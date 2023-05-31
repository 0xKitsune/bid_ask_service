use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::Debug,
    rc::Weak,
    sync::{Arc, RwLock},
};

use ordered_float::{Float, OrderedFloat};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::exchanges::Exchange;

use self::{
    error::OrderBookError,
    price_level::{
        ask::{self, Ask},
        bid::Bid,
        PriceLevelUpdate,
    },
};
pub mod btree_set;
pub mod error;
pub mod price_level;

//TODO: add a variant of the order book data structure where the order book will have a hashmap for quick update/removal

//TODO: if you want to read the order book, you will need this to be send/sync, if you just want updates from a channel you dont need this

//TODO: second off, this makes things a little bit easier, allowing you to have a rbtree or avl tree or other intrusive collection, without it needing to be thread safe

//TODO: add comment where it explains this represents the buy and sell side

//TODO: would need to implement order on bid and ask

//TODO: FIXME: we might still need this
pub trait Order: Ord {
    fn get_price(&self) -> &OrderedFloat<f64>;
    fn get_quantity(&self) -> &OrderedFloat<f64>;
    fn set_quantity(&mut self, quantity: OrderedFloat<f64>);
    fn get_exchange(&self) -> &Exchange;
}

pub trait OrderBook: Debug {
    fn update_bids(&mut self, bid: Bid, max_depth: usize);
    fn update_asks(&mut self, ask: Ask, max_depth: usize);
    fn get_best_bid(&self) -> Option<&Bid>;
    fn get_best_n_bids(&self, n: usize) -> Vec<Option<Bid>>;
    fn get_best_ask(&self) -> Option<&Ask>;
    fn get_best_n_asks(&self, n: usize) -> Vec<Option<Ask>>;

    //TODO: maybe add some functions that get best n bids and get best n asks?
}

pub struct AggregatedOrderBook<B: OrderBook + Send> {
    pub pair: [String; 2],
    pub exchanges: Vec<Exchange>,
    pub order_book: Arc<Mutex<B>>,
}

impl<B> AggregatedOrderBook<B>
where
    B: OrderBook + Send + 'static,
{
    pub fn new(pair: [&str; 2], exchanges: Vec<Exchange>, order_book: B) -> Self {
        AggregatedOrderBook {
            pair: [pair[0].to_string(), pair[1].to_string()],
            exchanges,
            order_book: Arc::new(Mutex::new(order_book)),
        }
    }

    pub async fn spawn_bid_ask_service(
        &self,

        best_n_orders: usize,
        max_order_book_depth: usize,
        order_book_stream_buffer: usize,
        price_level_buffer: usize,
    ) -> Result<Vec<JoinHandle<Result<(), OrderBookError>>>, OrderBookError> {
        let (price_level_tx, mut price_level_rx) =
            tokio::sync::mpsc::channel::<PriceLevelUpdate>(price_level_buffer);

        let mut handles = vec![];

        dbg!("spawning service for exchanges");

        for exchange in self.exchanges.iter() {
            handles.extend(
                exchange
                    .spawn_order_book_service(
                        [&self.pair[0], &self.pair[1]],
                        max_order_book_depth,
                        order_book_stream_buffer,
                        price_level_tx.clone(),
                    )
                    .await?,
            )
        }

        dbg!("spawning handler for orderbook");

        let order_book = self.order_book.clone();
        handles.push(tokio::spawn(async move {
            let mut best_bid = Bid::default();
            let mut best_ask = Ask::default();
            let mut best_n_bids: Vec<Option<Bid>> = vec![None; best_n_orders];
            let mut best_n_asks: Vec<Option<Ask>> = vec![None; best_n_orders];

            while let Some(price_level_update) = price_level_rx.recv().await {
                //TODO: FIXME: you can add a var here to check if the best ask is succeeded and then update the best ask,
                //best n asks, best bid, etc

                let mut update_best_bids = false;
                let mut update_best_asks = false;

                //TODO: can we make this concurrent so that these things happen at the same time?
                for ask in price_level_update.asks {
                    order_book
                        .lock()
                        .await
                        .update_asks(ask, max_order_book_depth);
                }
                //TODO: can we make this concurrent so that these things happen at the same time?
                for bid in price_level_update.bids {
                    order_book
                        .lock()
                        .await
                        .update_bids(bid, max_order_book_depth);
                }

                //TODO: do this concurrently
                if update_best_bids {
                    best_n_bids = order_book.lock().await.get_best_n_bids(best_n_orders);
                    if let Some(bid) = &best_n_bids[0] {
                        best_bid = bid.clone() //TODO: see if you need to clone here
                    }
                }

                if update_best_asks {
                    best_n_asks = order_book.lock().await.get_best_n_asks(best_n_orders);
                    if let Some(ask) = &best_n_asks[0] {
                        best_ask = ask.clone() //TODO: see if you need to clone here
                    }
                }

                //TODO: if bid or ask has been updated, send through the channel to the grpc server

                //TODO: look at caching the top 10 bids and send this all through to the grpc server
                //TODO:FIXME: if the top bids or asks change, recalc the spread and send the update
            }

            Ok::<(), OrderBookError>(())
        }));

        Ok(handles)
    }
}

#[cfg(test)]
mod tests {
    use crate::order_book::btree_set::BTreeSetOrderBook;
    use crate::{exchanges::Exchange, order_book::AggregatedOrderBook};
    #[tokio::test]
    async fn test_aggregated_order_book() {
        let order_book = BTreeSetOrderBook::new();

        let aggregated_order_book =
            AggregatedOrderBook::new(["eth", "btc"], vec![Exchange::Bitstamp], order_book);

        let join_handles = aggregated_order_book
            .spawn_bid_ask_service(10, 1000, 100, 20)
            .await
            .expect("TODO: handle this error");

        for handle in join_handles {
            handle
                .await
                .expect("TODO: handle this error")
                .expect("handle this error");
        }
    }
}

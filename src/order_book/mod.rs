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
    fn update_bids(&mut self, bid: Bid);
    fn update_asks(&mut self, ask: Ask);
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

    pub async fn listen_to_bid_ask_spread(
        &self,

        order_book_depth: usize,
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
                        order_book_depth,
                        order_book_stream_buffer,
                        price_level_tx.clone(),
                    )
                    .await?,
            )
        }

        dbg!("spawning handler for orderbook");

        let order_book = self.order_book.clone();
        handles.push(tokio::spawn(async move {
            while let Some(price_level_update) = price_level_rx.recv().await {
                for ask in price_level_update.asks {
                    order_book.lock().await.update_asks(ask);
                }

                for bid in price_level_update.bids {
                    order_book.lock().await.update_bids(bid);
                }

                //TODO: look at caching the top 10 bids and send this all through to the grpc server
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
            .listen_to_bid_ask_spread(10, 1000, 100)
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

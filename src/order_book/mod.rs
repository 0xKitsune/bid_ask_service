use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use ordered_float::OrderedFloat;
use tokio::{
    sync::{broadcast::Sender, mpsc::Receiver, Mutex},
    task::JoinHandle,
};

use crate::{
    error::BidAskServiceError,
    exchanges::Exchange,
    server::orderbook_service::{Level, Summary},
};

use self::{
    error::OrderBookError,
    price_level::{ask::Ask, bid::Bid, PriceLevelUpdate},
};
pub mod btree_set;
pub mod error;
pub mod price_level;

pub trait Order: Ord {
    fn get_price(&self) -> &OrderedFloat<f64>;
    fn get_quantity(&self) -> &OrderedFloat<f64>;
    fn set_quantity(&mut self, quantity: OrderedFloat<f64>);
    fn get_exchange(&self) -> &Exchange;
}

#[async_trait]
pub trait OrderBook: Debug {
    fn update_bids(&mut self, bid: Bid, max_depth: usize);
    fn update_asks(&mut self, ask: Ask, max_depth: usize);
    fn get_best_bid(&self) -> Option<&Bid>;
    fn get_best_n_bids(&self, n: usize) -> Vec<Option<Bid>>;
    fn get_best_ask(&self) -> Option<&Ask>;
    fn get_best_n_asks(&self, n: usize) -> Vec<Option<Ask>>;
}

pub trait BuySide: Debug {
    fn update_bids(&mut self, bid: Bid, max_depth: usize);
    fn get_best_bid(&self) -> Option<&Bid>;
    fn get_best_n_bids(&self, n: usize) -> Vec<Option<Bid>>;
}

pub trait SellSide: Debug {
    fn update_asks(&mut self, ask: Ask, max_depth: usize);
    fn get_best_ask(&self) -> Option<&Ask>;
    fn get_best_n_asks(&self, n: usize) -> Vec<Option<Ask>>;
}

pub struct AggregatedOrderBook<B: BuySide + Send, S: SellSide + Send> {
    pub pair: [String; 2],
    pub exchanges: Vec<Exchange>,
    pub bids: Arc<Mutex<B>>,
    pub asks: Arc<Mutex<S>>,
}

impl<B, S> AggregatedOrderBook<B, S>
where
    B: BuySide + Send + 'static,
    S: SellSide + Send + 'static,
{
    /// Creates a new instance of AggregatedOrderBook with the specified pair, exchanges, bids, and asks.
    pub fn new(pair: [&str; 2], exchanges: Vec<Exchange>, bids: B, asks: S) -> Self {
        AggregatedOrderBook {
            pair: [pair[0].to_string(), pair[1].to_string()],
            exchanges,
            bids: Arc::new(Mutex::new(bids)),
            asks: Arc::new(Mutex::new(asks)),
        }
    }

    /// Spawns the bid-ask service for the order book, with the specified configurations and channels,
    /// returning a vec of join handles for each exchange service and orderbook update logic
    pub fn spawn_bid_ask_service(
        &self,
        max_order_book_depth: usize,
        exchange_stream_buffer: usize,
        price_level_buffer: usize,
        best_n_orders: usize,
        summary_tx: Sender<Summary>,
    ) -> Vec<JoinHandle<Result<(), BidAskServiceError>>> {
        let (price_level_tx, price_level_rx) =
            tokio::sync::mpsc::channel::<PriceLevelUpdate>(price_level_buffer);
        let mut handles = vec![];

        //Spawn the order book service for each exchange, handling order book updates and sending them to the aggregated order book
        for exchange in self.exchanges.iter() {
            handles.extend(exchange.spawn_order_book_service(
                [&self.pair[0], &self.pair[1]],
                max_order_book_depth,
                exchange_stream_buffer,
                price_level_tx.clone(),
            ))
        }

        //Handle order book updates from the exchange streams, aggregating the order book and sending the summary to the gRPC server
        handles.push(self.handle_order_book_updates(
            price_level_rx,
            max_order_book_depth,
            best_n_orders,
            summary_tx,
        ));

        handles
    }

    pub fn handle_order_book_updates(
        &self,
        mut price_level_rx: Receiver<PriceLevelUpdate>,
        max_order_book_depth: usize,
        best_n_orders: usize,
        summary_tx: Sender<Summary>,
    ) -> JoinHandle<Result<(), BidAskServiceError>> {
        let bids = self.bids.clone();
        let asks = self.asks.clone();
        tokio::spawn(async move {
            let mut best_bid_price = 0.0;
            let mut best_ask_price = f64::MAX;

            //Track of the best n bids and asks to send to the gRPC server
            let mut best_n_bids: Vec<Level> = vec![];
            let mut best_n_asks: Vec<Level> = vec![];

            //Track the last bid and ask to determine if the best n bids and asks need to be updated when a new bid/ask comes in
            let mut last_bid = Bid::default();
            let mut last_ask = Ask::default();

            while let Some(price_level_update) = price_level_rx.recv().await {
                let bids_fut = async {
                    //Add each bid to the aggregated order book, checking if the bid is better than the "worst" bid in the top n bids
                    let mut update_best_bids = false;
                    for bid in price_level_update.bids {
                        if bid.cmp(&last_bid).is_ge() {
                            update_best_bids = true;
                        }
                        bids.lock().await.update_bids(bid, max_order_book_depth);
                    }

                    //If the bid is better than the "worst" bid in the top bids, update the best n bids
                    if update_best_bids {
                        let mut best_bids = bids.lock().await.get_best_n_bids(best_n_orders);
                        if let Some(_) = &best_bids[0] {
                            let mut best_n_levels = vec![];

                            //Get the best "n" bids and add the level to the best n levels
                            let mut last_bid = 0;
                            for bid_option in best_bids.iter() {
                                if let Some(bid) = bid_option {
                                    best_n_levels.push(Level {
                                        price: bid.price.0,
                                        amount: bid.quantity.0,
                                        exchange: bid.exchange.to_string(),
                                    });

                                    last_bid += 1;
                                } else {
                                    break;
                                }
                            }

                            //Return the best levels, the first bid, and the last bid
                            Some((
                                best_n_levels,
                                best_bids[0].take().unwrap().price.0,
                                best_bids[last_bid - 1].take().unwrap(),
                            ))
                        } else {
                            tracing::error!("No bids in aggregated order book");
                            None
                        }
                    } else {
                        None
                    }
                };

                let asks_fut = async {
                    let mut update_best_asks = false;

                    for ask in price_level_update.asks {
                        if ask.cmp(&last_ask).is_le() {
                            update_best_asks = true;
                        }
                        asks.lock().await.update_asks(ask, max_order_book_depth);
                    }

                    //If the ask is better than the "worst" ask in the top asks, update the best n bids
                    if update_best_asks {
                        let mut best_asks = asks.lock().await.get_best_n_asks(best_n_orders);

                        if let Some(_) = &best_asks[0] {
                            let mut best_n_levels = vec![];

                            //Get the best "n" asks and add the level to the best n levels
                            let mut last_ask = 0;
                            for ask_option in best_asks.iter() {
                                if let Some(ask) = ask_option {
                                    best_n_levels.push(Level {
                                        price: ask.price.0,
                                        amount: ask.quantity.0,
                                        exchange: ask.exchange.to_string(),
                                    });

                                    last_ask += 1;
                                } else {
                                    break;
                                }
                            }

                            //Return the best levels, the first ask, and the last ask
                            Some((
                                best_n_levels,
                                best_asks[0].take().unwrap().price.0,
                                best_asks[last_ask - 1].take().unwrap(),
                            ))
                        } else {
                            tracing::error!("No asks in aggregated order book");
                            None
                        }
                    } else {
                        None
                    }
                };

                //Join the futures so that the bids and asks can be updated concurrently
                let (updated_bids, updated_asks) = tokio::join!(bids_fut, asks_fut);

                //Update the best n bids and asks if they have been updated
                if let Some((best_bids, first_price, last)) = updated_bids {
                    best_n_bids = best_bids;
                    best_bid_price = first_price;
                    last_bid = last;
                }

                //Update the best n asks and asks if they have been updated
                if let Some((best_asks, first_price, last)) = updated_asks {
                    best_n_asks = best_asks;
                    best_ask_price = first_price;
                    last_ask = last;
                }

                //Calculate the bid-ask spread and send the updated summary to the gRPC server
                let bid_ask_spread = best_ask_price - best_bid_price;

                let summary = Summary {
                    spread: bid_ask_spread,
                    bids: best_n_bids.clone(),
                    asks: best_n_asks.clone(),
                };

                tracing::info!("Publishing summary: {:?}", summary);

                summary_tx
                    .send(summary)
                    .map_err(OrderBookError::SummarySendError)?;
            }

            Ok::<(), BidAskServiceError>(())
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use std::collections::BTreeSet;

//     use futures::FutureExt;

//     use crate::order_book::Ask;
//     use crate::order_book::Bid;
//     use crate::{exchanges::Exchange, order_book::AggregatedOrderBook};
//     #[tokio::test]
//     async fn test_aggregated_order_book() {
//         let bids = BTreeSet::<Bid>::new();
//         let asks = BTreeSet::<Ask>::new();

//         let aggregated_order_book = AggregatedOrderBook::new(
//             ["eth", "btc"],
//             vec![Exchange::Bitstamp, Exchange::Binance],
//             bids,
//             asks,
//         );

//         let join_handles = aggregated_order_book
//             .spawn_bid_ask_service(10, 1000, 100, 20)
//             .await
//             .expect("TODO: handle this error");

//         let futures = join_handles
//             .into_iter()
//             .map(|handle| handle.boxed())
//             .collect::<Vec<_>>();

//         //Wait for the first future to be finished
//         let (result, _, _) = futures::future::select_all(futures).await;

//         //TODO: update his handling and test
//         result.expect("error").expect("errr");
//     }
// }

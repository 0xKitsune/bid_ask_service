use std::{
    cmp::Ordering,
    collections::BTreeMap,
    rc::Weak,
    sync::{Arc, RwLock},
};

use ordered_float::{Float, OrderedFloat};
use tokio::task::JoinHandle;

use crate::exchanges::Exchange;

use self::{
    error::OrderBookError,
    price_level::{ask::Ask, bid::Bid, PriceLevelUpdate},
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

pub trait OrderBook {
    fn update_bids(&mut self, bid: Bid) -> Result<(), OrderBookError>;
    fn update_asks(&mut self, ask: Ask) -> Result<(), OrderBookError>;
}

// pub struct AggregatedOrderBook<B: OrderBook> {
//     pub pair: [String; 2],
//     pub exchanges: Vec<Exchange>,
//     pub order_book: B,
// }

// impl<B> AggregatedOrderBook<B>
// where
//     B: OrderBook,
// {
//     pub fn new(pair: [&str; 2], exchanges: Vec<Exchange>, order_book: B) -> Self {
//         AggregatedOrderBook {
//             pair: [pair[0].to_string(), pair[1].to_string()],
//             exchanges,
//             order_book,
//         }
//     }

//     pub async fn listen_to_bid_ask_spread(
//         &self,
//         order_book_depth: usize,
//         order_book_stream_buffer: usize,
//         price_level_buffer: usize,
//     ) -> Result<Vec<JoinHandle<Result<(), OrderBookError>>>, OrderBookError> {
//         let (price_level_tx, mut price_level_rx) =
//             tokio::sync::mpsc::channel::<PriceLevelUpdate>(price_level_buffer);

//         let mut handles = vec![];

//         for exchange in self.exchanges.iter() {
//             handles.extend(
//                 exchange
//                     .spawn_order_book_service(
//                         [&self.pair[0], &self.pair[1]],
//                         order_book_depth,
//                         order_book_stream_buffer,
//                         price_level_tx.clone(),
//                     )
//                     .await?,
//             )
//         }

//         // handles.push(tokio::spawn(async move {

//         //     while let Some(price_level_update) = price_level_rx.recv().await {
//         //         order_book.update_book(price_level_update)?;

//         //     }

//         //     Ok::<(), OrderBookError>(())
//         // }));

//         Ok(handles)
//     }
// }

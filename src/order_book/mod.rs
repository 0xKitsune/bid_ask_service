use std::{
    collections::BTreeMap,
    rc::Weak,
    sync::{Arc, RwLock},
};

pub mod rbtree;

use ordered_float::{Float, OrderedFloat};
use tokio::task::JoinHandle;

use crate::exchanges::Exchange;

use self::error::OrderBookError;
pub mod error;

//TODO: add a variant of the order book data structure where the order book will have a hashmap for quick update/removal

//TODO: if you want to read the order book, you will need this to be send/sync, if you just want updates from a channel you dont need this

//TODO: second off, this makes things a little bit easier, allowing you to have a rbtree or avl tree or other intrusive collection, without it needing to be thread safe

pub trait OrderBook {
    fn update_book(&self, price_level_update: PriceLevelUpdate) -> Result<(), OrderBookError>;
}

pub struct AggregatedOrderBook<B: OrderBook + 'static> {
    pub pair: [String; 2],
    pub exchanges: Vec<Exchange>,
    pub order_book: B, //TODO: you dont need this
}

impl<B> AggregatedOrderBook<B>
where
    B: OrderBook,
{
    pub fn new(pair: [&str; 2], exchanges: Vec<Exchange>, order_book: B) -> Self {
        AggregatedOrderBook {
            pair: [pair[0].to_string(), pair[1].to_string()],
            exchanges,
            order_book,
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

        // handles.push(tokio::spawn(async move {

        //     while let Some(price_level_update) = price_level_rx.recv().await {
        //         order_book.update_book(price_level_update)?;

        //     }

        //     Ok::<(), OrderBookError>(())
        // }));

        Ok(handles)
    }
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
    pub exchange: Exchange,
}

impl PriceLevel {
    pub fn new(price: f64, quantity: f64, exchange: Exchange) -> Self {
        PriceLevel {
            price,
            quantity,
            exchange,
        }
    }
}

#[derive(Debug)]
pub enum PriceLevelUpdate {
    Bid(PriceLevel),
    Ask(PriceLevel),
}

#[cfg(test)]
mod tests {}

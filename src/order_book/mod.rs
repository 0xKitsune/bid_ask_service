use std::{
    cmp::Ordering,
    collections::BTreeMap,
    rc::Weak,
    sync::{Arc, RwLock},
};

pub mod binary_tree;
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
    type Bids;
    type Asks;

    fn update_book(&self, price_level_update: PriceLevelUpdate) -> Result<(), OrderBookError> {
        match price_level_update {
            PriceLevelUpdate::Bid(_) => self.update_bids(price_level_update),
            PriceLevelUpdate::Ask(_) => self.update_asks(price_level_update),
        }
    }

    fn update_bids(&self, price_level_update: PriceLevelUpdate) -> Result<(), OrderBookError>;
    fn update_asks(&self, price_level_update: PriceLevelUpdate) -> Result<(), OrderBookError>;
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
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
    pub exchange: Exchange,
}

impl PriceLevel {
    pub fn new(price: f64, quantity: f64, exchange: Exchange) -> Self {
        PriceLevel {
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
            exchange,
        }
    }
}

#[derive(Debug)]
pub enum PriceLevelUpdate {
    Bid(PriceLevel),
    Ask(PriceLevel),
}

impl PartialEq for PriceLevel {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.quantity == other.quantity
    }
}

impl Eq for PriceLevel {}

impl PartialOrd for PriceLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriceLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Equal => self.quantity.cmp(&other.quantity),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {

    //TODO: add tests to compare price level

    use ordered_float::OrderedFloat;

    use crate::exchanges::Exchange;

    use super::PriceLevel;

    #[test]
    pub fn test_price_level_greater() {
        //test when the price is greater but the quantity is the same
        let price_level_0 = PriceLevel {
            price: OrderedFloat(1.23),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        let price_level_1 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        assert_eq!(price_level_0 > price_level_1, true);

        //Test when the price is the same but the quantity is greater
        let price_level_2 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1300.23),
            exchange: Exchange::Binance,
        };

        let price_level_3 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        assert_eq!(price_level_2 > price_level_3, true);

        //test when the price is greater but the quantity is the same and the exchanges are different
        let price_level_4 = PriceLevel {
            price: OrderedFloat(1.23),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Bitstamp,
        };

        let price_level_5 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        assert_eq!(price_level_4 > price_level_5, true);

        //Test when the price is the same but the quantity is greater and the exchanges are different
        let price_level_6 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1300.23),
            exchange: Exchange::Bitstamp,
        };

        let price_level_7 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        assert_eq!(price_level_6 > price_level_7, true);
    }
    #[test]
    pub fn test_price_level_less_than() {
        //test when the price is less but the quantity is the same
        let price_level_0 = PriceLevel {
            price: OrderedFloat(1.23),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        let price_level_1 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        assert_eq!(price_level_1 < price_level_0, true);

        //Test when the price is the same but the quantity is less
        let price_level_2 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1300.23),
            exchange: Exchange::Binance,
        };

        let price_level_3 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        assert_eq!(price_level_3 < price_level_2, true);

        //test when the price is less but the quantity is the same and the exchanges are different
        let price_level_4 = PriceLevel {
            price: OrderedFloat(1.23),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Bitstamp,
        };

        let price_level_5 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        assert_eq!(price_level_5 < price_level_4, true);

        //Test when the price is the same but the quantity is less and the exchanges are different
        let price_level_6 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1300.23),
            exchange: Exchange::Bitstamp,
        };

        let price_level_7 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        assert_eq!(price_level_7 < price_level_6, true);
    }
    #[test]
    pub fn test_price_level_equal() {
        //test when the price, quantity and the exchanges are the same
        let price_level_0 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        let price_level_1 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        assert_eq!(price_level_0 == price_level_1, true);

        //test when the price and quantity are the same but the exchange is different
        let price_level_2 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Binance,
        };

        let price_level_3 = PriceLevel {
            price: OrderedFloat(1.20),
            quantity: OrderedFloat(1200.56),
            exchange: Exchange::Bitstamp,
        };

        assert_eq!(price_level_2 == price_level_3, true);
    }
}

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    rc::Weak,
    sync::{Arc, RwLock},
};

use ordered_float::{Float, OrderedFloat};
use tokio::task::JoinHandle;

use crate::exchanges::Exchange;

#[derive(Debug, Clone)]
pub enum PriceLevel {
    Bid(Bid),
    Ask(Ask),
}

impl PriceLevel {
    pub fn new(price: f64, quantity: f64, exchange: Exchange, order_type: OrderType) -> Self {
        match order_type {
            OrderType::Bid => PriceLevel::Bid(Bid::new(price, quantity, exchange)),
            OrderType::Ask => PriceLevel::Ask(Ask::new(price, quantity, exchange)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bid {
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
    pub exchange: Exchange,
}

impl Bid {
    pub fn new(price: f64, quantity: f64, exchange: Exchange) -> Self {
        Bid {
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
            exchange,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ask {
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
    pub exchange: Exchange,
}

impl Ask {
    pub fn new(price: f64, quantity: f64, exchange: Exchange) -> Self {
        Ask {
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
            exchange,
        }
    }
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]

pub struct PriceLevelUpdate {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

impl PriceLevelUpdate {
    pub fn new(bids: Vec<PriceLevel>, asks: Vec<PriceLevel>) -> Self {
        PriceLevelUpdate { bids, asks }
    }
}

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.quantity == other.quantity
    }
}

impl Eq for Bid {}
impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Equal => self.quantity.cmp(&other.quantity),
            other => other,
        }
    }
}

impl PartialEq for Ask {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.quantity == other.quantity
    }
}

impl Eq for Ask {}
impl PartialOrd for Ask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

//When ordering asks, we want the lowest price with the highest quantity to be the best
//so a price level with the same price but higher quantity should be considered less than a price level
//with the same price but lower quantity in order to ensure that the best price is considered the ask that is lesser than the other
impl Ord for Ask {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Equal => self.quantity.cmp(&other.quantity).reverse(),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {

    //TODO: add tests to compare price level

    use crate::{
        exchanges::Exchange,
        order_book::{Ask, Bid},
    };

    use super::PriceLevel;

    //TODO: add comparison for asks update and test everything
    #[test]
    pub fn test_bid_greater() {
        //the price is greater but the quantity is the same
        let bid_0 = Bid::new(1.23, 1200.56, Exchange::Binance);
        let bid_1 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(bid_0 > bid_1, true);

        //the price is the same but the quantity is greater
        let bid_2 = Bid::new(1.20, 1300.56, Exchange::Binance);
        let bid_3 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(bid_2 > bid_3, true);

        //the price is greater but the quantity is the same and the exchanges are different
        let bid_4 = Bid::new(1.24, 1200.56, Exchange::Binance);
        let bid_5 = Bid::new(1.20, 1200.56, Exchange::Bitstamp);
        assert_eq!(bid_4 > bid_5, true);

        //the price is the same but the quantity is greater and the exchanges are different
        let bid_6 = Bid::new(1.20, 1300.56, Exchange::Binance);
        let bid_7 = Bid::new(1.20, 1200.56, Exchange::Bitstamp);

        assert_eq!(bid_6 > bid_7, true);

        //TODO: add case for price and quant are different
    }
    #[test]
    pub fn test_bid_less() {
        //the price is less but the quantity is the same
        let bid_0 = Bid::new(1.23, 1200.56, Exchange::Binance);
        let bid_1 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(bid_1 < bid_0, true);

        //the price is the same but the quantity is less
        let bid_2 = Bid::new(1.20, 1300.56, Exchange::Binance);
        let bid_3 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(bid_3 < bid_2, true);

        //the price is less but the quantity is the same and the exchanges are different
        let bid_4 = Bid::new(1.25, 1200.56, Exchange::Bitstamp);
        let bid_5 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(bid_5 < bid_4, true);

        //the price is the same but the quantity is less and the exchanges are different
        let bid_6 = Bid::new(1.20, 1300.56, Exchange::Binance);
        let bid_7 = Bid::new(1.20, 1200.56, Exchange::Bitstamp);

        assert_eq!(bid_7 < bid_6, true);
        //TODO: add case for price and quant are different
    }
    #[test]
    pub fn test_bid_equal() {
        //the price, quantity and the exchanges are the same
        let bid_0 = Bid::new(1.20, 1200.56, Exchange::Binance);
        let bid_1 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(bid_0 == bid_1, true);

        //the price and quantity are the same but the exchange is different
        let bid_2 = Bid::new(1.20, 1200.56, Exchange::Binance);
        let bid_3 = Bid::new(1.20, 1200.56, Exchange::Bitstamp);

        assert_eq!(bid_2 == bid_3, true);
    }

    #[test]
    pub fn test_ask_less() {
        //the price is less but the quantity is the same
        let ask_0 = Ask::new(1.23, 1200.56, Exchange::Binance);
        let ask_1 = Ask::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(ask_1 < ask_0, true);

        //the price is the same but the quantity is less
        let ask_2 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_3 = Ask::new(1.20, 1300.56, Exchange::Binance);

        assert_eq!(ask_3 < ask_2, true);

        //the price is less but the quantity is the same and the exchanges are different
        let ask_4 = Ask::new(1.25, 1200.56, Exchange::Bitstamp);
        let ask_5 = Ask::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(ask_5 < ask_4, true);

        //the price is the same but the quantity is less and the exchanges are different
        let ask_6 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);
        let ask_7 = Ask::new(1.20, 1300.56, Exchange::Binance);

        assert_eq!(ask_7 < ask_6, true);

        //TODO: add case for price and quant are different
    }

    #[test]
    pub fn test_ask_greater() {
        //the price is greater but the quantity is the same
        let ask_0 = Ask::new(1.23, 1200.56, Exchange::Binance);
        let ask_1 = Ask::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(ask_0 > ask_1, true);

        //the price is the same but the quantity is greater
        //TODO: add some comments as to why the ask is less when the quantity is greater
        let ask_2 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_3 = Ask::new(1.20, 1300.56, Exchange::Binance);

        assert_eq!(ask_2 > ask_3, true);

        //the price is greater but the quantity is the same and the exchanges are different
        let ask_4 = Ask::new(1.24, 1200.56, Exchange::Binance);
        let ask_5 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);
        assert_eq!(ask_4 > ask_5, true);

        //the price is the same but the quantity is greater and the exchanges are different
        //TODO: add some comments why this is the case
        let ask_6 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);
        let ask_7 = Ask::new(1.20, 1300.56, Exchange::Binance);

        //TODO: should fail
        assert_eq!(ask_6 > ask_7, true);

        //TODO: add case for price and quant are different
    }
    #[test]
    pub fn test_ask_equal() {
        //the price, quantity and the exchanges are the same
        let ask_0 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_1 = Ask::new(1.20, 1200.56, Exchange::Binance);

        assert_eq!(ask_0 == ask_1, true);

        //the price and quantity are the same but the exchange is different
        let ask_2 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_3 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert_eq!(ask_2 == ask_3, true);

        //TODO: add case for price and quant are different
    }
}
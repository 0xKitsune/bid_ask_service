use std::{
    cmp::Ordering,
    collections::BTreeMap,
    rc::Weak,
    sync::{Arc, RwLock},
};

use ordered_float::{Float, OrderedFloat};
use tokio::task::JoinHandle;

use crate::exchanges::Exchange;

use super::Order;

//TODO: prob refactor this

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

impl Order for Bid {
    fn get_price(&self) -> &OrderedFloat<f64> {
        &self.price
    }
    fn get_quantity(&self) -> &OrderedFloat<f64> {
        &self.quantity
    }
    fn set_quantity(&mut self, quantity: OrderedFloat<f64>) {
        self.quantity = quantity;
    }
    fn get_exchange(&self) -> &Exchange {
        &self.exchange
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
    pub bids: Vec<Bid>,
    pub asks: Vec<Ask>,
}

impl PriceLevelUpdate {
    pub fn new(bids: Vec<Bid>, asks: Vec<Ask>) -> Self {
        PriceLevelUpdate { bids, asks }
    }
}

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
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
        //First check if the price is equal
        match self.price.cmp(&other.price) {
            //If the price is equal, check the exchange, this allows the order book structure to know to replace the quantity for this value
            Ordering::Equal => match self.exchange.cmp(&other.exchange) {
                Ordering::Equal => Ordering::Equal,

                //If the price is the same but the exchange is different, compare the quantity
                exchange_order => match self.quantity.cmp(&other.quantity) {
                    Ordering::Equal => exchange_order,
                    other => other,
                },
            },
            other => other,
        }
    }
}

impl PartialEq for Ask {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
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
        //First check if the price is equal
        match self.price.cmp(&other.price) {
            //If the price is equal, check the exchange, this allows the order book structure to know to replace the quantity for this value
            Ordering::Equal => match self.exchange.cmp(&other.exchange).reverse() {
                Ordering::Equal => Ordering::Equal,

                //If the price is the same but the exchange is different, compare the quantity
                exchange_order => match self.quantity.cmp(&other.quantity).reverse() {
                    Ordering::Equal => exchange_order,
                    other => other,
                },
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        exchanges::Exchange,
        order_book::{Ask, Bid},
    };

    #[test]
    pub fn test_bid_greater() {
        //the price is greater but the quantity is the same
        let bid_0 = Bid::new(1.23, 1200.56, Exchange::Binance);
        let bid_1 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_0 > bid_1);

        //the price is greater but the quantity is the same and the exchanges are different
        let bid_4 = Bid::new(1.24, 1200.56, Exchange::Bitstamp);
        let bid_5 = Bid::new(1.20, 1200.56, Exchange::Binance);
        assert!(bid_4 > bid_5);

        //the price is the same but the quantity is greater and the exchanges are different
        let bid_6 = Bid::new(1.20, 1300.56, Exchange::Binance);
        let bid_7 = Bid::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(bid_6 > bid_7);

        //the price and quantity are different
        let bid_8 = Bid::new(1.23, 1000.56, Exchange::Binance);
        let bid_9 = Bid::new(1.20, 1200.56, Exchange::Bitstamp);
        assert!(bid_8 > bid_9);

        //the price and quantity are the same but the exchange is different
        let bid_10 = Bid::new(1.20, 1000.56, Exchange::Binance);
        let bid_11 = Bid::new(1.20, 1000.56, Exchange::Bitstamp);

        assert!(bid_10 > bid_11);
    }

    #[test]
    pub fn test_bid_less() {
        //the price is less but the quantity is the same
        let bid_0 = Bid::new(1.23, 1200.56, Exchange::Binance);
        let bid_1 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_1 < bid_0);

        //the price is less but the quantity is the same and the exchanges are different
        let bid_4 = Bid::new(1.25, 1200.56, Exchange::Bitstamp);
        let bid_5 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_5 < bid_4);

        //the price is the same but the quantity is less and the exchanges are different
        let bid_6 = Bid::new(1.20, 1300.56, Exchange::Bitstamp);
        let bid_7 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_7 < bid_6);

        //the price and quantity are different
        let bid_8 = Bid::new(1.23, 1000.56, Exchange::Bitstamp);
        let bid_9 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_9 < bid_8);

        //the price and quantity are the same but the exchange is different
        let bid_10 = Bid::new(1.20, 1000.56, Exchange::Binance);
        let bid_11 = Bid::new(1.20, 1000.56, Exchange::Bitstamp);

        assert!(bid_11 < bid_10);
    }
    #[test]
    pub fn test_bid_equal() {
        //the price, quantity and the exchanges are the same
        let bid_0 = Bid::new(1.20, 1200.56, Exchange::Binance);
        let bid_1 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_0 == bid_1);

        //the price and exchange are the same but the quantity is different
        let bid_2 = Bid::new(1.20, 12309.56, Exchange::Binance);
        let bid_3 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_2 == bid_3);
    }

    #[test]
    pub fn test_ask_less() {
        //the price is less but the quantity is the same
        let ask_0 = Ask::new(1.23, 1200.56, Exchange::Binance);
        let ask_1 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_1 < ask_0);

        //the price is the same but the quantity is less
        let ask_2 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_3 = Ask::new(1.20, 1300.56, Exchange::Bitstamp);

        assert!(ask_3 < ask_2);

        //the price is less but the quantity is the same and the exchanges are different
        let ask_4 = Ask::new(1.25, 1200.56, Exchange::Binance);
        let ask_5 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_5 < ask_4);

        //the price is the same but the quantity is less and the exchanges are different
        let ask_6 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);
        let ask_7 = Ask::new(1.20, 1300.56, Exchange::Binance);

        assert!(ask_7 < ask_6);

        //the price and quantity are different
        let ask_8 = Ask::new(1.23, 1500.56, Exchange::Binance);
        let ask_9 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_9 < ask_8);

        //the price and quantity are the same but the exchange is different
        let ask_10 = Ask::new(1.20, 1000.56, Exchange::Bitstamp);
        let ask_11 = Ask::new(1.20, 1000.56, Exchange::Binance);

        assert!(ask_11 < ask_10);
    }

    #[test]
    pub fn test_ask_greater() {
        //the price is greater but the quantity is the same
        let ask_0 = Ask::new(1.23, 1200.56, Exchange::Binance);
        let ask_1 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_0 > ask_1);

        //the price is the same but the quantity is greater
        let ask_2 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_3 = Ask::new(1.20, 1300.56, Exchange::Bitstamp);

        assert!(ask_2 > ask_3);

        //the price is greater but the quantity is the same and the exchanges are different
        let ask_4 = Ask::new(1.25, 1200.56, Exchange::Binance);
        let ask_5 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_4 > ask_5);

        //the price is the same but the quantity is greater and the exchanges are different
        let ask_6 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);
        let ask_7 = Ask::new(1.20, 1300.56, Exchange::Binance);

        assert!(ask_6 > ask_7);

        //the price and quantity are the same but the exchange is different
        let ask_8 = Ask::new(1.20, 1000.56, Exchange::Bitstamp);
        let ask_9 = Ask::new(1.20, 1000.56, Exchange::Binance);
        assert!(ask_9 < ask_8);
    }
    #[test]
    pub fn test_ask_equal() {
        //the price, quantity and the exchanges are the same
        let ask_0 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_1 = Ask::new(1.20, 1200.56, Exchange::Binance);

        assert!(ask_0 == ask_1);

        //the price and exchange are the same but the quantity is different
        let ask_2 = Ask::new(1.20, 234235.56, Exchange::Bitstamp);
        let ask_3 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_2 == ask_3);
    }
}

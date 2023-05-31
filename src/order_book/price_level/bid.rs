use std::{
    cmp::Ordering,
    collections::BTreeMap,
    rc::Weak,
    sync::{Arc, RwLock},
};

use ordered_float::{Float, OrderedFloat};
use tokio::task::JoinHandle;

use crate::{exchanges::Exchange, order_book::Order};

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

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
            && self.quantity == other.quantity
            && self.exchange == other.exchange
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

#[cfg(test)]
mod tests {

    use crate::{
        exchanges::Exchange,
        order_book::{Ask, Bid},
    };




    the update to the partial eq will break some things just a heads up, we will need to update the gt lt and eq tests to use cmp
    since eq is a strict equality check now

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
}

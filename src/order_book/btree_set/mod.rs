use std::collections::{BTreeMap, BTreeSet};

use ordered_float::OrderedFloat;

use crate::exchanges::Exchange;

use super::{
    error::OrderBookError,
    price_level::{ask::Ask, bid::Bid},
    Order, OrderBook,
};

pub struct BTreeSetOrderBook {
    pub bids: BTreeSet<Bid>,
    pub asks: BTreeSet<Ask>,
}

impl BTreeSetOrderBook {
    pub fn new() -> Self {
        BTreeSetOrderBook {
            bids: BTreeSet::new(),
            asks: BTreeSet::new(),
        }
    }
}

impl OrderBook for BTreeSetOrderBook {
    fn update_bids(&mut self, bid: Bid) -> Result<(), OrderBookError> {
        if bid.get_quantity().0 == 0.0 {
            self.bids.remove(&bid);
        } else {
            self.bids.insert(bid);
        }

        Ok(())
    }

    fn update_asks(&mut self, ask: Ask) -> Result<(), OrderBookError> {
        if ask.get_quantity().0 == 0.0 {
            self.asks.remove(&ask);
        } else {
            self.asks.insert(ask);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{exchanges::Exchange, order_book::price_level::bid::Bid};

    use super::BTreeSetOrderBook;

    #[test]
    fn test_insert_bid() {
        let mut order_book = BTreeSetOrderBook::new();

        let bid_0 = Bid::new(100.00, 50.0, Exchange::Binance);
        let bid_1 = Bid::new(100.00, 50.0, Exchange::Bitstamp);
        let bid_2 = Bid::new(101.00, 50.0, Exchange::Binance);
        let bid_3 = Bid::new(101.00, 50.0, Exchange::Bitstamp);
        let bid_4 = Bid::new(103.00, 50.0, Exchange::Binance);
        let bid_5 = Bid::new(102.00, 50.0, Exchange::Binance);
        let bid_6 = Bid::new(104.00, 50.0, Exchange::Binance);

        // create an expected bids vector
        let mut expected_bids = vec![
            bid_0.clone(),
            bid_1.clone(),
            bid_2.clone(),
            bid_3.clone(),
            bid_4.clone(),
            bid_5.clone(),
            bid_6.clone(),
        ];
        // sort the vector because BTreeSet is ordered
        expected_bids.sort();

        order_book.bids.insert(bid_0);
        order_book.bids.insert(bid_1);
        order_book.bids.insert(bid_2);
        order_book.bids.insert(bid_3);
        order_book.bids.insert(bid_4);
        order_book.bids.insert(bid_5);
        order_book.bids.insert(bid_6);

        // collect the actual bids from the BTreeSet into a vector
        let actual_bids: Vec<Bid> = order_book.bids.iter().cloned().collect();

        assert_eq!(actual_bids, expected_bids);
    }

    #[test]
    fn test_remove_bid() {
        let mut order_book = BTreeSetOrderBook::new();

        let bid_0 = Bid::new(100.00, 50.0, Exchange::Binance);
        let bid_1 = Bid::new(100.00, 50.0, Exchange::Bitstamp);
        let bid_2 = Bid::new(101.00, 50.0, Exchange::Binance);
        let bid_3 = Bid::new(101.00, 50.0, Exchange::Bitstamp);
        let bid_4 = Bid::new(103.00, 50.0, Exchange::Binance);
        let bid_5 = Bid::new(102.00, 50.0, Exchange::Binance);
        let bid_6 = Bid::new(104.00, 50.0, Exchange::Binance);

        // create an expected bids vector
        let mut expected_bids = vec![
            bid_0.clone(),
            bid_1.clone(),
            bid_2.clone(),
            bid_3.clone(),
            bid_4.clone(),
        ];

        // sort the vector because BTreeSet is ordered
        expected_bids.sort();

        order_book.bids.insert(bid_0);
        order_book.bids.insert(bid_1.clone());
        order_book.bids.insert(bid_2);
        order_book.bids.insert(bid_3);
        order_book.bids.insert(bid_4.clone());
        order_book.bids.insert(bid_5);
        order_book.bids.insert(bid_6.clone());

        order_book.bids.remove(&bid_1);
        order_book.bids.remove(&bid_4);
        order_book.bids.remove(&bid_6);

        // collect the actual bids from the BTreeSet into a vector
        let actual_bids: Vec<Bid> = order_book.bids.iter().cloned().collect();

        assert_eq!(actual_bids, expected_bids);
    }

    #[test]
    fn test_update_bid() {
        let mut order_book = BTreeSetOrderBook::new();

        let bid_0 = Bid::new(100.00, 50.0, Exchange::Binance);
        let bid_1 = Bid::new(100.00, 50.0, Exchange::Bitstamp);
        let bid_2 = Bid::new(101.00, 50.0, Exchange::Binance);
        let bid_3 = Bid::new(101.00, 499.0, Exchange::Bitstamp);
        let bid_4 = Bid::new(103.00, 50.0, Exchange::Binance);
        let bid_5 = Bid::new(102.00, 50.0, Exchange::Binance);
        let bid_6 = Bid::new(104.00, 50.0, Exchange::Binance);

        let replacement_bid_1 = Bid::new(100.00, 3404.0, Exchange::Bitstamp);
        let replacement_bid_3 = Bid::new(101.00, 12309.0, Exchange::Binance);
        let replacement_bid_6 = Bid::new(104.00, 20.0, Exchange::Bitstamp);

        // create an expected bids vector
        let mut expected_bids = vec![
            bid_0.clone(),
            replacement_bid_1.clone(),
            bid_2.clone(),
            replacement_bid_3.clone(),
            bid_4.clone(),
            bid_5.clone(),
            replacement_bid_6.clone(),
        ];

        // sort the vector because BTreeSet is ordered
        expected_bids.sort();

        order_book.bids.insert(bid_0);
        order_book.bids.insert(bid_1);
        order_book.bids.insert(bid_2);
        order_book.bids.insert(bid_3);
        order_book.bids.insert(bid_4);
        order_book.bids.insert(bid_5);
        order_book.bids.insert(bid_6);

        //insert the replacement bids
        order_book.bids.insert(replacement_bid_6);
        order_book.bids.insert(replacement_bid_3);
        order_book.bids.insert(replacement_bid_1);

        // collect the actual bids from the BTreeSet into a vector
        let actual_bids: Vec<Bid> = order_book.bids.iter().cloned().collect();

        assert_eq!(actual_bids, expected_bids);

        dbg!("here");
    }
}

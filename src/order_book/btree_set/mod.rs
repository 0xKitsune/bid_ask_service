use std::collections::{BTreeMap, BTreeSet};

use ordered_float::OrderedFloat;

use crate::exchanges::Exchange;

use super::{
    error::OrderBookError,
    price_level::{ask::Ask, bid::Bid},
    Order, OrderBook,
};

#[derive(Debug, Clone)] //Clone used for benching
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

//TODO: we can do something like check if the bid is better than the worst bid, if it is, then update the tree, and remove the worst bid if its full
//TODO: then you can return the top m bids or asks if its updated

impl OrderBook for BTreeSetOrderBook {
    fn update_bids(&mut self, bid: Bid, max_depth: usize) {
        if bid.get_quantity().0 == 0.0 {
            self.bids.remove(&bid);
        } else if self.bids.len() < max_depth {
            if self.bids.contains(&bid) {
                //We have to remove and insert because the replace method replaces the value at the pointer.
                //Since the two are seen as equal, it does not reorder the tree
                self.bids.remove(&bid);
                self.bids.insert(bid);
            } else {
                self.bids.insert(bid);
            }
        } else {
            // check if the bid is better than the worst bid
            let bid_is_better = {
                //We can unwrap this because we have already asserted that the bids.len() is not less than the max depth
                //signifying that there is at least one value
                let worst_bid = self.bids.iter().next().unwrap();
                bid > *worst_bid
            };

            if bid_is_better {
                self.bids.pop_first();
                self.bids.insert(bid);
            }
        }
    }

    fn update_asks(&mut self, ask: Ask, max_depth: usize) {
        if ask.get_quantity().0 == 0.0 {
            self.asks.remove(&ask);
        } else if self.asks.len() < max_depth {
            if self.asks.contains(&ask) {
                //We have to remove and insert because the replace method replaces the value at the pointer.
                //Since the two are seen as equal, it does not reorder the tree
                self.asks.remove(&ask);
                self.asks.insert(ask);
            } else {
                self.asks.insert(ask);
            }
        } else {
            // check if the bid is better than the worst bid
            let ask_is_better = {
                //We can unwrap this because we have already asserted that the bids.len() is not less than the max depth
                //signifying that there is at least one value
                let worst_ask = self.asks.iter().next_back().unwrap();
                ask < *worst_ask
            };

            if ask_is_better {
                self.asks.pop_last();
                self.asks.insert(ask);
            }
        }
    }

    fn get_best_bid(&self) -> Option<&Bid> {
        self.bids.iter().last()
    }
    fn get_best_ask(&self) -> Option<&Ask> {
        self.asks.iter().next()
    }

    fn get_best_n_asks(&self, n: usize) -> Vec<Option<Ask>> {
        todo!()
    }
    fn get_best_n_bids(&self, n: usize) -> Vec<Option<Bid>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use crate::{
        exchanges::Exchange,
        order_book::{
            btree_set::tests,
            price_level::{ask::Ask, bid::Bid},
            Order, OrderBook,
        },
    };

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

        order_book.update_bids(bid_0, 10);
        order_book.update_bids(bid_1, 10);
        order_book.update_bids(bid_2, 10);
        order_book.update_bids(bid_3, 10);
        order_book.update_bids(bid_4, 10);
        order_book.update_bids(bid_5, 10);
        order_book.update_bids(bid_6.clone(), 10);

        // collect the actual bids from the BTreeSet into a vector
        let actual_bids: Vec<Bid> = order_book.bids.iter().cloned().collect();

        let best_bid = order_book.get_best_bid();
        assert!(*best_bid.expect("Could not get best bid") == bid_6);

        assert_eq!(actual_bids, expected_bids);
    }

    #[test]
    fn test_remove_bid() {
        let mut order_book = BTreeSetOrderBook::new();

        let bid_0 = Bid::new(100.00, 50.0, Exchange::Binance);
        let mut bid_1 = Bid::new(100.50, 50.0, Exchange::Bitstamp);
        let bid_2 = Bid::new(101.00, 50.0, Exchange::Binance);
        let bid_3 = Bid::new(101.00, 50.0, Exchange::Bitstamp);
        let mut bid_4 = Bid::new(103.00, 50.0, Exchange::Binance);
        let bid_5 = Bid::new(103.50, 50.0, Exchange::Binance);
        let mut bid_6 = Bid::new(104.00, 50.0, Exchange::Binance);

        // create an expected bids vector
        let mut expected_bids = vec![bid_0.clone(), bid_2.clone(), bid_3.clone(), bid_5.clone()];

        // sort the vector because BTreeSet is ordered
        expected_bids.sort();

        order_book.update_bids(bid_0, 10);
        order_book.update_bids(bid_1.clone(), 10);
        order_book.update_bids(bid_2, 10);
        order_book.update_bids(bid_3, 10);
        order_book.update_bids(bid_4.clone(), 10);
        order_book.update_bids(bid_5.clone(), 10);
        order_book.update_bids(bid_6.clone(), 10);

        bid_1.set_quantity(OrderedFloat(0.0));
        bid_4.set_quantity(OrderedFloat(0.0));
        bid_6.set_quantity(OrderedFloat(0.0));

        order_book.update_bids(bid_1, 10);
        order_book.update_bids(bid_4, 10);
        order_book.update_bids(bid_6, 10);

        // collect the actual bids from the BTreeSet into a vector
        let actual_bids: Vec<Bid> = order_book.bids.iter().cloned().collect();

        let best_bid = order_book.get_best_bid();
        assert!(*best_bid.expect("Could not get best bid") == bid_5);

        assert_eq!(actual_bids, expected_bids);
    }

    #[test]
    fn test_update_bid() {
        let mut order_book = BTreeSetOrderBook::new();

        let bid_0 = Bid::new(100.00, 50.0, Exchange::Binance);
        let bid_1 = Bid::new(100.00, 50.0, Exchange::Bitstamp);
        let bid_2 = Bid::new(100.50, 400.0, Exchange::Bitstamp);
        let bid_3 = Bid::new(101.00, 499.0, Exchange::Binance);
        let bid_4 = Bid::new(103.00, 50.0, Exchange::Binance);
        let bid_5 = Bid::new(102.00, 50.0, Exchange::Binance);
        let bid_6 = Bid::new(104.00, 50.0, Exchange::Bitstamp);

        let replacement_bid_1 = Bid::new(100.00, 3404.0, Exchange::Bitstamp);
        let replacement_bid_3 = Bid::new(101.00, 250.0, Exchange::Binance);
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

        order_book.update_bids(bid_0, 10);
        order_book.update_bids(bid_1, 10);
        order_book.update_bids(bid_2, 10);
        order_book.update_bids(bid_3, 10);
        order_book.update_bids(bid_4, 10);
        order_book.update_bids(bid_5, 10);
        order_book.update_bids(bid_6, 10);

        //insert the replacement bids
        order_book.update_bids(replacement_bid_6.clone(), 10);
        order_book.update_bids(replacement_bid_3, 10);
        order_book.update_bids(replacement_bid_1, 10);

        // collect the actual bids from the BTreeSet into a vector
        let actual_bids: Vec<Bid> = order_book.bids.iter().cloned().collect();

        let best_bid = order_book.get_best_bid();
        assert!(*best_bid.expect("Could not get best bid") == replacement_bid_6);

        assert_eq!(actual_bids, expected_bids);
    }

    #[test]
    fn test_insert_ask() {
        let mut order_book = BTreeSetOrderBook::new();

        let ask_0 = Ask::new(100.00, 50.0, Exchange::Binance);
        let ask_1 = Ask::new(100.00, 1000.0, Exchange::Bitstamp);
        let ask_2 = Ask::new(101.00, 50.0, Exchange::Binance);
        let ask_3 = Ask::new(101.00, 50.0, Exchange::Bitstamp);
        let ask_4 = Ask::new(103.00, 50.0, Exchange::Binance);
        let ask_5 = Ask::new(102.00, 50.0, Exchange::Binance);
        let ask_6 = Ask::new(104.00, 50.0, Exchange::Binance);

        // create an expected bids vector
        let mut expected_asks = vec![
            ask_0.clone(),
            ask_1.clone(),
            ask_2.clone(),
            ask_3.clone(),
            ask_4.clone(),
            ask_5.clone(),
            ask_6.clone(),
        ];
        // sort the vector because BTreeSet is ordered
        expected_asks.sort();

        order_book.update_asks(ask_0, 10);
        order_book.update_asks(ask_1.clone(), 10);
        order_book.update_asks(ask_2, 10);
        order_book.update_asks(ask_3, 10);
        order_book.update_asks(ask_4, 10);
        order_book.update_asks(ask_5, 10);
        order_book.update_asks(ask_6, 10);

        // collect the actual bids from the BTreeSet into a vector
        let actual_asks: Vec<Ask> = order_book.asks.iter().cloned().collect();

        let best_ask = order_book.get_best_ask();
        assert!(*best_ask.expect("Could not get best ask") == ask_1);

        assert_eq!(actual_asks, expected_asks);
    }

    #[test]
    fn test_remove_ask() {
        let mut order_book = BTreeSetOrderBook::new();

        let ask_0 = Ask::new(100.00, 50.0, Exchange::Binance);
        let mut ask_1 = Ask::new(100.00, 1000.0, Exchange::Bitstamp);
        let ask_2 = Ask::new(101.00, 50.0, Exchange::Binance);
        let ask_3 = Ask::new(101.00, 50.0, Exchange::Bitstamp);
        let mut ask_4 = Ask::new(103.00, 50.0, Exchange::Binance);
        let ask_5 = Ask::new(102.00, 50.0, Exchange::Binance);
        let mut ask_6 = Ask::new(104.00, 50.0, Exchange::Binance);

        // create an expected bids vector
        let mut expected_asks = vec![ask_0.clone(), ask_2.clone(), ask_3.clone(), ask_5.clone()];

        // sort the vector because BTreeSet is ordered
        expected_asks.sort();

        order_book.update_asks(ask_0.clone(), 10);
        order_book.update_asks(ask_1.clone(), 10);
        order_book.update_asks(ask_2, 10);
        order_book.update_asks(ask_3, 10);
        order_book.update_asks(ask_4.clone(), 10);
        order_book.update_asks(ask_5.clone(), 10);
        order_book.update_asks(ask_6.clone(), 10);

        ask_1.set_quantity(OrderedFloat(0.0));
        ask_4.set_quantity(OrderedFloat(0.0));
        ask_6.set_quantity(OrderedFloat(0.0));

        order_book.update_asks(ask_1, 10);
        order_book.update_asks(ask_4, 10);
        order_book.update_asks(ask_6, 10);

        // collect the actual bids from the BTreeSet into a vector
        let actual_asks: Vec<Ask> = order_book.asks.iter().cloned().collect();

        let best_ask = order_book.get_best_ask();
        assert!(*best_ask.expect("Could not get best ask") == ask_0);

        assert_eq!(actual_asks, expected_asks);
    }

    #[test]
    fn test_update_ask() {
        let mut order_book = BTreeSetOrderBook::new();
        let ask_0 = Ask::new(100.00, 50.0, Exchange::Binance);
        let ask_1 = Ask::new(100.00, 1000.0, Exchange::Bitstamp);
        let ask_2 = Ask::new(101.00, 50.0, Exchange::Binance);
        let ask_3 = Ask::new(101.00, 50.0, Exchange::Bitstamp);
        let ask_4 = Ask::new(103.00, 50.0, Exchange::Binance);
        let ask_5 = Ask::new(102.00, 50.0, Exchange::Binance);
        let ask_6 = Ask::new(104.00, 50.0, Exchange::Binance);

        let replacement_ask_1 = Ask::new(100.00, 3404.0, Exchange::Bitstamp);
        let replacement_ask_3 = Ask::new(101.00, 250.0, Exchange::Bitstamp);
        let replacement_ask_6 = Ask::new(104.00, 20.0, Exchange::Binance);

        // create an expected bids vector
        let mut expected_asks = vec![
            ask_0.clone(),
            replacement_ask_1.clone(),
            ask_2.clone(),
            replacement_ask_3.clone(),
            ask_4.clone(),
            ask_5.clone(),
            replacement_ask_6.clone(),
        ];

        // sort the vector because BTreeSet is ordered
        expected_asks.sort();

        order_book.update_asks(ask_0, 10);
        order_book.update_asks(ask_1, 10);
        order_book.update_asks(ask_2, 10);
        order_book.update_asks(ask_3, 10);
        order_book.update_asks(ask_4, 10);
        order_book.update_asks(ask_5, 10);
        order_book.update_asks(ask_6, 10);

        //insert the replacement bids
        order_book.update_asks(replacement_ask_6, 10);
        order_book.update_asks(replacement_ask_3, 10);
        order_book.update_asks(replacement_ask_1.clone(), 10);

        // collect the actual bids from the BTreeSet into a vector
        let actual_asks: Vec<Ask> = order_book.asks.iter().cloned().collect();

        let best_ask = order_book.get_best_ask();

        dbg!(best_ask.clone());
        assert!(*best_ask.expect("Could not get best ask") == replacement_ask_1);

        assert_eq!(actual_asks, expected_asks);
    }

    //TODO: add tests for find best n asks

    // TODO: add test for find best n bids

    //TODO: add test for inserting past max depth
}

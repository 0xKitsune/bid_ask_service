use std::collections::{BTreeSet};





use super::{
    price_level::{ask::Ask, bid::Bid},
    BuySide, Order, SellSide,
};

impl BuySide for BTreeSet<Bid> {
    fn update_bids(&mut self, bid: Bid, max_depth: usize) {
        if bid.get_quantity().0 == 0.0 {
            self.remove(&bid);
        } else if self.len() < max_depth {
            if self.contains(&bid) {
                //We have to remove and insert because the replace method replaces the value at the pointer.
                //Since the two are seen as equal, it does not reorder the tree
                self.remove(&bid);
                self.insert(bid);
            } else {
                self.insert(bid);
            }
        } else {
            // check if the bid is better than the worst bid
            let bid_is_better = {
                //We can unwrap this because we have already asserted that the bids.len() is not less than the max depth
                //signifying that there is at least one value
                let worst_bid = self.iter().next().unwrap();
                bid > *worst_bid
            };

            if bid_is_better {
                self.pop_first();
                self.insert(bid);
            }
        }
    }

    fn get_best_bid(&self) -> Option<&Bid> {
        self.iter().last()
    }

    fn get_best_n_bids(&self, n: usize) -> Vec<Option<Bid>> {
        let mut best_bids = Vec::new();

        for bid in self.iter().rev().take(n) {
            best_bids.push(Some(bid.clone()));
        }

        while best_bids.len() < n {
            best_bids.push(None);
        }

        best_bids
    }
}

impl SellSide for BTreeSet<Ask> {
    fn update_asks(&mut self, ask: Ask, max_depth: usize) {
        if ask.get_quantity().0 == 0.0 {
            self.remove(&ask);
        } else if self.len() < max_depth {
            if self.contains(&ask) {
                //We have to remove and insert because the replace method replaces the value at the pointer.
                //Since the two are seen as equal, it does not reorder the tree
                self.remove(&ask);
                self.insert(ask);
            } else {
                self.insert(ask);
            }
        } else {
            // check if the bid is better than the worst bid
            let ask_is_better = {
                //We can unwrap this because we have already asserted that the bids.len() is not less than the max depth
                //signifying that there is at least one value
                let worst_ask = self.iter().next_back().unwrap();
                ask < *worst_ask
            };

            if ask_is_better {
                self.pop_last();
                self.insert(ask);
            }
        }
    }

    fn get_best_ask(&self) -> Option<&Ask> {
        self.iter().next()
    }

    fn get_best_n_asks(&self, n: usize) -> Vec<Option<Ask>> {
        let mut best_asks = Vec::new();

        for ask in self.iter().take(n) {
            best_asks.push(Some(ask.clone()));
        }

        while best_asks.len() < n {
            best_asks.push(None);
        }

        best_asks
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use ordered_float::OrderedFloat;

    use crate::{
        exchanges::Exchange,
        order_book::{
            price_level::{ask::Ask, bid::Bid},
            BuySide, Order, OrderBook, SellSide,
        },
    };

    #[test]
    fn test_insert_bid() {
        let mut order_book = BTreeSet::<Bid>::new();

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
        let actual_bids: Vec<Bid> = order_book.iter().cloned().collect();

        let best_bid = order_book.get_best_bid();
        assert!(*best_bid.expect("Could not get best bid") == bid_6);

        assert_eq!(actual_bids, expected_bids);
    }

    #[test]
    fn test_insert_bid_past_max_depth() {
        let mut order_book = BTreeSet::<Bid>::new();

        let bid_0 = Bid::new(100.00, 50.0, Exchange::Binance);
        let bid_1 = Bid::new(100.00, 50.0, Exchange::Bitstamp);
        let bid_2 = Bid::new(101.00, 50.0, Exchange::Binance);
        let bid_3 = Bid::new(101.00, 50.0, Exchange::Bitstamp);
        let bid_4 = Bid::new(103.00, 50.0, Exchange::Binance);
        let bid_5 = Bid::new(102.00, 50.0, Exchange::Binance);
        let bid_6 = Bid::new(104.00, 50.0, Exchange::Binance);

        // create an expected bids vector
        let mut expected_bids = vec![
            bid_2.clone(),
            bid_3.clone(),
            bid_4.clone(),
            bid_5.clone(),
            bid_6.clone(),
        ];
        // sort the vector because BTreeSet is ordered
        expected_bids.sort();

        order_book.update_bids(bid_0, 5);
        order_book.update_bids(bid_1, 5);
        order_book.update_bids(bid_2, 5);
        order_book.update_bids(bid_3, 5);
        order_book.update_bids(bid_4, 5);
        order_book.update_bids(bid_5, 5);
        order_book.update_bids(bid_6.clone(), 5);

        // collect the actual bids from the BTreeSet into a vector
        let actual_bids: Vec<Bid> = order_book.iter().cloned().collect();

        let best_bid = order_book.get_best_bid();
        assert!(*best_bid.expect("Could not get best bid") == bid_6);
        assert!(order_book.len() == 5);
        assert_eq!(actual_bids, expected_bids);
    }

    #[test]
    fn test_remove_bid() {
        let mut order_book = BTreeSet::<Bid>::new();

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
        let actual_bids: Vec<Bid> = order_book.iter().cloned().collect();

        let best_bid = order_book.get_best_bid();
        assert!(*best_bid.expect("Could not get best bid") == bid_5);

        assert_eq!(actual_bids, expected_bids);
    }

    #[test]
    fn test_update_bid() {
        let mut order_book = BTreeSet::<Bid>::new();

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
        let actual_bids: Vec<Bid> = order_book.iter().cloned().collect();

        let best_bid = order_book.get_best_bid();
        assert!(*best_bid.expect("Could not get best bid") == replacement_bid_6);

        assert_eq!(actual_bids, expected_bids);
    }

    #[test]
    fn test_get_best_n_bids() {
        let mut order_book = BTreeSet::<Bid>::new();
        let bid_0 = Bid::new(100.00, 50.0, Exchange::Binance);
        let bid_1 = Bid::new(100.00, 1000.0, Exchange::Bitstamp);
        let bid_2 = Bid::new(101.00, 50.0, Exchange::Binance);
        let bid_3 = Bid::new(101.00, 50.0, Exchange::Bitstamp);
        let bid_4 = Bid::new(103.00, 50.0, Exchange::Binance);
        let bid_5 = Bid::new(102.00, 50.0, Exchange::Binance);
        let bid_6 = Bid::new(104.00, 50.0, Exchange::Binance);

        let replacement_bid_1 = Bid::new(100.00, 3404.0, Exchange::Bitstamp);
        let replacement_bid_3 = Bid::new(101.00, 250.0, Exchange::Bitstamp);
        let replacement_bid_6 = Bid::new(104.00, 20.0, Exchange::Binance);

        // create an expected bids vector
        let expected_bids = vec![
            Some(replacement_bid_6.clone()),
            Some(bid_4.clone()),
            Some(bid_5.clone()),
        ];

        order_book.update_bids(bid_4, 5);
        order_book.update_bids(bid_5, 5);
        order_book.update_bids(bid_6, 5);
        order_book.update_bids(bid_0, 5);
        order_book.update_bids(bid_1, 5);
        order_book.update_bids(bid_2, 5);
        order_book.update_bids(bid_3, 5);

        //insert the replacement bids
        order_book.update_bids(replacement_bid_6, 10);
        order_book.update_bids(replacement_bid_3, 10);
        order_book.update_bids(replacement_bid_1, 10);

        let best_bids = order_book.get_best_n_bids(3);

        assert_eq!(expected_bids, best_bids);

        let empty_order_book = BTreeSet::<Bid>::new();

        let best_bids = empty_order_book.get_best_n_bids(10);
        let expected_bids = vec![None; 10];

        assert_eq!(best_bids, expected_bids);
    }

    #[test]
    fn test_insert_ask() {
        let mut order_book = BTreeSet::<Ask>::new();

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
        let actual_asks: Vec<Ask> = order_book.iter().cloned().collect();

        let best_ask = order_book.get_best_ask();
        assert!(*best_ask.expect("Could not get best ask") == ask_1);

        assert_eq!(actual_asks, expected_asks);
    }

    #[test]
    fn test_insert_ask_past_max_depth() {
        let mut order_book = BTreeSet::<Ask>::new();

        let ask_0 = Ask::new(100.00, 50.0, Exchange::Binance);
        let ask_1 = Ask::new(100.00, 1000.0, Exchange::Bitstamp);
        let ask_2 = Ask::new(101.00, 50.0, Exchange::Binance);
        let ask_3 = Ask::new(101.00, 50.0, Exchange::Bitstamp);
        let ask_4 = Ask::new(102.00, 50.0, Exchange::Binance);
        let ask_5 = Ask::new(103.00, 50.0, Exchange::Binance);
        let ask_6 = Ask::new(104.00, 50.0, Exchange::Binance);

        // create an expected bids vector
        let mut expected_asks = vec![
            ask_0.clone(),
            ask_1.clone(),
            ask_2.clone(),
            ask_3.clone(),
            ask_4.clone(),
        ];
        // sort the vector because BTreeSet is ordered
        expected_asks.sort();
        order_book.update_asks(ask_6, 5);
        order_book.update_asks(ask_5, 5);
        order_book.update_asks(ask_2, 5);
        order_book.update_asks(ask_3, 5);
        order_book.update_asks(ask_4, 5);
        order_book.update_asks(ask_0, 5);
        order_book.update_asks(ask_1.clone(), 5);

        // collect the actual bids from the BTreeSet into a vector
        let actual_asks: Vec<Ask> = order_book.iter().cloned().collect();

        let best_ask = order_book.get_best_ask();
        assert!(*best_ask.expect("Could not get best ask") == ask_1);
        assert!(order_book.len() == 5);
        assert_eq!(actual_asks, expected_asks);
    }

    #[test]
    fn test_remove_ask() {
        let mut order_book = BTreeSet::<Ask>::new();

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
        order_book.update_asks(ask_5, 10);
        order_book.update_asks(ask_6.clone(), 10);

        ask_1.set_quantity(OrderedFloat(0.0));
        ask_4.set_quantity(OrderedFloat(0.0));
        ask_6.set_quantity(OrderedFloat(0.0));

        order_book.update_asks(ask_1, 10);
        order_book.update_asks(ask_4, 10);
        order_book.update_asks(ask_6, 10);

        // collect the actual bids from the BTreeSet into a vector
        let actual_asks: Vec<Ask> = order_book.iter().cloned().collect();

        let best_ask = order_book.get_best_ask();
        assert!(*best_ask.expect("Could not get best ask") == ask_0);

        assert_eq!(actual_asks, expected_asks);
    }

    #[test]
    fn test_update_ask() {
        let mut order_book = BTreeSet::<Ask>::new();
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
        let actual_asks: Vec<Ask> = order_book.iter().cloned().collect();

        let best_ask = order_book.get_best_ask();

        dbg!(best_ask);
        assert!(*best_ask.expect("Could not get best ask") == replacement_ask_1);

        assert_eq!(actual_asks, expected_asks);
    }

    #[test]
    fn test_get_best_n_asks() {
        let mut order_book = BTreeSet::<Ask>::new();
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
            Some(ask_0.clone()),
            Some(replacement_ask_1.clone()),
            Some(replacement_ask_3.clone()),
        ];

        // sort the vector because BTreeSet is ordered
        expected_asks.sort();

        order_book.update_asks(ask_4, 5);
        order_book.update_asks(ask_5, 5);
        order_book.update_asks(ask_6, 5);
        order_book.update_asks(ask_0, 5);
        order_book.update_asks(ask_1, 5);
        order_book.update_asks(ask_2, 5);
        order_book.update_asks(ask_3, 5);

        //insert the replacement bids
        order_book.update_asks(replacement_ask_6, 10);
        order_book.update_asks(replacement_ask_3, 10);
        order_book.update_asks(replacement_ask_1, 10);

        let best_asks = order_book.get_best_n_asks(3);

        assert_eq!(expected_asks, best_asks);

        let empty_order_book = BTreeSet::<Ask>::new();

        let best_asks = empty_order_book.get_best_n_asks(10);
        let expected_asks = vec![None; 10];

        assert_eq!(best_asks, expected_asks);
    }
}

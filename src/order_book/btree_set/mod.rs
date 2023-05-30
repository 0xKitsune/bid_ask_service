use std::collections::{BTreeMap, BTreeSet};

use ordered_float::OrderedFloat;

use crate::exchanges::Exchange;

use super::{
    error::OrderBookError,
    price_level::{ask::Ask, bid::Bid},
    Order, OrderBook,
};

pub struct BTreeSetOrderBook {
    bids: BTreeSet<Bid>,
    asks: BTreeSet<Ask>,
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

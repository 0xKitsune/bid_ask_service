use super::{error::OrderBookError, OrderBook, PriceLevelUpdate};

pub struct RBTreeOrderBook;

impl RBTreeOrderBook {
    pub fn new() -> Self {
        todo!()
    }
}

// impl OrderBook for RBTreeOrderBook {
//     type Bids = RBTree;
//     type Asks = RBTree;

//     fn update_bids(&self, price_level_update: PriceLevelUpdate) -> Result<(), OrderBookError> {
//         todo!()
//     }

//     fn update_asks(&self, price_level_update: PriceLevelUpdate) -> Result<(), OrderBookError> {
//         todo!()
//     }
// }

// pub struct RBTree;

// impl RBTree {
//     pub fn new() -> Self {
//         todo!()
//     }
// }

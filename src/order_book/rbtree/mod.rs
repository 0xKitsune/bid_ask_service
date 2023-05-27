use super::{error::OrderBookError, OrderBook, PriceLevelUpdate};

pub struct RBTreeOrderBook;

impl RBTreeOrderBook {
    pub fn new() -> Self {
        todo!()
    }
}

impl OrderBook for RBTreeOrderBook {
    fn update_book(&self, price_level_update: PriceLevelUpdate) -> Result<(), OrderBookError> {
        todo!()
    }
}

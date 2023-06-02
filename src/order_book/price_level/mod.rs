pub mod ask;
pub mod bid;

use self::{ask::Ask, bid::Bid};

#[derive(Debug, Clone)]
pub enum OrderType {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]

// Data type to be sent from an exchange's stream handler, to the aggregated order book
pub struct PriceLevelUpdate {
    pub bids: Vec<Bid>,
    pub asks: Vec<Ask>,
}

impl PriceLevelUpdate {
    pub fn new(bids: Vec<Bid>, asks: Vec<Ask>) -> Self {
        PriceLevelUpdate { bids, asks }
    }
}

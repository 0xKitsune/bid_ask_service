pub mod binance;
pub mod error;

pub mod bitstamp;
pub mod exchange_utils;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::error::BidAskServiceError;
use crate::order_book::error::OrderBookError;
use crate::order_book::price_level::PriceLevelUpdate;

use self::binance::Binance;
use self::bitstamp::Bitstamp;

const BINANCE: &str = "binance";
const BITSTAMP: &str = "bitstamp";

#[async_trait]
pub trait OrderBookService {
    fn spawn_order_book_service(
        pair: [&str; 2],
        order_book_depth: usize,
        order_book_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Vec<JoinHandle<Result<(), BidAskServiceError>>>;
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
//TODO: add a note in the walkthrough that the top 10 bids are ordered by exchange preference here
pub enum Exchange {
    Bitstamp,
    Binance,
}

impl Exchange {
    pub fn spawn_order_book_service(
        &self,
        pair: [&str; 2],
        order_book_depth: usize,
        order_book_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Vec<JoinHandle<Result<(), BidAskServiceError>>> {
        match self {
            Exchange::Binance => Binance::spawn_order_book_service(
                pair,
                order_book_depth,
                order_book_stream_buffer,
                price_level_tx,
            ),
            Exchange::Bitstamp => Bitstamp::spawn_order_book_service(
                pair,
                order_book_depth,
                order_book_stream_buffer,
                price_level_tx,
            ),
        }
    }
}

impl ToString for Exchange {
    fn to_string(&self) -> String {
        match self {
            Exchange::Bitstamp => "bitstamp".to_string(),
            Exchange::Binance => "binance".to_string(),
        }
    }
}

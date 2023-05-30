pub mod binance;

pub mod bitstamp;
pub mod exchange_utils;

use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::order_book::error::OrderBookError;
use crate::order_book::price_level::PriceLevelUpdate;

use self::binance::Binance;

const BINANCE: &str = "binance";
const BITSTAMP: &str = "bitstamp";

#[async_trait]
pub trait OrderBookService {
    async fn spawn_order_book_service(
        pair: [&str; 2],
        order_book_depth: usize,
        order_book_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<Vec<JoinHandle<Result<(), OrderBookError>>>, OrderBookError>;
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Exchange {
    Bitstamp,
    Binance,
}

impl Exchange {
    pub async fn spawn_order_book_service(
        &self,
        pair: [&str; 2],
        order_book_depth: usize,
        order_book_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<Vec<JoinHandle<Result<(), OrderBookError>>>, OrderBookError> {
        match self {
            Exchange::Binance => Ok(Binance::spawn_order_book_service(
                pair,
                order_book_depth,
                order_book_stream_buffer,
                price_level_tx,
            )
            .await?),

            Exchange::Bitstamp => {
                todo!()
            }
        }
    }
}

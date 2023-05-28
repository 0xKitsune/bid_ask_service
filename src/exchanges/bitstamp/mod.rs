pub mod error;
pub mod stream;

use async_trait::async_trait;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::order_book::{error::OrderBookError, PriceLevelUpdate};

use super::OrderBookService;

pub struct Bitstamp;

impl Bitstamp {
    pub fn new() -> Self {
        Bitstamp {}
    }
}

#[async_trait]
impl OrderBookService for Bitstamp {
    async fn spawn_order_book_service(
        pair: [&str; 2],
        order_book_depth: usize,
        order_book_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<Vec<JoinHandle<Result<(), OrderBookError>>>, OrderBookError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU32, AtomicU8, Ordering},
        Arc,
    };

    use crate::{
        exchanges::{binance::Binance, OrderBookService},
        order_book::{error::OrderBookError, PriceLevel, PriceLevelUpdate},
    };
    use futures::FutureExt;

    #[tokio::test]

    //Test the Binance WS connection for 1000 price level updates
    async fn test_spawn_order_book_service() {}
}

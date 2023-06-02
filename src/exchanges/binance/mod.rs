pub mod error;
mod stream;

use crate::order_book::price_level::PriceLevelUpdate;
use crate::{error::BidAskServiceError, order_book::error::OrderBookError};

use async_trait::async_trait;

use tokio::{sync::mpsc::Sender, task::JoinHandle};

use self::stream::{spawn_order_book_stream, spawn_stream_handler};

use super::OrderBookService;

pub struct Binance;

impl Binance {
    pub fn new() -> Self {
        Binance {}
    }
}

#[async_trait]
impl OrderBookService for Binance {
    fn spawn_order_book_service(
        pair: [&str; 2],
        order_book_depth: usize,
        order_book_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Vec<JoinHandle<Result<(), BidAskServiceError>>> {
        let pair = pair.join("");
        //When subscribing to a stream of order book updates, the pair is required to be formatted as a single string with all lowercase letters
        let stream_pair = pair.to_lowercase();
        //When getting a snapshot, Binance requires that the pair si a single string with all uppercase letters
        let snapshot_pair = pair.to_uppercase();

        //Spawn a task to handle a buffered stream of the order book and reconnects to the exchange
        let (ws_stream_rx, stream_handle) =
            spawn_order_book_stream(stream_pair, order_book_stream_buffer);

        //Spawn a task to handle updates from the buffered stream, cleaning the data and sending it to the aggregated order book
        let order_book_update_handle = spawn_stream_handler(
            snapshot_pair,
            order_book_depth,
            ws_stream_rx,
            price_level_tx,
        );

        vec![stream_handle, order_book_update_handle]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    use crate::{
        error::BidAskServiceError,
        exchanges::{binance::Binance, OrderBookService},
        order_book::{error::OrderBookError, price_level::PriceLevelUpdate},
    };
    use futures::FutureExt;

    #[tokio::test]

    //Test the Binance WS connection for 1000 price level updates
    async fn test_spawn_order_book_service() {
        let atomic_counter_0 = Arc::new(AtomicU32::new(0));
        let atomic_counter_1 = atomic_counter_0.clone();
        let target_counter = 2100;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<PriceLevelUpdate>(500);
        let mut join_handles = Binance::spawn_order_book_service(["eth", "btc"], 1000, 500, tx);

        let price_level_update_handle = tokio::spawn(async move {
            while let Some(_) = rx.recv().await {
                dbg!(atomic_counter_0.load(Ordering::Relaxed));
                atomic_counter_0.fetch_add(1, Ordering::Relaxed);
                if atomic_counter_0.load(Ordering::Relaxed) >= target_counter {
                    break;
                }
            }

            Ok::<(), BidAskServiceError>(())
        });

        join_handles.push(price_level_update_handle);

        let futures = join_handles
            .into_iter()
            .map(|handle| handle.boxed())
            .collect::<Vec<_>>();

        //Wait for the first future to be finished
        let (result, _, _) = futures::future::select_all(futures).await;
        if atomic_counter_1.load(Ordering::Relaxed) != target_counter {
            result
                .expect("Join handle error")
                .expect("Error when handling WS connection");
        }
    }
}

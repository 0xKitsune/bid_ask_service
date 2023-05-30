use core::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub mod error;
mod stream;

use crate::exchanges::Exchange;
use crate::order_book::{self, PriceLevelUpdate};
use crate::{
    order_book::error::OrderBookError,
    order_book::{OrderBook, PriceLevel},
};

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserializer,
};
use serde_derive::Deserialize;

use tokio::{
    net::TcpStream,
    sync::mpsc::{error::SendError, Receiver, Sender},
    task::JoinHandle,
};

use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::protocol::frame::Frame;
use tungstenite::{protocol::WebSocketConfig, Message};

use self::error::BinanceError;
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
    async fn spawn_order_book_service(
        pair: [&str; 2],
        order_book_depth: usize,
        order_book_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Result<Vec<JoinHandle<Result<(), OrderBookError>>>, OrderBookError> {
        let pair = pair.join("");
        //TODO: add comment to explain why we do this
        let stream_pair = pair.to_lowercase();
        let snapshot_pair = pair.to_uppercase();

        let (ws_stream_rx, stream_handle) =
            spawn_order_book_stream(stream_pair, order_book_stream_buffer).await?;

        let order_book_update_handle = spawn_stream_handler(
            snapshot_pair,
            order_book_depth,
            ws_stream_rx,
            price_level_tx,
        )
        .await?;

        Ok(vec![stream_handle, order_book_update_handle])
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
    async fn test_spawn_order_book_service() {
        let atomic_counter_0 = Arc::new(AtomicU32::new(0));
        let atomic_counter_1 = atomic_counter_0.clone();
        let target_counter = 2100;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<PriceLevelUpdate>(500);
        let mut join_handles = Binance::spawn_order_book_service(["eth", "btc"], 1000, 500, tx)
            .await
            .expect("TODO: handle this error");

        let price_level_update_handle = tokio::spawn(async move {
            while let Some(_) = rx.recv().await {
                dbg!(atomic_counter_0.load(Ordering::Relaxed));
                atomic_counter_0.fetch_add(1, Ordering::Relaxed);
                if atomic_counter_0.load(Ordering::Relaxed) >= target_counter {
                    break;
                }
            }

            return Ok::<(), OrderBookError>(());
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

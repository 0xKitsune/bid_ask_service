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
        let (mut order_book_rx, stream_handles) =
            Binance::spawn_order_book_stream(pair, order_book_depth, order_book_stream_buffer)
                .await?;

        let mut last_update_id = 0;
        let price_level_update_handle = tokio::spawn(async move {
            while let Some(order_book_update) = order_book_rx.recv().await {
                if order_book_update.final_updated_id <= last_update_id {
                    continue;
                } else {
                    //TODO:
                    // make a note that the first update id will always be zero
                    if order_book_update.first_update_id <= last_update_id + 1
                        && order_book_update.final_updated_id >= last_update_id + 1
                    {
                        for bid in order_book_update.bids.into_iter() {
                            price_level_tx
                                .send(PriceLevelUpdate::Bid(PriceLevel::new(
                                    bid[0],
                                    bid[1],
                                    Exchange::Binance,
                                )))
                                .await?;
                        }

                        for ask in order_book_update.asks.into_iter() {
                            price_level_tx
                                .send(PriceLevelUpdate::Ask(PriceLevel::new(
                                    ask[0],
                                    ask[1],
                                    Exchange::Binance,
                                )))
                                .await?;
                        }
                    } else {
                        return Err(BinanceError::InvalidUpdateId.into());
                    }

                    last_update_id = order_book_update.final_updated_id;
                }
            }

            Ok::<(), OrderBookError>(())
        });

        let mut order_book_service_handles = vec![];
        order_book_service_handles.extend(stream_handles);
        order_book_service_handles.push(price_level_update_handle);

        Ok(order_book_service_handles)
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
    async fn test_ws_stream() {
        let atomic_counter_0 = Arc::new(AtomicU32::new(0));
        let atomic_counter_1 = atomic_counter_0.clone();
        let target_counter = 1000;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<PriceLevelUpdate>(500);
        let mut join_handles = Binance::spawn_order_book_service(["bnb", "btc"], 1000, 500, tx)
            .await
            .expect("handle this error");

        let price_level_update_handle = tokio::spawn(async move {
            while let Some(price_level_update) = rx.recv().await {
                dbg!(price_level_update);

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

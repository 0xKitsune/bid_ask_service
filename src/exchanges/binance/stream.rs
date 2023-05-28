use serde_derive::Deserialize;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

use super::Binance;
use crate::exchanges::binance::error::BinanceError;
use crate::order_book::error::OrderBookError;

use core::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::exchanges::Exchange;
use crate::order_book::{self, PriceLevelUpdate};
use crate::order_book::{OrderBook, PriceLevel};

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserializer,
};

use tokio::{
    net::TcpStream,
    sync::mpsc::{error::SendError, Sender},
};

use crate::exchanges::exchange_utils;

use tungstenite::Message;

const WS_BASE_ENDPOINT: &str = "wss://stream.binance.com:9443/ws/";
const ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT: &str = "https://api.binance.com/api/v3/depth?symbol=";

//TODO: Add a comment for what this is also there are more efficient ways to do this, update this

const GET_ORDER_BOOK_SNAPSHOT: Vec<u8> = vec![];

// Websocket Market Streams

// The base endpoint is: wss://stream.binance.com:9443 or wss://stream.binance.com:443
// Streams can be accessed either in a single raw stream or in a combined stream.
// Users can listen to multiple streams.
// Raw streams are accessed at /ws/<streamName>
// Combined streams are accessed at /stream?streams=<streamName1>/<streamName2>/<streamName3>
// Combined stream events are wrapped as follows: {"stream":"<streamName>","data":<rawPayload>}
// All symbols for streams are lowercase
// A single connection to stream.binance.com is only valid for 24 hours; expect to be disconnected at the 24 hour mark
// The websocket server will send a ping frame every 3 minutes. If the websocket server does not receive a pong frame back from the connection within a 10 minute period, the connection will be disconnected. Unsolicited pong frames are allowed.
// The base endpoint wss://data-stream.binance.com can be subscribed to receive market data messages. Users data stream is NOT available from this URL.

impl Binance {
    pub async fn spawn_order_book_stream(
        pair: [&str; 2],
        order_book_depth: usize,
        order_book_stream_buffer: usize,
    ) -> Result<
        (
            Receiver<OrderBookUpdate>,
            Vec<JoinHandle<Result<(), OrderBookError>>>,
        ),
        OrderBookError,
    > {
        let pair = pair.join("");
        //TODO: add comment to explain why we do this
        let stream_pair = pair.to_lowercase();
        let snapshot_pair = pair.to_uppercase();

        let (ws_stream_tx, mut ws_stream_rx) =
            tokio::sync::mpsc::channel::<Message>(order_book_stream_buffer);

        //spawn a thread that handles the stream and buffers the results
        let stream_handle = tokio::spawn(async move {
            let ws_stream_tx = ws_stream_tx.clone();
            loop {
                //Establish an infinite loop to handle a ws stream with reconnects
                let order_book_endpoint = WS_BASE_ENDPOINT.to_owned() + &stream_pair + "@depth";

                let (mut order_book_stream, _) =
                    tokio_tungstenite::connect_async(order_book_endpoint).await?;
                log::info!("Ws connection established");

                ws_stream_tx
                    .send(Message::Binary(GET_ORDER_BOOK_SNAPSHOT))
                    .await
                    .map_err(BinanceError::MessageSendError)?; //TODO: we prob dont need a binance error for this

                while let Some(Ok(message)) = order_book_stream.next().await {
                    match message {
                        tungstenite::Message::Text(_) => {
                            ws_stream_tx
                                .send(message)
                                .await
                                .map_err(BinanceError::MessageSendError)?;
                        }

                        tungstenite::Message::Ping(_) => {
                            log::info!("Ping received");
                            order_book_stream.send(Message::Pong(vec![])).await.ok();
                            log::info!("Pong sent");
                        }

                        tungstenite::Message::Close(_) => {
                            log::info!("Ws connection closed, reconnecting...");
                            break;
                        }

                        other => {
                            log::warn!("{other:?}");
                        }
                    }
                }
            }
        });

        let (order_book_update_tx, order_book_update_rx) =
            tokio::sync::mpsc::channel::<OrderBookUpdate>(order_book_stream_buffer);

        let order_book_update_handle = tokio::spawn(async move {
            while let Some(message) = ws_stream_rx.recv().await {
                match message {
                    tungstenite::Message::Text(message) => {
                        order_book_update_tx
                            .send(serde_json::from_str(&message)?)
                            .await
                            .map_err(BinanceError::OrderBookUpdateSendError)?;
                    }

                    tungstenite::Message::Binary(message) => {
                        //This is an internal message signaling that we should get the depth snapshot and send it through the channel
                        if message.is_empty() {
                            let snapshot =
                                get_order_book_snapshot(&snapshot_pair, order_book_depth).await?;

                            //TODO: there might be a more efficient way to do this, we are making sure we are not missing any orders using redundant logic with this approach but it is prob a little slow
                            order_book_update_tx
                                .send(OrderBookUpdate {
                                    event_type: OrderBookEventType::DepthUpdate,
                                    event_time: 0,
                                    first_update_id: 0,
                                    final_updated_id: snapshot.last_update_id,
                                    bids: snapshot.bids,
                                    asks: snapshot.asks,
                                })
                                .await
                                .map_err(BinanceError::OrderBookUpdateSendError)?;
                        }
                    }

                    _ => {}
                }
            }

            Ok::<(), OrderBookError>(())
        });

        Ok((
            order_book_update_rx,
            vec![stream_handle, order_book_update_handle],
        ))
    }
}

#[derive(Debug, Deserialize)]
pub struct OrderBookSnapshot {
    #[serde(rename = "lastUpdateId")]
    last_update_id: u64,
    #[serde(deserialize_with = "exchange_utils::convert_array_items_to_f64")]
    bids: Vec<[f64; 2]>,
    #[serde(deserialize_with = "exchange_utils::convert_array_items_to_f64")]
    asks: Vec<[f64; 2]>,
}

#[derive(Deserialize, Debug)]
pub struct OrderBookUpdate {
    #[serde(rename = "e")]
    pub event_type: OrderBookEventType,
    #[serde(rename = "E")]
    pub event_time: usize,
    #[serde(rename = "U")]
    pub first_update_id: u64, //NOTE: not positive what the largest order id from the exchange will possibly grow to, it can probably be covered by u32, but using u64 just to be safe
    #[serde(rename = "u")]
    pub final_updated_id: u64,
    #[serde(
        rename = "b",
        deserialize_with = "exchange_utils::convert_array_items_to_f64"
    )]
    pub bids: Vec<[f64; 2]>,
    #[serde(
        rename = "a",
        deserialize_with = "exchange_utils::convert_array_items_to_f64"
    )]
    pub asks: Vec<[f64; 2]>,
}

impl OrderBookUpdate {
    pub fn new(
        event_type: OrderBookEventType,
        event_time: usize,
        first_update_id: u64,
        final_updated_id: u64,
        bids: Vec<[f64; 2]>,
        asks: Vec<[f64; 2]>,
    ) -> Self {
        OrderBookUpdate {
            event_type,
            event_time,
            first_update_id,
            final_updated_id,
            bids,
            asks,
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum OrderBookEventType {
    #[serde(rename = "depthUpdate")]
    DepthUpdate,
}

async fn get_order_book_snapshot(
    pair: &str,
    order_book_depth: usize,
) -> Result<OrderBookSnapshot, OrderBookError> {
    let snapshot_endpoint = ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT.to_owned()
        + &pair
        + "&limit="
        + order_book_depth.to_string().as_str();

    // Get the depth snapshot
    let snapshot_response = reqwest::get(snapshot_endpoint).await?;

    if snapshot_response.status().is_success() {
        Ok(snapshot_response.json::<OrderBookSnapshot>().await?)
    } else {
        Err(OrderBookError::HTTPError(String::from_utf8(
            snapshot_response.bytes().await?.to_vec(),
        )?))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU32, AtomicU8, Ordering},
        Arc,
    };

    use crate::exchanges::binance::stream::OrderBookUpdate;
    use crate::{
        exchanges::{binance::Binance, OrderBookService},
        order_book::{error::OrderBookError, PriceLevel, PriceLevelUpdate},
    };
    use futures::FutureExt;
    use tokio::sync::mpsc::Receiver;
    //TODO: add test for get snapshot

    #[tokio::test]

    //Test the Binance WS connection for 1000 order book updates
    async fn test_spawn_order_book_stream() {
        let atomic_counter_0 = Arc::new(AtomicU32::new(0));
        let atomic_counter_1 = atomic_counter_0.clone();
        let target_counter = 1000;

        let (mut order_book_update_rx, mut join_handles) =
            Binance::spawn_order_book_stream(["eth", "btc"], 1000, 500)
                .await
                .expect("handle this error");

        let order_book_update_handle = tokio::spawn(async move {
            while let Some(_) = order_book_update_rx.recv().await {
                atomic_counter_0.fetch_add(1, Ordering::Relaxed);
                if atomic_counter_0.load(Ordering::Relaxed) >= target_counter {
                    break;
                }
            }

            return Ok::<(), OrderBookError>(());
        });

        join_handles.push(order_book_update_handle);

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

//TODO: add some tests for error cases

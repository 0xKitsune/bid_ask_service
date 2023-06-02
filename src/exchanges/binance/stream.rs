use serde_derive::Deserialize;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

use crate::order_book::price_level::ask::Ask;
use crate::order_book::price_level::bid::Bid;
use crate::order_book::price_level::PriceLevelUpdate;
use crate::{error::BidAskServiceError, exchanges::binance::error::BinanceError};

use crate::exchanges::Exchange;

use futures::{SinkExt, StreamExt};

use tokio::sync::mpsc::Sender;

use crate::exchanges::exchange_utils;

use tungstenite::Message;

const WS_BASE_ENDPOINT: &str = "wss://stream.binance.com:9443/ws/";
const ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT: &str = "https://api.binance.com/api/v3/depth?symbol=";
const DEPTH_UPDATE_EVENT: &str = "depthUpdate";
const GET_ORDER_BOOK_SNAPSHOT: Vec<u8> = vec![];

// Websocket Market Streams

// The base endpoint is: wss://stream.binance.com:9443 or wss://stream.binance.com:443
// All symbols for streams are lowercase
// A single connection to stream.binance.com is only valid for 24 hours; expect to be disconnected at the 24 hour mark
// The websocket server will send a ping frame every 3 minutes. If the websocket server does not receive a pong frame back from the connection within a 10 minute period, the connection will be disconnected. Unsolicited pong frames are allowed.
// The base endpoint wss://data-stream.binance.com can be subscribed to receive market data messages. Users data stream is not available from this URL.

//Spawns a thread to stream order book updates from Binance
pub fn spawn_order_book_stream(
    pair: String,
    exchange_stream_buffer: usize,
) -> (
    Receiver<Message>,
    JoinHandle<Result<(), BidAskServiceError>>,
) {
    let (ws_stream_tx, ws_stream_rx) =
        tokio::sync::mpsc::channel::<Message>(exchange_stream_buffer);

    //spawn a thread that handles the stream and buffers the results
    let stream_handle = tokio::spawn(async move {
        let ws_stream_tx = ws_stream_tx.clone();
        loop {
            //Establish an infinite loop to handle a ws stream with reconnects
            let order_book_endpoint = WS_BASE_ENDPOINT.to_owned() + &pair + "@depth"; //TODO: see if we can specify the depth to listen to

            // Connect to the order book stream endpoint and start the stream
            let (mut order_book_stream, _) = tokio_tungstenite::connect_async(order_book_endpoint)
                .await
                .map_err(BinanceError::TungsteniteError)?;
            tracing::info!("Ws connection established");

            //Notify the stream handler to get a snapshot of the order book
            //This will be the first message that the stream handler receives, so a
            //snapshot of the orderbook will be retrieved before any order book updates are handled
            ws_stream_tx
                .send(Message::Binary(GET_ORDER_BOOK_SNAPSHOT))
                .await
                .map_err(BinanceError::MessageSendError)?;

            //Send messages through a channel to be handled by the stream handler, respond to ping requests and handle reconnects
            while let Some(Ok(message)) = order_book_stream.next().await {
                match message {
                    tungstenite::Message::Text(_) => {
                        ws_stream_tx
                            .send(message)
                            .await
                            .map_err(BinanceError::MessageSendError)?;
                    }

                    tungstenite::Message::Ping(_) => {
                        tracing::info!("Ping received");
                        order_book_stream.send(Message::Pong(vec![])).await.ok();
                        tracing::info!("Pong sent");
                    }

                    tungstenite::Message::Close(_) => {
                        tracing::warn!("Ws connection closed, reconnecting...");
                        break;
                    }

                    other => {
                        tracing::warn!("{other:?}");
                    }
                }
            }
        }
    });

    (ws_stream_rx, stream_handle)
}

//Spawns a thread to handle order book updates from Binance
pub fn spawn_stream_handler(
    pair: String,
    order_book_depth: usize,
    mut ws_stream_rx: Receiver<Message>,
    price_level_tx: Sender<PriceLevelUpdate>,
) -> JoinHandle<Result<(), BidAskServiceError>> {
    tokio::spawn(async move {
        let mut last_update_id = 0;

        while let Some(message) = ws_stream_rx.recv().await {
            match message {
                //Deserialize the event, verify the order Id is valid and and send it through to the aggregated order book
                tungstenite::Message::Text(message) => {
                    let order_book_event = serde_json::from_str::<OrderBookEvent>(&message)
                        .map_err(BinanceError::SerdeJsonError)?;

                    if order_book_event.event == DEPTH_UPDATE_EVENT {
                        let order_book_update = serde_json::from_str::<OrderBookUpdate>(&message)
                            .map_err(BinanceError::SerdeJsonError)?;

                        if order_book_update.final_updated_id <= last_update_id {
                            tracing::warn!("Update id is <= last update id");
                            continue;
                        } else {
                            if order_book_update.first_update_id <= last_update_id + 1
                                && order_book_update.final_updated_id > last_update_id
                            {
                                //Collect bids and asks, sending the batch of price level updates through a channel to the aggregated order book
                                let mut bids = vec![];
                                for bid in order_book_update.bids.into_iter() {
                                    bids.push(Bid::new(bid[0], bid[1], Exchange::Binance));
                                }

                                let mut asks = vec![];
                                for ask in order_book_update.asks.into_iter() {
                                    asks.push(Ask::new(ask[0], ask[1], Exchange::Binance));
                                }

                                price_level_tx
                                    .send(PriceLevelUpdate::new(bids, asks))
                                    .await
                                    .map_err(BinanceError::PriceLevelUpdateSendError)?;
                            } else {
                                return Err(BinanceError::InvalidUpdateId.into());
                            }

                            last_update_id = order_book_update.final_updated_id;
                        }
                    }
                }

                tungstenite::Message::Binary(message) => {
                    // This is an internal message signifying that the stream has reconnected so we need to get a snapshot
                    // First get a snapshot of the order book, handle all of the bids/asks and send it through the channel to the aggregated orderbook
                    if message.is_empty() {
                        tracing::info!("Getting order book snapshot");
                        let snapshot = get_order_book_snapshot(&pair, order_book_depth).await?;

                        let mut bids = vec![];
                        for bid in snapshot.bids.into_iter() {
                            bids.push(Bid::new(bid[0], bid[1], Exchange::Binance));
                        }

                        let mut asks = vec![];
                        for ask in snapshot.asks.into_iter() {
                            asks.push(Ask::new(ask[0], ask[1], Exchange::Binance));
                        }

                        price_level_tx
                            .send(PriceLevelUpdate::new(bids, asks))
                            .await
                            .map_err(BinanceError::PriceLevelUpdateSendError)?;

                        //Update the last seen update id
                        last_update_id = snapshot.last_update_id;
                    }
                }

                _ => {}
            }
        }

        Ok::<(), BidAskServiceError>(())
    })
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
        event_time: usize,
        first_update_id: u64,
        final_updated_id: u64,
        bids: Vec<[f64; 2]>,
        asks: Vec<[f64; 2]>,
    ) -> Self {
        OrderBookUpdate {
            event_time,
            first_update_id,
            final_updated_id,
            bids,
            asks,
        }
    }
}

#[derive(Deserialize, Debug)]

pub struct OrderBookEvent {
    #[serde(rename = "e")]
    pub event: String,
}

async fn get_order_book_snapshot(
    pair: &str,
    order_book_depth: usize,
) -> Result<OrderBookSnapshot, BinanceError> {
    let snapshot_endpoint = ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT.to_owned()
        + pair
        + "&limit="
        + order_book_depth.to_string().as_str();

    // Get the depth snapshot, deserialize and return the result
    let snapshot_response = reqwest::get(snapshot_endpoint).await?;

    if snapshot_response.status().is_success() {
        Ok(snapshot_response.json::<OrderBookSnapshot>().await?)
    } else {
        Err(BinanceError::HTTPError(String::from_utf8(
            snapshot_response.bytes().await?.to_vec(),
        )?))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    use crate::{error::BidAskServiceError, exchanges::binance::spawn_order_book_stream};

    use futures::FutureExt;

    use crate::exchanges::binance::stream::get_order_book_snapshot;

    #[tokio::test]
    async fn test_get_order_book_snapshot() {
        let snapshot = get_order_book_snapshot("ETHBTC", 50)
            .await
            .expect("Could not get order book snapshot");

        assert!(!snapshot.bids.is_empty());
        assert!(!snapshot.asks.is_empty());
    }

    #[tokio::test]
    //Test the Binance WS connection for 50 order book updates
    async fn test_spawn_order_book_stream() {
        let atomic_counter_0 = Arc::new(AtomicU32::new(0));
        let atomic_counter_1 = atomic_counter_0.clone();
        let target_counter = 50;

        let mut join_handles = vec![];

        let (mut order_book_update_rx, order_book_stream_handle) =
            spawn_order_book_stream("ethbtc".to_owned(), 500);

        let order_book_update_handle = tokio::spawn(async move {
            while let Some(_) = order_book_update_rx.recv().await {
                dbg!(atomic_counter_0.load(Ordering::Relaxed));
                atomic_counter_0.fetch_add(1, Ordering::Relaxed);
                if atomic_counter_0.load(Ordering::Relaxed) >= target_counter {
                    break;
                }
            }

            Ok::<(), BidAskServiceError>(())
        });

        join_handles.push(order_book_stream_handle);
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

            panic!("Unexpected error");
        }
    }
}

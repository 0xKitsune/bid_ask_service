use crate::{
    error::BidAskServiceError,
    exchanges::{exchange_utils, Exchange},
    order_book::price_level::{ask::Ask, bid::Bid, PriceLevelUpdate},
};

use futures::{SinkExt, StreamExt};
use serde_derive::{Deserialize, Serialize};

use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tungstenite::Message;

use crate::{exchanges::bitstamp::error::BitstampError, order_book::error::OrderBookError};

const WS_BASE_ENDPOINT: &str = "wss://ws.bitstamp.net/";
const SUBSCRIBE_EVENT: &str = "bts:subscribe";
const DIFF_ORDER_BOOK: &str = "diff_order_book";
const ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT: &str = "https://www.bitstamp.net/api/v2/order_book/";
const DATA_EVENT: &str = "data";
const GET_ORDER_BOOK_SNAPSHOT: Vec<u8> = vec![];

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
        let ws_stream_tx: Sender<Message> = ws_stream_tx.clone();
        loop {
            //Connect to the websocket endpoint
            let (mut order_book_stream, _) = tokio_tungstenite::connect_async(WS_BASE_ENDPOINT)
                .await
                .map_err(BitstampError::TungsteniteError)?;

            //Create a subscription message to notify Bitstamp to send order book updates
            let subscription_message =
                serde_json::to_string(&SubscribeMessage::new(&format!("{DIFF_ORDER_BOOK}_{pair}")))
                    .map_err(BitstampError::SerdeJsonError)?;

            //Send a subscribe message to start the stream
            order_book_stream
                .send(tungstenite::Message::Text(subscription_message))
                .await
                .map_err(BitstampError::TungsteniteError)?;

            tracing::info!("Ws connection established");

            //Notify the stream handler to get a snapshot of the order book
            //This will be the first message that the stream handler receives, so a
            //snapshot of the orderbook will be retrieved before any order book updates are handled
            ws_stream_tx
                .send(Message::Binary(GET_ORDER_BOOK_SNAPSHOT))
                .await
                .map_err(BitstampError::MessageSendError)?;

            //Send messages through a channel to be handled by the stream handler, respond to ping requests and handle reconnects
            while let Some(Ok(message)) = order_book_stream.next().await {
                match message {
                    tungstenite::Message::Text(_) => {
                        ws_stream_tx
                            .send(message)
                            .await
                            .map_err(BitstampError::MessageSendError)?;
                    }

                    tungstenite::Message::Ping(_) => {
                        tracing::info!("Ping received");
                        order_book_stream.send(Message::Pong(vec![])).await.ok();
                        tracing::info!("Pong sent");
                    }

                    tungstenite::Message::Close(_) => {
                        tracing::error!("Ws connection closed, reconnecting...");
                        break;
                    }

                    other => {
                        log::warn!("{other:?}");
                    }
                }
            }
        }
    });

    (ws_stream_rx, stream_handle)
}

pub fn spawn_stream_handler(
    pair: String,
    mut ws_stream_rx: Receiver<Message>,
    price_level_tx: Sender<PriceLevelUpdate>,
) -> JoinHandle<Result<(), BidAskServiceError>> {
    let order_book_update_handle = tokio::spawn(async move {
        let mut last_microtimestamp = 0;

        while let Some(message) = ws_stream_rx.recv().await {
            match message {
                tungstenite::Message::Text(message) => {
                    //Deserialize the event and check if it is a data event
                    let order_book_event = serde_json::from_str::<OrderBookEvent>(&message)
                        .map_err(BitstampError::SerdeJsonError)?;

                    if order_book_event.event == DATA_EVENT {
                        //Deserialize the order book update to extract the bids and asks
                        let order_book_update = serde_json::from_str::<OrderBookUpdate>(&message)
                            .map_err(BitstampError::SerdeJsonError)?;

                        let order_book_data = order_book_update.data;

                        // If the microtimestamp of the order book data is not newer than the last microtimestamp we skip
                        //processing it and continue with the next message
                        if order_book_data.microtimestamp <= last_microtimestamp {
                            //TODO: potentially add some error logging here
                            continue;
                        } else {
                            //Collect all of the bids from the update
                            let mut bids = vec![];
                            for bid in order_book_data.bids.into_iter() {
                                bids.push(Bid::new(bid[0], bid[1], Exchange::Bitstamp));
                            }

                            //Collect all of the asks from the update
                            let mut asks = vec![];
                            for ask in order_book_data.asks.into_iter() {
                                asks.push(Ask::new(ask[0], ask[1], Exchange::Bitstamp));
                            }

                            //Send the batched price level update to the aggregated order book
                            price_level_tx
                                .send(PriceLevelUpdate::new(bids, asks))
                                .await
                                .map_err(BitstampError::PriceLevelUpdateSendError)?;

                            last_microtimestamp = order_book_data.microtimestamp;
                        }
                    }
                }

                tungstenite::Message::Binary(message) => {
                    // This is an internal message signifying that the stream has reconnected so we need to get a snapshot
                    // First get a snapshot of the order book, handle all of the bids/asks and send it through the channel to the aggregated orderbook
                    if message.is_empty() {
                        let snapshot = get_order_book_snapshot(&pair).await?;

                        let mut bids = vec![];
                        for bid in snapshot.bids.into_iter() {
                            bids.push(Bid::new(bid[0], bid[1], Exchange::Bitstamp));
                        }

                        let mut asks = vec![];
                        for ask in snapshot.asks.into_iter() {
                            asks.push(Ask::new(ask[0], ask[1], Exchange::Bitstamp));
                        }

                        price_level_tx
                            .send(PriceLevelUpdate::new(bids, asks))
                            .await
                            .map_err(BitstampError::PriceLevelUpdateSendError)?;

                        //Update the last seen microtimestamp
                        last_microtimestamp = snapshot.microtimestamp;
                    }
                }

                _ => {}
            }
        }

        Ok::<(), BidAskServiceError>(())
    });

    order_book_update_handle
}

#[derive(Serialize, Debug)]
pub struct SubscriptionData {
    channel: String,
}
impl SubscriptionData {
    pub fn new(channel: &str) -> SubscriptionData {
        SubscriptionData {
            channel: String::from(channel),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct SubscribeMessage {
    event: String,
    data: SubscriptionData,
}
impl SubscribeMessage {
    pub fn new(channel: &str) -> SubscribeMessage {
        SubscribeMessage {
            event: SUBSCRIBE_EVENT.to_owned(),
            data: SubscriptionData::new(channel),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OrderBookSnapshot {
    #[serde(
        rename = "timestamp",
        deserialize_with = "exchange_utils::convert_from_string_to_u64"
    )]
    timestamp: u64,
    #[serde(
        rename = "microtimestamp",
        deserialize_with = "exchange_utils::convert_from_string_to_u64"
    )]
    microtimestamp: u64,
    #[serde(
        rename = "bids",
        deserialize_with = "exchange_utils::convert_array_items_to_f64"
    )]
    bids: Vec<[f64; 2]>,
    #[serde(
        rename = "asks",
        deserialize_with = "exchange_utils::convert_array_items_to_f64"
    )]
    asks: Vec<[f64; 2]>,
}

#[derive(Deserialize, Debug)]

pub struct OrderBookEvent {
    pub event: String,
}

#[derive(Deserialize, Debug)]
pub struct OrderBookUpdate {
    pub data: OrderBookUpdateData,
}

#[derive(Deserialize, Debug)]
pub struct OrderBookUpdateData {
    #[serde(
        rename = "timestamp",
        deserialize_with = "exchange_utils::convert_from_string_to_u64"
    )]
    microtimestamp: u64,
    #[serde(
        rename = "bids",
        deserialize_with = "exchange_utils::convert_array_items_to_f64"
    )]
    pub bids: Vec<[f64; 2]>,
    #[serde(
        rename = "asks",
        deserialize_with = "exchange_utils::convert_array_items_to_f64"
    )]
    pub asks: Vec<[f64; 2]>,
}

async fn get_order_book_snapshot(pair: &str) -> Result<OrderBookSnapshot, BitstampError> {
    let snapshot_endpoint = ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT.to_owned() + pair;

    // Get the depth snapshot, deserialize and return the result
    let snapshot_response = reqwest::get(snapshot_endpoint).await?;
    if snapshot_response.status().is_success() {
        Ok(snapshot_response.json::<OrderBookSnapshot>().await?)
    } else {
        Err(BitstampError::HTTPError(String::from_utf8(
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

    use crate::exchanges::bitstamp::stream::get_order_book_snapshot;
    use crate::{error::BidAskServiceError, exchanges::bitstamp::stream::spawn_order_book_stream};
    use futures::FutureExt;

    #[tokio::test]
    async fn test_get_order_book_snapshot() {
        let snapshot = get_order_book_snapshot("ethbtc")
            .await
            .expect("Could not get order book snapshot");

        assert!(!snapshot.bids.is_empty());
        assert!(!snapshot.asks.is_empty());
    }

    #[tokio::test]
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

use std::{fs::File, io::Write};

use super::Bitstamp;
use crate::{
    exchanges::{exchange_utils, Exchange},
    order_book::{OrderType, PriceLevel},
};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tungstenite::Message;

use crate::{exchanges::bitstamp::error::BitstampError, order_book::error::OrderBookError};

use super::OrderBookService;

const WS_BASE_ENDPOINT: &str = "wss://ws.bitstamp.net/";
const SUBSCRIBE_EVENT: &str = "bts:subscribe";
const DIFF_ORDER_BOOK: &str = "diff_order_book";
const ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT: &str = "https://www.bitstamp.net/api/v2/order_book/";
const DATA_EVENT: &str = "data";
//TODO: Add a comment for what this is also there are more efficent ways to do this, update this
const GET_ORDER_BOOK_SNAPSHOT: Vec<u8> = vec![];

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

//TODO: note to self, I think we are going to want to get the full live order book data

//TODO: in this case, it seems liek the live order book endpoint gives the list of the top 100 bids at a time, we dont need this if its repeating data.

//TODO: the live detail order book does the same thing but gives more data

//TODO: the live full order book just gives you the list of changed bids/asks since the last broadcast.

//TODO: we can prob couple this with the snapshot as well just like binance and follow almost the exact same order to get a buffered stream with reconnects

pub async fn spawn_order_book_stream(
    pair: String,
    order_book_stream_buffer: usize,
) -> Result<(Receiver<Message>, JoinHandle<Result<(), OrderBookError>>), OrderBookError> {
    let (ws_stream_tx, ws_stream_rx) =
        tokio::sync::mpsc::channel::<Message>(order_book_stream_buffer);

    //spawn a thread that handles the stream and buffers the results
    let stream_handle = tokio::spawn(async move {
        let ws_stream_tx: Sender<Message> = ws_stream_tx.clone();
        loop {
            let (mut order_book_stream, _) =
                tokio_tungstenite::connect_async(WS_BASE_ENDPOINT).await?;

            let subscription_message = serde_json::to_string(&SubscribeMessage::new(&format!(
                "{DIFF_ORDER_BOOK}_{pair}"
            )))?;

            order_book_stream
                .send(tungstenite::Message::Text(subscription_message))
                .await?;

            log::info!("Ws connection established");

            //TODO: send a message telling to get the orderbook snapshot

            ws_stream_tx
                .send(Message::Binary(GET_ORDER_BOOK_SNAPSHOT))
                .await
                .map_err(BitstampError::MessageSendError)?; //TODO: we prob dont need a binance error for this

            while let Some(Ok(message)) = order_book_stream.next().await {
                match message {
                    tungstenite::Message::Text(_) => {
                        ws_stream_tx
                            .send(message)
                            .await
                            .map_err(BitstampError::MessageSendError)?;
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

    Ok((ws_stream_rx, stream_handle))
}

pub async fn spawn_stream_handler(
    pair: String,
    mut ws_stream_rx: Receiver<Message>,
    price_level_tx: Sender<PriceLevel>,
) -> Result<JoinHandle<Result<(), OrderBookError>>, OrderBookError> {
    let order_book_update_handle = tokio::spawn(async move {
        //TODO: update heuristic to check if orders are gtg
        let mut last_microtimestamp = 0;

        while let Some(message) = ws_stream_rx.recv().await {
            match message {
                tungstenite::Message::Text(message) => {
                    let order_book_event = serde_json::from_str::<OrderBookEvent>(&message)?;

                    if order_book_event.event == DATA_EVENT {
                        dbg!(&message);
                        let order_book_update = serde_json::from_str::<OrderBookUpdate>(&message)?;

                        let order_book_data = order_book_update.data;

                        if order_book_data.microtimestamp <= last_microtimestamp {
                            //TODO: potentially add some error logging here
                            continue;
                        } else {
                            for bid in order_book_data.bids.into_iter() {
                                price_level_tx
                                    .send(PriceLevel::new(
                                        bid[0],
                                        bid[1],
                                        Exchange::Binance,
                                        OrderType::Bid,
                                    ))
                                    .await?;
                            }

                            for ask in order_book_data.asks.into_iter() {
                                price_level_tx
                                    .send(PriceLevel::new(
                                        ask[0],
                                        ask[1],
                                        Exchange::Binance,
                                        OrderType::Ask,
                                    ))
                                    .await?;
                            }

                            last_microtimestamp = order_book_data.microtimestamp;
                        }
                    }
                }

                tungstenite::Message::Binary(message) => {
                    //This is an internal message signaling that we should get the depth snapshot and send it through the channel
                    if message.is_empty() {
                        let snapshot = get_order_book_snapshot(&pair).await?;

                        for bid in snapshot.bids.iter() {
                            price_level_tx
                                .send(PriceLevel::new(
                                    bid[0],
                                    bid[1],
                                    Exchange::Bitstamp,
                                    OrderType::Bid,
                                ))
                                .await?;
                        }

                        for ask in snapshot.asks.iter() {
                            price_level_tx
                                .send(PriceLevel::new(
                                    ask[0],
                                    ask[1],
                                    Exchange::Bitstamp,
                                    OrderType::Ask,
                                ))
                                .await?;
                        }

                        //TODO: update timestamp if needed or whatever metric we are using to check if valid order
                        // last_update_id = snapshot.last_update_id;
                    }
                }

                _ => {}
            }
        }

        Ok::<(), OrderBookError>(())
    });

    Ok(order_book_update_handle)
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
    pub bids: Vec<[f64; 2]>,
    #[serde(
        rename = "asks",
        deserialize_with = "exchange_utils::convert_array_items_to_f64"
    )]
    pub asks: Vec<[f64; 2]>,
}

async fn get_order_book_snapshot(pair: &str) -> Result<OrderBookSnapshot, OrderBookError> {
    let snapshot_endpoint = ORDER_BOOK_SNAPSHOT_BASE_ENDPOINT.to_owned() + &pair;
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

    use crate::exchanges::bitstamp::stream::spawn_order_book_stream;
    use crate::{
        exchanges::{binance::Binance, bitstamp::Bitstamp, OrderBookService},
        order_book::{error::OrderBookError, PriceLevel},
    };
    use futures::FutureExt;
    #[tokio::test]

    //TODO: add a test for order book snapshot

    //TODO: add some failure tests

    async fn test_spawn_order_book_stream() {
        let atomic_counter_0 = Arc::new(AtomicU32::new(0));
        let atomic_counter_1 = atomic_counter_0.clone();
        let target_counter = 50;
        let mut join_handles = vec![];

        let (mut order_book_update_rx, order_book_stream_handle) =
            spawn_order_book_stream("ethbtc".to_owned(), 500)
                .await
                .expect("TODO: handle this error");

        let order_book_update_handle = tokio::spawn(async move {
            while let Some(_) = order_book_update_rx.recv().await {
                dbg!(atomic_counter_0.load(Ordering::Relaxed));
                atomic_counter_0.fetch_add(1, Ordering::Relaxed);
                if atomic_counter_0.load(Ordering::Relaxed) >= target_counter {
                    break;
                }
            }

            return Ok::<(), OrderBookError>(());
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
        }
    }
}

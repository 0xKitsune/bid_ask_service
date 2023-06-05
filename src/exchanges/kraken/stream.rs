use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_derive::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tungstenite::Message;

use crate::{
    error::BidAskServiceError, exchanges::exchange_utils, exchanges::kraken::error::KrakenError,
    order_book::price_level::PriceLevelUpdate,
};

pub const SUBSCRIBE: &str = "subscribe";
pub const BOOK: &str = "book";
const WS_ENDPOINT: &str = "wss://ws.kraken.com";
const GET_DEPTH_SNAPSHOT: Vec<u8> = vec![];

pub fn spawn_order_book_stream(
    pair: String,
    exchange_stream_buffer: usize,
    order_book_depth: usize,
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
            let (mut order_book_stream, _) = tokio_tungstenite::connect_async(WS_ENDPOINT).await?;
            tracing::info!("Ws connection established");

            order_book_stream
                .send(Message::Text(
                    serde_json::to_string(&SubscribeMessage::new(&pair, order_book_depth))
                        .expect("TODO: handle this error"),
                ))
                .await?;

            while let Some(Ok(message)) = order_book_stream.next().await {
                match message {
                    tungstenite::Message::Text(_) => {
                        ws_stream_tx
                            .send(message)
                            .await
                            .map_err(KrakenError::MessageSendError)?;
                    }

                    tungstenite::Message::Ping(_) => {
                        tracing::info!("Ping received");
                        order_book_stream.send(Message::Pong(vec![])).await.ok();
                        tracing::info!("Pong sent");
                    }

                    tungstenite::Message::Close(_) => {
                        tracing::info!("Ws connection closed, reconnecting...");
                        //The kraken docs, mention to wait 5 seconds before reconnecting, should prob experiment with this
                        //to see if we can reconnect faster without penalty
                        tokio::time::sleep(Duration::from_secs(5)).await;
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

#[derive(Serialize, Debug)]
pub struct SubscribeMessage {
    event: String,
    pair: Vec<String>,
    subscription: Subscription,
}

#[derive(Serialize, Debug)]
pub struct Subscription {
    name: String,
    depth: usize,
}

impl Subscription {
    pub fn new(name: &str, depth: usize) -> Self {
        Subscription {
            name: name.to_string(),
            depth,
        }
    }
}

impl SubscribeMessage {
    pub fn new(pair: &str, depth: usize) -> Self {
        SubscribeMessage {
            event: SUBSCRIBE.to_string(),
            pair: vec![pair.to_owned()],
            subscription: Subscription::new(BOOK, depth),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct OrderBookSnapshot {
    #[serde(
        rename = "bs",
        deserialize_with = "exchange_utils::convert_array_len_3_to_f64"
    )]
    pub bids: Vec<[f64; 3]>,
    #[serde(
        rename = "as",
        deserialize_with = "exchange_utils::convert_array_len_3_to_f64"
    )]
    pub asks: Vec<[f64; 3]>,
}

#[derive(Deserialize, Debug)]
pub struct OrderBookUpdate {
    #[serde(
        rename = "b",
        deserialize_with = "exchange_utils::convert_array_len_3_to_f64",
        default
    )]
    pub bids: Vec<[f64; 3]>,
    #[serde(
        rename = "a",
        deserialize_with = "exchange_utils::convert_array_len_3_to_f64",
        default
    )]
    pub asks: Vec<[f64; 3]>,
}

pub fn spawn_stream_handler(
    pair: String,
    mut ws_stream_rx: Receiver<Message>,
    price_level_tx: Sender<PriceLevelUpdate>,
) -> JoinHandle<Result<(), BidAskServiceError>> {
    tokio::spawn(async move { Ok::<(), BidAskServiceError>(()) });

    todo!()
}

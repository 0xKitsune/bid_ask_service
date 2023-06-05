use tokio::sync::mpsc::error::SendError;
use tungstenite::Message;

use crate::order_book::price_level::PriceLevelUpdate;

#[derive(thiserror::Error, Debug)]
pub enum KrakenError {
    #[error("Error when sending tungstenite message")]
    MessageSendError(#[from] SendError<Message>),
    #[error("Serde json error")]
    SerdeJsonError(#[from] serde_json::Error),
}

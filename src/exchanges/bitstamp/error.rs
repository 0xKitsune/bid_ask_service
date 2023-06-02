use tokio::sync::mpsc::error::SendError;

use crate::order_book::price_level::PriceLevelUpdate;

#[derive(thiserror::Error, Debug)]
pub enum BitstampError {
    #[error("Error when sending tungstenite message")]
    MessageSendError(#[from] SendError<tungstenite::Message>),
    #[error("Invalid update id")]
    InvalidUpdateId,
    #[error("Tungstenite error")]
    TungsteniteError(#[from] tungstenite::Error),
    #[error("Error when sending price level update")]
    PriceLevelUpdateSendError(#[from] tokio::sync::mpsc::error::SendError<PriceLevelUpdate>),
    #[error("Serde json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("HTTP error")]
    HTTPError(String),
    #[error("Error when converting to Utf8 from string")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

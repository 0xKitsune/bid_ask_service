use crate::exchanges::{
    binance::error::{self, BinanceError},
    bitstamp::error::BitstampError,
};

use super::PriceLevelUpdate;

#[derive(thiserror::Error, Debug)]
pub enum OrderBookError {
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Tungstenite error")]
    TungsteniteError(#[from] tungstenite::Error),
    #[error("HTTP error")]
    HTTPError(String),
    #[error("Serde json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Binance error")]
    BinanceError(#[from] BinanceError),
    #[error("Bitstamp error")]
    BitstampError(#[from] BitstampError),
    #[error("Error when sending price level update")]
    PriceLevelUpdateSendError(#[from] tokio::sync::mpsc::error::SendError<PriceLevelUpdate>),
    #[error("Poisoned lock")]
    PoisonedLock,
    #[error("Error when converting to Utf8 from string")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

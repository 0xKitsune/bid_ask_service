use crate::{
    exchanges::{binance::error::BinanceError, bitstamp::error::BitstampError},
    server::orderbook_service::Summary,
};

use super::price_level::PriceLevelUpdate;

#[derive(thiserror::Error, Debug)]
pub enum OrderBookError {
    #[error("Poisoned lock")]
    PoisonedLock,
    #[error("Error when sending summary through channel")]
    SummarySendError(#[from] tokio::sync::broadcast::error::SendError<Summary>),
}

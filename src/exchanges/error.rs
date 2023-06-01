use crate::exchanges::{binance::error::BinanceError, bitstamp::error::BitstampError};

#[derive(thiserror::Error, Debug)]
pub enum ExchangeError {
    #[error("Binance error")]
    BinanceError(#[from] BinanceError),
    #[error("Bitstamp error")]
    BitstampError(#[from] BitstampError),
}

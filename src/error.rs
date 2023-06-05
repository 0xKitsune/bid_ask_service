use crate::{
    exchanges::{
        binance::error::BinanceError, bitstamp::error::BitstampError, kraken::error::KrakenError,
    },
    order_book::error::OrderBookError,
    server::error::ServerError,
};

#[derive(thiserror::Error, Debug)]
pub enum BidAskServiceError {
    #[error("Order book error")]
    OrderBookError(#[from] OrderBookError),
    #[error("Binance error")]
    BinanceError(#[from] BinanceError),
    #[error("Bitstamp error")]
    BitstampError(#[from] BitstampError),
    #[error("Kraken error")]
    KrakenError(#[from] KrakenError),
    #[error("Server error")]
    ServerError(#[from] ServerError),
}

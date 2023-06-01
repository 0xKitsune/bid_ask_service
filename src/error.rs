use crate::{
    exchanges::{
        binance::error::BinanceError, bitstamp::error::BitstampError, error::ExchangeError,
    },
    order_book::error::OrderBookError,
    server::error::ServerError,
};

#[derive(thiserror::Error, Debug)]
pub enum BidAskServiceError {
    #[error("Order book error")]
    OrderBookError(#[from] OrderBookError),
    #[error("Exchange error")]
    ExchangeError(#[from] ExchangeError),
    #[error("Server error")]
    ServerError(#[from] ServerError),
}

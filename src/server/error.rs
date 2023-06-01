use crate::{
    exchanges::{binance::error::BinanceError, bitstamp::error::BitstampError},
    order_book::error::OrderBookError,
};

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Transport error")]
    TransportError(#[from] tonic::transport::Error),
}

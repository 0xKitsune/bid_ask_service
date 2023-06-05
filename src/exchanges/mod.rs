pub mod binance;
pub mod bitstamp;
pub mod error;
pub mod exchange_utils;
pub mod kraken;

use core::fmt;
use std::str::FromStr;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::error::BidAskServiceError;
use crate::order_book::price_level::PriceLevelUpdate;

use self::binance::Binance;
use self::bitstamp::Bitstamp;

const BINANCE: &str = "binance";
const BITSTAMP: &str = "bitstamp";

#[async_trait]
pub trait OrderBookService {
    /// Spawns an order book service to stream order book data and handle stream events for a specified pair.
    fn spawn_order_book_service(
        pair: [&str; 2],
        order_book_depth: usize,
        exchange_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Vec<JoinHandle<Result<(), BidAskServiceError>>>;
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Exchange {
    Bitstamp,
    Binance,
}

impl Exchange {
    //Spawn the order book service for the specified exchange
    pub fn spawn_order_book_service(
        &self,
        pair: [&str; 2],
        order_book_depth: usize,
        exchange_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Vec<JoinHandle<Result<(), BidAskServiceError>>> {
        match self {
            Exchange::Binance => Binance::spawn_order_book_service(
                pair,
                order_book_depth,
                exchange_stream_buffer,
                price_level_tx,
            ),
            Exchange::Bitstamp => Bitstamp::spawn_order_book_service(
                pair,
                order_book_depth,
                exchange_stream_buffer,
                price_level_tx,
            ),
        }
    }

    //Return all available exchanges
    pub fn all_exchanges() -> Vec<Exchange> {
        vec![Exchange::Bitstamp, Exchange::Binance]
    }

    //Parse a list of exchanges from a comma separated String into a Vec<Exchange>
    pub fn parse_exchanges(exchanges: String) -> Result<Vec<Exchange>, ParseExchangeError> {
        exchanges
            .split(',')
            .map(|s| s.parse::<Exchange>())
            .collect::<Result<Vec<_>, _>>()
    }
}

impl ToString for Exchange {
    fn to_string(&self) -> String {
        match self {
            Exchange::Bitstamp => BITSTAMP.to_owned(),
            Exchange::Binance => BINANCE.to_owned(),
        }
    }
}

impl FromStr for Exchange {
    type Err = ParseExchangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bitstamp" => Ok(Exchange::Bitstamp),
            "binance" => Ok(Exchange::Binance),
            _ => Err(ParseExchangeError::UnrecognizedExchange),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParseExchangeError {
    UnrecognizedExchange,
}

impl fmt::Display for ParseExchangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not parse the exchange")
    }
}

impl std::error::Error for ParseExchangeError {}

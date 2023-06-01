use std::error::Error;

use kbas::{
    exchanges::Exchange,
    order_book::{rbtree::RBTreeOrderBook, AggregatedOrderBook},
    server::{
        self, orderbook_service::orderbook_aggregator_server::OrderbookAggregatorServer,
        spawn_order_book_aggregator_service, OrderbookAggregatorService,
    },
};
use tonic::transport::Server;

pub const PRICE_LEVEL_CHANNEL_BUFFER: usize = 100;

//TODO: add clap and parse args to determine which exchanges, which order book variant, log file, etc
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let default_exchanges = vec![Exchange::Binance, Exchange::Bitstamp];

    //TODO: add the summary buffer as a clap arg
    let (order_book_aggregator_service, summary_tx) = server::OrderbookAggregatorService::new(300);
    let router = Server::builder().add_service(OrderbookAggregatorServer::new(
        order_book_aggregator_service,
    ));
    let socket_addr = "[::1]:50051".parse()?;

    //TODO: add the pair as a clap arg

    let pair = ["", ""];

    //TODO: add the depth as a clap arg

    let order_book_depth = 10;
    //TODO: add the order book stream buffer as a clap arg

    let order_book_stream_buffer = 100;

    let aggregated_order_book =
        AggregatedOrderBook::new(pair, default_exchanges, RBTreeOrderBook::new());

    aggregated_order_book
        .listen_to_bid_ask_spread(
            order_book_depth,
            order_book_stream_buffer,
            PRICE_LEVEL_CHANNEL_BUFFER,
        )
        .await?;

    //TODO: initializes the exchanges we want, grabs the order book that we want, uses all of this to spin up the aggregated order book

    //TODO: spawns the aggregated order book, returns the rx

    //TODO: spawn the grpc server, passes in the rx, this triggers any listeners to be served whenever an rx comes through

    //TODO: rename this function
    let service_handle = spawn_order_book_aggregator_service(router, socket_addr);
    Ok(())
}

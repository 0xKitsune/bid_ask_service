use std::{collections::BTreeSet, error::Error};

use futures::FutureExt;
use kbas::{
    exchanges::Exchange,
    order_book::{
        price_level::{ask::Ask, bid::Bid},
        AggregatedOrderBook,
    },
    server::{
        self, orderbook_service::orderbook_aggregator_server::OrderbookAggregatorServer,
        spawn_order_book_service, OrderbookAggregatorService,
    },
};
use tonic::transport::Server;

pub const PRICE_LEVEL_CHANNEL_BUFFER: usize = 100;

//TODO: add clap and parse args to determine which exchanges, which order book variant, log file, etc
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //TODO: handle exchanges from clap
    let default_exchanges = vec![Exchange::Binance, Exchange::Bitstamp];

    //TODO: add the summary buffer as a clap arg
    let (order_book_aggregator_service, summary_tx) = server::OrderbookAggregatorService::new(300);
    let router = Server::builder().add_service(OrderbookAggregatorServer::new(
        order_book_aggregator_service,
    ));
    let socket_addr = "[::1]:50051".parse()?;

    //TODO: add the pair as a clap arg

    let pair = ["eth", "btc"];

    //TODO: add the depth as a clap arg

    let order_book_depth = 10;
    //TODO: add the order book stream buffer as a clap arg

    let order_book_stream_buffer = 100;

    //TODO: add best orders size as a clap arg
    let best_n_orders = 10;

    let aggregated_order_book = AggregatedOrderBook::new(
        pair,
        default_exchanges,
        BTreeSet::<Bid>::new(),
        BTreeSet::<Ask>::new(),
    );

    let mut join_handles = vec![];

    join_handles.extend(aggregated_order_book.spawn_bid_ask_service(
        order_book_depth,
        order_book_stream_buffer,
        PRICE_LEVEL_CHANNEL_BUFFER,
        best_n_orders,
        summary_tx,
    ));

    join_handles.push(spawn_order_book_service(router, socket_addr));

    let futures = join_handles
        .into_iter()
        .map(|handle| handle.boxed())
        .collect::<Vec<_>>();

    //TODO: handle an error if it pops up
    let (result, _, _) = futures::future::select_all(futures).await;

    dbg!(result);

    Ok(())
}

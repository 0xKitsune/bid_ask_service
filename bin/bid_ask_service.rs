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
        spawn_grpc_server, OrderbookAggregatorService,
    },
};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "Bid ask service")]
struct Opts {
    /// List of exchanges, separated by commas
    #[clap(long, short)]
    exchanges: Option<String>,

    /// Summary buffer size
    #[clap(long, default_value = "300")]
    summary_buffer: usize,

    /// Trading pair
    #[clap(long, short)]
    pair: String,

    /// The max depth of the aggregated order book
    #[clap(long, default_value = "25")]
    order_book_depth: usize,

    /// Size of best orders
    #[clap(long, default_value = "10")]
    best_n_orders: usize,

    /// Order book stream buffer size
    #[clap(long, default_value = "100")]
    order_book_stream_buffer: usize,

    ///
    #[clap(long, default_value = "100")]
    price_level_channel_buffer: usize,

    /// Socket address for the gRPC server
    #[clap(long, default_value = "[::1]:50051")]
    socket_address: String,
}

use tonic::transport::Server;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    //Parse the command line args, extract the exchanges and the pair
    let opts = Opts::parse();

    let exchanges = if let Some(values) = opts.exchanges {
        Exchange::parse_exchanges(values)?
    } else {
        Exchange::all_exchanges()
    };

    let tickers = opts
        .pair
        .split(",")
        .map(|s| s.replace(" ", "").to_lowercase())
        .collect::<Vec<String>>();

    let pair: [&str; 2] = [&tickers[0], &tickers[1]];

    //Create a new orderbook aggregator service and build the gRPC server
    let (order_book_aggregator_service, summary_tx) =
        server::OrderbookAggregatorService::new(opts.summary_buffer);
    let router = Server::builder().add_service(OrderbookAggregatorServer::new(
        order_book_aggregator_service,
    ));

    //Initialize a new aggregated orderbook, specifying the data structure to represent the bids and asks
    let aggregated_order_book = AggregatedOrderBook::new(
        pair,
        exchanges,
        BTreeSet::<Bid>::new(),
        BTreeSet::<Ask>::new(),
    );

    //Spawn the bid ask service from the orderbook and the gRPC server
    let mut join_handles = vec![];
    join_handles.extend(aggregated_order_book.spawn_bid_ask_service(
        opts.order_book_depth,
        opts.order_book_stream_buffer,
        opts.price_level_channel_buffer,
        opts.best_n_orders,
        summary_tx,
    ));

    join_handles.push(spawn_grpc_server(router, opts.socket_address.parse()?));

    //Collect all of the join handles and await the futures to handle any errors
    let futures = join_handles
        .into_iter()
        .map(|handle| handle.boxed())
        .collect::<Vec<_>>();

    let (future_result, _, _) = futures::future::select_all(futures).await;

    match future_result {
        Ok(task_result) => match task_result {
            Ok(_) => {
                eyre::bail!("Program exited unexpectedly");
            }
            Err(e) => Err(eyre::Report::new(e)),
        },
        Err(join_error) => Err(eyre::Report::new(join_error)),
    }
}

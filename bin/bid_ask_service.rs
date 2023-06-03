use bid_ask_service::{
    exchanges::Exchange,
    order_book::{
        price_level::{ask::Ask, bid::Bid},
        AggregatedOrderBook,
    },
    server::{
        self, orderbook_service::orderbook_aggregator_server::OrderbookAggregatorServer,
        spawn_grpc_server,
    },
};
use clap::Parser;
use futures::FutureExt;
use std::collections::BTreeSet;
use tonic::transport::Server;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::Format;

#[derive(Parser, Debug)]
#[clap(name = "Bid ask service")]
struct Opts {
    /// List of exchanges, separated by commas, ie. binance,bitstamp
    #[clap(long, short)]
    exchanges: Option<String>,

    /// Channel buffer size for the tokio broadcast channel used to stream the aggregated order book to the gRPC server
    #[clap(long, default_value = "300")]
    summary_buffer: usize,

    /// Trading pair to listen to updates to separated by commas, ie. eth,btc
    #[clap(long, short)]
    pair: String,

    /// The max depth of the aggregated order book
    #[clap(long, default_value = "25")]
    order_book_depth: usize,

    /// The number of best bids and asks to stream via the gRPC server
    #[clap(long, default_value = "10")]
    best_n_orders: usize,

    /// Channel buffer size for streaming live order book data from exchanges
    #[clap(long, default_value = "100")]
    exchange_stream_buffer: usize,

    /// Channel buffer size to pass the price level updates from the exchange module to the aggregated order book
    #[clap(long, default_value = "100")]
    price_level_channel_buffer: usize,

    /// Socket address for the gRPC server
    #[clap(long, default_value = "[::1]:50051")]
    socket_address: String,

    /// Level of logging, options are trace, debug, info, warn, error
    #[clap(long, default_value = "info")]
    level: tracing::metadata::LevelFilter,

    /// Path to output file for logging
    #[clap(long, default_value = "output.log")]
    log_file_path: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    //Parse the command line args, extract the exchanges and the pair
    let opts = Opts::parse();
    let _tracing_guard = initialize_tracing(&opts.log_file_path, opts.level)?;

    let exchanges = if let Some(values) = opts.exchanges {
        Exchange::parse_exchanges(values)?
    } else {
        Exchange::all_exchanges()
    };

    let tickers = opts
        .pair
        .split(',')
        .map(|s| s.replace(' ', "").to_lowercase())
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

    tracing::info!("Spawning aggregated order book bid-ask service");
    //Spawn the bid ask service from the orderbook and the gRPC server
    let mut join_handles = vec![];
    join_handles.extend(aggregated_order_book.spawn_bid_ask_service(
        opts.order_book_depth,
        opts.exchange_stream_buffer,
        opts.price_level_channel_buffer,
        opts.best_n_orders,
        summary_tx,
    ));

    tracing::info!("Spawning gRPC server");
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

fn initialize_tracing(
    file_path: &str,
    level: tracing::metadata::LevelFilter,
) -> eyre::Result<WorkerGuard> {
    let file_appender = tracing_appender::rolling::never("log", file_path);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let format = Format::default()
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_ansi(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_level(true)
        .compact();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(level)
        .event_format(format)
        .with_writer(non_blocking)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(guard)
}

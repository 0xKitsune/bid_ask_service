use std::{
    collections::BTreeSet,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::FutureExt;
use kbas::{
    error::BidAskServiceError,
    exchanges::Exchange,
    order_book::{
        price_level::{ask::Ask, bid::Bid},
        AggregatedOrderBook,
    },
    server::{
        self, orderbook_service::orderbook_aggregator_client::OrderbookAggregatorClient,
        orderbook_service::orderbook_aggregator_server::OrderbookAggregatorServer,
        orderbook_service::Empty, spawn_grpc_server,
    },
};
use tokio::{task::JoinHandle, time};
use tonic::transport::{Channel, Server};

#[tokio::test]
async fn test_bid_ask_service() {
    let mut join_handles = vec![];
    let atomic_counter_0 = Arc::new(AtomicU32::new(0));
    let atomic_counter_1 = atomic_counter_0.clone();

    //specify the amount of stream events that should be successfully handled
    let target_count = 500;

    let server_address = "[::1]:50051".to_owned();

    //spawn the bid ask service, opening a port at the server address
    let server_handle = spawn_bid_ask_service(server_address.clone());

    //allow the server to start
    time::sleep(Duration::from_secs(10)).await;

    //spawn the client connection which will handle 500 updates from the server
    let client_handle = spawn_client(server_address, atomic_counter_0, target_count).await;

    join_handles.extend(server_handle);
    join_handles.push(client_handle);

    let futures = join_handles
        .into_iter()
        .map(|handle| handle.boxed())
        .collect::<Vec<_>>();

    //Wait for the first future to be finished
    let (result, _, _) = futures::future::select_all(futures).await;

    if atomic_counter_1.load(Ordering::Relaxed) != target_count {
        result
            .expect("Join handle error")
            .expect("Error when handling gRPC stream");

        panic!("Unexpected error");
    }
}

fn spawn_bid_ask_service(
    server_address: String,
) -> Vec<JoinHandle<Result<(), BidAskServiceError>>> {
    let order_book_depth = 25;
    let summary_buffer = 100;
    let order_book_stream_buffer = 100;
    let price_level_channel_buffer = 100;
    let best_n_orders = 10;

    let socket_address = server_address
        .parse::<SocketAddr>()
        .expect("error initializing socket address");

    //Create a new orderbook aggregator service and build the gRPC server
    let (order_book_aggregator_service, summary_tx) =
        server::OrderbookAggregatorService::new(summary_buffer);
    let router = Server::builder().add_service(OrderbookAggregatorServer::new(
        order_book_aggregator_service,
    ));

    //Initialize a new aggregated orderbook, specifying the data structure to represent the bids and asks
    let aggregated_order_book = AggregatedOrderBook::new(
        ["eth", "btc"],
        vec![Exchange::Bitstamp, Exchange::Binance],
        BTreeSet::<Bid>::new(),
        BTreeSet::<Ask>::new(),
    );

    //Spawn the bid ask service from the orderbook and the gRPC server
    let mut join_handles = vec![];
    join_handles.extend(aggregated_order_book.spawn_bid_ask_service(
        order_book_depth,
        order_book_stream_buffer,
        price_level_channel_buffer,
        best_n_orders,
        summary_tx,
    ));

    join_handles.push(spawn_grpc_server(router, socket_address));

    join_handles
}

async fn spawn_client(
    server_address: String,
    atomic_counter: Arc<AtomicU32>,
    target_count: u32,
) -> JoinHandle<Result<(), BidAskServiceError>> {
    tokio::spawn(async move {
        // connect to the gRPC server
        let channel = Channel::from_shared("http://".to_owned() + &server_address)
            .expect("could not form channel from server address")
            .connect()
            .await
            .expect("could not connect to channel");

        let mut client = OrderbookAggregatorClient::new(channel);

        // call the BookSummary endpoint
        let mut stream = client
            .book_summary(tonic::Request::new(Empty {}))
            .await
            .expect("could not make request")
            .into_inner();

        // handle the responses
        while let Some(response) = stream
            .message()
            .await
            .expect("could not get message from stream")
        {
            atomic_counter.fetch_add(1, Ordering::Relaxed);
            let counter = atomic_counter.load(Ordering::Relaxed);
            println!("Counter: {:?}", counter);
            println!("Response: {:?}", response);
            if counter >= target_count {
                break;
            }
        }
        Ok::<(), BidAskServiceError>(())
    })
}

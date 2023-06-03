# Code Walkthrough / Reflection

The Bid-Ask service is broken up into three major modules, `exchanges`, `order_book` and `server`. 

- The `exchanges` module handles the connections to various CeFi exchanges, retrieving live updates from their respective order books, and standardizing this data into a consistent format.

- The `order_book` module is in charge of aggregating the standardized data from all exchanges into a comprehensive and updated view of of the top bids, top asks and bid-ask spread across all of the exchanges for a given pair.

- Lastly, the `server` module serves this aggregated data to clients via a gRPC stream. 

Throughout this walkthrough, we will dive deeper into how each of these modules function, starting with the exchanges module.


## Exchanges

In the `exchanges` module, every exchange implements the `OrderBookService` trait. The trait consists of a single function, `spawn_order_book_service`, which is tasked with starting an order book stream, managing reconnects, handling updates, and sending the cleaned data to the aggregated order book.

```rust
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
```

Lets take a quick look at how the `OrderBookService` trait is implemented for Binance.


```rust

#[async_trait]
impl OrderBookService for Binance {
    fn spawn_order_book_service(
        pair: [&str; 2],
        order_book_depth: usize,
        exchange_stream_buffer: usize,
        price_level_tx: Sender<PriceLevelUpdate>,
    ) -> Vec<JoinHandle<Result<(), BidAskServiceError>>> {
        let pair = pair.join("");
        //When subscribing to a stream of order book updates, the pair is required to be formatted as a single string with all lowercase letters
        let stream_pair = pair.to_lowercase();
        //When getting a snapshot, Binance requires that the pair si a single string with all uppercase letters
        let snapshot_pair = pair.to_uppercase();

        tracing::info!("Spawning Binance order book stream");
        //Spawn a task to handle a buffered stream of the order book and reconnects to the exchange
        let (ws_stream_rx, stream_handle) =
            spawn_order_book_stream(stream_pair, exchange_stream_buffer);

        tracing::info!("Spawning Binance order book stream handler");
        //Spawn a task to handle updates from the buffered stream, cleaning the data and sending it to the aggregated order book
        let order_book_update_handle = spawn_stream_handler(
            snapshot_pair,
            order_book_depth,
            ws_stream_rx,
            price_level_tx,
        );

        vec![stream_handle, order_book_update_handle]
    }
}
```

Lets take a closer look at what is happening in this function. 

The `spawn_order_book_stream` function subscribes to a WebSocket stream of order book updates for the given pair. The WebSocket stream uses the `exchange_stream_buffer` variable to define how many messages to buffer the stream by, which helps manage potential back pressure or sudden bursts of updates. When a disconnection happens, this function handles reconnecting to the stream, ensuring no data is missed during the downtime by retrieving a snapshot of the orderbook before handling further stream updates. This is crucial because while the stream is attempting to reconnect, the state of the order book could have changed significantly.

The `spawn_order_book_service` function returns a vector of the spawned tasks, allowing them to be managed, monitored, or awaited elsewhere in the application. All of the exchanges throughout the application implement the same approach, using a websocket stream and order book snapshots to retrieve real-time order book data, clean the data into a consistent format and send the price level updates to the aggregated order book.


## Order Book
The `order_book` module is divided into a few main components, price levels, order book data structures and the aggregated order book. Starting with price levels, the `price_level` sub-module defines the `Bid` and `Ask` structs, lays out the rules for their ordering and contains various trait definitions that define an order. Lets take a quick look at what the `Bid` and `Ask` structs look like.

```rust

pub struct Bid {
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
    pub exchange: Exchange,
}

pub struct Ask {
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
    pub exchange: Exchange,
}

```
Each struct has a `price`, `quantity` and `exchange`. Both the `Bid` and `Ask` structs implement various traits like `Eq`, `PartialEq` and `Ord` to allow them to be ordered correctly within the aggregated orderbook. 

The second major component to the `order_book` module are the buy and sell side traits. Any struct that implements the `BuySide` or `SellSide` traits, can act as the data structure that holds the bids or asks in the aggregated order book. Lets take a quick look at these traits.

```rust
pub trait BuySide: Debug {
    fn update_bids(&mut self, bid: Bid, max_depth: usize);
    fn get_best_bid(&self) -> Option<&Bid>;
    fn get_best_n_bids(&self, n: usize) -> Vec<Option<Bid>>;
}

pub trait SellSide: Debug {
    fn update_asks(&mut self, ask: Ask, max_depth: usize);
    fn get_best_ask(&self) -> Option<&Ask>;
    fn get_best_n_asks(&self, n: usize) -> Vec<Option<Ask>>;
}
```

The program currently uses a `BTreeSet` to represent bids and asks within the aggregated order book. A `BTreeSet` was chosen because it is a self balancing tree with O(log n) insert, removal and traversal.

The final major component of the `order_book` module is the `AggregatedOrderBook`. This struct is responsible for aggregating all of the bids and asks from each exchange stream and storing the best `n` orders on each side of the market. 

```rust
pub struct AggregatedOrderBook<B: BuySide + Send, S: SellSide + Send> {
    pub pair: [String; 2],
    pub exchanges: Vec<Exchange>,
    pub bids: Arc<Mutex<B>>,
    pub asks: Arc<Mutex<S>>,
}
```

The `AggregatedOrderBook` contains a method called `spawn_bid_ask_service` which is responsible for calling the `spawn_order_book_service` on each exchange and handling price level updates through it's `handle_order_book_updates`. 

```rust

impl<B, S> AggregatedOrderBook<B, S>
where
    B: BuySide + Send + 'static,
    S: SellSide + Send + 'static,
{
    // --snip--
 pub fn spawn_bid_ask_service(
        &self,
        max_order_book_depth: usize,
        exchange_stream_buffer: usize,
        price_level_buffer: usize,
        best_n_orders: usize,
        summary_tx: Sender<Summary>,
    ) -> Vec<JoinHandle<Result<(), BidAskServiceError>>> {
        let (price_level_tx, price_level_rx) =
            tokio::sync::mpsc::channel::<PriceLevelUpdate>(price_level_buffer);
        let mut handles = vec![];

        //Spawn the order book service for each exchange, handling order book updates and sending them to the aggregated order book
        for exchange in self.exchanges.iter() {
            handles.extend(exchange.spawn_order_book_service(
                [&self.pair[0], &self.pair[1]],
                max_order_book_depth,
                exchange_stream_buffer,
                price_level_tx.clone(),
            ))
        }

        //Handle order book updates from the exchange streams, aggregating the order book and sending the summary to the gRPC server
        handles.push(self.handle_order_book_updates(
            price_level_rx,
            max_order_book_depth,
            best_n_orders,
            summary_tx,
        ));

        handles
    }

    // --snip--
}
```

The `handle_order_book_updates` function receives the a channel receiver, which feeds all of the price updates from each exchange to the function. Upon each new update, the aggregated order book adds the order to the buy or sell side, updates a summary of the bid-ask spread as well as the top `n` orders from both the bids and the asks and sends this summary through a channel to the gRPC server logic, which streams the summary to any clients that have connected to the gRPC server.


## Server
  
The final module of the application is the `server` module. This module is responsible for managing client connections to the gRPC server and streaming order book summary updates to each client. To start the server, the `spawn_grpc_server` function is called.

```rust
pub fn spawn_grpc_server(
    router: Router,
    socket_address: SocketAddr,
) -> JoinHandle<Result<(), BidAskServiceError>> {
    tokio::spawn(async move {
        router
            .serve(socket_address)
            .await
            .map_err(ServerError::TransportError)?;
        Ok::<_, BidAskServiceError>(())
    })
}

```

Following this, each time a client connects to the server, a receiver channel containing the aggregated order book summary updates is used to serve the summary to the client as a stream.

```rust
#[tonic::async_trait]
impl orderbook_service::orderbook_aggregator_server::OrderbookAggregator
    for OrderbookAggregatorService
{
    type BookSummaryStream =
        Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send + Sync + 'static>>;

    //Send a stream receiver to the client that will send the latest summary of the aggregated order book on each update
    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        tracing::info!("New client connected to book summary stream");

        let rx = self.summary_rx.resubscribe();

        let stream =
            tokio_stream::wrappers::BroadcastStream::new(rx).map(|summary| match summary {
                Ok(summary) => Ok(summary),
                Err(e) => match e {
                    BroadcastStreamRecvError::Lagged(_) => {
                        Err(Status::internal("Stream lagged too far behind"))
                    }
                },
            });

        Ok(Response::new(Box::pin(stream)))
    }
}
```

In summary, the bid ask service initializes an aggregated orderbook,  which manages the order book streams for each exchange, passing the updates through a channel where it then reaches clients connected to the gRPC server. If you would like to see all of these components in action, feel free to check out [bin/bid_ask_service.rs](bin/bid_ask_service.rs).



## Reflection / Post Build Thoughts

After finishing this build, there are a few considerations for upgrades/improvements to the codebase. 


### Concurrency Model
In the initial design of the program, channels were used to pass data between concurrent threads. Channels offer several benefits such as ease of use, particularly for those new to concurrent programming. They fit nicely into the producer-consumer pattern, simplifying the flow of data between different parts of the system. Additionally, since sequential the order book is updated frequently and sequentially, I originally thought that channels could avoid the overhead of locking/unlocking a mutex or lock.

However, there can be potential benefits in considering options like `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for certain situations. With channels, synchronization is implicit. You can send and receive messages, and the channel takes care of the rest. This can be great for preventing data races and other concurrency-related bugs.

In contrast, approaches like `Arc<Mutex<T>>` involve explicit synchronization. You explicitly acquire the lock to a mutex to control access to the data. This gives you more control and can sometimes be more efficient, because you can avoid the overhead of sending and receiving messages.

It would be worthwhile to see how the two compare in performance. 

### Orderbook Data Structures
To represent the buy and sell side of the aggregated orderbook, I chose a `BTreeSet` as a data structure. A `BTreeSet` offers efficient insertion, removal, and retrieval operations with a time complexity of O(log n). In the context of an order book where prices are constantly updating and orders are continuously being added and removed, having efficient operations is important.

The `BTreeSet` also maintains its elements in a sorted order, a feature particularly useful for an order book where we often want to quickly access the highest bid or the lowest ask. This sort of access can be performed in constant time O(1) due to the maintained order.

Even with these benefits, there are areas where the current approach could be improved or modified for potential performance gains or features. Some alternative data structures to consider could be other balanced binary trees like red-black trees, AVL trees, hashmaps, or heaps, each with its own strengths and trade-offs. Additionally, concurrent data structures that allow concurrent read/writes could potentially be used depending on the design. 

Additionally, advanced techniques such as the use of raw pointers could be explored to further optimize performance. Using raw pointers allows for more direct memory management and can reduce the overhead of some operations. However, this comes at the cost of the safety guarantees provided by Rust.

I designed the program so that it is easy to implement the `BuySide` and `SellSide` traits so that other data structures can be used in the orderbook in the future. It would be worth testing the performance of other data structures other than current `BTreeSet` implementation in the future.


### Logging

The current logging system uses the `tracing` library and `tracing-appender` for logging to a file, which provides a good base for collecting and preserving diagnostic information about the application's execution. `tracing` is particularly suited to asynchronous systems.

However, like all systems, there are trade-offs to consider. While tracing is highly performant, the process of logging itself - particularly to a file - can be an I/O-bound operation and could potentially become a bottleneck under high load. Additionally, depending on the data that needs to be logged, the log file could get large very quickly.

Also, with logging, it's crucial to strike a balance between having enough information to diagnose issues and avoid being overwhelmed with noise.

In a larger scale or production system, it would be worth considering sending logs to a database or a real-time monitoring service where proper analytics can be carried out, especially if other systems need access to the logs for analysis.

### Additional Error Handling for Exchanges

Currently, the application implements a basic level of error handling, primarily focusing on capturing and managing errors originating from the exchanges. However, given the wide range of potential errors each exchange could produce, there is room for expanding the error handling mechanism.

Implementing more extensive error handling would not only provide finer control over error reporting but would also be instrumental in diagnosing issues more effectively. This could involve capturing more specific error types from each exchange, handling more specific scenarios and better error propagation throughout the system.
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
The `order_book` module is divided into two main components, price levels and order book structures. The `price_level` sub-module defines the `Bid` and `Ask` structs, lays out the rules for their ordering and contains trait definitions for order types.

The other major component to the `order_book` module are the 'order book structures'.



## Server
  

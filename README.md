# bid_ask_service


`bid_ask_service` is a Rust program designed to aggregate real-time orderbook data from multiple CeFi exchanges in order to publish the bid-ask spread, top `n` bids and top `n` asks via a gRPC server.  

This README provides a step-by-step guide on how to install/run the program. Additionally, this repo also contains a code walkthrough and a post build reflection detailing thoughts, upgrades and improvements to the current codebase. If you would like to jump to any of these sections, feel free to use the table of contents below.


## Table of Contents

1. [Installing](#installing)
2. [Usage](#usage)
3. [Code Walkthrough/Reflections](docs/walkthrough.md)





<br>

## Installing 

Installing the program is quick and easy. First, make sure that you have [Rust installed](https://www.rust-lang.org/tools/install). Once you have Rust installed, you can run the following commands to install the Bid-Ask Service from source.
```bash
git clone https://github.com/0xKitsune/bid_ask_service
cd bid_ask_service
cargo install --path .
```

Since this program uses a gRPC server to stream aggregated order book updates to clients, you will also need to install the `protoc` Protocol Buffers compiler, along with Protocol Buffers resource files.
### Ubuntu
```
sudo apt update && sudo apt upgrade -y
sudo apt install -y protobuf-compiler libprotobuf-dev
```
### Alpine Linux
```
sudo apk add protoc protobuf-dev
```
### macOS
Assuming Homebrew is already installed. (If not, see instructions for installing Homebrew on the Homebrew website.)

```
brew install protobuf
```

With that out of the way, you are ready to run the program!


<br>


## Usage

The Bid Ask Service is configurable via command-line arguments. Here's a rundown of each option:

- `--exchanges, -e`: Specifies the list of exchanges the service should connect to. They should be separated by commas. For example, if you wanted to connect t Binance and Bitstamp, you would specify `--exchanges binance,bitstamp`.

- `--summary_buffer`: Sets the buffer size for the tokio broadcast channel used to stream the aggregated order book to the gRPC server. The default size is 300.

- `--pair, -p`: Specifies the trading pair to listen to updates. Trading pairs should be separated by commas. For example, if you wanted to listen to updates for the ETH/BTC pairing, you would specify `--pair eth,btc`.

- `--order_book_depth`: Determines the max depth of the aggregated order book. This specifies the maximum amount of bids or asks the book will hold. For example, if the depth is set to 20, there will be a maximum of 20 bids and 20 asks in the orderbook. The default depth is 25.

- `--best_n_orders`: Determines the number of best bids and asks to stream via the gRPC server. The default number is 10.

- `--exchange_stream_buffer`: Sets the channel buffer size for streaming live order book data from exchanges. The default size is 100.

- `--price_level_channel_buffer`: Sets the channel buffer size to pass the price level updates from the exchange module to the aggregated order book. The default size is 100.

- `--socket_address`: Specifies the socket address for the gRPC server. The default address is `[::1]:50051`.

- `--level`: Sets the level of logging. The options are trace, debug, info, warn, and error. The default level is info.

- `--log_file_path`: Specifies the path to the output file for logging. The default path is `output.log`.



Here is an example showing how to use the command line arguments together.
```bash
bid_ask_service --exchanges binance,bitstamp --pair eth,btc --order_book_depth 50 --best_n_orders 20 --level info --log_file_path my_log.log
```



### Reflection Section/ What I would change
- Concurrency model to use mutexes instead of channels for exchange/order book communication
- Logging, right now it writes often and could be to a database instead
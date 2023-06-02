# Bid Ask Service


**Bid Ask Service** is a Rust program designed to aggregate real-time orderbook data from multiple CeFi exchanges in order to publish the bid-ask spread, top 10 bids and top 10 asks via a gRPC server.  

This document provides a step-by-step guide on how to install/run the program, add a new exchange, and add different types of order books. The system is designed to be modular so that you can add new exchanges without changing any of the core logic.  This document also contains a robust code walkthrough and a post build reflection detailing thoughts, upgrades and improvements to the current codebase. If you would like to jump to any of these sections, feel free to use the table of contents below.


## Table of Contents

1. [Installing](#installing)
2. [Usage](#usage)
3. [Adding a New Exchange](docs/add_an_exchange.md)
4. [Adding a New Orderbook](docs/add_an_exchange.md)
5. [Code Walkthrough](#walkthrough)
6. [Reflections](#reflections)


## Installing 

Installing the program is quick and easy. First, make sure that you have [Rust installed](https://www.rust-lang.org/tools/install). Once you have Rust installed, you can run the following commands to install the Bid-Ask Service from source.
```bash
git clone https://github.com/0xKitsune/bid_ask_service
cd bid_ask_service
cargo install --path .
```

## Usage

## Walkthrough

## Reflections 

## Notes for post build writeup

### Structure
- Overview, table of contents and hot links to adding an excchange, adding a new order book, upgrades reflection sections (and whatever else makes sense)

### Reflection Section/ What I would change
- Concurrency model to use mutexes instead of channels for exchange/order book communication
- The Bid/Ask Ord trait
- Logging, right now it writes often and could be to a database instead
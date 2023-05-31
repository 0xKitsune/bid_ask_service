# Keyrock Technical Challenge










## TODO: for tonight:
- test feed all of the updates to the aggregated order book 
- add tests for ask order book order
- 
- think through how to cache the top 10 bids/asks and the spread
- gRPC server






### Notes:
- TODO: work on order book structures, handle concurrency where it makes sense
- make some notes highlighting the importance of the ord trait for bid and ask
- look at updating the order book trait to return the top 10 values in bid or ask? or an option if it hasnt changed? 
- Also there should be a depth limit


## Notes for post build writeup

### Reflection Section/ What I would change
- Concurrency model to use mutexes instead of channels for exchange/order book communication
- The Bid/Ask Ord trait
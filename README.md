# Keyrock Technical Challenge










## TODO: for tonight:
- Update all of the ord tests for bid/ask
- feed all of the updates to the aggregated order book test
- think through how to cache the top 10 bids/asks and the spread
- add benches to order book impl
- gRPC server






### Notes:
- TODO: work on order book structures, handle concurrency where it makes sense
- make some notes highlighting the importance of the ord trait for bid and ask
- look at updating the order book trait to return the top 10 values in bid or ask? or an option if it hasnt changed? 
- Also there should be a depth limit
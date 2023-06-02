# Keyrock Technical Challenge





## TODO:
- Add logging 
- address all todos
- update first bid and ask to just be a price to calc the spread or just store the spread and pass the first bid and ask price
- Add more error handling where applicable
- write writeup


## Notes for post build writeup

### Structure
- Overview, table of contents and hot links to adding an excchange, adding a new order book, upgrades reflection sections (and whatever else makes sense)

### Reflection Section/ What I would change
- Concurrency model to use mutexes instead of channels for exchange/order book communication
- The Bid/Ask Ord trait
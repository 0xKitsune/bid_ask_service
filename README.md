# Keyrock Technical Challenge


## Notes for post build writeup

### Structure
- Overview, table of contents and hot links to adding an excchange, adding a new order book, upgrades reflection sections (and whatever else makes sense)

### Reflection Section/ What I would change
- Concurrency model to use mutexes instead of channels for exchange/order book communication
- The Bid/Ask Ord trait
- Logging, right now it writes often and could be to a database instead
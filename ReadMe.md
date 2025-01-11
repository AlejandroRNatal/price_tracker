# Price Tracker
A Command Line Interface for chacking the daily price of cards in a file.

## Usage

```
POKEMON_TCG_API_KEY=YOUR_KEY_HERE cargo run -- price /path/to/cards/for/pricing.txt
```

## Dependencies
- serde
- reqwest
- clap
- tokio
- regex

# Price Tracker
A Command Line Interface for checking the daily price of cards in a file.

The primary CLI component is the main.rs file.
However, the project contains a client subdirectory that stores all the querying logic for the opensource [Pokemon TCG API](https://docs.pokemontcg.io/). There are future plans to convert the client module into its own standalone library.

## CLI Usage

```
POKEMON_TCG_API_KEY=YOUR_KEY_HERE cargo run -- price /path/to/cards/for/pricing.txt
```

> The API Key here is provided by the wonderful people over at [PokemonTCG.io](https://pokemontcg.io/).

## API

### Usage

```rust
use crate::client::client::{ Card, CardSet, Pokemon, Type };

let api_key = "".into();
let api = Pokemon::new(api_key);

let card: Option<Card> = api.find::<Card>("xy1-1".into()).await;
let set: Option<CardSet> = api.find::<CardSet>("xy1-1".into()).await;
let _types: Vec<Type> = api.all::<Type>().await;
```

## Dependencies
- serde
- reqwest
- clap
- tokio
- regex
- sqlx

> NOTE: tokio, clap and regex are used almost exclusively for the CLI whereas reqwest & serde are considered core.

## Dev

Testing command
```bash
POKEMON_TCG_API_KEY=KEY_HERE DATABASE_URL=URL_HERE RUST_BACKTRACE=full cargo test -- --show-output
```

Running the local price tracker with SQLite DB 
```bash
cargo add sqlx-cli
DATABASE_URL=sqlite://URL_HERE sqlx db create #create the DB first
POKEMON_TCG_API_KEY=KEY_HERE DATABASE_URL=sqlite://URL_HERE cargo run -- price /path/to/cards_to_track.txt
```

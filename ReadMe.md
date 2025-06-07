# Price Tracker
A Command Line Interface for checking the daily price of cards in a file.

However, the project contains a client subdirectory that stores all the querying logic for the opensource [Pokemon TCG API](https://docs.pokemontcg.io/). 

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

We require OpenSSL installed if we want to develop Leptos

# macOS
`$ brew install openssl@1.1`

# Arch Linux
`$ sudo pacman -S pkg-config openssl`

# Debian and Ubuntu
`$ sudo apt-get install pkg-config libssl-dev`

# Fedora
`$ sudo dnf install pkg-config openssl-devel`

To compile, you must include wasm32 in your rust toolchain

`rustup target wasm32-unknown-unknown`


---

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

## Running Leptos
```bash
trunk serve --open
```

use std::io::Write;
use std::fs::OpenOptions;
use std::path::PathBuf;

use clap::{arg, Command, Parser};
use chrono::prelude::*;

use regex::Regex;
use serde::{Serialize, Deserialize};

use pokemon_tcg_sdk::{ Client, Query};
use pokemon_tcg_sdk::models::models::{ sv_sets, swsh_sets, Card, CardToPrice, Set};
use pokemon_tcg_sdk::models::errors::Error;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    cards_to_price_path: String,

    #[arg(short, long, default_value_t = format!(""))]
    config_file_path: String,

    #[arg(short, long)]
    card: String,

    #[arg(short, long)]
    market: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    cards_to_price: String,
    path_to_dump_prices: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            cards_to_price: String::from(""),
            path_to_dump_prices: String::from(""),
        }
    }
}

async fn fetch_prices(key: String, cards: Vec<CardToPrice>) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .append(true)
        .open("./prices.txt")
        .map_err(|_| { Error::FailedOpeningFile })?;

    let api = Client::new(key);
    let time = Utc::now();
    for card in cards.iter() {
        let set_id: String = card.setId.clone().unwrap_or("N/A".into());
        let number = card.number.unwrap_or(999999);
        let id = format!("{set_id}-{number}");
        let maybe_card = api.find::<Card>(&id).await;
        if let Some(c) = maybe_card {
            let buff = format!("{} {}", time, c);
            file.write_all(buff.as_str().as_bytes()).map_err(|_| { Error::FailedParsingFile })?;    
        }
    }
    
    Ok(())
}

async fn fetch_card(api: String, card: String) -> Result< (), Error> {
    let client = Client::new(api);
    
    let response = client.find::<Card>(&card ).await;
    let card = response.unwrap();

    let name = card.name.clone().unwrap();

    let mut price = 0.0;
    if let Some(tcg_player) = card.tcgplayer.clone() {
        if let Some( prices) = tcg_player.prices.clone() {
            if let Some(tcg_pricing) = prices.normal.clone() {
                if let Some(market) = tcg_pricing.market.clone() {
                    price = market;
                }
            }
        }
    }

    if let Some(cardmarket) = card.cardmarket.clone() {
        if let Some(prices) = cardmarket.prices.clone() {
            if let Some(p) = prices.averageSellPrice.clone() {
                price = p;
            }
        }
    }

    println!("{name} {price}");
    Ok(())
}

async fn fetch_sets(api_key: String) {
    let mut client = Client::new(api_key);

    let sets = client.all::<Set>().await;

    let sets: Vec<(String, String)> = sets.iter().map(|s| {(s.name.clone().unwrap(), s.id.clone().unwrap())} ).collect();
    sets.iter().for_each(|s| println!("{}\t\t\t{}", s.0, s.1));
}

// Expected Format: [\"]\w+[\"]\s+\w{3}\d{1-3}
fn parse_pricing_file(file: &PathBuf) -> Result<Vec<CardToPrice>, Error> {
    let contents = std::fs::read_to_string(file).map_err(|_| { Error::FailedOpeningFile })?;
    let mut result = Vec::new();
    let pattern = Regex::new(r"'(?P<name>[^']+)'\s+(?P<set_code>\w{3})\s+(?P<number>\d{1,3})").unwrap();

    let sv_sets = sv_sets();
    let sw_sets = swsh_sets();

    for line in contents.lines() {
        let _processed = match pattern.captures(line) {
            Some(captures) => {
                let name = captures.name("name").unwrap().as_str();
                let set_code = captures.name("set_code").unwrap().as_str();
                let number = captures.name("number").unwrap().as_str();
                let id: String = {
                    if let Some(s) = sv_sets.get(set_code) {
                        s.clone()
                    }else if let Some(s) = sw_sets.get(set_code) {
                        s.clone()
                    }else {
                        return Err(Error::MissingSetMapping { set: format!("{set_code}").into() });
                    }
                };
                
                result.push(
                    CardToPrice {
                        name: Some(String::from(name)),
                        ptcgoCode: Some(String::from(set_code)),
                        number: Some(number.parse::<u32>().unwrap_or(99999)),
                        setId: Some(id),
                });
            },
            None => {
                continue;
            },
        };
    }

    Ok(result)
}

fn cli() -> Command {
    Command::new("price_tracker")
            .subcommand(
                Command::new("config")
                        .about("Specifies Configuration File Path to load")
                        .arg(arg!(<CONFIG_PATH> "The path to the configuration file"))
            )
            .subcommand(
                Command::new("price")
                        .about("Specifies Cards To Price File Path")
                        .arg(arg!(<PRICE> "The path to the cards to be priced file"))
            )
            .subcommand(
                Command::new("card")
                .about("Single card to price")
                .arg(arg!(<CARD> "The name of the card to be priced")),
            )
            .subcommand(
                Command::new("sets")
                .about("List sets")
                .arg(arg!(-s --sets ... "Lists all sets Available"))
            )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("config", submatches)) => {
            let config_path = submatches.get_one::<String>("CONFIG_PATH").map(|s| s.to_string());
        },
        Some(("price", submatches)) => {
            let price = submatches.get_one::<String>("PRICE").unwrap();
            let price_path = PathBuf::from(price);

            let cards = parse_pricing_file(&price_path).unwrap();
            let response = fetch_prices(api_key, cards).await;

            match response {
                Ok(_) => { println!("Success: Fetched Prices")},
                Err(e) => { eprintln!("Failed to process prices: {e}"); }
            }
        },
        Some(("card", submatches)) => {
            let card = submatches.get_one::<String>("CARD").unwrap();
            fetch_card(api_key, card.clone()).await;
        },
        Some(("sets", _)) => {
            fetch_sets(api_key).await;
        },
        _ => {
            panic!("Invalid Argument");
        },
    }

    Ok(())
}

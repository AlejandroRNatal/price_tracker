use std::io::Write;
use std::fs::OpenOptions;
use std::path::PathBuf;

use clap::{arg, Command, Parser};
use regex::Regex;
use serde::{Serialize, Deserialize};

use pokemon_tcg_sdk::{Pokemon, Query};
use pokemon_tcg_sdk::models::models::{ sv_sets, swsh_sets, Card, CardToPrice };
use pokemon_tcg_sdk::models::errors::Error;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    cards_to_price_path: String,

    #[arg(short, long, default_value_t = format!(""))]
    config_file_path: String,
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

    let api = Pokemon::new(key);

    for card in cards.iter() {
        let set_id: String = card.setId.clone().unwrap_or("N/A".into());
        let number = card.number.unwrap_or(999999);
        let id = format!("{set_id}-{number}");
        let maybe_card = api.find::<Card>(&id).await;
        if let Some(c) = maybe_card {
            dbg!(&c);
            let buff = format!("{}", c);
            dbg!(&buff);
            file.write_all(buff.as_str().as_bytes()).map_err(|_| { Error::FailedParsingFile })?;    
        }
    }
    
    Ok(())
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
                result.push(CardToPrice{
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
                        .arg_required_else_help(true),
            )
            .subcommand(
                Command::new("price")
                        .about("Specifies Cards To Price File Path")
                        .arg(arg!(<PRICE> "The path to the cards to be priced file"))
                        .arg_required_else_help(true),
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
        _ => {
            panic!("Invalid Argument");
        },
    }

    Ok(())
}

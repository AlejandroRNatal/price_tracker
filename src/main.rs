use std::io::{ Write};
use std::fs::OpenOptions;

use client::client::{Pokemon, Query};
use reqwest::{ Client };

use std::path::PathBuf;
use std::fs::{File};

use clap::{arg, Command, Parser};
use regex::Regex;
use serde::{Serialize, Deserialize};

mod models;
mod client;

use crate::models::models::{ sv_sets, swsh_sets, Card, DataCardMap, CardToPrice };
use crate::client::client::{ POKEMON_TCG_URL, };

const SET_MAPPINGS_DIR: &'static str = "./set_mappings";
const CONFIG_DIR: &'static str = "./config";

const SET_MAPPING_CONVENTION: &'static str = "_set_mappings.json";

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

async fn fetch_prices(key: String, cards: Vec<CardToPrice>) -> Result<(), reqwest::Error> {
    let mut file = OpenOptions::new()
        .append(true)
        .open("./prices.txt")
        .expect("Unable to open file");

    let api = Pokemon::new(key);

    for card in cards.iter() {
        let set_id: String = card.setId.clone().unwrap_or("N/A".into());
        let number = card.number.unwrap_or(999999);
        let id = String::from(format!("{set_id}-{number}"));
        let may_card = api.find::<Card>(id).await;
        if let Some(c) = may_card {
            dbg!(&c);
            let buff = format!("{}", c);
            dbg!(&buff);
            file.write_all(buff.as_str().as_bytes()).expect("Failed to write out pricing to file");    
        }
    }
    
    Ok(())
}

// Expected Format: [\"]\w+[\"]\s+\w{3}\d{1-3}
fn parse_pricing_file(file: &PathBuf) -> Result<Vec<CardToPrice>, String> {
    let contents = std::fs::read_to_string(file).expect("File Does Not Exist");
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
                        return Err(format!("Set Not Found: {set_code}").into());
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
async fn main() -> Result<(), String> {
    let api_key = std::env::var("POKEMON_TCG_API_KEY").expect("Expected API Key Env Var");    
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
            panic!("Invalid Argument")
        },
    }

    Ok(())
}

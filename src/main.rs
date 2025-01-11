use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::fs::OpenOptions;

use reqwest::{ Client };

use std::path::PathBuf;
use std::fs::{File};

use clap::{arg, Command, Parser};
use regex::Regex;
use serde::{Serialize, Deserialize};
use url::Url;

mod models;
use models::cards::{DataCardMap, CardToPrice, SetMapping};


const POKEMON_TCG_URL: &'static str = "https://api.pokemontcg.io/v2/cards";
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

async fn fetch_prices(client: Client, key: String, cards: Vec<CardToPrice>) -> Result<(), reqwest::Error> {
    // let mut file = File::open(PathBuf::from("/Users/magno-mestizo/Documents/GitHub/price_tracker/prices.txt")).expect("Failed to open Pricing file");
    let mut file = OpenOptions::new()
        .append(true)
        .open("/Users/magno-mestizo/Documents/GitHub/price_tracker/prices.txt")
        .expect("Unable to open file");
    
    let query_urls: Vec<String> = 
        cards.into_iter()
             .map(|card| {
                let name = card.name.unwrap_or("N/A".into());
                let _set = card.ptcgoCode.unwrap_or("N/A".into());
                let set_id: String = card.setId.unwrap_or("N/A".into());
                let number = card.number.unwrap_or(999999);

                // let url = String::from(format!("{POKEMON_TCG_URL}?q=name:{name} set.id:{set_id} number:{number}"));
                // let url = String::from(format!("{POKEMON_TCG_URL}/xy1-1"));
                let url = String::from(format!("{POKEMON_TCG_URL}/{set_id}-{number}"));
                // let url = "api.pokemontcg.io/v2/cards?page=1&pageSize=250".to_string();
                dbg!(&url);
                url
              })
             .collect();

    // here we query the server with the baked URLs
    for url in query_urls.into_iter() {
        if url.is_empty() { continue; }

        let response = client.get(url)
                                    .header("X-Api-Key", format!("{key}"))
                                    .header("User-Agent", "Mozilla/5.0")
                                    .send()
                                    .await?
                                    ;
        let data_card_map =  response.json::<DataCardMap>().await?;
    
        dbg!(&data_card_map);
        if let Some(card) = data_card_map.data {
            let buff = format!("{}",card);
            dbg!(&buff);
            file.write_all(buff.as_str().as_bytes()).expect("Failed to write out pricing to file");
        }
        
    }
    
    Ok(())
}

// Expected Format: [\"]\w+[\"]\s+\w{3}\d{1-3}
fn parse_pricing_file(file: &PathBuf) -> Result<Vec<models::cards::CardToPrice>, String> {
    let contents = std::fs::read_to_string(file).expect("File Does Not Exist");
    let mut result = Vec::new();
    let pattern = Regex::new(r"'(?P<name>[^']+)'\s+(?P<set_code>\w{3})\s+(?P<number>\d{1,3})").unwrap();
    let sv_set_name: Vec<SetMapping> = serde_json::from_str(std::fs::read_to_string(PathBuf::from(format!("{SET_MAPPINGS_DIR}/sv_set_mappings.json"))).unwrap().as_str()).unwrap();
    let sw_set_name: Vec<SetMapping> = serde_json::from_str(std::fs::read_to_string(PathBuf::from(format!("{SET_MAPPINGS_DIR}/swsh_set_mappings.json"))).unwrap().as_str()).unwrap();

    for line in contents.lines() {
        let _processed = match pattern.captures(line) {
            Some(captures) => {
                let name = captures.name("name").unwrap().as_str();
                let set_code = captures.name("set_code").unwrap().as_str();
                let number = captures.name("number").unwrap().as_str();
                let id: String = {
                    sv_set_name.iter()
                               .filter(|set| { set.code.clone().unwrap() == set_code })
                               .map(|s| s.id.clone().unwrap())
                               .collect()
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
    let client = Client::new();
    
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("config", submatches)) => {
            let config_path = submatches.get_one::<String>("CONFIG_PATH").map(|s| s.to_string());
        },
        Some(("price", submatches)) => {
            let price = submatches.get_one::<String>("PRICE").unwrap();
            let price_path = PathBuf::from(price);
            let cards = parse_pricing_file(&price_path).unwrap();

            let response = fetch_prices(client, api_key, cards).await;
            match response {
                Ok(_) => {},
                Err(e) => { eprintln!("Failed to process prices: {e}"); }
            }
            
        },
        _ => {
            panic!("Invalid Argument")
        },
    }

    Ok(())
}

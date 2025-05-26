use std::io::{ Write };
use std::fs::OpenOptions;

use std::path::PathBuf;
use std::fs::{File};

use clap::{arg, Command, Parser};
use chrono::{DateTime, TimeZone, Utc};

use regex::Regex;
use reqwest::{ Client };
use serde::{Serialize, Deserialize};


mod models;
mod client;


use crate::models::models::{ sv_sets, swsh_sets,DataCardMap, CardToPrice };
use crate::client::client::{ POKEMON_TCG_URL };


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

#[derive(Clone, FromRow, Debug)]
struct DBCard {
    pub id: u64;
    pub name: String;
    pub card_id: String;
    pub last_price: f64;
    pub date: u64;
}

impl DBCard {
    pub fn new(name: String, card_id: String, date: u64) -> Self {
        Self {
            id: 0,
            name: name,
            card_id: card_id,
            last_price: 0.0,
            date: date,
        }
    } 
}

fn extract_price(card: &Card) -> f64 {
    match card.tcgplayer {
        Some(tcgp) => {
            match tcgp.prices {
                Some(prices) => {
                    if let Some(normal) = prices.normal {
                        if let Some(market) = normal.market {
                            return market as f64;
                        } else {
                            return -999.99;
                        }
                    }
                    /*else if let Some(reverse) = prices.reverseHolofoil {
                        if let Some(market) = reverse.market {
                            return market as f64;
                        } else {
                            return -999.99;
                        }
                    }*/
                    else {
                        return -999.99;
                    }
                },
                None => { -999.99 },
            }
        },
        None => { eprintln!("Not priced: {}", card.name.unwrap()); -999.99 },
    }
}

async fn check_db_data(db_url: String) -> Result<(), &str> {
    let db = SqlitePool::connect(&db_url).await.unwrap();
    let result = sqlx::query(
        "SELECT name
         FROM sqlite_schema
         WHERE type ='table' 
         AND name NOT LIKE 'sqlite_%';",
    )
    .fetch_all(&db)
    .await
    .unwrap();
    
    for (idx, row) in result.iter().enumerate() {
        println!("[{}]: {:?}", idx, row.get::<String, &str>("name"));
    }
    Ok(())
}

async fn insert_card(db_url: String, card: DBCard) -> Result<(), &str> {
    let db = SqlitePool::connect(&db_url).await.unwrap();
    let result = sqlx::query("INSERT INTO cards (name, card_id, last_price, date) VALUES (?)")
        .bind(&card.name)
        .bind(&card.card_id)
        .bind(&card.last_price)
        .bind(&card.date)
        .execute(&db)
        .await
        .unwrap();
    println!("Query result: {:?}", result);
    Ok(())
}

async fn setup_db(db_url: String) -> Result<(), &str> {
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        println!("Creating database {}", &db_url);
        match Sqlite::create_database(&db_url).await {
            Ok(_) => println!("Create DB success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
        let db = SqlitePool::connect(&db_url).await.unwrap();
        let result = sqlx::query("CREATE TABLE IF NOT EXISTS cards (id INTEGER PRIMARY KEY NOT NULL, name VARCHAR(250) NOT NULL, card_id VARCHAR(250) NOT NULL, last_price REAL NOT NULL, date INTEGER);").execute(&db).await.unwrap();
        println!("Create user table result: {:?}", result);
    }

    Ok()
}

async fn fetch_prices(client: Client, key: String, cards: Vec<CardToPrice>) -> Result<(), reqwest::Error> {
    let mut file = OpenOptions::new()
                               .append(true)
                               .open("./prices.txt")
                               .expect("Unable to open file");
    let mut db_cards: HashMap<String, DBCard> = HashMap::new(); 
    let query_urls: Vec<String> = 
        cards.into_iter()
             .map(|card| {
                let name = card.name.unwrap_or("N/A".into());
                let _set = card.ptcgoCode.unwrap_or("N/A".into());
                let set_id: String = card.setId.unwrap_or("N/A".into());
                let number = card.number.unwrap_or(999999);
                let date = Utc::now().timestamp() as u64;
                let db_card = DBCard::new(name, format!("{set_id}-{number}", date));
                db_cards.push(db_card);
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
                             .await?;
        let data_card_map =  response.json::<DataCardMap>().await?;
    
        dbg!(&data_card_map);
        if let Some(card) = data_card_map.data {
            let set_id = format("{}-{}", card.set.unwrap()
                                             .name.unwrap(), card.number.unwrap());
            if let Some(db_card) = db_cards.get_mut(&set_id) {
                db_card.last_price = extract_price(card);
            }

            insert_card(db_card).await;
            
            let buff = format!("{}",card);
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
    let api_key = std::env::var("POKEMON_TCG_API_KEY")
                     .expect("Expected API Key Env Var");
    let client = Client::new();
    
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("config", submatches)) => {
            let config_path = submatches.get_one::<String>("CONFIG_PATH")
                                        .map(|s| s.to_string());
        },
        Some(("price", submatches)) => {
            let price = submatches.get_one::<String>("PRICE").unwrap();
            let price_path = PathBuf::from(price);
            let cards = parse_pricing_file(&price_path).unwrap();

            let response = fetch_prices(client, api_key, cards).await;
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

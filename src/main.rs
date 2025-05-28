use std::io::Write;
use std::fs::OpenOptions;
use std::path::PathBuf;

use clap::{ arg, Command, Parser };
use chrono::prelude::*;
use dotenv::dotenv;

use regex::Regex;

use serde::{ Serialize, Deserialize };

use pokemon_tcg_sdk_rs::{ Client, Query };
use pokemon_tcg_sdk_rs::models::models::{ sv_sets, swsh_sets, DataCardMap, Card, CardToPrice, Set };
use pokemon_tcg_sdk_rs::models::errors::Error;

use sqlx::{ migrate::MigrateDatabase, FromRow, Row, Sqlite, SqlitePool };


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

#[derive(Clone, FromRow, Debug)]
struct DBCard {
    pub id: u64,
    pub name: String,
    pub card_id: String,
    pub last_price: f64,
    pub date: i64,
}

impl DBCard {
    pub fn new(name: String, card_id: String, date: i64) -> Self {
        Self {
            id: 0,
            name: name,
            card_id: card_id,
            last_price: 0.0,
            date: date,
        }
    } 
}

/// Extract the TCG Player price of a Card if it exists.
fn extract_price(card: &Card) -> f64 {
    match &card.tcgplayer {
        Some(tcgp) => {
            match &tcgp.prices {
                Some(prices) => {
                    if let Some(normal) = prices.normal.clone() {
                        if let Some(market) = normal.market.clone() {
                            return market as f64;
                        } else {
                            return -999.99;
                        }
                    }
                    else {
                        return -999.99;
                    }
                },
                None => { -999.99 },
            }
        },
        None => { eprintln!("Not priced: {}", card.name.clone().unwrap()); -999.99 },
    }
}

async fn setup_db(db_url: &String) -> Result<(), String> {
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        println!("Creating database {}", db_url);
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

    Ok(())
}

async fn check_db_data(db_url: &String) -> Result<(), String> {
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

async fn insert_card(db_url: &String, card: DBCard) -> Result<(), String> {
    let db = SqlitePool::connect(&db_url).await.unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS cards (id INTEGER PRIMARY KEY NOT NULL, name VARCHAR(250) NOT NULL, card_id VARCHAR(250) NOT NULL, last_price REAL NOT NULL, date INTEGER);").execute(&db).await.unwrap();
    let result = sqlx::query("INSERT INTO cards (name, card_id, last_price, date) VALUES (?, ?, ?, ?)")
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

async fn fetch_prices(key: String, db_url: &String, cards: Vec<CardToPrice>) -> Result<(), Error> {
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
        
        let number = card.number.unwrap_or(999999);
        let date = Utc::now().timestamp();

        if let Some(c) = maybe_card {
            let mut db_card = DBCard::new(c.name.clone().unwrap(), format!("{id}"), date);
            db_card.last_price = extract_price(&c);
            insert_card(db_url, db_card).await;

            let buff = format!("{} {}", time, c);
            file.write_all(buff.as_str().as_bytes())
                .map_err(|_| { Error::FailedParsingFile })?;
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

    let sets: Vec<(String, String)> = sets.iter()
                                          .map(|s|
                                                { (s.name.clone().unwrap(),
                                                   s.id.clone().unwrap())} )
                                          .collect();
    sets.iter().for_each(|s| println!("{}\t\t\t{}", s.0, s.1));
}

// Expected Format: [\"]\w+[\"]\s+\w{3}\d{1-3}
fn parse_pricing_file(file: &PathBuf) -> Result<Vec<CardToPrice>, Error> {
    let contents = std::fs::read_to_string(file)
                           .map_err(|_| { Error::FailedOpeningFile })?;
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
                    }
                );
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
    let DB_URL = std::env::var("DATABASE_URL").unwrap();
    let api_key = std::env::var("POKEMON_TCG_API_KEY")
                     .map_err(|_| { Error::ApiKeyNotFound })
                     .unwrap();
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("config", submatches)) => {
            let config_path = submatches.get_one::<String>("CONFIG_PATH")
                                        .map(|s| s.to_string());
        },
        Some(("price", submatches)) => {
            setup_db(&DB_URL).await;
            let price = submatches.get_one::<String>("PRICE").unwrap();
            let price_path = PathBuf::from(price);

            let cards = parse_pricing_file(&price_path).unwrap();
            let response = fetch_prices(api_key, &DB_URL, cards).await;

            match response {
                Ok(_) => { println!("Success: Fetched Prices"); },
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_database() {
        let db_name = "card_prices.db".to_string();
        let url = "sqlite://card_prices.db".to_string();
        setup_db(&url).await;

        // check that the DB file exists
        assert!(fs::exists(db_name).unwrap_or(false));
    }
}

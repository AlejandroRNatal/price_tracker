
use std::{ collections::HashMap };
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::models::models::{Card, CardSet, Type };

pub const POKEMON_TCG_URL: &'static str = "https://api.pokemontcg.io/v2/cards";


#[derive(Debug)]
pub struct Pokemon {
    pub client: reqwest::Client,
    pub key: String,
    pub args: HashMap<String, String>,
}

impl Pokemon {
    pub fn new(key: String) -> Self {
            Self{
            client: reqwest::Client::new(),
            key: key,
            args: HashMap::new(),
        }
    }
}

trait Url {
    fn path() -> String;
}

impl Url for Card {
    fn path () -> String { "cards".into() }
}

impl Url for Type {
    fn path() -> String { "types".into() }
}

impl Url for CardSet {
    fn path() -> String {"sets".into() }
}

#[derive(Serialize, Deserialize)]
struct Container<U> {
    pub data: U,
}

#[derive(Serialize, Deserialize)]
struct VecContainer<U> {
    pub data: Vec<U>,
}

impl Query for Pokemon {

    async fn find<T: Url + DeserializeOwned + Clone>(&self, id: String) -> Option<T> {
        let _url: String = T::path();
        let url: String = format!("{POKEMON_TCG_URL}/{_url}/{id}").into();
        let key = self.key.clone();

        let response: Result<reqwest::Response, reqwest::Error> = self.client.get(url)
        .header("X-Api-Key", format!("{key}"))
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await;
        if let Ok(resp) =  response {
            if let Ok(container) = resp.json::<Container<T>>().await {
                return Some(container.data.clone());
            }
        };       

        None
    }
    
    async fn _where<T: Url + DeserializeOwned + Clone>(&mut self, args: HashMap<String, String>) -> Vec<T> {
        self.args.clear();
        for(k, v) in args.iter() {
            self.args.insert(k.clone(), v.clone());
        }

        self.all::<T>().await
    }

    async fn all<T: Url + DeserializeOwned + Clone>(&mut self) -> Vec<T> {
        let mut res = Vec::<T>::new();
        let mut fetch_all = true;

        let key = self.key.clone();
        let u = T::path();
        let url: String = format!("{POKEMON_TCG_URL}/{u}").into();
        
        if let Some(_) = self.args.get(&String::from("page")) {
            fetch_all = false;
        } else { self.args.insert(String::from("page"), String::from("1"));}

        loop {
            let response: Result<reqwest::Response, reqwest::Error> = self.client.get(&url).query(&self.args)
                        .header("X-Api-Key", format!("{key}"))
                        .header("User-Agent", "Mozilla/5.0")
                        .send()
                        .await;
                        if let Ok(resp) =  response {
                            if let Ok(container) = resp.json::<VecContainer<T>>().await {
                                for item in container.data {
                                    res.push(
                                        item.clone()
                                    );
                                }
                                
                                if fetch_all {
                                    if let Some(val) = self.args.get_mut(&String::from("page")) {
                                        let mut num = val.parse().unwrap_or(0);
                                        num += 1;
                                        *val = String::from(format!("{num}"));
                                    }
                                } else {
                                    break;
                                }
                            }
                        }else {
                            break;
                        };    
        }
        self.args.clear();
        res
    }
}



pub trait Query {
    async fn find<T: Url + DeserializeOwned + Clone>(&self, id: String) -> Option<T>;
    async fn _where<T: Url + DeserializeOwned + Clone>(&mut self, args: HashMap<String, String>) -> Vec<T>;
    async fn all<T: Url + DeserializeOwned + Clone>(&mut self) -> Vec<T>; 
    // fn transform<T>(&self, func: Option<F>, response: reqwest::Response)where F: FnOnce() -> T ;
}
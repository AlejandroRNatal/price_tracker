
use std::{ collections::HashMap };
use std::fmt::Debug;
use serde::{ de::DeserializeOwned, Deserialize, Serialize };

use crate::models::models::{ Card, CardSet, Subtype, Supertype, Type };

pub const POKEMON_TCG_URL: &'static str = "https://api.pokemontcg.io/v2";


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

impl Url for Supertype {
    fn path() -> String { "supertypes".into() }
}

impl Url for Subtype {
    fn path() -> String { "subtypes".into() }
}

impl Url for CardSet {
    fn path() -> String {"sets".into() }
}

#[derive(Serialize, Deserialize)]
struct Container<U> {
    pub data: U,
}

#[derive(Debug, Serialize, Deserialize)]
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

        dbg!("{:?}", &response);
        if let Ok(resp) =  response {
            if let Ok(container) = resp.json::<Container<T>>().await {
                return Some(container.data.clone());
            }
        };       

        None
    }
    
    async fn _where<T: Url + DeserializeOwned + Clone + Debug>(&mut self, args: HashMap<String, String>) -> Vec<T> {
        self.args.clear();
        for(k, v) in args.iter() {
            self.args.insert(k.clone(), v.clone());
        }

        self.all::<T>().await
    }

    async fn all<T: Url + DeserializeOwned + Clone + Debug>(&mut self) -> Vec<T> {
        let mut res = Vec::<T>::new();
        let mut fetch_all = false;// TODO: Fix paging, it causes an infinite loop, should be true

        let key = self.key.clone();
        let u = T::path();
        let url: String = format!("{POKEMON_TCG_URL}/{u}").into();

        dbg!(&url);
        
        if let Some(_) = self.args.get(&String::from("page")) {
            fetch_all = true;
        } else {
             self.args.insert(String::from("page"), String::from("1"));
        }

        dbg!(&fetch_all);

        'paging: loop {
            let response: Result<reqwest::Response, reqwest::Error> = if self.args.len() > 0 { 
                self.client.get(&url).query(&self.args)
                        .header("X-Api-Key", format!("{key}"))
                        .header("User-Agent", "Mozilla/5.0")
                        .send()
                        .await
            } else {
                self.client.get(&url)
                        .header("X-Api-Key", format!("{key}"))
                        .header("User-Agent", "Mozilla/5.0")
                        .send()
                        .await
            };

            dbg!(&response);
            if let Ok(resp) =  response {
                dbg!(&resp);
                if let Ok(container) = resp.json::<VecContainer<T>>().await {
                    dbg!(&container);
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
                        } else {
                            fetch_all = false;
                        }
                    } else {
                        break 'paging;
                    }
                }
            }else {
                break 'paging;
            };    
        }
        self.args.clear();
        res
    }
}

pub trait Query {
    async fn find<T: Url + DeserializeOwned + Clone>(&self, id: String) -> Option<T>;
    async fn _where<T: Url + DeserializeOwned + Clone + Debug>(&mut self, args: HashMap<String, String>) -> Vec<T>;
    async fn all<T: Url + DeserializeOwned + Clone + Debug>(&mut self) -> Vec<T>; 
    // fn transform<T>(&self, func: Option<F>, response: reqwest::Response)where F: FnOnce() -> T ;
}

#[cfg(test)]
mod tests {
    use ntest::timeout;

    use super::*;

    #[tokio::test]
    async fn test_find_card_by_id() {
        let key = std::env::var("POKEMON_TCG_API_KEY").expect("Expected API Key Env Var");
        let mut api = Pokemon::new(key);

        let card = api.find::<Card>("xy1-1".into()).await;

        assert!(card.is_some());
        assert!(card.unwrap().id == Some(String::from("xy1-1")))
    }

    #[tokio::test]
    async fn test_find_set_by_id() {
        let key = std::env::var("POKEMON_TCG_API_KEY").expect("Expected API Key Env Var");
        let mut api = Pokemon::new(key);

        let set = api.find::<CardSet>("xy1".into()).await;

        assert!(set.is_some());
        assert!(set.unwrap().id == Some(String::from("xy1")))
    }

    #[tokio::test]
    #[timeout(1000)]
    async fn test_fetch_all_types() {
        let key = std::env::var("POKEMON_TCG_API_KEY").expect("Expected API Key Env Var");
        let mut api = Pokemon::new(key);

        let types = api.all::<Type>().await;
        
        assert!(types.len() > 0);
        assert!(types.contains(&Type("Colorless".into())));
    }

    #[tokio::test]
    #[timeout(1000)]
    async fn test_fetch_all_subtypes() {
        let key = std::env::var("POKEMON_TCG_API_KEY").expect("Expected API Key Env Var");
        let mut api = Pokemon::new(key);

        let types = api.all::<Subtype>().await;
        
        assert!(types.len() > 0);
        assert!(types.contains(&Subtype("Item".into())));
        assert!(types.contains(&Subtype("Supporter".into())));
    }

    #[tokio::test]
    #[timeout(1000)]
    async fn test_fetch_all_supertypes() {
        let key = std::env::var("POKEMON_TCG_API_KEY").expect("Expected API Key Env Var");
        let mut api = Pokemon::new(key);

        let types = api.all::<Supertype>().await;
        
        assert!(types.len() > 0);
        assert!(types.contains(&Supertype("Trainer".into())));
    }
}

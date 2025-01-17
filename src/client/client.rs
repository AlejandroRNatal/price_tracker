
use std::{ collections::HashMap };
use std::fmt::Debug;
use serde::{ de::DeserializeOwned, Deserialize, Serialize };

use crate::models::models::{ Card, CardSet, Rarity, Subtype, Supertype, Type };
use crate::models::errors::Error;

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

impl Url for Rarity {
    fn path() -> String {"rarities".into()}
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

    async fn find<T: Url + DeserializeOwned + Clone>(&self, id: &str) -> Option<T> {
        let _url: String = T::path();
        if _url == "types" || _url == "supertypes" || _url == "subtypes" || _url == "rarities" {
            return None;
        }

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
        
        if let Some(_) = self.args.get(&String::from("page")) {
            fetch_all = true;
        } else {
             self.args.insert(String::from("page"), String::from("1"));
        }

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
    async fn find<T: Url + DeserializeOwned + Clone>(&self, id: &str) -> Option<T>;
    async fn _where<T: Url + DeserializeOwned + Clone + Debug>(&mut self, args: HashMap<String, String>) -> Vec<T>;
    async fn all<T: Url + DeserializeOwned + Clone + Debug>(&mut self) -> Vec<T>; 
}

#[cfg(test)]
mod tests {
    use ntest::timeout;

    use super::*;

    #[tokio::test]
    async fn test_find_card_by_id() {
        let cwd = std::env::current_dir().unwrap();
        let mocks = cwd.join("src/mock/xy1-1.json");
        let mock_data_string = std::fs::read_to_string(mocks).unwrap();
        let expected: Container<Card> = serde_json::from_str(mock_data_string.as_str()).unwrap();

        let key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
        let api = Pokemon::new(key);

        let card = api.find::<Card>("xy1-1".into()).await.unwrap();

        assert!(card.id == expected.data.id);
    }

    #[tokio::test]
    async fn test_find_set_by_id() { 
        let cwd = std::env::current_dir().unwrap();
        let mocks = cwd.join("src/mock/xy1.json");
        let mock_data_string = std::fs::read_to_string(mocks).unwrap();
        let expected: Container<CardSet> = serde_json::from_str(mock_data_string.as_str()).unwrap();

        let key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
        let api = Pokemon::new(key);

        let set = api.find::<CardSet>("xy1".into()).await.unwrap();

        assert!(set.id == expected.data.id);
        assert!(set.name == expected.data.name);
    }

    #[tokio::test]
    #[timeout(1000)]
    async fn test_fetch_all_types() {
        let cwd = std::env::current_dir().unwrap();
        let mocks = cwd.join("src/mock/types.json");
        let mock_data_string = std::fs::read_to_string(mocks).unwrap();
        let expected: VecContainer<Type> = serde_json::from_str(mock_data_string.as_str()).unwrap();

        let key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
        let mut api = Pokemon::new(key);

        let types = api.all::<Type>().await;
        
        assert!(types.len() == expected.data.len());
        assert!(types.contains(&Type("Colorless".into())) == types.contains(&Type("Colorless".into())));
    }

    #[tokio::test]
    #[timeout(1000)]
    async fn test_fetch_all_subtypes() {
        let cwd = std::env::current_dir().unwrap();
        let mocks = cwd.join("src/mock/subtypes.json");
        let mock_data_string = std::fs::read_to_string(mocks).unwrap();
        let expected: VecContainer<Subtype> = serde_json::from_str(mock_data_string.as_str()).unwrap();

        let key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
        let mut api = Pokemon::new(key);

        let types = api.all::<Subtype>().await;
        
        assert!(types.len() == expected.data.len());
        assert!(types.contains(&Subtype("Item".into())) == expected.data.contains(&Subtype("Item".into())));
        assert!(types.contains(&Subtype("Supporter".into())) == expected.data.contains(&Subtype("Item".into())));
    }

    #[tokio::test]
    #[timeout(1000)]
    async fn test_fetch_all_supertypes() {
        let cwd = std::env::current_dir().unwrap();
        let mocks = cwd.join("src/mock/supertypes.json");
        let mock_data_string = std::fs::read_to_string(mocks).unwrap();
        let expected: VecContainer<Supertype> = serde_json::from_str(mock_data_string.as_str()).unwrap();

        let key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
        let mut api = Pokemon::new(key);

        let types = api.all::<Supertype>().await;
        
        assert!(types.len() == expected.data.len());
        assert!(types.contains(&Supertype("Trainer".into())));
    }

    #[tokio::test]
    async fn test_no_find_by_id_on_types() {    
        let key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
        let api = Pokemon::new(key);

        let types = api.find::<Type>("Colorless".into()).await;
        assert!(types.is_none());
    }

    #[tokio::test]
    async fn test_no_find_by_id_on_subtypes() {
        let key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
        let api = Pokemon::new(key);

        let subtypes = api.find::<Subtype>("Supporter".into()).await;
        assert!(subtypes.is_none());
    }

    #[tokio::test]
    async fn test_no_find_by_id_on_supertypes() {
        let key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
        let api = Pokemon::new(key);

        let supertypes = api.find::<Supertype>("Trainer".into()).await;
        assert!(supertypes.is_none());
    }

    #[tokio::test]
    async fn test_no_find_by_id_on_rarities() {
        let key = std::env::var("POKEMON_TCG_API_KEY").map_err(|_| { Error::ApiKeyNotFound }).unwrap();
        let api = Pokemon::new(key);

        let rarities = api.find::<Rarity>("Rare".into()).await;
        assert!(rarities.is_none());
    }
}

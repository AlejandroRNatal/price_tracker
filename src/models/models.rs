#![allow(non_snake_case)]
// We must keep the non-snake-case since the other clients use non-snake-case
use std::{ fmt };
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataCardMap {
    pub data: Option<Card>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: Option<String>,
    pub name: Option<String>,
    pub supertype: Option<String>,
    pub subtypes: Option<Vec<String>>,
    pub hp: Option<String>,
    pub types: Option<Vec<String>>,
    pub evolvesTo: Option<Vec<String>>,
    pub evolvesFrom:  Option<String>,
    pub rules: Option<Vec<String>>,
    pub abilities: Option<Vec<Ability>>,
    pub attacks: Option<Vec<Attack>>,
    pub weaknesses: Option<Vec<Weakness>>,
    pub convertedRetreatCost: Option<u32>,
    pub retreatCost: Option<Vec<String>>,
    pub set: Option<CardSet>,
    pub number: Option<String>,
    pub artist: Option<String>,
    pub rarity: Option<String>,
    pub flavorText: Option<String>,
    pub nationalPokedexNumbers: Option<Vec<u32>>,
    pub legalities: Option<Legalities>,
    pub images: Option<Images>,
    pub tcgplayer: Option<TcgPlayer>,
    pub cardmarket: Option<CardMarket>,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = self.clone();
        let price = extract_price(c.clone());

        write!(f, "{} {} {} {}\n", c.name.unwrap_or("N/A".into()), c.id.unwrap_or("N/A".into()), c.number.unwrap_or("N/A".into()), price)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Ability {
    name: Option<String>,
    text: Option<String>,
    r#type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Attack {
    name: Option<String>,
    cost: Option<Vec<String>>,
    convertedEnergyCost: Option<u32>,
    damage: Option<String>,
    text: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Weakness {
    r#type: Option<String>,
    value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardSet {
    pub id: Option<String>,
    pub name: Option<String>,
    pub series: Option<String>,
    pub printedTotal: Option<u32>,
    pub total: Option<u32>,
    pub legalities: Legalities,
    pub ptcgoCode: Option<String>,
    pub releaseDate: Option<String>,
    pub updatedAt: Option<String>,
    pub images: Option<SetImages>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Legalities {
    unlimited: Option<String>,
    standard: Option<String>,
    expanded: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SetImages {
    symbol: Option<String>,
    logo: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Images {
    small: Option<String>,
    large: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TcgPlayer {
    url: Option<String>,
    updatedAt: Option<String>,
    prices: Option<TcgPlayerPrices>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardMarket {
    url: Option<String>,
    date: Option<String>,
    prices: Option<Prices>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Prices {
    averageSellPrice: Option<f32>,
    lowPrice: Option<f32>,
    trendPrice: Option<f32>,
    germanProLow: Option<f32>,
    suggestedPrice: Option<f32>,
    reverseHoloSell: Option<f32>,
    reverseHoloLow: Option<f32>,
    reverseHoloTrend: Option<f32>,
    lowPriceExPlus: Option<f32>,
    avg1: Option<f32>,
    avg7: Option<f32>,
    avg30: Option<f32>,
    reverseHoloAvg1: Option<f32>,
    reverseHoloAvg7: Option<f32>,
    reverseHoloAvg30: Option<f32>, 
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TcgPlayerPrices {
    normal: Option<TcgPricing>,
    reverseHolofoil: Option<TcgPricing>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TcgPricing {
    low: Option<f32>,
    mid: Option<f32>,
    high: Option<f32>,
    market: Option<f32>,
    directLow: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardToPrice {
    pub ptcgoCode: Option<String>,
    pub number: Option<u32>,
    pub name: Option<String>,
    pub setId: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetMapping {
    name: Option<String>,
    pub id: Option<String>,
    pub code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Type(pub String);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Supertype(pub String);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Subtype(pub String);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Rarity(pub String);

pub fn extract_price(c: Card) -> f32 {
    let res = -99.9;
    if let Some(card_market) = c.cardmarket {
        if let Some(prices) = card_market.prices {
            if let Some(avg) = prices.averageSellPrice {
                return avg;
            }else if let Some(avg1) = prices.avg1 {
                return avg1;
            } else if let Some(avg30) = prices.avg30 {
                return avg30;
            }
        }
    } else if let Some(tcg_player) = c.tcgplayer {
        if let Some(tcg_player_prices) = tcg_player.prices {
            if let Some(normal) = tcg_player_prices.normal {
                if let Some(market) = normal.market {
                    return market;
                }
                else if let Some(mid) = normal.mid {
                    return mid;
                }
                else if let Some(high) = normal.high {
                    return high;
                }
                else if let Some(low) = normal.high {
                    return low;
                }
            }
        }
    };
    res
}

pub fn sv_sets() -> HashMap<String, String> {
    HashMap::from([
        ("SVI".into(), "sv1".into()),
        ("PAL".into(), "sv2".into()),
        ("OBF".into(), "sv3".into()),
        ("MEW".into(), "sv3.5".into()),
        ("PAR".into(), "sv4".into()),
        ("PAF".into(), "sv4.5".into()),
        ("TEF".into(), "sv5".into()),
        ("TWM".into(), "sv6".into()),
        ("SFA".into(), "sv6.5".into()),
        ("SCR".into(), "sv7".into()),
        ("SSP".into(), "sv8".into()),
        ("PRE".into(), "sv8.5".into()),
    ])
}

pub fn swsh_sets() -> HashMap<String, String> {
    HashMap::from([
        ("SWSH".into(), "swsh1".into()),
        ("RCL".into(), "swsh2".into()),
        ("DAA".into(), "swsh3".into()),
        ("CPA".into(), "swsh3.5".into()),
        ("VIV".into(), "swsh4".into()),
        ("SHF".into(), "swsh4.5".into()),
        ("BST".into(), "swsh5".into()),
        ("CRE".into(), "swsh6".into()),
        ("EVS".into(), "swsh7".into()),
        ("CEL".into(), "swsh7.5".into()),
        ("FST".into(), "swsh8".into()),
        ("BRS".into(), "swsh9".into()),
        ("ASR".into(), "swsh10".into()),
        ("PGO".into(), "swsh10.5".into()),
        ("LOR".into(), "swsh11".into()),
        ("SIT".into(), "swsh12".into()),
        ("CRZ".into(), "swsh12.5".into()),
    ])
}
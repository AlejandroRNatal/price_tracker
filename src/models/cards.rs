#![allow(non_snake_case)]
// We must keep the non-snake-case since the other clients use non-snake-case
use std:: fmt;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataCardMap {
    pub data: Option<Card>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    id: Option<String>,
    name: Option<String>,
    supertype: Option<String>,
    subtypes: Option<Vec<String>>,
    hp: Option<String>,
    types: Option<Vec<String>>,
    evolvesTo: Option<Vec<String>>,
    evolvesFrom:  Option<String>,
    rules: Option<Vec<String>>,
    abilities: Option<Vec<Ability>>,
    attacks: Option<Vec<Attack>>,
    weaknesses: Option<Vec<Weakness>>,
    convertedRetreatCost: Option<u32>,
    retreatCost: Option<Vec<String>>,
    set: Option<CardSet>,
    number: Option<String>,
    artist: Option<String>,
    rarity: Option<String>,
    flavorText: Option<String>,
    nationalPokedexNumbers: Option<Vec<u32>>,
    legalities: Option<Legalities>,
    images: Option<Images>,
    tcgplayer: Option<TcgPlayer>,
    cardmarket: Option<CardMarket>,
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
    id: Option<String>,
    name: Option<String>,
    series: Option<String>,
    printedTotal: Option<u32>,
    total: Option<u32>,
    legalities: Legalities,
    ptcgoCode: Option<String>,
    releaseDate: Option<String>,
    updatedAt: Option<String>,
    images: Option<SetImages>,
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
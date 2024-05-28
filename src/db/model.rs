use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, TimeZone, Timelike, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sui_sdk::{rpc_types::SuiCoinMetadata, types::digests};

use crate::utils::convert_chart_timestamp;

pub type CoinType = String;
//토큰 기본 정보
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub icon_url: Option<String>,
    pub description: String,
    pub total_supply: u64,
    pub coin_type: CoinType,
    pub create_time: u64,
    pub recent_trade: Option<u64>,
    pub create_digest: String,
}
impl Token {
    pub fn new(
        metadata: SuiCoinMetadata,
        coin_type: String,
        total_supply: u64,
        create_time: u64,
        create_digest: String,
    ) -> Self {
        Token {
            name: metadata.name,
            symbol: metadata.symbol,
            decimals: metadata.decimals,
            icon_url: metadata.icon_url,
            description: metadata.description,
            total_supply,
            coin_type,
            create_time,
            recent_trade: None,
            create_digest,
        }
    }
    pub fn update_recent_trade(&mut self, timestamp: u64) {
        self.recent_trade = Some(timestamp);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Swap {
    pub account: String,
    pub pool_id: String,
    pub meme_in_amount: u64,
    pub meme_out_amount: u64,
    pub sui_in_amount: u64,
    pub sui_out_amount: u64,
    pub reserve_meme: u64,
    pub reserve_sui: u64,
    pub timestamp: u64,
    pub coin_type: CoinType,
    pub account_meme_balance: u64,
    pub digest: String,
}
impl Swap {
    pub fn new(event: SwapEvent) -> Self {
        Swap {
            account: event.account,
            pool_id: event.pool_id,
            meme_in_amount: event.meme_in_amount.parse().unwrap(),
            meme_out_amount: event.meme_out_amount.parse().unwrap(),
            sui_in_amount: event.sui_in_amount.parse().unwrap(),
            sui_out_amount: event.sui_out_amount.parse().unwrap(),
            reserve_meme: event.reserve_meme.parse().unwrap(),
            reserve_sui: event.reserve_sui.parse().unwrap(),
            timestamp: event.timestamp.unwrap(),
            coin_type: event.coin_type.unwrap(),
            account_meme_balance: event.account_meme_balance.unwrap(),
            digest: event.digest.unwrap(),
        }
    }
}
//Account 정보
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub account: String,
    pub nickname: String,
    pub image_url: String,
    // pub tokens: Vec<CoinType>,
}

//Trading 정보
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeData {
    pub trades: Vec<Trade>,
}

impl TradeData {
    pub fn new() -> Self {
        TradeData { trades: vec![] }
    }

    pub fn add_trade(&mut self, trade: Trade) {
        self.trades.insert(0, trade);
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TradeType {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    #[serde(rename = "account")]
    pub account: String,
    #[serde(rename = "tradeType")]
    pub trade_type: TradeType,
    #[serde(rename = "suiAmount")]
    pub sui_amount: u64,
    #[serde(rename = "updatedTimeStampAt")]
    pub timestamp: u64,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
}

impl Trade {
    pub fn new(event: SwapEvent) -> Self {
        let trade_type = if event.sui_out_amount == "0" && event.meme_in_amount == "0" {
            TradeType::Buy
        } else {
            TradeType::Sell
        };
        match trade_type {
            TradeType::Buy => Trade {
                account: event.account,
                trade_type,
                sui_amount: event.sui_in_amount.parse().unwrap(),
                timestamp: event.timestamp.unwrap(),
                transaction_hash: event.digest.unwrap(),
            },
            TradeType::Sell => Trade {
                account: event.account,
                trade_type,
                sui_amount: event.sui_out_amount.parse().unwrap(),
                timestamp: event.timestamp.unwrap(),
                transaction_hash: event.digest.unwrap(),
            },
        }
    }
}

//@@ Pool 정보
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolInfo {
    pub coin_type: CoinType,
    pub pool_id: String,
    pub reserve_meme: u64,
    pub reserve_sui: u64,
    pub time_stamp: u64,
}

impl PoolInfo {
    pub fn new(event: CreatePoolEvent) -> Self {
        PoolInfo {
            coin_type: event.coin_type.unwrap(),
            pool_id: event.pool_id,
            reserve_meme: event.reserve_meme.parse().unwrap(),
            reserve_sui: event.reserve_sui.parse().unwrap(),
            time_stamp: event.timestamp.unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartData {
    pub charts: Vec<Chart>,
}
impl ChartData {
    pub fn add_chart(&mut self, chart: Chart) {
        self.charts.insert(0, chart);
    }

    pub fn update_latest_chart(&mut self, timestamp: u64, price: Decimal) {
        if let Some(latest_chart) = self.charts.first_mut() {
            if latest_chart.chart_timestamp == convert_chart_timestamp(timestamp) {
                latest_chart.update(price);
            } else {
                self.add_chart(Chart::new(timestamp, price));
            }
        } else {
            self.add_chart(Chart::new(timestamp, price));
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chart {
    #[serde(rename = "timeStamp")]
    pub chart_timestamp: u64,
    #[serde(rename = "highPrice")]
    pub high_price: String,
    #[serde(rename = "lowPrice")]
    pub low_price: String,
    #[serde(rename = "currentPrice")]
    pub current_price: String,
    #[serde(rename = "openPrice")]
    pub open_price: String,
    pub close_price: String,
}

impl Chart {
    pub fn new(timestamp: u64, current_price: Decimal) -> Self {
        let current_price = current_price.to_string();
        Chart {
            chart_timestamp: convert_chart_timestamp(timestamp),
            high_price: current_price.clone(),
            low_price: current_price.clone(),
            current_price: current_price.clone(),
            open_price: current_price.clone(),
            close_price: current_price,
        }
    }
    pub fn update(&mut self, current_price: Decimal) {
        let current_price_str = current_price.to_string();

        if current_price > Decimal::from_str(&self.high_price).unwrap() {
            self.high_price = current_price_str.clone();
        }

        if current_price < Decimal::from_str(&self.low_price).unwrap() {
            self.low_price = current_price_str.clone();
        }

        self.current_price = current_price_str.clone();
        self.close_price = current_price_str;
    }
}

//Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapEvent {
    pub account: String,
    pub pool_id: String,
    pub meme_in_amount: String,
    pub meme_out_amount: String,
    pub sui_in_amount: String,
    pub sui_out_amount: String,
    pub reserve_meme: String,
    pub reserve_sui: String,
    pub timestamp: Option<u64>,
    pub coin_type: Option<String>,
    pub account_meme_balance: Option<u64>,
    pub digest: Option<String>,
    pub current_price: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePoolEvent {
    pub coin_type: Option<CoinType>,
    pub metadata_id: String,
    pub pool_id: String,
    pub reserve_meme: String,
    pub reserve_sui: String,
    pub account: String,
    pub treasury_id: String,
    pub timestamp: Option<u64>,
    pub digest: Option<String>,
}

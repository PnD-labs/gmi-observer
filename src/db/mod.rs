pub mod model;

use crate::db::model::{CreatePoolEvent, PoolInfo};
use crate::env::DBEnv;
use crate::utils::convert_chart_timestamp;
// use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sui_sdk::rpc_types::SuiCoinMetadata;
use sui_sdk::SuiClient;
use surrealdb::sql::Thing;
use surrealdb::Response;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Result, Surreal,
};
use tracing::info;

use self::model::{Chart, ChartData, Swap, SwapEvent, Token, Trade, TradeData};

static POOL_INFO: &str = "POOL_INFO";
static TRADE: &str = "TRADE";
static CHART: &str = "CHART";
static TOKEN: &str = "TOKEN";
static TRADE_DATA: &str = "TRADE_DATA";
static CHART_DATA: &str = "CHART_DATA";
static DB: Lazy<Surreal<Client>> = Lazy::new(|| Surreal::init());

#[derive(Debug, Clone)]
pub struct Database {
    db: Surreal<Client>,
}

impl Database {
    /// 새로운 Database 인스턴스를 생성합니다.
    pub async fn new() -> Result<Self> {
        let env = DBEnv::new();
        DB.connect::<Ws>(&env.db_url).await?;
        DB.signin(Root {
            username: env.username.as_str(),
            password: env.password.as_str(),
        })
        .await?;
        DB.use_ns(env.name_space).use_db(env.db_name).await?;

        Ok(Self { db: DB.clone() })
    }

    // Pool 관련 메서드들

    pub async fn save_pool(&self, create_pool_event: CreatePoolEvent) -> Result<()> {
        let pool = PoolInfo::new(create_pool_event);

        let pool_opt: Option<PoolInfo> = self
            .db
            .create((POOL_INFO, pool.coin_type.as_str()))
            .content(pool)
            .await?;
        // info!("Create Pool {:?}", pool_opt);
        Ok(())
    }

    /// Pool의 reserve 값을 업데이트합니다.
    pub async fn update_pool_info_reserve(&self, swap_event: SwapEvent) -> Result<()> {
        let coin_type = swap_event.coin_type.clone().unwrap();
        let pool_id = swap_event.pool_id.clone();
        let swap = Swap::new(swap_event);
        let pool_info: Option<PoolInfo> = self.db.select((POOL_INFO, coin_type.as_str())).await?;

        match pool_info {
            Some(mut pool_info) => {
                pool_info.reserve_meme = swap.reserve_meme;
                pool_info.reserve_sui = swap.reserve_sui;
                let pool_info_opt: Option<PoolInfo> = self
                    .db
                    .update((POOL_INFO, coin_type))
                    .content(pool_info)
                    .await?;

                // info!("Update PoolInfo {:?}", pool_info_opt)
            }
            None => {
                let new_pool_info = PoolInfo {
                    coin_type: coin_type.clone(),
                    pool_id,
                    reserve_meme: swap.reserve_meme,
                    reserve_sui: swap.reserve_sui,
                    time_stamp: swap.timestamp,
                };
                let pool_info_opt: Option<PoolInfo> = self
                    .db
                    .create((POOL_INFO, coin_type))
                    .content(new_pool_info)
                    .await?;
                // info!("Create PoolInfo {:?}", pool_info_opt)
            }
        }

        Ok(())
    }

    // Swap 관련 메서드들

    /// Swap 데이터를 저장합니다.
    pub async fn save_trade_data(&self, swap_event: SwapEvent) -> Result<()> {
        let coin_type = swap_event.coin_type.clone().unwrap();
        let trade = Trade::new(swap_event);
        info!("coin_type = {:?}\ntrade = {:?}\n \n", coin_type, trade);

        let trades: Option<TradeData> = self.db.select((TRADE_DATA, coin_type.as_str())).await?;

        match trades {
            Some(mut trade_data) => {
                info!("Trade Update");
                trade_data.add_trade(trade);
                let trades: Option<TradeData> = self
                    .db
                    .update((TRADE_DATA, coin_type.as_str()))
                    .content(trade_data)
                    .await?;
                // info!("update trades {:?}", trades)
            }
            None => {
                let mut new_trade_data = TradeData::new();
                new_trade_data.add_trade(trade);
                info!("Trade Create");
                let trades: Option<TradeData> = self
                    .db
                    .create((TRADE_DATA, coin_type.as_str()))
                    .content(new_trade_data)
                    .await?;

                // info!("create trades {:?}", trades)
            }
        }

        Ok(())
    }

    pub async fn save_chart_data(&self, swap_event: SwapEvent) -> Result<()> {
        let coin_type = swap_event.coin_type.clone().unwrap();
        let timestamp = swap_event.timestamp.unwrap();
        let current_price = swap_event.current_price.unwrap();
        // info!("timestamp is = {:?}", timestamp);
        let chart_data: Option<ChartData> =
            self.db.select((CHART_DATA, coin_type.as_str())).await?;

        match chart_data {
            Some(mut chart_data) => {
                info!("Chart Update");
                chart_data.update_latest_chart(timestamp, current_price);
                let chart_opt: Option<ChartData> = self
                    .db
                    .update((CHART_DATA, coin_type.as_str()))
                    .content(chart_data)
                    .await?;

                info!("Chart Update{:?}", chart_opt);
            }

            None => {
                let new_chart_data = ChartData {
                    charts: vec![Chart::new(swap_event.timestamp.unwrap(), current_price)],
                };
                let chart_opt: Option<ChartData> = self
                    .db
                    .create((CHART_DATA, coin_type.as_str()))
                    .content(new_chart_data)
                    .await?;
                info!("Chart Create {:?}", chart_opt);
            }
        }

        Ok(())
    }

    // Token 관련 메서드들

    /// Token 데이터를 저장합니다.
    pub async fn save_token(
        &self,
        create_pool_event: CreatePoolEvent,
        metadata: SuiCoinMetadata,
        coin_type: String,
        total_supply: u64,
    ) -> Result<()> {
        let CreatePoolEvent {
            timestamp, digest, ..
        } = create_pool_event;
        let token = Token::new(
            metadata,
            coin_type.clone(),
            total_supply,
            timestamp.unwrap(),
            digest.unwrap(),
        );

        let token_opt: Option<Token> = self
            .db
            .create(("TOKEN", token.coin_type.as_str()))
            .content(token)
            .await?;
        // info!("Save Token {:?}", token_opt);
        Ok(())
    }

    pub async fn update_token_recent_trade(&self, coin_type: String, timestamp: u64) -> Result<()> {
        let token: Option<Token> = self.db.select((TOKEN, coin_type.as_str())).await?;

        match token {
            Some(mut token) => {
                token.update_recent_trade(timestamp);
                let token_opt: Option<Token> = self
                    .db
                    .update((TOKEN, coin_type.as_str()))
                    .content(token)
                    .await?;
                // info!("Update Token {:?}", token_opt);
            }
            None => {
                info!("Token not found");
            }
        }

        Ok(())
    }
}

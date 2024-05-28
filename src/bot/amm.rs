use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::json;
use std::fmt::format;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use sui_keys::keystore::AccountKeystore;
use sui_move_types::ident_str;
use sui_move_types::language_storage::StructTag;
use sui_sdk::json::SuiJsonValue;
use sui_sdk::rpc_types::{
    SuiObjectDataOptions, SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions,
    SuiTypeTag,
};
use sui_sdk::types::base_types::{ObjectID, ObjectRef, SuiAddress};
use sui_sdk::types::object::Owner;
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_sdk::types::sui_serde::SuiStructTag;
use sui_sdk::types::transaction::{
    Argument, Command, ObjectArg, ProgrammableMoveCall, ProgrammableTransaction, Transaction,
    TransactionData,
};
use sui_sdk::types::{Identifier, TypeTag};
use sui_sdk::SuiClient;
use sui_shared_crypto::intent::Intent;
use tracing::info;

use super::bot::Bot;
use super::meme_coin::MemeCoin;
use crate::env;
const AMM_SWAP_MODULE: &str = "amm_swap";
const AMM_CREATE_POOL: &str = "create_pool";
const AMM_SELL_MEME_COIN: &str = "sell_meme_coin";
const AMM_BUY_MEME_COIN: &str = "buy_meme_coin";

#[derive(Debug, Clone)]
pub struct AMM {
    pub package_id: ObjectID,
    pub config_id: ObjectID,
    pub pool_id: Option<ObjectID>,
}

impl AMM {
    pub fn new() -> Self {
        let amm_env = env::AMMEnv::new();
        let package_id = amm_env.amm_package_id;
        let config_id = amm_env.amm_config_id;

        AMM {
            package_id: ObjectID::from_str(package_id.as_str()).unwrap(),
            config_id: ObjectID::from_str(config_id.as_str()).unwrap(),
            pool_id: None,
        }
    }

    pub async fn create_pool(&mut self, bot: Arc<Bot>, meme: &MemeCoin) -> Result<()> {
        // let call_args = vec![SuiJsonValue::from_object_id(self.self.)];
        let meme_coin_type = format!("{}::meme_coin::MEME_COIN", meme.package_id);
        let meme_coin = bot
            .get_coin(Some(meme_coin_type.clone()))
            .await?
            .next_cursor
            .unwrap();

        let sui_coin = bot.get_coin(None).await?.next_cursor.unwrap();
        bot.request_tokens_from_faucet().await?;

        let call_args = vec![
            SuiJsonValue::from_object_id(meme.treasury_id),
            SuiJsonValue::from_object_id(meme.metadata_id),
            SuiJsonValue::from_object_id(meme_coin),
            SuiJsonValue::from_object_id(sui_coin),
        ];
        let create_pool_call = bot
            .client
            .transaction_builder()
            .move_call(
                bot.address,
                self.package_id,
                AMM_SWAP_MODULE,
                AMM_CREATE_POOL,
                vec![SuiTypeTag::new(meme_coin_type)],
                call_args,
                None,
                10_000_000,
                None,
            )
            .await?;
        let signature =
            bot.key
                .sign_secure(&bot.address, &create_pool_call, Intent::sui_transaction())?;

        let response = bot
            .client
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(create_pool_call, vec![signature]),
                SuiTransactionBlockResponseOptions::default().with_events(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await?;

        let pool_id = ObjectID::from_str(get_pool_id(&response).await?)?;
        self.pool_id = Some(pool_id);
        info!("Create Pool Complete pool ={:?}", pool_id);
        Ok(())
    }
    pub async fn sell_meme_coin(
        &self,
        bot: Arc<Bot>,
        meme_coin: &MemeCoin,
        // amount: u64,
    ) -> Result<()> {
        // let call_args = vec![SuiJsonValue::from_object_id(self.self.)];
        let meme_coin_type = format!("{}::meme_coin::MEME_COIN", meme_coin.package_id);
        bot.split_coin(&meme_coin_type).await?;
        let meme_coin = bot
            .get_coin(Some(meme_coin_type.clone()))
            .await?
            .next_cursor
            .unwrap();
        bot.request_tokens_from_faucet().await?;
        let pool_id = match self.pool_id {
            Some(pool_id) => pool_id,
            None => return Err(anyhow!("pool_id not found")),
        };
        let call_args = vec![
            SuiJsonValue::from_object_id(pool_id),
            SuiJsonValue::from_object_id(self.config_id),
            SuiJsonValue::from_object_id(meme_coin),
        ];

        let create_pool_call = bot
            .client
            .transaction_builder()
            .move_call(
                bot.address,
                self.package_id,
                AMM_SWAP_MODULE,
                AMM_SELL_MEME_COIN,
                vec![SuiTypeTag::new(meme_coin_type)],
                call_args,
                None,
                10_000_000,
                None,
            )
            .await?;
        let signature =
            bot.key
                .sign_secure(&bot.address, &create_pool_call, Intent::sui_transaction())?;
        let response = bot
            .client
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(create_pool_call, vec![signature]),
                SuiTransactionBlockResponseOptions::default().with_events(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await?;

        // info!("Sell Swap Complete Response {:?}", response);
        // println!("{:?}", response.events);
        // println!("{:?}", response.events);
        // println!("Swap response = {:?}", response.events);
        Ok(())
    }
    pub async fn buy_meme_coin(&self, bot: Arc<Bot>, meme_coin: &MemeCoin) -> Result<()> {
        // let call_args = vec![SuiJsonValue::from_object_id(self.self.)];
        let meme_coin_type = format!("{}::meme_coin::MEME_COIN", meme_coin.package_id);

        bot.request_tokens_from_faucet().await?;
        let sui_coin_obj = bot.get_coin(None).await?.next_cursor.unwrap();
        //1을 받으면 0.5씩 계속 buy 하는거

        let pool_id = match self.pool_id {
            Some(pool_id) => pool_id,
            None => return Err(anyhow!("pool_id not found")),
        };

        let call_args = vec![
            SuiJsonValue::from_object_id(pool_id),
            SuiJsonValue::from_object_id(self.config_id),
            SuiJsonValue::from_object_id(sui_coin_obj),
        ];

        let create_pool_call = bot
            .client
            .transaction_builder()
            .move_call(
                bot.address,
                self.package_id,
                AMM_SWAP_MODULE,
                AMM_BUY_MEME_COIN,
                vec![SuiTypeTag::new(meme_coin_type)],
                call_args,
                None,
                10_000_000,
                None,
            )
            .await?;

        let signature =
            bot.key
                .sign_secure(&bot.address, &create_pool_call, Intent::sui_transaction())?;
        let response = bot
            .client
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(create_pool_call, vec![signature]),
                SuiTransactionBlockResponseOptions::default().with_events(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await?;
        // info!("Buy Swap Complete Response {:?}", response);

        Ok(())
    }
}

async fn get_pool_id(response: &SuiTransactionBlockResponse) -> Result<&str> {
    if let Some(events) = &response.events {
        for event in &events.data {
            if event.type_.name == ident_str!("CreatePoolEvent").into() {
                if let Some(pool_id) = event.parsed_json.get("pool_id") {
                    return Ok(pool_id.as_str().unwrap());
                }
            }
        }
    }
    Err(anyhow!("No events found"))
}

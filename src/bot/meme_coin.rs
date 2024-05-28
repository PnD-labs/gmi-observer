use std::{str::FromStr, sync::Arc};

use anyhow::{anyhow, Error, Result};
use serde::Deserialize;
use std::process::Command;

use sui_keys::keystore::AccountKeystore;
use sui_sdk::{
    json::SuiJsonValue,
    rpc_types::SuiTransactionBlockResponseOptions,
    types::{
        base_types::ObjectID, quorum_driver_types::ExecuteTransactionRequestType,
        transaction::Transaction,
    },
};
use sui_shared_crypto::intent::Intent;

use super::bot::Bot;

pub const SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub const MEME_MODULE: &str = "meme_coin";
pub const MEME_INIT_FUNCTION: &str = "init_coin";
pub const MEME_INIT_MINT: &str = "init_mint";
#[derive(Debug, Clone)]
pub struct CoinData {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image_url: String,
}
#[derive(Debug, Deserialize)]
struct Event {
    id: EventId,
    packageId: String,
    transactionModule: String,
    sender: String,
    #[serde(rename = "type")]
    event_type: String,
    parsedJson: MemeCoinParsedJson,
    bcs: String,
}
#[derive(Debug, Deserialize)]
struct EventId {
    txDigest: String,
    eventSeq: String,
}
#[derive(Debug, Deserialize)]
struct MemeCoinParsedJson {
    metadata: String,
    treasury: String,
}

#[derive(Debug, Clone)]
pub struct MemeCoin {
    // pub digest: String,
    pub package_id: ObjectID,
    pub treasury_id: ObjectID,
    pub metadata_id: ObjectID,
}

impl MemeCoin {
    pub async fn new(directory: String) -> Result<Self> {
        println!("Deploying...");

        let output = Command::new("sui")
            .arg("client")
            .arg("publish")
            .arg("--json")
            .current_dir(directory)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_value: serde_json::Value = serde_json::from_str(&stdout)?;

        println!("Deploying complete!");
        let events = json_value
            .get("events")
            .ok_or(anyhow!("Missing 'events' field"))?;

        let events: Vec<Event> = serde_json::from_value(events.clone())?;

        if let Some(event) = events.get(0) {
            // Create and write to the .env file

            Ok(MemeCoin {
                // digest,
                package_id: ObjectID::from_str(event.packageId.as_str())?,
                metadata_id: ObjectID::from_str(event.parsedJson.metadata.as_str())?,
                treasury_id: ObjectID::from_str(event.parsedJson.treasury.as_str())?,
            })
        } else {
            Err(anyhow!("No events found"))
        }
    }

    pub async fn init_coin(&self, bot: Arc<Bot>, coin_data: CoinData) -> Result<(), Error> {
        println!("Init coin!!",);
        let call_args = vec![
            SuiJsonValue::from_object_id(self.treasury_id),
            SuiJsonValue::from_object_id(self.metadata_id),
            SuiJsonValue::from_str(coin_data.symbol.as_str())?,
            SuiJsonValue::from_str(coin_data.name.as_str())?,
            SuiJsonValue::from_str(coin_data.description.as_str())?,
            SuiJsonValue::from_str(coin_data.image_url.as_str())?,
        ];

        let init_coin_call = bot
            .client
            .transaction_builder()
            .move_call(
                bot.address,
                self.package_id,
                MEME_MODULE,
                MEME_INIT_FUNCTION,
                vec![],
                call_args,
                None,
                1_000_000_000,
                None,
            )
            .await?;
        // println!("{:?}", init_coin_call);
        // println!("init_coin_call = {:?}", init_coin_call);
        let signature =
            bot.key
                .sign_secure(&bot.address, &init_coin_call, Intent::sui_transaction())?;
        // println!("signature = {:?}", signature);
        let response = bot
            .client
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(init_coin_call, vec![signature]),
                SuiTransactionBlockResponseOptions::default().with_events(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await?;

        println!("Init coin complete!!");
        // println!("response ={:?}", response);
        Ok(())
    }

    pub async fn init_mint(&self, bot: Arc<Bot>) -> Result<(), Error> {
        println!("Init Min!!");
        let call_args = vec![SuiJsonValue::from_object_id(self.treasury_id)];

        let init_mint_call = bot
            .client
            .transaction_builder()
            .move_call(
                bot.address,
                self.package_id,
                MEME_MODULE,
                MEME_INIT_MINT,
                vec![],
                call_args,
                None,
                4_000_000,
                None,
            )
            .await?;
        let signature =
            bot.key
                .sign_secure(&bot.address, &init_mint_call, Intent::sui_transaction())?;

        let response = bot
            .client
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(init_mint_call, vec![signature]),
                SuiTransactionBlockResponseOptions::default().with_events(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await?;

        println!("Init Mint Complete");
        // println!("init mint response = {:?}", response);
        Ok(())
    }
}

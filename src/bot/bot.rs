use std::sync::Arc;

use crate::sui::get_client;
use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use sui_config::{sui_config_dir, SUI_KEYSTORE_FILENAME};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::{
    rpc_types::{Coin, Page, SuiTransactionBlockResponseOptions},
    types::{
        base_types::{ObjectID, SuiAddress},
        quorum_driver_types::ExecuteTransactionRequestType,
        transaction::Transaction,
    },
    SuiClient,
};
use sui_shared_crypto::intent::Intent;

pub const SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub const BOT_INDEX: u8 = 0;
const SUI_FAUCET: &str = "https://faucet.testnet.sui.io/gas"; //testnet
pub struct Bot {
    pub key: FileBasedKeystore,
    pub address: SuiAddress,
    pub client: Arc<SuiClient>,
}

impl Bot {
    pub fn new(client: Arc<SuiClient>) -> Self {
        let key = FileBasedKeystore::new(&sui_config_dir().unwrap().join(SUI_KEYSTORE_FILENAME))
            .unwrap_or_else(|e| panic!("Failed to initialize keystore: {}", e));

        let addresses = key.addresses();
        if addresses.is_empty() {
            panic!("No addresses found in keystore");
        }

        let address = addresses.get(BOT_INDEX as usize).unwrap();
        println!("address = {:?}", address);

        Bot {
            key,
            address: *address,
            client,
        }
    }
    pub async fn get_coin(&self, coin_type: Option<String>) -> Result<Page<Coin, ObjectID>> {
        let coin = match coin_type {
            Some(coin_type) => {
                self.client
                    .coin_read_api()
                    .get_coins(self.address, Some(coin_type), None, None)
                    .await?
            }
            None => {
                self.client
                    .coin_read_api()
                    .get_coins(self.address, None, None, None)
                    .await?
            }
        };

        Ok(coin)
    }
    pub async fn get_gas_coin(&self) -> Result<Coin> {
        let coins = self
            .client
            .coin_read_api()
            .get_coins(self.address, Some(SUI_COIN_TYPE.to_string()), None, None)
            .await?;
        // println!("{:?}", coin);
        let coin = coins.data.into_iter().next().unwrap();
        Ok(coin)
    }

    pub async fn split_coin(&self, coin_type: &str) -> Result<()> {
        let coins = self
            .client
            .coin_read_api()
            .select_coins(self.address, Some(coin_type.to_string()), 1, vec![])
            .await?;

        let coin = match coins.get(0) {
            Some(coin) => coin,
            None => return Err(anyhow::anyhow!("Coin not found")),
        };

        let split_tx = self
            .client
            .transaction_builder()
            .split_coin(
                self.address,
                coin.coin_object_id,
                vec![coin.balance / 2],
                None,
                1_000_000,
            )
            .await?;

        let signature =
            self.key
                .sign_secure(&self.address, &split_tx, Intent::sui_transaction())?;

        let response = self
            .client
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(split_tx, vec![signature]),
                SuiTransactionBlockResponseOptions::default().with_events(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await?;
        println!("split coin response = {:?}", response);
        Ok(())
    }

    pub async fn merge_coin(&self, coin_type: &str) -> Result<()> {
        let coins = self
            .client
            .coin_read_api()
            .select_coins(self.address, Some(coin_type.to_string()), 1, vec![])
            .await?;
        let coins = self
            .client
            .coin_read_api()
            .get_coins(self.address, Some(coin_type.to_string()), None, None)
            .await?;

        let coin_objs = coins
            .data
            .into_iter()
            .map(|c| c.coin_object_id)
            .collect::<Vec<ObjectID>>();
        let primary_coin = match coin_objs.get(0) {
            Some(coin) => coin,
            None => return Err(anyhow::anyhow!("Coin not found")),
        };
        let coin_to_merge = match coin_objs.get(1) {
            Some(coin) => coin,
            None => return Err(anyhow::anyhow!("Coin not found")),
        };
        let merge_tx = self
            .client
            .transaction_builder()
            .merge_coins(self.address, *primary_coin, *coin_to_merge, None, 10000000)
            .await?;

        let signature =
            self.key
                .sign_secure(&self.address, &merge_tx, Intent::sui_transaction())?;

        let response = self
            .client
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(merge_tx, vec![signature]),
                SuiTransactionBlockResponseOptions::default().with_events(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await?;
        println!("split coin response = {:?}", response);
        Ok(())
    }

    // pub const SUI_FAUCET: &str = "https://faucet.testnet.sui.io/v1/gas"; // testnet faucet
    /// Request tokens from the Faucet for the given address
    #[allow(unused_assignments)]
    pub async fn request_tokens_from_faucet(&self) -> Result<(), anyhow::Error> {
        let address_str = self.address.to_string();
        let json_body = json![{
            "FixedAmountRequest": {
                "recipient": &address_str
            }
        }];

        let client = Client::new();
        let resp = client
            .post(SUI_FAUCET)
            .header("Content-Type", "application/json")
            .json(&json_body)
            .send()
            .await?;

        // println!(
        //     "Faucet request for address {address_str} has status: {}",
        //     resp.status()
        // );

        Ok(())
    }
}

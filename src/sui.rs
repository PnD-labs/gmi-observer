use std::time::Duration;

use sui_sdk::rpc_types::{EventFilter, SuiEvent};
use sui_sdk::{types::base_types::ObjectID, SuiClient, SuiClientBuilder};
use tracing::info;

const SUI_MAINNET_HTTPS: &str = "https://fullnode.mainnet.sui.io:443";
const SUI_MAINNET_WSS: &str = "wss://fullnode.mainnet.sui.io:443";
const SUI_DEVNET_WSS: &str = "wss://fullnode.devnet.sui.io:443";

const SUI_TESTNET_WSS: &str = "wss://testnet.suiet.app:443";

pub async fn get_client(build: &str) -> SuiClient {
    let client = match build {
        "testnet" => SuiClientBuilder::default()
            .ws_url(SUI_TESTNET_WSS)
            .ws_ping_interval(Duration::from_secs(1))
            .build_testnet()
            .await
            .unwrap_or_else(|e| panic!("Failed to build testnet client: {}", e)),
        "devnet" => SuiClientBuilder::default()
            .ws_url(SUI_DEVNET_WSS)
            .ws_ping_interval(Duration::from_secs(1))
            .build_devnet()
            .await
            .unwrap_or_else(|e| panic!("Failed to build devnet client: {}", e)),
        "mainnet" => SuiClientBuilder::default()
            .ws_url(SUI_MAINNET_WSS)
            .ws_ping_interval(Duration::from_secs(1))
            .build(SUI_MAINNET_HTTPS)
            .await
            .unwrap_or_else(|e| panic!("Failed to build mainnet client: {}", e)),
        _ => panic!("Invalid build type: {}", build),
    };
    info!("Sui Client initialized!");
    client
}

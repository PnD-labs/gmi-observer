use crate::{
    db::{
        model::{CreatePoolEvent, SwapEvent},
        Database,
    },
    env,
};

// use crate::bot::amm::AMM;
use anyhow::Result;
use regex::Regex;
use rust_decimal::Decimal;

use std::{str::FromStr, sync::Arc};
use sui_sdk::{
    rpc_types::{EventFilter, SuiEvent, SuiObjectDataOptions},
    types::base_types::{ObjectID, SuiAddress},
    SuiClient,
};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio_stream::StreamExt;
use tracing::info;
pub async fn subscribe_package_event(
    sui: Arc<SuiClient>,
    event_sender: Sender<SuiEvent>,
) -> Result<()> {
    info!("Subscribing to package events start");
    let amm_package_id = ObjectID::from_str(&env::get_env("AMM_PACKAGE_ID"))?;
    loop {
        let mut event_stream = sui
            .event_api()
            .subscribe_event(EventFilter::Package(amm_package_id))
            .await?;

        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(event) => {
                    info!("Event Send");
                    if let Err(e) = event_sender.send(event) {
                        eprintln!("Error sending event: {:?}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving event: {:?}", e);
                    break; // Exit inner loop and restart subscription
                }
            }
        }

        // Log and sleep before retrying subscription
        info!("Re-subscribing to package events");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

pub async fn receive_event(
    sui: Arc<SuiClient>,
    mut event_receiver: Receiver<SuiEvent>,
    db: Arc<Database>,
) -> Result<()> {
    info!("Receive Event Start");

    loop {
        match event_receiver.recv().await {
            Ok(event) => {
                println!("Receive Event!!\n");
                let event_name = event.type_.name.to_string();

                match event_name.as_str() {
                    "CreatePoolEvent" => {
                        if let Err(e) = create_pool_event(sui.clone(), db.clone(), event).await {
                            eprintln!("Error handling CreatePoolEvent: {:?}", e);
                        }
                    }
                    "SwapEvent" => {
                        if let Err(e) = control_swap_event(sui.clone(), db.clone(), event).await {
                            eprintln!("Error handling SwapEvent: {:?}", e);
                        }
                    }
                    _ => {
                        eprintln!("Unknown Event: {}", event_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving from channel: {:?}", e);
            }
        }
    }
}

/// 스왑 이벤트 제어 함수
pub async fn control_swap_event(
    sui: Arc<SuiClient>,
    db: Arc<Database>,
    event: SuiEvent,
) -> Result<()> {
    if let Ok(mut swap_event) = serde_json::from_value::<SwapEvent>(event.parsed_json) {
        let coin_type = get_coin_type_by_pool_id(sui.clone(), swap_event.pool_id.clone()).await?;
        let account_meme_balance = sui
            .coin_read_api()
            .get_balance(
                SuiAddress::from_str(&swap_event.account)?,
                Some(coin_type.clone()),
            )
            .await?
            .total_balance;
        swap_event.account_meme_balance = Some(account_meme_balance as u64);
        swap_event.coin_type = Some(coin_type.clone());
        swap_event.timestamp = event.timestamp_ms.clone();
        // info!(
        //     "rounded timestamp = {:?} , now_timestamp = {:?}",
        //     convert_chart_timestamp(event.timestamp_ms.unwrap()),
        //     event.timestamp_ms.unwrap()
        // );
        swap_event.digest = Some(event.id.tx_digest.to_string());

        //@@ price 구하는 방법은?

        let reserve_meme = Decimal::from_str(&swap_event.reserve_meme)?;
        let reserve_sui = Decimal::from_str(&swap_event.reserve_sui)?;
        let price = reserve_sui / reserve_meme;
        info!("Price is ={:?}", price);
        swap_event.current_price = Some(price);
        db.update_pool_info_reserve(swap_event.clone()).await?;
        db.save_trade_data(swap_event.clone()).await?;
        db.save_chart_data(swap_event.clone()).await?;
        db.update_token_recent_trade(coin_type, event.timestamp_ms.unwrap())
            .await?;
    } else {
        eprintln!("Failed to parse SwapEvent data");
    }
    Ok(())
}

/// 풀 생성 이벤트 제어 함수
pub async fn create_pool_event(
    sui: Arc<SuiClient>,
    db: Arc<Database>,
    event: SuiEvent,
) -> Result<()> {
    info!("Create Pool Event!! \n\n");
    if let Ok(mut create_pool_event) = serde_json::from_value::<CreatePoolEvent>(event.parsed_json)
    {
        let coin_type =
            get_coin_type_by_pool_id(sui.clone(), create_pool_event.pool_id.clone()).await?;

        let coin_metadata = sui
            .coin_read_api()
            .get_coin_metadata(coin_type.to_string())
            .await?;

        let total_supply = sui
            .coin_read_api()
            .get_total_supply(coin_type.to_string())
            .await?
            .value;

        create_pool_event.coin_type = Some(coin_type.to_string());
        create_pool_event.timestamp = event.timestamp_ms;
        create_pool_event.digest = Some(event.id.tx_digest.to_string());

        db.save_pool(create_pool_event.clone()).await?;
        db.save_token(
            create_pool_event,
            coin_metadata.unwrap(),
            coin_type.to_string(),
            total_supply,
        )
        .await?;
    } else {
        eprintln!("Failed to parse CreatePoolEvent data");
    }
    Ok(())
}

async fn get_coin_type_by_pool_id(sui: Arc<SuiClient>, pool_id: String) -> Result<String> {
    let pool_type = sui
        .read_api()
        .get_object_with_options(
            ObjectID::from_str(&pool_id)?,
            SuiObjectDataOptions::new().with_type(),
        )
        .await?
        .data
        .unwrap()
        .object_type()?
        .to_string();
    let re = Regex::new(r"::Pool<([^>]+)>")?;
    let captures = re.captures(&pool_type).unwrap();
    let coin_type = captures.get(1).unwrap().as_str().to_string();
    Ok(coin_type)
}

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tracing::info;

use crate::env::get_env;
use reqwest;
use serde::Deserialize;
#[derive(Deserialize, Debug)]
struct Price {
    price: String,
    conf: String,
    expo: i64,
    publish_time: i64,
}
#[derive(Deserialize, Debug)]
struct ApiResponse {
    id: String,
    price: Price,
    ema_price: Price,
}
static SUI_PRICE: Lazy<Mutex<Option<Price>>> = Lazy::new(|| Mutex::new(None));

pub async fn get_sui_price() -> Result<(), anyhow::Error> {
    loop {
        // let sui_price_id = get_env("SUI_PRICE_ID");
        let sui_price_id = "23d7315113f5b1d3ba7a83604c44b94d79f4fd69af77f804fc7f920a6dc65744";
        println!("{:?}", sui_price_id);
        let url = format!(
            "https://hermes.pyth.network/api/latest_price_feeds?ids[]={}",
            sui_price_id
        );

        let response = reqwest::get(&url).await?;
        println!("{:?}", response.status());
        let price_data: Vec<ApiResponse> = response.json().await?;
        println!("{:?}", price_data);
        // let price_data = response.json().await?;
        // info!("Price is ={:?}", price_data);
    }
    Ok(())
}

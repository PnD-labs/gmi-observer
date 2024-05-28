use anyhow::Result;

use gmi_server::{
    bot::{
        amm::AMM,
        bot::Bot,
        execute_bot,
        meme_coin::{CoinData, MemeCoin},
    },
    db::Database,
    env::{self, get_env},
    observe::{receive_event, subscribe_package_event},
    pyth::get_sui_price,
    sui,
};
use log::info;
use std::{str::FromStr, sync::Arc};
use sui_sdk::{
    rpc_types::{EventFilter, SuiEvent},
    types::{base_types::ObjectID, Identifier},
};
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    task::{self, JoinSet},
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    dotenv::dotenv().ok();
    let db = Arc::new(Database::new().await?);
    // get_sui_price().await?;
    let sui = Arc::new(sui::get_client(get_env("SUI_RPC").as_str()).await);
    // info!("Sui client initialized");
    let mut set = JoinSet::new();
    let (event_sender, event_reciver): (Sender<SuiEvent>, Receiver<SuiEvent>) =
        broadcast::channel(100000);
    set.spawn(execute_bot(sui.clone()));
    set.spawn(subscribe_package_event(sui.clone(), event_sender));
    set.spawn(receive_event(sui.clone(), event_reciver, db.clone()));
    // recieve_event(sui.clone(), event_reciver, db.clone()).await?;

    while let Some(res) = set.join_next().await {
        match res {
            Ok(_) => println!("Task completed successfully"),
            Err(e) => eprintln!("Task panicked: {:?}", e),
        }
    }

    Ok(())
}

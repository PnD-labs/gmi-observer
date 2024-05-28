use std::sync::Arc;

use log::info;
use sui_sdk::SuiClient;

use crate::bot::{
    amm::AMM,
    bot::Bot,
    meme_coin::{CoinData, MemeCoin},
};

pub mod amm;
pub mod bot;
pub mod meme_coin;

pub async fn execute_bot(sui: Arc<SuiClient>) -> Result<(), anyhow::Error> {
    info!("Bot Start!!\n\n");

    let mut amm = AMM::new();
    let bot = Arc::new(Bot::new(sui.clone()));
    let meme_coin = MemeCoin::new("/Users/gyu/project/gmi/contracts/meme_coin".to_string()).await?;
    let coin_data = CoinData{ name: "pepe".to_string(), symbol: "pepe".to_string(), description: "pepe".to_string(), image_url: "data:image/jpeg;base64,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".to_string() };

    let _ = meme_coin.init_coin(bot.clone(), coin_data).await?;
    let _ = meme_coin.init_mint(bot.clone()).await?;
    amm.create_pool(bot.clone(), &meme_coin).await?;
    for _i in 0..100 {
        amm.buy_meme_coin(bot.clone(), &meme_coin).await?;
        amm.buy_meme_coin(bot.clone(), &meme_coin).await?;
        amm.sell_meme_coin(bot.clone(), &meme_coin).await?;
    }
    info!("BOT WORK END!!\n\n");

    Ok(())
}
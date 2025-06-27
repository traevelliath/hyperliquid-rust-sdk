use hyperliquid_sdk::{InfoClient, Message, NetworkType, Subscription, shutdown_signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::str::FromStr;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(tracing_subscriber::fmt::format().compact())
                .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339()),
        )
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::DEBUG.into())
                .from_env_lossy(),
        )
        .init();

    let mut info_client = InfoClient::new(NetworkType::Mainnet).await.unwrap();
    let user =
        alloy::primitives::Address::from_str("0xc64cc00b46101bd40aa1c3121195e85c0b0918d8").unwrap();

    let mut receiver = info_client
        .subscribe(Subscription::ActiveAssetData {
            user,
            coin: "ETH".to_string(),
        })
        .await
        .unwrap();

    //                               duplicate     typo: XRP
    for coin in ["BTC", "ETH", "SOL", "XPR", "SUI", "DOGE"] {
        match info_client
            .subscribe(Subscription::ActiveAssetData {
                user,
                coin: coin.to_string(),
            })
            .await
        {
            Ok(_) => {
                tracing::info!("Subscribed to {}", coin);
            }
            Err(e) => {
                tracing::error!("Error: {}", e);
            }
        }
    }

    loop {
        tokio::select! {
            _ = shutdown_signal() => {
                break;
            }
            Ok(m) = receiver.recv() => {
                match m {
                    Message::ActiveAssetData(active_asset_data) => {
                        tracing::info!(
                            coin = %active_asset_data.data.coin,
                            leverage = ?active_asset_data.data.leverage,
                            max_trade_szs = ?active_asset_data.data.max_trade_szs,
                            available_to_trade = ?active_asset_data.data.available_to_trade,
                            "NEW ACTIVE ASSET DATA:"
                        );
                    }
                    Message::Error(error) => {
                        tracing::error!("Error: {}", error.data);
                    }
                    _ => {}
                }
            }
        }
    }

    info_client.shutdown().await;
}

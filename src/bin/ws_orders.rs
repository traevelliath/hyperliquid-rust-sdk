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
        .subscribe(Subscription::OrderUpdates { user })
        .await
        .unwrap();

    let mut batch_id = 0;
    loop {
        tokio::select! {
            _ = shutdown_signal() => {
                break;
            }
            Ok(m) = receiver.recv() => {
                if let Message::OrderUpdates(order_updates) = m {
                    batch_id += 1;
                    let batch_size = order_updates.data.len();
                    for update in order_updates.data {
                        let order = update.order;
                        let side = if order.side == "B" { "BUY" } else { "SELL" };
                        tracing::info!(
                            coin = %order.coin,
                            status = %update.status,
                            limit_price = %order.limit_px,
                            size = %order.sz,
                            batch_id = %batch_id,
                            batch_size = %batch_size,
                            "NEW {side} ORDER:"
                        );
                    }
                }
            }
        }
    }

    info_client.shutdown().await;
}

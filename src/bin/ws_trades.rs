use hyperliquid_sdk::{InfoClient, Message, NetworkType, Subscription, shutdown_signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let mut receiver = info_client
        .subscribe(Subscription::Trades {
            coin: "ETH".to_string(),
        })
        .await
        .unwrap();

    loop {
        tokio::select! {
            _ = shutdown_signal() => {
                break;
            }
            Ok(m) = receiver.recv() => {
                if let Message::Trades(trades) = m {
                    for trade in trades.data {
                        let side = if trade.side == "B" { "BUY" } else { "SELL" };
                        tracing::info!(
                            coin = %trade.coin,
                            price = %trade.px,
                            size = %trade.sz,
                            "NEW {side} TRADE:"
                        );
                    }
                }
            }
        }
    }

    info_client.shutdown().await;
}

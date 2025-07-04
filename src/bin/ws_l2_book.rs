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

    let mut info_client = InfoClient::builder().network(NetworkType::Mainnet).build();

    let mut receiver = info_client
        .subscribe(Subscription::L2Book {
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
                if let Message::L2Book(l2_book) = m {
                    tracing::info!(
                        coin = %l2_book.data.coin,
                        time = %l2_book.data.time,
                        levels = ?l2_book.data.levels,
                        "NEW L2 BOOK:"
                    );
                }
            }
        }
    }

    info_client.shutdown().await;
}

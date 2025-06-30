use hyperliquid_sdk::{InfoClient, Message, NetworkType, Subscription, shutdown_signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::DEBUG.into())
                .from_env_lossy(),
        )
        .init();

    let mut info_client = InfoClient::builder().network(NetworkType::Mainnet).build();
    let user =
        "0x21b5eb0ca859383f7d8b6906cddb115e92e80913".parse().unwrap();

    let mut receiver = info_client
        .subscribe(Subscription::OrderUpdates { user })
        .await
        .unwrap();

    loop {
        tokio::select! {
            _ = shutdown_signal() => {
                break;
            }
            Ok(m) = receiver.recv() => {
                if let Message::OrderUpdates(order_updates) = m {
                    for update in order_updates.data {
                        let order = update.order;
                        let side = if order.side == "B" { "BUY" } else { "SELL" };
                        tracing::info!(
                            coin = %order.coin,
                            status = %update.status,
                            limit_price = %order.limit_px,
                            size = %order.sz,
                            oid = %order.oid,
                            cloid = ?order.cloid,
                            "NEW {side} ORDER:"
                        );
                    }
                }
            }
        }
    }

    info_client.shutdown().await;
}

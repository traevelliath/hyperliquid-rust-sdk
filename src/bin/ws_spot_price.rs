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
        .subscribe(Subscription::ActiveAssetCtx {
            coin: "@107".to_string(), //spot index for hype token
        })
        .await
        .unwrap();

    loop {
        tokio::select! {
            _ = shutdown_signal() => {
                break;
            }
            Ok(m) = receiver.recv() => {
                if let Message::ActiveSpotAssetCtx(active_spot_asset_ctx) = m {
                    tracing::info!(
                        coin = %active_spot_asset_ctx.data.coin,
                        circulating_supply = %active_spot_asset_ctx.data.ctx.circulating_supply,
                        "NEW SPOT ASSET CTX:"
                    );
                }
            }
        }
    }

    info_client.shutdown().await;
}

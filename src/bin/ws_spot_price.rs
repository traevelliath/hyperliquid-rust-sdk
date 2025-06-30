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
                        mark_px = %active_spot_asset_ctx.data.ctx.shared.mark_px,
                        mid_px = ?active_spot_asset_ctx.data.ctx.shared.mid_px,
                        day_ntl_vlm = %active_spot_asset_ctx.data.ctx.shared.day_ntl_vlm,
                        prev_day_px = %active_spot_asset_ctx.data.ctx.shared.prev_day_px,
                        "NEW SPOT ASSET CTX:"
                    );
                }
            }
        }
    }

    info_client.shutdown().await;
}

use ethers::signers::LocalWallet;
use hyperliquid_sdk::{ExchangeClient, NetworkType};
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
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: LocalWallet =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let exchange_client = ExchangeClient::builder()
        .wallet(wallet)
        .network(NetworkType::Testnet)
        .build()
        .await
        .unwrap();

    let amount = "1"; // 1 USD
    let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";

    let res = exchange_client
        .usdc_transfer(amount, destination, None)
        .await
        .unwrap();

    tracing::info!("Usdc transfer result: {res:?}");
}

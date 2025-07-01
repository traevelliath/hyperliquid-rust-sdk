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
        .vault_address(
            "0x1962905b0a2d0ce7907ae1a0d17f3e4a1f63dfb7"
                .parse()
                .unwrap(),
        )
        .build()
        .await
        .unwrap();

    let usd = 5_000_000; // at least 5 USD
    let is_deposit = true;

    let res = exchange_client
        .vault_transfer(is_deposit, usd)
        .await
        .unwrap();

    tracing::info!("Vault transfer result: {res:?}");
}

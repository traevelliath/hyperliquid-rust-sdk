use alloy::signers::local::PrivateKeySigner;
use hyperliquid_sdk::{ExchangeClient, NetworkType};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let signer: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let exchange_client = ExchangeClient::builder()
        .wallet(signer)
        .network(NetworkType::Testnet)
        .build()
        .await
        .unwrap();

    let usd = "5"; // 5 USD
    let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";

    let res = exchange_client
        .withdraw_from_bridge(usd, destination, None)
        .await
        .unwrap();
    tracing::info!("Withdraw from bridge result: {res:?}");
}

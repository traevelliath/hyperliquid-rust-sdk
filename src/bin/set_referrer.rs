use alloy::signers::local::PrivateKeySigner;
use hyperliquid_sdk::{ExchangeClient, NetworkType};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let exchange_client = ExchangeClient::builder()
        .wallet(wallet)
        .network(NetworkType::Testnet)
        .build()
        .await
        .unwrap();

    let code = "TESTNET".to_string();

    let res = exchange_client.set_referrer(code).await;

    if let Ok(res) = res {
        tracing::info!("Exchange response: {res:#?}");
    } else {
        tracing::info!("Got error: {:#?}", res.err().unwrap());
    }
}

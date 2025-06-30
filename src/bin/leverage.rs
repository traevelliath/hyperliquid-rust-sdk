use alloy::signers::local::PrivateKeySigner;
use hyperliquid_sdk::{ExchangeClient, InfoClient, NetworkType};

#[tokio::main]
async fn main() {
    // Example assumes you already have a position on ETH so you can update margin
    tracing_subscriber::fmt::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let address = wallet.address();
    let exchange_client = ExchangeClient::builder()
        .wallet(wallet)
        .network(NetworkType::Testnet)
        .build()
        .await
        .unwrap();
    let info_client = InfoClient::builder().network(NetworkType::Testnet).build();

    let response = exchange_client
        .update_leverage(5, "ETH", false)
        .await
        .unwrap();
    tracing::info!("Update leverage response: {response:?}");

    let response = exchange_client
        .update_isolated_margin(1.0, "ETH")
        .await
        .unwrap();

    tracing::info!("Update isolated margin response: {response:?}");

    let user_state = info_client.user_state(address).await.unwrap();
    tracing::info!("User state: {user_state:?}");
}

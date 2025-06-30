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

    let max_fee_rate = "0.1%";

    let resp = exchange_client
        .approve_builder_fee(
            "0x1ab189B7801140900C711E458212F9c76F8dAC79"
                .parse()
                .unwrap(),
            max_fee_rate.to_string(),
        )
        .await;
    tracing::info!("resp: {resp:#?}");
}

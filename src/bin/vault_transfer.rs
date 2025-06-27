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

    let exchange_client = ExchangeClient::new(wallet, NetworkType::Testnet, None, None)
        .await
        .unwrap();

    let usd = 5_000_000; // at least 5 USD
    let is_deposit = true;

    let res = exchange_client
        .vault_transfer(
            is_deposit,
            usd,
            Some(
                "0x1962905b0a2d0ce7907ae1a0d17f3e4a1f63dfb7"
                    .parse()
                    .unwrap(),
            ),
            None,
        )
        .await
        .unwrap();

    tracing::info!("Vault transfer result: {res:?}");
}

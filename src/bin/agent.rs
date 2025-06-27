use alloy::signers::local::PrivateKeySigner;

use hyperliquid_sdk::{ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, NetworkType};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let signer: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let exchange_client = ExchangeClient::new(signer, NetworkType::Testnet, None, None)
        .await
        .unwrap();
    /*
        Create a new wallet with the agent.
        This agent cannot transfer or withdraw funds, but can for example place orders.
    */
    let (private_key, response) = exchange_client.approve_agent(None).await.unwrap();
    tracing::info!("Agent creation response: {response:?}");

    let wallet: PrivateKeySigner = PrivateKeySigner::from_signing_key(private_key);

    tracing::info!("Agent address: {:?}", wallet.address());

    let exchange_client = ExchangeClient::new(wallet, NetworkType::Testnet, None, None)
        .await
        .unwrap();

    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1795.0,
        sz: 0.01,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client.order(order, None).await.unwrap();

    tracing::info!("Order placed: {response:?}");
}

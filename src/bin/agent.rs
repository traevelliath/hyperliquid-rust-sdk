use ethers::signers::{LocalWallet, Signer};

use hyperliquid_sdk::{
    ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, LimitTif, NetworkType,
};
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
    let signer: LocalWallet =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let exchange_client = ExchangeClient::builder()
        .wallet(signer)
        .network(NetworkType::Testnet)
        .build()
        .await
        .unwrap();
    /*
        Create a new wallet with the agent.
        This agent cannot transfer or withdraw funds, but can for example place orders.
    */
    let (wallet, response) = exchange_client.approve_agent().await.unwrap();
    tracing::info!("Agent creation response: {response:?}");

    tracing::info!("Agent address: {:?}", wallet.address());

    let exchange_client = ExchangeClient::builder()
        .wallet(wallet)
        .network(NetworkType::Testnet)
        .build()
        .await
        .unwrap();

    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1795.0,
        sz: 0.01,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit { tif: LimitTif::Gtc }),
    };

    let response = exchange_client.order(order).await.unwrap();

    tracing::info!("Order placed: {response:?}");
}

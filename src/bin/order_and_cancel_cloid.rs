use ethers::signers::LocalWallet;
use hyperliquid_sdk::{
    ClientCancelRequestCloid, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient,
    LimitTif, NetworkType,
};
use std::{thread::sleep, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::DEBUG.into())
                .from_env_lossy(),
        )
        .init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    let exchange_client = ExchangeClient::builder()
        .wallet(wallet)
        .network(NetworkType::Testnet)
        .build()
        .await
        .unwrap();

    // Order and Cancel with cloid
    let cloid = Uuid::new_v4();
    let order = ClientOrderRequest {
        asset: "ETH",
        is_buy: true,
        reduce_only: false,
        limit_px: 1800.0,
        sz: 0.01,
        cloid: Some(cloid),
        order_type: ClientOrder::Limit(ClientLimit { tif: LimitTif::Gtc }),
    };

    let response = exchange_client.order(order).await.unwrap();
    tracing::info!("Order placed: {response:?}");

    // So you can see the order before it's cancelled
    sleep(Duration::from_secs(10));

    let cancel = ClientCancelRequestCloid {
        asset: "ETH",
        cloid,
    };

    // This response will return an error if order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = exchange_client.cancel_by_cloid(cancel).await.unwrap();
    tracing::info!("Order potentially cancelled: {response:?}");
}

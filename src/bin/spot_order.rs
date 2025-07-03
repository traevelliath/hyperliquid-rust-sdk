use ethers::signers::LocalWallet;
use hyperliquid_sdk::{
    ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient,
    ExchangeDataStatus, ExchangeResponseStatus, LimitTif, NetworkType,
};
use std::{thread::sleep, time::Duration};
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

    let order = ClientOrderRequest {
        asset: "XYZTWO/USDC".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 0.00002378,
        sz: 1000000.0,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit { tif: LimitTif::Gtc }),
    };

    let response = exchange_client.order(order).await.unwrap();
    tracing::info!("Order placed: {response:?}");

    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    let oid = match status {
        ExchangeDataStatus::Filled(order) => order.oid,
        ExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("Error: {status:?}"),
    };

    // So you can see the order before it's cancelled
    sleep(Duration::from_secs(10));

    let cancel = ClientCancelRequest {
        asset: "HFUN/USDC".to_string(),
        oid,
    };

    // This response will return an error if order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = exchange_client.cancel(cancel).await.unwrap();
    tracing::info!("Order potentially cancelled: {response:?}");
}

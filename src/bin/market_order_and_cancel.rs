use ethers::signers::LocalWallet;

use hyperliquid_sdk::{
    ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, MarketCloseParams,
    MarketOrderParams, NetworkType,
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

    // Market open order
    let market_open_params = MarketOrderParams {
        asset: "ETH",
        is_buy: true,
        sz: 0.01,
        px: None,
        slippage: Some(0.01), // 1% slippage
        cloid: None,
        wallet: None,
    };

    let response = exchange_client
        .market_open(market_open_params)
        .await
        .unwrap();
    tracing::info!("Market open order placed: {response:?}");

    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("Error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    match status {
        ExchangeDataStatus::Filled(order) => tracing::info!("Order filled: {order:?}"),
        ExchangeDataStatus::Resting(order) => tracing::info!("Order resting: {order:?}"),
        _ => panic!("Unexpected status: {status:?}"),
    };

    // Wait for a while before closing the position
    sleep(Duration::from_secs(10));

    // Market close order
    let market_close_params = MarketCloseParams {
        asset: "ETH",
        sz: None, // Close entire position
        px: None,
        slippage: Some(0.01), // 1% slippage
        cloid: None,
        wallet: None,
    };

    let response = exchange_client
        .market_close(market_close_params)
        .await
        .unwrap();
    tracing::info!("Market close order placed: {response:?}");

    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("Error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    match status {
        ExchangeDataStatus::Filled(order) => tracing::info!("Close order filled: {order:?}"),
        ExchangeDataStatus::Resting(order) => tracing::info!("Close order resting: {order:?}"),
        _ => panic!("Unexpected status: {status:?}"),
    };
}

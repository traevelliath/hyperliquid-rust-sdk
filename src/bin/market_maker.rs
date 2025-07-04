use ethers::signers::LocalWallet;
/*
This is an example of a basic market making strategy.

We subscribe to the current mid price and build a market around this price. Whenever our market becomes outdated, we place and cancel orders to renew it.
*/
use hyperliquid_sdk::{MarketMaker, MarketMakerInput};
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
    let market_maker_input = MarketMakerInput {
        asset: "ETH".to_string(),
        target_liquidity: 0.25,
        max_bps_diff: 2,
        half_spread: 1,
        max_absolute_position_size: 0.5,
        decimals: 1,
        wallet,
    };
    MarketMaker::new(market_maker_input).await.start().await
}

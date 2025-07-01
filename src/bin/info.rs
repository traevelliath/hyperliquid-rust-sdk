use ethers::types::H160;
use hyperliquid_sdk::{InfoClient, Interval, NetworkType};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const ADDRESS: &str = "0x97E626F1B3639c6B131527F586A101a56D365F46";

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
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let info_client = InfoClient::builder().network(NetworkType::Testnet).build();
    open_orders_example(&info_client).await;
    user_state_example(&info_client).await;
    user_states_example(&info_client).await;
    recent_trades(&info_client).await;
    meta_example(&info_client).await;
    all_mids_example(&info_client).await;
    user_fills_example(&info_client).await;
    funding_history_example(&info_client).await;
    l2_snapshot_example(&info_client).await;
    candles_snapshot_example(&info_client).await;
    user_token_balances_example(&info_client).await;
    user_fees_example(&info_client).await;
    user_funding_example(&info_client).await;
    spot_meta_example(&info_client).await;
    spot_meta_and_asset_contexts_example(&info_client).await;
    query_order_by_oid_example(&info_client).await;
    query_referral_state_example(&info_client).await;
    historical_orders_example(&info_client).await;
}

fn address() -> H160 {
    ADDRESS.to_string().parse().unwrap()
}

async fn open_orders_example(info_client: &InfoClient) {
    let user = address();

    tracing::info!(
        "Open order data for {user}: {:?}",
        info_client.open_orders(user).await.unwrap()
    );
}

async fn user_state_example(info_client: &InfoClient) {
    let user = address();

    tracing::info!(
        "User state data for {user}: {:?}",
        info_client.user_state(user).await.unwrap()
    );
}

async fn user_states_example(info_client: &InfoClient) {
    let user = address();

    tracing::info!(
        "User state data for {user}: {:?}",
        info_client.user_states(vec![user]).await.unwrap()
    );
}

async fn user_token_balances_example(info_client: &InfoClient) {
    let user = address();

    tracing::info!(
        "User token balances data for {user}: {:?}",
        info_client.user_token_balances(user).await.unwrap()
    );
}

async fn user_fees_example(info_client: &InfoClient) {
    let user = address();

    tracing::info!(
        "User fees data for {user}: {:?}",
        info_client.user_fees(user).await.unwrap()
    );
}

async fn recent_trades(info_client: &InfoClient) {
    let coin = "ETH";

    tracing::info!(
        "Recent trades for {coin}: {:?}",
        info_client.recent_trades(coin).await.unwrap()
    );
}

async fn meta_example(info_client: &InfoClient) {
    tracing::info!("Metadata: {:?}", info_client.meta().await.unwrap());
}

async fn all_mids_example(info_client: &InfoClient) {
    tracing::info!("All mids: {:?}", info_client.all_mids().await.unwrap());
}

async fn user_fills_example(info_client: &InfoClient) {
    let user = address();

    tracing::info!(
        "User fills data for {user}: {:?}",
        info_client.user_fills(user).await.unwrap()
    );
}

async fn funding_history_example(info_client: &InfoClient) {
    let coin = "ETH";

    let start_timestamp = 1690540602225;
    let end_timestamp = 1690569402225;
    tracing::info!(
        "Funding data history for {coin} between timestamps {start_timestamp} and {end_timestamp}: {:?}",
        info_client
            .funding_history(coin, start_timestamp, Some(end_timestamp))
            .await
            .unwrap()
    );
}

async fn l2_snapshot_example(info_client: &InfoClient) {
    let coin = "ETH";

    tracing::info!(
        "L2 snapshot data for {coin}: {:?}",
        info_client.l2_snapshot(coin).await.unwrap()
    );
}

async fn candles_snapshot_example(info_client: &InfoClient) {
    let coin = "ETH";
    let start_timestamp = 1690540602225;
    let end_timestamp = 1690569402225;
    let interval = Interval::OneHour;

    tracing::info!(
        "Candles snapshot data: {:?}",
        info_client
            .candles_snapshot(coin, interval, start_timestamp, end_timestamp)
            .await
            .unwrap()
    );
}

async fn user_funding_example(info_client: &InfoClient) {
    let user = address();
    let start_timestamp = 1690540602225;
    let end_timestamp = 1690569402225;
    tracing::info!(
        "Funding data history for {user} between timestamps {start_timestamp} and {end_timestamp}: {:?}",
        info_client
            .user_funding_history(user, start_timestamp, Some(end_timestamp))
            .await
            .unwrap()
    );
}

async fn spot_meta_example(info_client: &InfoClient) {
    tracing::info!("SpotMeta: {:?}", info_client.spot_meta().await.unwrap());
}

async fn spot_meta_and_asset_contexts_example(info_client: &InfoClient) {
    tracing::info!(
        "SpotMetaAndAssetContexts: {:?}",
        info_client.spot_meta_and_asset_contexts().await.unwrap()
    );
}

async fn query_order_by_oid_example(info_client: &InfoClient) {
    let user = address();
    let oid = 26342632321;
    tracing::info!(
        "Order status for {user} for oid {oid}: {:?}",
        info_client.query_order_by_oid(user, oid).await.unwrap()
    );
}

async fn query_referral_state_example(info_client: &InfoClient) {
    let user = address();
    tracing::info!(
        "Referral state for {user}: {:?}",
        info_client.query_referral_state(user).await.unwrap()
    );
}

async fn historical_orders_example(info_client: &InfoClient) {
    let user = address();
    tracing::info!(
        "Historical orders for {user}: {:?}",
        info_client.historical_orders(user).await.unwrap()
    );
}

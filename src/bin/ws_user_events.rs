use hyperliquid_sdk::{InfoClient, Message, NetworkType, Subscription, UserData, shutdown_signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::str::FromStr;

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

    let mut info_client = InfoClient::new(NetworkType::Mainnet).await.unwrap();
    let user =
        alloy::primitives::Address::from_str("0xc64cc00b46101bd40aa1c3121195e85c0b0918d8").unwrap();

    let mut receiver = info_client
        .subscribe(Subscription::UserEvents { user })
        .await
        .unwrap();

    loop {
        tokio::select! {
            _ = shutdown_signal() => {
                break;
            }
            Ok(m) = receiver.recv() => {
                if let Message::User(user) = m {
                    match user.data {
                        UserData::Fills(fills) => {
                            for fill in fills {
                                tracing::info!(
                                    coin = %fill.coin,
                                    price = %fill.px,
                                    size = %fill.sz,
                                    side = %fill.side,
                                    hash = %fill.hash,
                                    oid = %fill.oid,
                                    "NEW FILL EVENT:"
                                );
                            }
                        }
                        UserData::Funding(funding) => {
                            tracing::info!(
                                coin = %funding.coin,
                                rate = %funding.funding_rate,
                                amount = %funding.usdc,
                                size = %funding.szi,
                                "NEW FUNDING EVENT:"
                            );
                        }
                        UserData::Liquidation(liquidation) => {
                            tracing::info!(
                                lid = %liquidation.lid,
                                liquidator = %liquidation.liquidator,
                                liquidated_user = %liquidation.liquidated_user,
                                liquidated_ntl_pos = %liquidation.liquidated_ntl_pos,
                                liquidated_account_value = %liquidation.liquidated_account_value,
                                "NEW LIQUIDATION EVENT:"
                            );
                        }
                        UserData::NonUserCancel(non_user_cancel) => {
                            for cancel in non_user_cancel {
                                tracing::info!(
                                    coin = %cancel.coin,
                                    oid = %cancel.oid,
                                    "NEW NON-USER CANCEL EVENT:"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    info_client.shutdown().await;
}

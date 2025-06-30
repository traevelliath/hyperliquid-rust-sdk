use hyperliquid_sdk::{InfoClient, Message, NetworkType, Subscription, UserData, shutdown_signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::DEBUG.into())
                .from_env_lossy(),
        )
        .init();

    let mut info_client = InfoClient::builder().network(NetworkType::Mainnet).build();
    let user =
        "0x21b5eb0ca859383f7d8b6906cddb115e92e80913".parse().unwrap();

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

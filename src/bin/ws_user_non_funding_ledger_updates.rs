use hyperliquid_sdk::{InfoClient, Message, NetworkType, Subscription, shutdown_signal};
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

    let mut info_client = InfoClient::builder().network(NetworkType::Mainnet).build();
    let user =
        "0x21b5eb0ca859383f7d8b6906cddb115e92e80913".parse().unwrap();

    let mut receiver = info_client
        .subscribe(Subscription::UserNonFundingLedgerUpdates { user })
        .await
        .unwrap();

    loop {
        tokio::select! {
            _ = shutdown_signal() => {
                break;
            }
            Ok(m) = receiver.recv() => {
                if let Message::UserNonFundingLedgerUpdates(user_non_funding_ledger_updates) = m {
                    for update in user_non_funding_ledger_updates.data.non_funding_ledger_updates {
                        tracing::info!(
                            time = %update.time,
                            delta = ?update.delta,
                            hash = %update.hash,
                            "NEW NON-FUNDING LEDGER UPDATE:"
                        );
                    }
                }
            }
        }
    }

    info_client.shutdown().await;
}

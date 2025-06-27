use crate::consts::*;

use lazy_static::lazy_static;
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

fn now_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time to be after Unix epoch")
        .as_millis() as u64
}

pub(crate) fn next_nonce() -> u64 {
    let nonce = CUR_NONCE.fetch_add(1, Ordering::Relaxed);
    let now_ms = now_timestamp_ms();
    if nonce > now_ms + 1000 {
        tracing::info!("nonce progressed too far ahead {nonce} {now_ms}");
    }
    // more than 300 seconds behind
    if nonce + 300000 < now_ms {
        CUR_NONCE.fetch_max(now_ms + 1, Ordering::Relaxed);

        return now_ms;
    }
    nonce
}

pub(crate) const WIRE_DECIMALS: u8 = 8;

pub(crate) fn float_to_string_for_hashing(x: f64) -> String {
    let mut x = format!("{:.*}", WIRE_DECIMALS.into(), x);
    while x.ends_with('0') {
        x.pop();
    }
    if x.ends_with('.') {
        x.pop();
    }
    if x == "-0" { "0".to_string() } else { x }
}

pub(crate) fn uuid_to_hex_string(uuid: Uuid) -> String {
    let hex_string = uuid
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join("");
    format!("0x{hex_string}")
}

pub fn truncate_float(float: f64, decimals: u32, round_up: bool) -> f64 {
    let pow10 = 10i64.pow(decimals) as f64;
    let mut float = (float * pow10) as u64;
    if round_up {
        float += 1;
    }
    float as f64 / pow10
}

pub fn bps_diff(x: f64, y: f64) -> u16 {
    if x.abs() < EPSILON {
        INF_BPS
    } else {
        (((y - x).abs() / (x)) * 10_000.0) as u16
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BaseUrl {
    Localhost,
    Testnet,
    Mainnet,
}

impl BaseUrl {
    pub(crate) fn get_url(&self) -> url::Url {
        match self {
            BaseUrl::Localhost => url::Url::parse(LOCAL_API_URL).unwrap(),
            BaseUrl::Mainnet => url::Url::parse(MAINNET_API_URL).unwrap(),
            BaseUrl::Testnet => url::Url::parse(TESTNET_API_URL).unwrap(),
        }
    }

    pub(crate) fn get_ws_url(&self) -> url::Url {
        match self {
            BaseUrl::Mainnet => url::Url::parse(MAINNET_WS_URL).unwrap(),
            BaseUrl::Testnet => url::Url::parse(TESTNET_WS_URL).unwrap(),
            _ => panic!("Unsupported network"),
        }
    }
}

lazy_static! {
    static ref CUR_NONCE: AtomicU64 = AtomicU64::new(now_timestamp_ms());
}

/// Utility function for graceful shutdown handling in websocket examples
pub async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_to_string_for_hashing_test() {
        assert_eq!(float_to_string_for_hashing(0.), "0".to_string());
        assert_eq!(float_to_string_for_hashing(-0.), "0".to_string());
        assert_eq!(float_to_string_for_hashing(-0.0000), "0".to_string());
        assert_eq!(
            float_to_string_for_hashing(0.00076000),
            "0.00076".to_string()
        );
        assert_eq!(
            float_to_string_for_hashing(0.00000001),
            "0.00000001".to_string()
        );
        assert_eq!(
            float_to_string_for_hashing(0.12345678),
            "0.12345678".to_string()
        );
        assert_eq!(
            float_to_string_for_hashing(87654321.12345678),
            "87654321.12345678".to_string()
        );
        assert_eq!(
            float_to_string_for_hashing(987654321.00000000),
            "987654321".to_string()
        );
        assert_eq!(
            float_to_string_for_hashing(87654321.1234),
            "87654321.1234".to_string()
        );
        assert_eq!(float_to_string_for_hashing(0.000760), "0.00076".to_string());
        assert_eq!(float_to_string_for_hashing(0.00076), "0.00076".to_string());
        assert_eq!(
            float_to_string_for_hashing(987654321.0),
            "987654321".to_string()
        );
        assert_eq!(
            float_to_string_for_hashing(987654321.),
            "987654321".to_string()
        );
    }
}

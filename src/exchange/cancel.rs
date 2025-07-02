use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ClientCancelRequest<'a> {
    pub asset: &'a str,
    pub oid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelRequest {
    #[serde(rename = "a", alias = "asset")]
    pub asset: u32,
    #[serde(rename = "o", alias = "oid")]
    pub oid: u64,
}

#[derive(Debug)]
pub struct ClientCancelRequestCloid<'a> {
    pub asset: &'a str,
    pub cloid: crate::Cloid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelRequestCloid {
    pub asset: u32,
    pub cloid: String,
}

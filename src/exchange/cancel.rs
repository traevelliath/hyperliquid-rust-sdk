use serde::Serialize;

#[derive(Debug)]
pub struct ClientCancelRequest {
    pub asset: String,
    pub oid: u64,
}

#[derive(Serialize, Debug, Clone)]
pub struct CancelRequest {
    #[serde(rename = "a", alias = "asset")]
    pub asset: u32,
    #[serde(rename = "o", alias = "oid")]
    pub oid: u64,
}

#[derive(Debug)]
pub struct ClientCancelRequestCloid {
    pub asset: String,
    pub cloid: crate::Cloid,
}

#[derive(Serialize, Debug, Clone)]
pub struct CancelRequestCloid {
    pub asset: u32,
    pub cloid: String,
}

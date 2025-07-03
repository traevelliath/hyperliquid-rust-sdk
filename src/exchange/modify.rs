use super::{ClientOrderRequest, order::OrderRequest};

#[derive(serde::Serialize, Debug, Clone)]
pub enum ModifyId {
    Oid(u64),
    Cloid(String),
}

#[derive(Debug)]
pub struct ClientModifyRequest {
    pub id: ModifyId,
    pub order: ClientOrderRequest,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct ModifyRequest {
    pub id: ModifyId,
    pub order: OrderRequest,
}

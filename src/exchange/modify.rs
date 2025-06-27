use super::{ClientOrderRequest, order::OrderRequest};
use serde::Serialize;

#[derive(Debug)]
pub struct ClientModifyRequest {
    pub oid: u64,
    pub order: ClientOrderRequest,
}

#[derive(Serialize, Debug, Clone)]
pub struct ModifyRequest<'a> {
    pub oid: u64,
    pub order: OrderRequest<'a>,
}

use super::{ClientOrderRequest, order::OrderRequest};
use serde::Serialize;

#[derive(Debug)]
pub struct ClientModifyRequest<'a> {
    pub oid: u64,
    pub order: ClientOrderRequest<'a>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ModifyRequest {
    pub oid: u64,
    pub order: OrderRequest,
}

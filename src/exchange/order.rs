use crate::{
    errors::Error,
    helpers::{float_to_string_for_hashing, uuid_to_hex_string},
    errors::Result,
};

#[derive(Default, serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuilderInfo {
    #[serde(rename = "b")]
    pub builder: String,
    #[serde(rename = "f")]
    pub fee: u64,
}

#[derive(serde::Serialize, Debug, Clone)]
pub enum LimitTif {
    Alo,
    Ioc,
    Gtc,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TriggerTpsl {
    Tp,
    Sl,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct Limit {
    pub tif: LimitTif,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    pub is_market: bool,
    pub trigger_px: String,
    pub tpsl: TriggerTpsl,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Order {
    Limit(Limit),
    Trigger(Trigger),
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    #[serde(rename = "a", alias = "asset")]
    pub asset: u32,
    #[serde(rename = "b", alias = "isBuy")]
    pub is_buy: bool,
    #[serde(rename = "p", alias = "limitPx")]
    pub limit_px: String,
    #[serde(rename = "s", alias = "sz")]
    pub sz: String,
    #[serde(rename = "r", alias = "reduceOnly", default)]
    pub reduce_only: bool,
    #[serde(rename = "t", alias = "orderType")]
    pub order_type: Order,
    #[serde(rename = "c", alias = "cloid", skip_serializing_if = "Option::is_none")]
    pub cloid: Option<String>,
}

#[derive(Debug)]
pub struct ClientLimit {
    pub tif: LimitTif,
}

#[derive(Debug)]
pub struct ClientTrigger {
    pub is_market: bool,
    pub trigger_px: f64,
    pub tpsl: TriggerTpsl,
}

#[derive(Debug)]
pub struct MarketOrderParams<'a> {
    pub asset: &'a str,
    pub is_buy: bool,
    pub sz: f64,
    pub px: Option<f64>,
    pub slippage: Option<f64>,
    pub cloid: Option<uuid::Uuid>,
    pub wallet: Option<&'a ethers::signers::LocalWallet>,
}

#[derive(Debug)]
pub struct MarketCloseParams<'a> {
    pub asset: &'a str,
    pub sz: Option<f64>,
    pub px: Option<f64>,
    pub slippage: Option<f64>,
    pub cloid: Option<uuid::Uuid>,
    pub wallet: Option<&'a ethers::signers::LocalWallet>,
}

#[derive(Debug)]
pub enum ClientOrder {
    Limit(ClientLimit),
    Trigger(ClientTrigger),
}

#[derive(Debug)]
pub struct ClientOrderRequest<'a> {
    pub asset: &'a str,
    pub is_buy: bool,
    pub reduce_only: bool,
    pub limit_px: f64,
    pub sz: f64,
    pub cloid: Option<uuid::Uuid>,
    pub order_type: ClientOrder,
}

impl<'a> ClientOrderRequest<'a> {
    pub(crate) fn to_order_request(
        &self,
        coin_to_asset: &scc::HashMap<String, u32>,
    ) -> Result<OrderRequest> {
        let order_type = match &self.order_type {
            ClientOrder::Limit(limit) => Order::Limit(Limit {
                tif: limit.tif.clone(),
            }),
            ClientOrder::Trigger(trigger) => Order::Trigger(Trigger {
                trigger_px: float_to_string_for_hashing(trigger.trigger_px),
                is_market: trigger.is_market,
                tpsl: trigger.tpsl.clone(),
            }),
        };
        let asset = coin_to_asset
            .read(self.asset, |_, asset| *asset)
            .ok_or(Error::AssetNotFound)?;

        let cloid = self.cloid.map(uuid_to_hex_string);

        Ok(OrderRequest {
            asset,
            is_buy: self.is_buy,
            reduce_only: self.reduce_only,
            limit_px: float_to_string_for_hashing(self.limit_px),
            sz: float_to_string_for_hashing(self.sz),
            order_type,
            cloid,
        })
    }
}

impl std::str::FromStr for TriggerTpsl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "tp" => Ok(TriggerTpsl::Tp),
            "sl" => Ok(TriggerTpsl::Sl),
            s => Err(Error::InvalidTriggerTpsl(s.to_string())),
        }
    }
}

impl std::str::FromStr for LimitTif {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Alo" => Ok(LimitTif::Alo),
            "Ioc" => Ok(LimitTif::Ioc),
            "Gtc" => Ok(LimitTif::Gtc),
            s => Err(Error::InvalidLimitTif(s.to_string())),
        }
    }
}

use crate::signature::sign_typed_data;
use crate::{
    BulkCancelCloid, Error, ExchangeResponseStatus,
    exchange::{
        ClientCancelRequest, ClientOrderRequest,
        actions::{
            ApproveAgent, ApproveBuilderFee, BulkCancel, BulkModify, BulkOrder, Grouping,
            SetReferrer, UpdateIsolatedMargin, UpdateLeverage, UsdSend,
        },
        cancel::{CancelRequest, CancelRequestCloid},
        modify::{ClientModifyRequest, ModifyRequest},
    },
    helpers::{next_nonce, uuid_to_hex_string},
    info::client::InfoClient,
    meta::Meta,
    prelude::*,
    req::{Endpoint, HttpClient},
    signature::sign_l1_action,
};
use crate::{ClassTransfer, SpotSend, SpotUser, VaultTransfer, Withdraw3};
use crate::{ExchangeClientBuilder, req::NetworkType};

use ethers::{
    signers::{LocalWallet, Signer},
    types::{H160, Signature},
};

use super::api::{Actions, ExchangePayload};
use super::cancel::ClientCancelRequestCloid;
use super::order::{BuilderInfo, LimitTif, MarketCloseParams, MarketOrderParams};
use super::{ClientLimit, ClientOrder};

#[derive(Debug, Clone)]
pub struct ExchangeClient {
    pub http_client: HttpClient,
    pub wallet: std::sync::Arc<LocalWallet>,
    pub vault_address: Option<H160>,
    pub meta: std::sync::Arc<scc::HashMap<String, u32>>,
    pub coin_to_asset: std::sync::Arc<scc::HashMap<String, u32>>,
}

impl ExchangeClient {
    pub fn builder() -> ExchangeClientBuilder {
        ExchangeClientBuilder::default()
    }
}

impl ExchangeClient {
    pub(crate) async fn new(
        http_client: HttpClient,
        wallet: LocalWallet,
        network: NetworkType,
        meta: Option<Meta>,
        vault_address: Option<H160>,
    ) -> Result<ExchangeClient> {
        let info = InfoClient::builder()
            .http_client(http_client.client.clone())
            .network(network)
            .build();
        let meta = if let Some(meta) = meta {
            meta
        } else {
            info.meta().await?
        };
        let mut perp_map = {
            let iter = meta
                .universe
                .iter()
                .enumerate()
                .map(|(idx, asset)| (asset.name.clone(), idx as u32));
            scc::HashMap::from_iter(iter)
        };
        info.spot_meta()
            .await?
            .add_to_coin_to_asset_map(&mut perp_map);

        let meta = {
            let iter = meta
                .universe
                .into_iter()
                .map(|asset| (asset.name, asset.sz_decimals));
            scc::HashMap::from_iter(iter)
        };
        Ok(ExchangeClient {
            http_client,
            wallet: std::sync::Arc::new(wallet),
            vault_address,
            meta: std::sync::Arc::new(meta),
            coin_to_asset: std::sync::Arc::new(perp_map),
        })
    }

    async fn post(
        &self,
        action: serde_json::Value,
        signature: Signature,
        nonce: u64,
    ) -> Result<ExchangeResponseStatus> {
        let exchange_payload = ExchangePayload {
            action,
            signature,
            nonce,
            vault_address: self.vault_address,
        };

        let res = serde_json::to_string(&exchange_payload)
            .map_err(|e| Error::JsonParse(e.to_string()))?;
        tracing::debug!("Sending request {res:?}");

        let output = &self
            .http_client
            .post(Endpoint::Exchange, res)
            .await
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        serde_json::from_str(output).map_err(|e| Error::JsonParse(e.to_string()))
    }

    pub async fn usdc_transfer(
        &self,
        amount: &str,
        destination: &str,
        signer: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        let signer = signer.unwrap_or(&self.wallet);
        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let timestamp = next_nonce();
        let usd_send = UsdSend {
            signature_chain_id: 421614.into(),
            hyperliquid_chain,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
        };
        let signature = sign_typed_data(&usd_send, signer)?;
        let action = serde_json::to_value(Actions::UsdSend(usd_send))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp).await
    }

    pub async fn class_transfer(
        &self,
        usdc: f64,
        to_perp: bool,
        signer: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        // payload expects usdc without decimals
        let usdc = (usdc * 1e6).round() as u64;
        let signer = signer.unwrap_or(&self.wallet);

        let timestamp = next_nonce();

        let action = Actions::SpotUser(SpotUser {
            class_transfer: ClassTransfer { usdc, to_perp },
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(signer, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn vault_transfer(
        &self,
        is_deposit: bool,
        usd: u64,
    ) -> Result<ExchangeResponseStatus> {
        let vault_address = match &self.vault_address {
            Some(vault_address) => vault_address,
            None => return Err(crate::Error::VaultAddressNotFound),
        };
        let timestamp = next_nonce();

        let action = Actions::VaultTransfer(VaultTransfer {
            vault_address: *vault_address,
            is_deposit,
            usd,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(&self.wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn market_open(
        &self,
        params: MarketOrderParams<'_>,
    ) -> Result<ExchangeResponseStatus> {
        let slippage = params.slippage.unwrap_or(0.05); // Default 5% slippage
        let (px, sz_decimals) = self
            .calculate_slippage_price(params.asset, params.is_buy, slippage, params.px)
            .await?;

        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: false,
            limit_px: px,
            sz: round_to_decimals(params.sz, sz_decimals),
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit { tif: LimitTif::Ioc }),
        };

        self.order(order).await
    }

    pub async fn market_open_with_builder(
        &self,
        params: MarketOrderParams<'_>,
        builder: BuilderInfo,
    ) -> Result<ExchangeResponseStatus> {
        let slippage = params.slippage.unwrap_or(0.05); // Default 5% slippage
        let (px, sz_decimals) = self
            .calculate_slippage_price(params.asset, params.is_buy, slippage, params.px)
            .await?;

        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: false,
            limit_px: px,
            sz: round_to_decimals(params.sz, sz_decimals),
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit { tif: LimitTif::Ioc }),
        };

        self.order_with_builder(order, builder).await
    }

    pub async fn market_close(
        &self,
        params: MarketCloseParams<'_>,
    ) -> Result<ExchangeResponseStatus> {
        let slippage = params.slippage.unwrap_or(0.05); // Default 5% slippage
        let signer = params.wallet.unwrap_or(&self.wallet);

        let info_client = InfoClient::builder()
            .network(self.http_client.network_type())
            .build();
        let user_state = info_client.user_state(signer.address()).await?;

        let position = user_state
            .asset_positions
            .iter()
            .find(|p| p.position.coin == params.asset)
            .ok_or(Error::AssetNotFound)?;

        let szi = position
            .position
            .szi
            .parse::<f64>()
            .map_err(|_| Error::FloatStringParse)?;

        let (px, sz_decimals) = self
            .calculate_slippage_price(params.asset, szi < 0.0, slippage, params.px)
            .await?;

        let sz = round_to_decimals(params.sz.unwrap_or_else(|| szi.abs()), sz_decimals);

        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: szi < 0.0,
            reduce_only: true,
            limit_px: px,
            sz,
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit { tif: LimitTif::Ioc }),
        };

        self.order(order).await
    }

    async fn calculate_slippage_price(
        &self,
        asset: &str,
        is_buy: bool,
        slippage: f64,
        px: Option<f64>,
    ) -> Result<(f64, u32)> {
        let info_client = InfoClient::builder()
            .network(self.http_client.network_type())
            .build();
        let meta = info_client.meta().await?;

        let asset_meta = meta
            .universe
            .iter()
            .find(|a| a.name == asset)
            .ok_or(Error::AssetNotFound)?;

        let sz_decimals = asset_meta.sz_decimals;
        let max_decimals: u32 = if self
            .coin_to_asset
            .read(asset, |_, asset| *asset)
            .unwrap_or_default()
            < 10000
        {
            6
        } else {
            8
        };
        let price_decimals = max_decimals.saturating_sub(sz_decimals);

        let px = if let Some(px) = px {
            px
        } else {
            let all_mids = info_client.all_mids().await?;
            all_mids
                .get(asset)
                .ok_or(Error::AssetNotFound)?
                .parse::<f64>()
                .map_err(|_| Error::FloatStringParse)?
        };

        tracing::debug!("px before slippage: {px:?}");
        let slippage_factor = if is_buy {
            1.0 + slippage
        } else {
            1.0 - slippage
        };
        let px = px * slippage_factor;

        // Round to the correct number of decimal places and significant figures
        let px = round_to_significant_and_decimal(px, 5, price_decimals);

        tracing::debug!("px after slippage: {px:?}");
        Ok((px, sz_decimals))
    }

    pub async fn order(&self, order: ClientOrderRequest<'_>) -> Result<ExchangeResponseStatus> {
        self.bulk_order(&[order]).await
    }

    pub async fn order_with_builder(
        &self,
        order: ClientOrderRequest<'_>,
        builder: BuilderInfo,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_order_with_builder(&[order], builder).await
    }

    pub async fn bulk_order(
        &self,
        orders: &[ClientOrderRequest<'_>],
    ) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        let transformed_orders = orders
            .iter()
            .filter_map(|order| order.to_order_request(&self.coin_to_asset).ok())
            .collect::<Vec<_>>();

        let action = Actions::Order(BulkOrder {
            orders: transformed_orders,
            grouping: Grouping::Na,
            builder: None,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(&self.wallet, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp).await
    }

    pub async fn bulk_order_with_builder(
        &self,
        orders: &[ClientOrderRequest<'_>],
        mut builder: BuilderInfo,
    ) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        builder.builder = builder.builder.to_lowercase();

        let transformed_orders = orders
            .iter()
            .filter_map(|order| order.to_order_request(&self.coin_to_asset).ok())
            .collect::<Vec<_>>();

        let action = Actions::Order(BulkOrder {
            orders: transformed_orders,
            grouping: Grouping::Na,
            builder: Some(builder),
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(&self.wallet, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp).await
    }

    pub async fn cancel(&self, cancel: ClientCancelRequest<'_>) -> Result<ExchangeResponseStatus> {
        self.bulk_cancel(&[cancel]).await
    }

    pub async fn bulk_cancel(
        &self,
        cancels: &[ClientCancelRequest<'_>],
    ) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        let mut transformed_cancels = Vec::new();
        for cancel in cancels.iter() {
            let asset = self
                .coin_to_asset
                .read(cancel.asset, |_, asset| *asset)
                .ok_or(Error::AssetNotFound)?;
            transformed_cancels.push(CancelRequest {
                asset,
                oid: cancel.oid,
            });
        }

        let action = Actions::Cancel(BulkCancel {
            cancels: transformed_cancels,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(&self.wallet, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp).await
    }

    pub async fn modify(&self, modify: ClientModifyRequest<'_>) -> Result<ExchangeResponseStatus> {
        self.bulk_modify(&[modify]).await
    }

    pub async fn bulk_modify(
        &self,
        modifies: &[ClientModifyRequest<'_>],
    ) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        let mut transformed_modifies = Vec::new();
        for modify in modifies.iter() {
            transformed_modifies.push(ModifyRequest {
                oid: modify.oid,
                order: modify.order.to_order_request(&self.coin_to_asset)?,
            });
        }

        let action = Actions::BatchModify(BulkModify {
            modifies: transformed_modifies,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(&self.wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn cancel_by_cloid(
        &self,
        cancel: ClientCancelRequestCloid<'_>,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_cancel_by_cloid(&[cancel]).await
    }

    pub async fn bulk_cancel_by_cloid(
        &self,
        cancels: &[ClientCancelRequestCloid<'_>],
    ) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        let mut transformed_cancels: Vec<CancelRequestCloid> = Vec::new();
        for cancel in cancels.iter() {
            let asset = self
                .coin_to_asset
                .read(cancel.asset, |_, asset| *asset)
                .ok_or(Error::AssetNotFound)?;
            transformed_cancels.push(CancelRequestCloid {
                asset,
                cloid: uuid_to_hex_string(cancel.cloid),
            });
        }

        let action = Actions::CancelByCloid(BulkCancelCloid {
            cancels: transformed_cancels,
        });

        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(&self.wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn update_leverage(
        &self,
        leverage: u32,
        coin: &str,
        is_cross: bool,
    ) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        let asset_index = self
            .coin_to_asset
            .read(coin, |_, asset| *asset)
            .ok_or(Error::AssetNotFound)?;
        let action = Actions::UpdateLeverage(UpdateLeverage {
            asset: asset_index,
            is_cross,
            leverage,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(&self.wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn update_isolated_margin(
        &self,
        amount: f64,
        coin: &str,
    ) -> Result<ExchangeResponseStatus> {
        let amount = (amount * 1_000_000.0).round() as i64;
        let timestamp = next_nonce();

        let asset_index = self
            .coin_to_asset
            .read(coin, |_, asset| *asset)
            .ok_or(Error::AssetNotFound)?;
        let action = Actions::UpdateIsolatedMargin(UpdateIsolatedMargin {
            asset: asset_index,
            is_buy: true,
            ntli: amount,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(&self.wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn approve_agent(&self) -> Result<(LocalWallet, ExchangeResponseStatus)> {
        let mut rng = ethers::core::rand::thread_rng();
        let wallet = LocalWallet::new(&mut rng);
        let address = wallet.address();

        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let nonce = next_nonce();
        let approve_agent = ApproveAgent {
            signature_chain_id: 421614.into(),
            hyperliquid_chain,
            agent_address: address,
            agent_name: Default::default(),
            nonce,
        };
        let signature = sign_typed_data(&approve_agent, &self.wallet)?;
        let action = serde_json::to_value(Actions::ApproveAgent(approve_agent))
            .map_err(|e| Error::JsonParse(e.to_string()))?;
        Ok((wallet, self.post(action, signature, nonce).await?))
    }

    pub async fn withdraw_from_bridge(
        &self,
        amount: &str,
        destination: &str,
    ) -> Result<ExchangeResponseStatus> {
        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let timestamp = next_nonce();
        let withdraw = Withdraw3 {
            signature_chain_id: 421614.into(),
            hyperliquid_chain,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
        };
        let signature = sign_typed_data(&withdraw, &self.wallet)?;
        let action = serde_json::to_value(Actions::Withdraw3(withdraw))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp).await
    }

    pub async fn spot_transfer(
        &self,
        amount: &str,
        destination: &str,
        token: &str,
    ) -> Result<ExchangeResponseStatus> {
        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let timestamp = next_nonce();
        let spot_send = SpotSend {
            signature_chain_id: 421614.into(),
            hyperliquid_chain,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
            token: token.to_string(),
        };
        let signature = sign_typed_data(&spot_send, &self.wallet)?;
        let action = serde_json::to_value(Actions::SpotSend(spot_send))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp).await
    }

    pub async fn set_referrer(&self, code: String) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        let action = Actions::SetReferrer(SetReferrer { code });

        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(&self.wallet, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp).await
    }

    pub async fn approve_builder_fee(
        &self,
        builder: H160,
        max_fee_rate: String,
    ) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let approve_builder_fee = ApproveBuilderFee {
            signature_chain_id: 421614.into(),
            hyperliquid_chain,
            builder,
            max_fee_rate,
            nonce: timestamp,
        };

        let signature = sign_typed_data(&approve_builder_fee, &self.wallet)?;
        let action = serde_json::to_value(Actions::ApproveBuilderFee(approve_builder_fee))
            .map_err(|e| Error::JsonParse(e.to_string()))?;
        self.post(action, signature, timestamp).await
    }
}

fn round_to_decimals(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

fn round_to_significant_and_decimal(value: f64, sig_figs: u32, max_decimals: u32) -> f64 {
    let abs_value = value.abs();
    let magnitude = abs_value.log10().floor() as i32;
    let scale = 10f64.powi(sig_figs as i32 - magnitude - 1);
    let rounded = (abs_value * scale).round() / scale;
    round_to_decimals(rounded.copysign(value), max_decimals)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{
        Order,
        exchange::order::{Limit, OrderRequest, Trigger},
    };

    fn get_wallet() -> Result<LocalWallet> {
        let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
        priv_key
            .parse::<LocalWallet>()
            .map_err(|e| Error::Wallet(e.to_string()))
    }

    #[test]
    fn test_limit_order_action_hashing() -> Result<()> {
        let wallet = get_wallet()?;
        let action = Actions::Order(BulkOrder {
            orders: vec![OrderRequest {
                asset: 1,
                is_buy: true,
                limit_px: "2000.0".to_string(),
                sz: "3.5".to_string(),
                reduce_only: false,
                order_type: Order::Limit(Limit { tif: LimitTif::Ioc }),
                cloid: None,
            }],
            grouping: Grouping::Na,
            builder: None,
        });
        let connection_id = action.hash(1583838, None)?;

        let signature = sign_l1_action(&wallet, connection_id, true)?;
        assert_eq!(
            signature.to_string(),
            "0x82f7e6747e4fcd0359efa9490426871693472d07ce404261f1c39084beb7aba02d8e9e3f618336c2287849b69e5021ac593bb94c00ae82815c7580a4256923a01c"
        );

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(
            signature.to_string(),
            "0x1760e47c9670cbc26ca6ad961818231fabcffb116e43feed75baa87c0307cc7c446131e4bd121caeba7fe1e8494410dbf149206988985bb66044f905450638821b"
        );

        Ok(())
    }

    #[test]
    fn test_limit_order_action_hashing_with_cloid() -> Result<()> {
        let cloid = uuid::Uuid::from_str("1e60610f-0b3d-4205-97c8-8c1fed2ad5ee")
            .map_err(|_e| uuid::Uuid::new_v4());
        let wallet = get_wallet()?;
        let action = Actions::Order(BulkOrder {
            orders: vec![OrderRequest {
                asset: 1,
                is_buy: true,
                limit_px: "2000.0".to_string(),
                sz: "3.5".to_string(),
                reduce_only: false,
                order_type: Order::Limit(Limit { tif: LimitTif::Ioc }),
                cloid: Some(uuid_to_hex_string(cloid.unwrap())),
            }],
            grouping: Grouping::Na,
            builder: None,
        });
        let connection_id = action.hash(1583838, None)?;

        let signature = sign_l1_action(&wallet, connection_id, true)?;
        assert_eq!(
            signature.to_string(),
            "0xc5cc6ca48c2c4223c89f62f1e6eff4c68546dfc7baa12073a8ddff5a38b3e62a6d2967e080698522863ca147685e5c68ff854348dd97cc032771ff5be301a2c21b"
        );

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(
            signature.to_string(),
            "0xeb99e4496d3897aa58c653044c543d347458edfc1a68182fd53c64bcf4c3a6e2429f53e4dee68214f32d3277f7454b77be963d3cb98b8462c79b47adf61185861b"
        );

        Ok(())
    }

    #[test]
    fn test_tpsl_order_action_hashing() -> Result<()> {
        use crate::exchange::order::TriggerTpsl;

        for (tpsl, mainnet_signature, testnet_signature) in [
            (
                "tp",
                "0x5061414c399533a5880f429362ee15511864401aefe00f8a1b6da937a0ecc2e058f90b1b72b26bb79d537785e49b37420b3a04d313ff7010ea8453bbf6e2383c1b",
                "0x09c29abc493d6144f1136f197194d0cdd87cd8c56c971ee38a69447da5d7a11773355e2d66afff017386e3143654dd07be365530ad382a246f14f818d66be5c81c",
            ),
            (
                "sl",
                "0x40fdee49426becc5cabebbcd61182cee72a0ae2c40df3de0cbbcf65621cf64b54dd9d720192c16dedf5aea10aca7bd5522ac76164a3a3f7d10b2054ff732a91e1b",
                "0x7f2cfdcaa1d8a0b47e4da1699f8aa4af8da749db6bd5ea29f18fdb880856b1705d52ec14e7fc1c37a728b3e6099c83a58c23d5d0775b800c175bf137e29dc0e91c",
            ),
        ] {
            let wallet = get_wallet()?;
            let action = Actions::Order(BulkOrder {
                orders: vec![OrderRequest {
                    asset: 1,
                    is_buy: true,
                    limit_px: "2000.0".to_string(),
                    sz: "3.5".to_string(),
                    reduce_only: false,
                    order_type: Order::Trigger(Trigger {
                        trigger_px: "2000.0".to_string(),
                        is_market: true,
                        tpsl: TriggerTpsl::from_str(tpsl).unwrap(),
                    }),
                    cloid: None,
                }],
                grouping: Grouping::Na,
                builder: None,
            });
            let connection_id = action.hash(1583838, None)?;

            let signature = sign_l1_action(&wallet, connection_id, true)?;
            assert_eq!(signature.to_string(), mainnet_signature);

            let signature = sign_l1_action(&wallet, connection_id, false)?;
            assert_eq!(signature.to_string(), testnet_signature);
        }
        Ok(())
    }

    #[test]
    fn test_cancel_action_hashing() -> Result<()> {
        let wallet = get_wallet()?;
        let action = Actions::Cancel(BulkCancel {
            cancels: vec![CancelRequest {
                asset: 1,
                oid: 82382,
            }],
        });
        let connection_id = action.hash(1583838, None)?;

        let signature = sign_l1_action(&wallet, connection_id, true)?;
        assert_eq!(
            signature.to_string(),
            "0x9f8b8530274f2f174adf8cd0f02e8bbf5c2987866fbac000e7e3e19214686dde1018d9d181e95a84246a7361ca1dd731a8cbd42dfb04cfc9b071e78aa487a6441c"
        );

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(
            signature.to_string(),
            "0x6b50910d58758f2a50f9629ac8f01c2ce533d9f5db21446c8f9d3a720e0e5f5e7a2bcf0de855c9af0ddb4959575add4478f5eb153eee8d35b640238ed71314381b"
        );

        Ok(())
    }
}

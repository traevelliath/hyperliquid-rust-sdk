use crate::req::NetworkType;
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
    req::{Endpoint, HttpClient},
    signature::sign_l1_action,
    errors::Result,
};
use crate::{ClassTransfer, SpotSend, SpotUser, VaultTransfer, Withdraw3};

use ethers::{
    signers::{LocalWallet, Signer},
    types::{H160, H256, Signature},
};

use super::cancel::ClientCancelRequestCloid;
use super::order::BuilderInfo;

#[derive(Debug, Clone)]
pub struct ExchangeApi {
    pub http_client: HttpClient,
    pub coin_to_asset: std::sync::Arc<scc::HashMap<String, u32>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::exchange) struct ExchangePayload {
    pub(in crate::exchange) action: serde_json::Value,
    pub(in crate::exchange) signature: Signature,
    pub(in crate::exchange) nonce: u64,
    pub(in crate::exchange) vault_address: Option<H160>,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Actions {
    UsdSend(UsdSend),
    UpdateLeverage(UpdateLeverage),
    UpdateIsolatedMargin(UpdateIsolatedMargin),
    Order(BulkOrder),
    Cancel(BulkCancel),
    CancelByCloid(BulkCancelCloid),
    BatchModify(BulkModify),
    ApproveAgent(ApproveAgent),
    Withdraw3(Withdraw3),
    SpotUser(SpotUser),
    VaultTransfer(VaultTransfer),
    SpotSend(SpotSend),
    SetReferrer(SetReferrer),
    ApproveBuilderFee(ApproveBuilderFee),
}

impl Actions {
    pub(in crate::exchange) fn hash(
        &self,
        timestamp: u64,
        vault_address: Option<H160>,
    ) -> Result<H256> {
        let mut bytes =
            rmp_serde::to_vec_named(self).map_err(|e| Error::RmpParse(e.to_string()))?;
        bytes.extend(timestamp.to_be_bytes());
        if let Some(vault_address) = vault_address {
            bytes.push(1);
            bytes.extend(vault_address.to_fixed_bytes());
        } else {
            bytes.push(0);
        }

        Ok(H256::from(ethers::utils::keccak256(bytes)))
    }
}

impl ExchangeApi {
    pub async fn new(http_client: &HttpClient, network: NetworkType) -> Result<ExchangeApi> {
        let info = InfoClient::builder()
            .http_client(http_client.client.clone())
            .network(network)
            .build();
        let meta = info.meta().await?;
        let mut perp_map = {
            let iter = meta
                .universe
                .into_iter()
                .map(|asset| (asset.name, asset.sz_decimals));
            scc::HashMap::from_iter(iter)
        };

        info.spot_meta()
            .await?
            .add_to_coin_to_asset_map(&mut perp_map);

        Ok(ExchangeApi {
            http_client: http_client.clone(),
            coin_to_asset: std::sync::Arc::new(perp_map),
        })
    }

    async fn post(
        &self,
        action: serde_json::Value,
        signature: Signature,
        nonce: u64,
        vault_address: Option<H160>,
    ) -> Result<ExchangeResponseStatus> {
        let exchange_payload = ExchangePayload {
            action,
            signature,
            nonce,
            vault_address,
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
        signer: &LocalWallet,
    ) -> Result<ExchangeResponseStatus> {
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

        self.post(action, signature, timestamp, None).await
    }

    pub async fn class_transfer(
        &self,
        usdc: f64,
        to_perp: bool,
        signer: &LocalWallet,
    ) -> Result<ExchangeResponseStatus> {
        // payload expects usdc without decimals
        let usdc = (usdc * 1e6).round() as u64;

        let timestamp = next_nonce();

        let action = Actions::SpotUser(SpotUser {
            class_transfer: ClassTransfer { usdc, to_perp },
        });
        let connection_id = action.hash(timestamp, None)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(signer, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp, None).await
    }

    pub async fn vault_transfer(
        &self,
        is_deposit: bool,
        usd: u64,
        vault_address: &H160,
        signer: &LocalWallet,
    ) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        let action = Actions::VaultTransfer(VaultTransfer {
            vault_address: *vault_address,
            is_deposit,
            usd,
        });
        let connection_id = action.hash(timestamp, Some(*vault_address))?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(signer, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp, Some(*vault_address))
            .await
    }

    pub async fn order(
        &self,
        order: ClientOrderRequest<'_>,
        signer: &LocalWallet,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_order(&[order], signer).await
    }

    pub async fn order_with_builder(
        &self,
        order: ClientOrderRequest<'_>,
        signer: &LocalWallet,
        builder: BuilderInfo,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_order_with_builder(&[order], signer, builder)
            .await
    }

    pub async fn bulk_order(
        &self,
        orders: &[ClientOrderRequest<'_>],
        signer: &LocalWallet,
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
        let connection_id = action.hash(timestamp, None)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(signer, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp, None).await
    }

    pub async fn bulk_order_with_builder(
        &self,
        orders: &[ClientOrderRequest<'_>],
        wallet: &LocalWallet,
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
        let connection_id = action.hash(timestamp, None)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp, None).await
    }

    pub async fn cancel(
        &self,
        cancel: ClientCancelRequest<'_>,
        wallet: &LocalWallet,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_cancel(&[cancel], wallet).await
    }

    pub async fn bulk_cancel(
        &self,
        cancels: &[ClientCancelRequest<'_>],
        wallet: &LocalWallet,
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
        let connection_id = action.hash(timestamp, None)?;

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp, None).await
    }

    pub async fn modify(
        &self,
        modify: ClientModifyRequest<'_>,
        wallet: &LocalWallet,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_modify(&[modify], wallet).await
    }

    pub async fn bulk_modify(
        &self,
        modifies: &[ClientModifyRequest<'_>],
        wallet: &LocalWallet,
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
        let connection_id = action.hash(timestamp, None)?;

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp, None).await
    }

    pub async fn cancel_by_cloid(
        &self,
        cancel: ClientCancelRequestCloid<'_>,
        wallet: &LocalWallet,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_cancel_by_cloid(&[cancel], wallet).await
    }

    pub async fn bulk_cancel_by_cloid(
        &self,
        cancels: &[ClientCancelRequestCloid<'_>],
        wallet: &LocalWallet,
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
                cloid: match &cancel.cloid {
                    crate::Cloid::Uuid(cloid) => uuid_to_hex_string(*cloid),
                    crate::Cloid::String(cloid) => cloid.clone(),
                },
            });
        }

        let action = Actions::CancelByCloid(BulkCancelCloid {
            cancels: transformed_cancels,
        });

        let connection_id = action.hash(timestamp, None)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp, None).await
    }

    pub async fn update_leverage(
        &self,
        leverage: u32,
        coin: &str,
        is_cross: bool,
        wallet: &LocalWallet,
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
        let connection_id = action.hash(timestamp, None)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp, None).await
    }

    pub async fn update_isolated_margin(
        &self,
        amount: f64,
        coin: &str,
        wallet: &LocalWallet,
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
        let connection_id = action.hash(timestamp, None)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp, None).await
    }

    pub async fn approve_agent(
        &self,
        wallet: &LocalWallet,
    ) -> Result<(LocalWallet, ExchangeResponseStatus)> {
        let mut rng = ethers::core::rand::thread_rng();
        let key = LocalWallet::new(&mut rng);
        let address = key.address();

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
        let signature = sign_typed_data(&approve_agent, wallet)?;
        let action = serde_json::to_value(Actions::ApproveAgent(approve_agent))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok((key, self.post(action, signature, nonce, None).await?))
    }

    pub async fn withdraw_from_bridge(
        &self,
        amount: &str,
        destination: &str,
        wallet: &LocalWallet,
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
        let signature = sign_typed_data(&withdraw, wallet)?;
        let action = serde_json::to_value(Actions::Withdraw3(withdraw))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp, None).await
    }

    pub async fn spot_transfer(
        &self,
        amount: &str,
        destination: &str,
        token: &str,
        wallet: &LocalWallet,
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
        let signature = sign_typed_data(&spot_send, wallet)?;
        let action = serde_json::to_value(Actions::SpotSend(spot_send))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp, None).await
    }

    pub async fn set_referrer(
        &self,
        code: String,
        wallet: &LocalWallet,
    ) -> Result<ExchangeResponseStatus> {
        let timestamp = next_nonce();

        let action = Actions::SetReferrer(SetReferrer { code });

        let connection_id = action.hash(timestamp, None)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp, None).await
    }

    pub async fn approve_builder_fee(
        &self,
        builder: H160,
        max_fee_rate: String,
        signer: &LocalWallet,
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

        let signature = sign_typed_data(&approve_builder_fee, signer)?;
        let action = serde_json::to_value(Actions::ApproveBuilderFee(approve_builder_fee))
            .map_err(|e| Error::JsonParse(e.to_string()))?;
        self.post(action, signature, timestamp, None).await
    }
}

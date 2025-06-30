use crate::{
    BaseUrl, Error, Message, OrderStatusResponse, ReferralResponse, UserFeesResponse,
    UserFundingResponse, UserTokenBalanceResponse,
    info::{
        CandlesSnapshotResponse, FundingHistoryResponse, L2SnapshotResponse, OpenOrdersResponse,
        OrderInfo, RecentTradesResponse, UserFillsResponse, UserStateResponse,
        builder::InfoClientBuilder,
        request::{CandleSnapshotRequest, InfoRequest},
    },
    meta::{Meta, SpotMeta, SpotMetaAndAssetCtxs},
    prelude::*,
    req::{HttpClient, Endpoint},
    ws::{Subscription, WsManager},
};

use alloy::primitives::Address;

#[derive(Debug)]
pub struct InfoClient {
    http_client: HttpClient,
    ws_manager: Option<WsManager>,
    base_url: BaseUrl,
}

impl InfoClient {
    pub fn builder() -> InfoClientBuilder {
        InfoClientBuilder::default()
    }
}

impl InfoClient {
    pub(crate) fn new(
        http_client: reqwest::Client,
        ws_manager: Option<WsManager>,
        base_url: BaseUrl,
    ) -> Self {
        Self {
            http_client: HttpClient {
                client: http_client,
                base_url,
            },
            ws_manager,
            base_url,
        }
    }

    pub async fn subscribe(
        &mut self,
        subscription: Subscription,
    ) -> Result<tokio::sync::broadcast::Receiver<Message>> {
        if self.ws_manager.is_none() {
            let ws_manager = WsManager::new(self.base_url.get_ws_url()).await?;
            self.ws_manager = Some(ws_manager);
        }

        self.ws_manager
            .as_mut()
            .ok_or(Error::WsManagerNotFound)?
            .subscribe(subscription)
            .await
    }

    pub async fn unsubscribe(
        &mut self,
        subscription: Subscription,
    ) -> Result<tokio::sync::broadcast::Receiver<Message>> {
        if self.ws_manager.is_none() {
            let ws_manager = WsManager::new(self.base_url.get_ws_url()).await?;
            self.ws_manager = Some(ws_manager);
        }

        self.ws_manager
            .as_mut()
            .ok_or(Error::WsManagerNotFound)?
            .unsubscribe(subscription)
            .await
    }

    pub fn get_listener(&self) -> Result<tokio::sync::broadcast::Receiver<Message>> {
        let receiver = self
            .ws_manager
            .as_ref()
            .ok_or(Error::WsManagerNotFound)?
            .get_listener();

        Ok(receiver)
    }

    async fn send_info_request<'a, T: for<'de> serde::Deserialize<'de>>(
        &self,
        info_request: InfoRequest<'a>,
    ) -> Result<T> {
        let data =
            serde_json::to_string(&info_request).map_err(|e| Error::JsonParse(e.to_string()))?;

        let return_data = self.http_client.post(Endpoint::Info, data).await?;
        serde_json::from_str(&return_data).map_err(|e| Error::JsonParse(e.to_string()))
    }

    pub async fn open_orders(&self, address: Address) -> Result<Vec<OpenOrdersResponse>> {
        let input = InfoRequest::OpenOrders { user: address };
        self.send_info_request(input).await
    }

    pub async fn user_state(&self, address: Address) -> Result<UserStateResponse> {
        let input = InfoRequest::UserState { user: address };
        self.send_info_request(input).await
    }

    pub async fn user_states(&self, addresses: Vec<Address>) -> Result<Vec<UserStateResponse>> {
        let input = InfoRequest::UserStates { users: addresses };
        self.send_info_request(input).await
    }

    pub async fn user_token_balances(&self, address: Address) -> Result<UserTokenBalanceResponse> {
        let input = InfoRequest::UserTokenBalances { user: address };
        self.send_info_request(input).await
    }

    pub async fn user_fees(&self, address: Address) -> Result<UserFeesResponse> {
        let input = InfoRequest::UserFees { user: address };
        self.send_info_request(input).await
    }

    pub async fn meta(&self) -> Result<Meta> {
        let input = InfoRequest::Meta;
        self.send_info_request(input).await
    }

    pub async fn spot_meta(&self) -> Result<SpotMeta> {
        let input = InfoRequest::SpotMeta;
        self.send_info_request(input).await
    }

    pub async fn spot_meta_and_asset_contexts(&self) -> Result<Vec<SpotMetaAndAssetCtxs>> {
        let input = InfoRequest::SpotMetaAndAssetCtxs;
        self.send_info_request(input).await
    }

    pub async fn all_mids(&self) -> Result<std::collections::HashMap<String, String>> {
        let input = InfoRequest::AllMids;
        self.send_info_request(input).await
    }

    pub async fn user_fills(&self, address: Address) -> Result<Vec<UserFillsResponse>> {
        let input = InfoRequest::UserFills { user: address };
        self.send_info_request(input).await
    }

    pub async fn funding_history(
        &self,
        coin: &str,
        start_time: u64,
        end_time: Option<u64>,
    ) -> Result<Vec<FundingHistoryResponse>> {
        let input = InfoRequest::FundingHistory {
            coin,
            start_time,
            end_time,
        };
        self.send_info_request(input).await
    }

    pub async fn user_funding_history(
        &self,
        user: Address,
        start_time: u64,
        end_time: Option<u64>,
    ) -> Result<Vec<UserFundingResponse>> {
        let input = InfoRequest::UserFunding {
            user,
            start_time,
            end_time,
        };
        self.send_info_request(input).await
    }

    pub async fn recent_trades(&self, coin: &str) -> Result<Vec<RecentTradesResponse>> {
        let input = InfoRequest::RecentTrades { coin };
        self.send_info_request(input).await
    }

    pub async fn l2_snapshot(&self, coin: &str) -> Result<L2SnapshotResponse> {
        let input = InfoRequest::L2Book { coin };
        self.send_info_request(input).await
    }

    pub async fn candles_snapshot(
        &self,
        coin: &str,
        interval: crate::ws::Interval,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<CandlesSnapshotResponse>> {
        let input = InfoRequest::CandleSnapshot {
            req: CandleSnapshotRequest::new(coin, interval, start_time, end_time),
        };
        self.send_info_request(input).await
    }

    pub async fn query_order_by_oid(
        &self,
        address: Address,
        oid: u64,
    ) -> Result<OrderStatusResponse> {
        let input = InfoRequest::OrderStatus { user: address, oid };
        self.send_info_request(input).await
    }

    pub async fn query_referral_state(&self, address: Address) -> Result<ReferralResponse> {
        let input = InfoRequest::Referral { user: address };
        self.send_info_request(input).await
    }

    pub async fn historical_orders(&self, address: Address) -> Result<Vec<OrderInfo>> {
        let input = InfoRequest::HistoricalOrders { user: address };
        self.send_info_request(input).await
    }

    /// Gracefully shuts down the WebSocket connection.
    ///
    /// This method is used to gracefully shut down the WebSocket connection.
    /// It is important to call this method when you are done using the InfoClient
    /// to ensure that the WebSocket connection is closed properly.
    pub async fn shutdown(self) {
        if let Some(ws) = self.ws_manager {
            ws.shutdown().await;
        }
    }
}

use crate::{BaseUrl, NetworkType, exchange::client::ExchangeClient, prelude::*, req::HttpClient};

pub struct ExchangeClientBuilder {
    http_client: HttpClient,
    network: NetworkType,
    wallet: alloy::signers::local::PrivateKeySigner,
    meta: Option<crate::meta::Meta>,
    vault_address: Option<alloy::primitives::Address>,
    coin_to_asset: scc::HashMap<String, u32>,
}

impl Default for ExchangeClientBuilder {
    fn default() -> Self {
        Self {
            http_client: HttpClient {
                client: reqwest::Client::new(),
                base_url: BaseUrl::Mainnet,
            },
            network: NetworkType::Mainnet,
            wallet: alloy::signers::local::PrivateKeySigner::random(),
            meta: None,
            vault_address: None,
            coin_to_asset: scc::HashMap::new(),
        }
    }
}

impl ExchangeClientBuilder {
    pub fn http_client(mut self, http_client: HttpClient) -> Self {
        self.http_client = http_client;
        self
    }

    pub fn network(mut self, network: NetworkType) -> Self {
        self.network = network;
        self
    }

    pub fn wallet(mut self, wallet: alloy::signers::local::PrivateKeySigner) -> Self {
        self.wallet = wallet;
        self
    }

    pub fn meta(mut self, meta: crate::meta::Meta) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn vault_address(mut self, vault_address: alloy::primitives::Address) -> Self {
        self.vault_address = Some(vault_address);
        self
    }

    pub fn coin_to_asset<T>(mut self, coin_to_asset: T) -> Self
    where
        T: Into<scc::HashMap<String, u32>>,
    {
        self.coin_to_asset = coin_to_asset.into();
        self
    }

    pub async fn build(self) -> Result<ExchangeClient> {
        ExchangeClient::new(
            self.http_client,
            self.wallet,
            self.network,
            self.meta,
            self.vault_address,
        )
        .await
    }
}

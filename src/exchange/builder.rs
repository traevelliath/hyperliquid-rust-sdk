use crate::{BaseUrl, NetworkType, exchange::client::ExchangeClient, req::HttpClient, errors::Result};

pub struct ExchangeClientBuilder {
    http_client: reqwest::Client,
    network: NetworkType,
    wallet: ethers::signers::LocalWallet,
    meta: Option<crate::meta::Meta>,
    vault_address: Option<ethers::types::H160>,
    coin_to_asset: scc::HashMap<String, u32>,
}

impl Default for ExchangeClientBuilder {
    fn default() -> Self {
        let mut rng = ethers::core::rand::thread_rng();
        Self {
            http_client: reqwest::Client::new(),
            network: NetworkType::Mainnet,
            wallet: ethers::signers::LocalWallet::new(&mut rng),
            meta: None,
            vault_address: None,
            coin_to_asset: scc::HashMap::new(),
        }
    }
}

impl ExchangeClientBuilder {
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = http_client;
        self
    }

    pub fn network(mut self, network: NetworkType) -> Self {
        self.network = network;
        self
    }

    pub fn wallet(mut self, wallet: ethers::signers::LocalWallet) -> Self {
        self.wallet = wallet;
        self
    }

    pub fn meta(mut self, meta: crate::meta::Meta) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn vault_address(mut self, vault_address: ethers::types::H160) -> Self {
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
            HttpClient {
                client: self.http_client,
                base_url: match self.network {
                    NetworkType::Mainnet => BaseUrl::Mainnet,
                    NetworkType::Testnet => BaseUrl::Testnet,
                    NetworkType::Localhost => BaseUrl::Localhost,
                },
            },
            self.wallet,
            self.network,
            self.meta,
            self.vault_address,
        )
        .await
    }
}

pub struct InfoClientBuilder {
    http_client: reqwest::Client,
    network: crate::NetworkType,
}

impl Default for InfoClientBuilder {
    fn default() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            network: crate::NetworkType::Mainnet,
        }
    }
}

impl InfoClientBuilder {
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = http_client;
        self
    }

    pub fn network(mut self, network: crate::NetworkType) -> Self {
        self.network = network;
        self
    }

    pub fn build(self) -> crate::info::client::InfoClient {
        let base_url = match self.network {
            crate::NetworkType::Mainnet => crate::BaseUrl::Mainnet,
            crate::NetworkType::Testnet => crate::BaseUrl::Testnet,
            crate::NetworkType::Localhost => crate::BaseUrl::Localhost,
        };

        crate::info::client::InfoClient::new(self.http_client, None, base_url)
    }
}

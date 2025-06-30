use crate::{BaseUrl, Error, prelude::*};

#[derive(serde::Deserialize, Debug)]
struct ErrorData {
    data: String,
    code: u16,
    msg: String,
}

#[derive(Debug, Clone)]
pub struct HttpClient {
    pub client: reqwest::Client,
    pub base_url: BaseUrl,
}

#[derive(Debug, Clone)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Localhost,
}

#[derive(Debug, Clone)]
pub enum Endpoint {
    Info,
    Exchange,
}

async fn parse_response(response: reqwest::Response) -> Result<String> {
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| Error::GenericRequest(e.to_string()))?;

    if status.is_success() {
        return Ok(text);
    }

    let status_code = status.as_u16();
    if status.is_client_error() {
        let client_error = match serde_json::from_str::<ErrorData>(&text) {
            Ok(ErrorData { data, code, msg }) => Error::ClientRequest {
                status_code,
                error_code: Some(code),
                error_message: msg,
                error_data: Some(data),
            },
            Err(_) => Error::ClientRequest {
                status_code,
                error_code: None,
                error_message: text,
                error_data: None,
            },
        };
        return Err(client_error);
    }

    Err(Error::ServerRequest {
        status_code,
        error_message: text,
    })
}

impl HttpClient {
    pub async fn post(&self, endpoint: Endpoint, data: String) -> Result<String> {
        let url = {
            let mut base = self.base_url.get_url().clone();
            base.set_path(endpoint.into());
            base
        };
        let request = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(data)
            .build()
            .map_err(|e| Error::GenericRequest(e.to_string()))?;

        let result = self
            .client
            .execute(request)
            .await
            .map_err(|e| Error::GenericRequest(e.to_string()))?;

        parse_response(result).await
    }

    pub fn is_mainnet(&self) -> bool {
        self.base_url == BaseUrl::Mainnet
    }

    pub fn network_type(&self) -> NetworkType {
        match self.base_url {
            BaseUrl::Mainnet => NetworkType::Mainnet,
            BaseUrl::Testnet => NetworkType::Testnet,
            BaseUrl::Localhost => NetworkType::Localhost,
        }
    }
}

impl From<Endpoint> for &str {
    fn from(endpoint: Endpoint) -> Self {
        match endpoint {
            Endpoint::Info => "info",
            Endpoint::Exchange => "exchange",
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    // TODO: turn some embedded types into errors instead of strings
    #[error(
        "Client error: status code: {status_code}, error code: {error_code:?}, error message: {error_message}, error data: {error_data:?}"
    )]
    ClientRequest {
        status_code: u16,
        error_code: Option<u16>,
        error_message: String,
        error_data: Option<String>,
    },
    #[error("Server error: status code: {status_code}, error message: {error_message}")]
    ServerRequest {
        status_code: u16,
        error_message: String,
    },
    #[error("Generic request error: {0:?}")]
    GenericRequest(String),
    #[error("Chain type not allowed for this function")]
    ChainNotAllowed,
    #[error("Asset not found")]
    AssetNotFound,
    #[error("Error from Eip712 struct: {0:?}")]
    Eip712(String),
    #[error("Json parse error: {0:?}")]
    JsonParse(String),
    #[error("Generic parse error: {0:?}")]
    GenericParse(String),
    #[error("Wallet error: {0:?}")]
    Wallet(String),
    #[error("Websocket error: {0:?}")]
    Websocket(String),
    #[error("Subscription not found")]
    SubscriptionNotFound,
    #[error("Subscription already exists: {0}")]
    SubscriptionAlreadyExists(String),
    #[error("WS manager not instantiated")]
    WsManagerNotFound,
    #[error("WS send error: {0:?}")]
    WsSend(String),
    #[error("Reader data not found")]
    ReaderDataNotFound,
    #[error("Reader error: {0:?}")]
    GenericReader(String),
    #[error("Reader text conversion error: {0:?}")]
    ReaderTextConversion(String),
    #[error("Order type not found")]
    OrderTypeNotFound,
    #[error("Issue with generating random data: {0:?}")]
    RandGen(String),
    #[error("Private key parse error: {0:?}")]
    PrivateKeyParse(String),
    #[error("Cannot subscribe to multiple user events")]
    UserEvents,
    #[error("Rmp parse error: {0:?}")]
    RmpParse(String),
    #[error("Invalid input number")]
    FloatStringParse,
    #[error("No cloid found in order request when expected")]
    NoCloid,
    #[error("ECDSA signature failed: {0:?}")]
    SignatureFailure(String),
    #[error("Vault address not found")]
    VaultAddressNotFound,

    #[error("Invalid URL: {0:?}")]
    InvalidUrl(String),
    #[error("Tls connector error: {0:?}")]
    TlsConnector(String),
    #[error("Tcp stream error: {0:?}")]
    TcpStream(String),

    #[error("Invalid trigger tpsl: {0}")]
    InvalidTriggerTpsl(String),
    #[error("Invalid limit tif: {0}")]
    InvalidLimitTif(String),
}

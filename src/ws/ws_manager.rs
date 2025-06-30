#[derive(serde::Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Interval {
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "3m")]
    ThreeMinutes,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "30m")]
    ThirtyMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "2h")]
    TwoHours,
    #[serde(rename = "4h")]
    FourHours,
    #[serde(rename = "8h")]
    EightHours,
    #[serde(rename = "12h")]
    TwelveHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "3d")]
    ThreeDays,
    #[serde(rename = "1w")]
    OneWeek,
    #[serde(rename = "1M")]
    OneMonth,
}

#[derive(serde::Serialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Subscription {
    AllMids,
    Notification {
        user: ethers::types::H160,
    },
    WebData2 {
        user: ethers::types::H160,
    },
    Candle {
        coin: String,
        interval: Interval,
    },
    L2Book {
        coin: String,
    },
    Trades {
        coin: String,
    },
    OrderUpdates {
        user: ethers::types::H160,
    },
    UserEvents {
        user: ethers::types::H160,
    },
    UserFills {
        user: ethers::types::H160,
    },
    UserFundings {
        user: ethers::types::H160,
    },
    UserNonFundingLedgerUpdates {
        user: ethers::types::H160,
    },
    ActiveAssetCtx {
        coin: String,
    },
    ActiveAssetData {
        user: ethers::types::H160,
        coin: String,
    },
    Bbo {
        coin: String,
    },
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Method {
    Subscribe,
    Unsubscribe,
}

impl Subscription {
    fn into_frame(self, method: Method) -> fastwebsockets::Frame<'static> {
        let json = serde_json::json!({
            "method": method,
            "subscription": self,
        });

        fastwebsockets::Frame::text(fastwebsockets::Payload::Owned(
            json.to_string().into_bytes(),
        ))
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(tag = "channel")]
#[serde(rename_all = "camelCase")]
pub enum Message {
    NoData,
    Error(crate::ws::message_types::Error),
    HyperliquidError(String),
    AllMids(crate::ws::message_types::AllMids),
    Trades(crate::ws::message_types::Trades),
    L2Book(crate::ws::message_types::L2Book),
    User(crate::ws::message_types::User),
    UserFills(crate::UserFills),
    Candle(crate::ws::message_types::Candle),
    SubscriptionResponse,
    OrderUpdates(crate::ws::message_types::OrderUpdates),
    UserFundings(crate::UserFundings),
    UserNonFundingLedgerUpdates(crate::UserNonFundingLedgerUpdates),
    Notification(crate::Notification),
    WebData2(crate::WebData2),
    ActiveAssetCtx(crate::ActiveAssetCtx),
    ActiveAssetData(crate::ws::message_types::ActiveAssetData),
    ActiveSpotAssetCtx(crate::ws::message_types::ActiveSpotAssetCtx),
    Bbo(crate::ws::message_types::Bbo),
    Pong,
}

#[derive(Debug)]
pub(crate) struct WsManager {
    response_tx: tokio::sync::broadcast::Sender<Message>,
    subscription_tx: tokio::sync::mpsc::Sender<(Subscription, Method)>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
    subscriptions: std::sync::Arc<scc::HashMap<Subscription, ()>>,
}

impl WsManager {
    const SEND_PING_INTERVAL: u64 = 50;
    const MAX_RECONNECT_ATTEMPTS: u64 = 10;

    /// Create a new WebSocket manager.
    ///
    /// Returns a WebSocket manager that can be used to subscribe to and unsubscribe from subscription events.
    ///
    /// Returns an error if the WebSocket connection cannot be established.
    pub(crate) async fn new(ws_url: url::Url) -> Result<Self, crate::Error> {
        let mut ws = Self::connect(&ws_url).await?;
        let (response_tx, _) = tokio::sync::broadcast::channel::<Message>(100);

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let (subscription_tx, mut subscription_rx) =
            tokio::sync::mpsc::channel::<(Subscription, Method)>(100);
        let subscriptions = std::sync::Arc::new(scc::HashMap::<Subscription, ()>::new());

        let response_tx1 = response_tx.clone();
        let subscription_tx1 = subscription_tx.clone();
        let subscriptions1 = subscriptions.clone();
        let task = tokio::spawn(async move {
            let mut heartbeat_interval =
                tokio::time::interval(std::time::Duration::from_secs(Self::SEND_PING_INTERVAL));
            heartbeat_interval.tick().await;

            let mut reconnect = false;
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("Shutting down...");
                        if let Err(e) = ws.write_frame(fastwebsockets::Frame::close(1000, b"bye")).await {
                            tracing::error!("Failed to send close frame: {}", e);
                        }

                        break;
                    }
                    _ = heartbeat_interval.tick() => {
                        let ping_message = serde_json::json!({
                            "method": "ping"
                        });
                        let ping_frame = fastwebsockets::Frame::text(
                            fastwebsockets::Payload::Owned(ping_message.to_string().into_bytes())
                        );
                        if let Err(e) = ws.write_frame(ping_frame).await {
                            tracing::error!("Failed to send heartbeat ping: {}", e);

                            break;
                        }
                        tracing::debug!("Sent heartbeat ping");
                    }
                    Some((subscription, method)) = subscription_rx.recv() => {
                        let frame = subscription.into_frame(method);
                        if let Err(e) = ws.write_frame(frame).await {
                            tracing::error!("Failed to send subscription frame: {}", e);
                        }

                        tracing::debug!("Sent subscription frame");
                    }
                    frame = ws.read_frame() => {
                        match frame {
                            Ok(frame) => {
                                heartbeat_interval.reset();

                                match frame.opcode {
                                    fastwebsockets::OpCode::Close => {
                                        tracing::info!("WebSocket connection closed");

                                        break;
                                    }
                                    fastwebsockets::OpCode::Text => {
                                        let text = String::from_utf8_lossy(&frame.payload);
                                        let message = match serde_json::from_str::<Message>(&text) {
                                            Ok(msg) => msg,
                                            Err(e) => {
                                                tracing::error!("Failed to parse message {}: {}", text, e);

                                                continue;
                                            }
                                        };

                                        if let Err(e) = response_tx1.send(message) {
                                            tracing::error!("Failed to send message to response channel: {}", e);
                                        }
                                    }
                                    _ => {
                                        tracing::debug!("Received frame with opcode: {:?}", frame.opcode);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to read frame: {}", e);
                                if let Err(e) = response_tx1.send(Message::NoData) {
                                    tracing::error!("Failed to send message to response channel: {}", e);
                                }
                                reconnect = true;
                            }
                        }
                    }
                }

                if reconnect {
                    tracing::info!("Reconnecting...");
                    let mut attempts_made = 0;
                    loop {
                        attempts_made += 1;
                        if attempts_made > Self::MAX_RECONNECT_ATTEMPTS {
                            tracing::error!(
                                "Failed to reconnect after {} attempts",
                                Self::MAX_RECONNECT_ATTEMPTS
                            );

                            return;
                        }

                        match Self::connect(&ws_url).await {
                            Ok(conn) => {
                                ws = conn;
                                reconnect = false;

                                let mut iter = subscriptions1.first_entry_async().await;
                                while let Some(entry) = iter {
                                    if let Err(e) = subscription_tx1
                                        .send((entry.key().clone(), Method::Subscribe))
                                        .await
                                    {
                                        tracing::error!(
                                            "Failed to send subscription to subscription channel: {}",
                                            e
                                        );
                                    }

                                    iter = entry.next_async().await;
                                }

                                break;
                            }
                            Err(e) => {
                                tracing::error!("Failed to reconnect: {}", e);
                                let delay = std::cmp::min(60, 1 << attempts_made);
                                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            response_tx,
            subscription_tx,
            shutdown_tx,
            task,
            subscriptions,
        })
    }

    /// Subscribe to a subscription event.
    ///
    /// Returns a receiver that will receive messages from the subscription.
    ///
    /// The receiver will be closed when the subscription is closed.
    /// Returns an error if the subscription already exists.
    pub(crate) async fn subscribe(
        &mut self,
        subscription: Subscription,
    ) -> Result<tokio::sync::broadcast::Receiver<Message>, crate::Error> {
        self.subscriptions
            .insert_async(subscription.clone(), ())
            .await
            .map_err(|_| {
                crate::Error::SubscriptionAlreadyExists(
                    serde_json::to_string(&subscription).unwrap_or_default(),
                )
            })?;

        self.send_subscription_data(subscription, Method::Subscribe)
            .await
    }

    /// Unsubscribe from a subscription event.
    ///
    /// Returns a receiver that will receive messages from the subscription.
    ///
    /// The receiver will be closed when the subscription is closed.
    /// Returns an error if the subscription does not exist.
    pub(crate) async fn unsubscribe(
        &mut self,
        subscription: Subscription,
    ) -> Result<tokio::sync::broadcast::Receiver<Message>, crate::Error> {
        if self
            .subscriptions
            .remove_async(&subscription)
            .await
            .is_none()
        {
            return Err(crate::Error::SubscriptionNotFound);
        }

        self.send_subscription_data(subscription, Method::Unsubscribe)
            .await
    }

    pub(crate) async fn shutdown(self) {
        if self.shutdown_tx.send(()).is_err() {
            tracing::error!("Error sending shutdown signal");
        }

        self.task.await.unwrap();
    }

    pub(crate) fn get_listener(&self) -> tokio::sync::broadcast::Receiver<Message> {
        self.response_tx.subscribe()
    }

    async fn send_subscription_data(
        &self,
        subscription: Subscription,
        method: Method,
    ) -> Result<tokio::sync::broadcast::Receiver<Message>, crate::Error> {
        if let Err(e) = self.subscription_tx.send((subscription, method)).await {
            tracing::error!("Failed to send subscription to subscription channel: {}", e);
            return Err(crate::Error::WsManagerNotFound);
        }

        Ok(self.response_tx.subscribe())
    }

    async fn connect(
        ws_url: &url::Url,
    ) -> Result<
        fastwebsockets::FragmentCollector<hyper_util::rt::TokioIo<hyper::upgrade::Upgraded>>,
        crate::Error,
    > {
        tracing::info!("Connecting to Hyperliquid WebSocket: {}", ws_url);

        // Parse the WebSocket URL
        let host = ws_url
            .host_str()
            .ok_or_else(|| crate::Error::InvalidUrl(ws_url.to_string()))?;
        let port = ws_url.port().unwrap_or(443);
        let addr = format!("{}:{}", host, port);

        tracing::debug!("Connecting to address: {}", addr);

        // Connect to the WebSocket (TCP)
        let tcp_stream = tokio::net::TcpStream::connect(&addr)
            .await
            .map_err(|e| crate::Error::TcpStream(e.to_string()))?;
        tracing::debug!("TCP connection established");

        // Wrap the TCP stream in a TLS stream for wss://
        let tls_connector = native_tls::TlsConnector::new()
            .map_err(|e| crate::Error::TlsConnector(e.to_string()))?;
        let tls_connector = tokio_native_tls::TlsConnector::from(tls_connector);
        let tls_stream = tls_connector
            .connect(host, tcp_stream)
            .await
            .map_err(|e| crate::Error::TlsConnector(e.to_string()))?;
        tracing::debug!("TLS handshake completed");

        // Build the WebSocket upgrade request
        let req = hyper::Request::builder()
            .method("GET")
            .uri(ws_url.as_str())
            .header("Host", host)
            .header(hyper::header::UPGRADE, "websocket")
            .header(hyper::header::CONNECTION, "upgrade")
            .header(
                "Sec-WebSocket-Key",
                fastwebsockets::handshake::generate_key(),
            )
            .header("Sec-WebSocket-Version", "13")
            .body(http_body_util::Empty::<hyper::body::Bytes>::new())
            .map_err(|e| crate::Error::Websocket(e.to_string()))?;

        tracing::debug!("WebSocket upgrade request built");

        // Perform WebSocket handshake
        let (ws, response) = fastwebsockets::handshake::client(&SpawnExecutor, req, tls_stream)
            .await
            .map_err(|e| crate::Error::Websocket(e.to_string()))?;
        tracing::debug!(
            "WebSocket handshake completed, response status: {}",
            response.status()
        );

        Ok(fastwebsockets::FragmentCollector::new(ws))
    }
}

// Tie hyper's executor to tokio runtime
struct SpawnExecutor;

impl<Fut> hyper::rt::Executor<Fut> for SpawnExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        tokio::task::spawn(fut);
    }
}

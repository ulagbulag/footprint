use std::f64;

use anyhow::{bail, Error, Result};
use footprint_api::{Base, GlobalLocation, LocalLocation, LocationVectorScale, ObjectLocation};
use footprint_provider_api::env::env_var;
use url::Url;

#[cfg(feature = "metrics")]
pub async fn spawn() -> Result<()> {
    let metrics = ::std::sync::Arc::new(Metrics::new().await?);
    let tick = ::footprint_provider_api::env::Tick::new()?;

    tick.spawn_async(move || {
        let metrics = metrics.clone();
        async move { metrics.next().await.map(::footprint_provider_api::update) }
    });
    Ok(())
}

#[derive(Debug)]
pub struct Metrics {
    base: Base,
    client: Client,
    #[cfg(feature = "metrics")]
    id: usize,
    key: String,
    scale: LocationVectorScale,
    url: Url,
}

impl Metrics {
    pub async fn new() -> Result<Self> {
        fn env_base(key: &str) -> Result<f64> {
            env_var(&format!("FOOTPRINT_BASE_{key}"))
        }

        let url: Url = env_var("FOOTPRINT_API_URL")?;
        let client = match url.scheme() {
            #[cfg(feature = "metrics")]
            "http" | "https" => Client::Metrics(::reqwest::Client::new()),
            #[cfg(feature = "websocket")]
            "ws" | "wss" => {
                use futures::{SinkExt, StreamExt};

                let (client, _) = ::tungstenite::connect_async(url)
                    .await
                    .map_err(|error| ::anyhow::anyhow!("failed to connect: {error}"))?;

                let (mut writer, reader) = client.split();

                // subscribe feeds
                let message = ::serde_json::json!({
                    "headers": {
                        "X-ApiKey": "17254faec6a60f58458308763",
                    },
                    "method": "subscribe",
                    "resource": "/feeds/",
                });
                let payload =
                    ::tungstenite::tungstenite::Message::Binary(::serde_json::to_vec(&message)?);
                writer.send(payload).await?;

                Client::Websocket {
                    reader: reader.into(),
                    writer: writer.into(),
                }
            }
            scheme => bail!("unsupported scheme: {scheme}"),
        };

        Ok(Self {
            base: Base {
                location: GlobalLocation {
                    error_m: env_base("ERROR_M")?,
                    latitude: env_base("LATITUDE")?,
                    longitude: env_base("LONGITUDE")?,
                },
                rotation: env_base("ROTATION")? / f64::consts::PI * 180.0,
            },
            client,
            #[cfg(feature = "metrics")]
            id: env_var("FOOTPRINT_API_ID")?,
            key: env_var("FOOTPRINT_API_KEY")?,
            scale: LocationVectorScale {
                latitude: env_var("FOOTPRINT_SCALE_LATITUDE")?,
                longitude: env_var("FOOTPRINT_SCALE_LONGITUDE")?,
            },
            url: env_var("FOOTPRINT_API_URL")?,
        })
    }

    pub async fn next(&self) -> Result<ObjectLocation> {
        match &self.client {
            #[cfg(feature = "metrics")]
            Client::Metrics(client) => {
                let url = format!("{url}/{id}", url = &self.url, id = self.id);
                let entity: Entity = client
                    .get(url)
                    .header("X-ApiKey", &self.key)
                    .send()
                    .await?
                    .json()
                    .await?;

                let local_location = LocalLocation::try_from(&entity)?;
                Ok(self.calibrate(entity.id.parse()?, local_location))
            }

            #[cfg(feature = "websocket")]
            Client::Websocket { reader, writer } => {
                use std::time::Duration;

                use futures::{SinkExt, TryStreamExt};
                use tokio::{select, time::sleep};
                use tungstenite::tungstenite::Message;

                loop {
                    let message = {
                        let mut reader = reader.lock().await;

                        loop {
                            let future_next = reader.try_next();
                            let future_timeout = sleep(Duration::from_secs(30));
                            select! {
                                message = future_next => break message?.ok_or_else(|| ::anyhow::anyhow!("connection closed"))?,
                                () = future_timeout => {
                                    let mut writer = writer.lock().await;

                                    // send ping
                                    let payload = Message::Ping(Vec::default());
                                    writer.send(payload).await?;
                                    continue
                                },
                            };
                        }
                    };

                    // verify message
                    let message = match message {
                        Message::Text(message) if !message.is_empty() => message,
                        _ => continue,
                    };

                    let entity: WebsocketEntity = ::serde_json::from_str(&message)?;
                    match LocalLocation::try_from(&entity.body) {
                        Ok(local_location) => {
                            break Ok(self.calibrate(entity.body.id.parse()?, local_location))
                        }
                        Err(_) => continue,
                    }
                }
            }
        }
    }

    fn calibrate(&self, id: usize, local_location: LocalLocation) -> ObjectLocation {
        ObjectLocation {
            id,
            location: self.base + local_location * self.scale,
        }
    }
}

#[cfg(feature = "websocket")]
type WebSocketStream =
    ::tungstenite::WebSocketStream<::tungstenite::MaybeTlsStream<::tokio::net::TcpStream>>;

#[derive(Debug)]
enum Client {
    #[cfg(feature = "metrics")]
    Metrics(::reqwest::Client),
    #[cfg(feature = "websocket")]
    Websocket {
        reader: ::tokio::sync::Mutex<::futures::stream::SplitStream<WebSocketStream>>,
        writer: ::tokio::sync::Mutex<
            ::futures::stream::SplitSink<WebSocketStream, ::tungstenite::tungstenite::Message>,
        >,
    },
}

#[cfg(feature = "websocket")]
#[derive(::serde::Deserialize)]
struct WebsocketEntity {
    body: Entity,
    // resource: String,
}

#[derive(::serde::Deserialize)]
struct Entity {
    id: String,
    datastreams: Vec<DataStream>,
}

impl TryFrom<&Entity> for LocalLocation {
    type Error = Error;

    fn try_from(entity: &Entity) -> Result<Self, Self::Error> {
        Ok(LocalLocation {
            error_m: 0.0,
            x: entity.parse_value("posX")?,
            y: -entity.parse_value("posY")?,
        })
    }
}

impl Entity {
    fn get(&self, key: &str) -> Result<&DataStream> {
        self.datastreams
            .iter()
            .find(|datastream| datastream.id == key)
            .ok_or_else(|| ::anyhow::anyhow!("failed to get datastream: {key}"))
    }

    fn parse_value(&self, key: &str) -> Result<f64> {
        self.get(key)
            .and_then(|datastream| datastream.parse_value())
    }
}

#[derive(::serde::Deserialize)]
struct DataStream {
    id: String,
    current_value: String,
}

impl DataStream {
    fn parse_value(&self) -> Result<f64> {
        self.current_value.trim().parse().map_err(Into::into)
    }
}

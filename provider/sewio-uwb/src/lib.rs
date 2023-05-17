use std::sync::Arc;

use anyhow::{anyhow, Result};
use footprint_api::{Base, Location};
use footprint_provider_api::env::{env_var, Tick};
use reqwest::{Client, Url};
use serde::Deserialize;

pub fn spawn() -> Result<()> {
    let metrics = Arc::new(Metrics::new()?);
    let tick = Tick::new()?;

    tick.spawn_async(move || {
        let metrics = metrics.clone();
        async move { metrics.next().await.map(::footprint_provider_api::update) }
    });
    Ok(())
}

struct Metrics {
    base: Base,
    client: Client,
    id: usize,
    key: String,
    url: Url,
}

impl Metrics {
    fn new() -> Result<Self> {
        fn env_base(key: &str) -> Result<f64> {
            env_var(&format!("FOOTPRINT_BASE_{key}"))
        }

        Ok(Self {
            base: Base {
                location: Location {
                    error_m: env_base("ERROR_M")?,
                    latitude: env_base("LATITUDE")?,
                    longitude: env_base("LATITUDE")?,
                },
                rotation: env_base("ROTATION")?,
            },
            client: Client::new(),
            id: env_var("FOOTPRINT_API_ID")?,
            key: env_var("FOOTPRINT_API_KEY")?,
            url: env_var("FOOTPRINT_API_URL")?,
        })
    }

    async fn next(&self) -> Result<Location> {
        let url = format!("{url}/{id}", url = &self.url, id = self.id);
        let response: Response = self
            .client
            .get(url)
            .header("X-ApiKey", &self.key)
            .send()
            .await?
            .json()
            .await?;

        let local_location = Location {
            error_m: 0.0,
            latitude: response.parse_value("posY")?,
            longitude: response.parse_value("posX")?,
        };
        Ok(self.base + local_location)
    }
}

#[derive(Deserialize)]
struct Response {
    datastreams: Vec<DataStream>,
}

impl Response {
    fn get(&self, key: &str) -> Result<&DataStream> {
        self.datastreams
            .iter()
            .find(|datastream| datastream.id == key)
            .ok_or_else(|| anyhow!("failed to get datastream: {key}"))
    }

    fn parse_value(&self, key: &str) -> Result<f64> {
        self.get(key)
            .and_then(|datastream| datastream.parse_value())
    }
}

#[derive(Deserialize)]
struct DataStream {
    id: String,
    current_value: String,
}

impl DataStream {
    fn parse_value(&self) -> Result<f64> {
        self.current_value.trim().parse().map_err(Into::into)
    }
}

use anyhow::{bail, Result};
use footprint_api::Location;
use reqwest::Url;

pub struct Client {
    inner: ::reqwest::Client,
    url: Url,
}

impl Client {
    pub fn new(url: Url) -> Result<Self> {
        ::reqwest::ClientBuilder::new()
            .build()
            .map(|inner| Self { inner, url })
            .map_err(Into::into)
    }

    pub async fn put(&self, location: &Location) -> Result<()> {
        let status = self
            .inner
            .put(self.url.clone())
            .json(location)
            .send()
            .await?
            .status();

        if status.is_success() {
            Ok(())
        } else {
            let reason = status.canonical_reason().unwrap_or_default();
            bail!("{reason}")
        }
    }
}

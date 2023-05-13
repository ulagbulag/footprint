use std::str::FromStr;

use anyhow::{bail, Result};
use chrono::Utc;
use footprint_api::{DataRef, Location};
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use tokio::try_join;

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

    pub async fn get_raw(
        &self,
        DataRef {
            kind,
            name,
            namespace,
        }: &DataRef,
    ) -> Result<Option<Location>> {
        let labels = format!(
            "kind={kind:?},name={name:?},namespace={namespace:?}",
            namespace = namespace.as_deref().unwrap_or_default(),
        );

        match try_join!(
            {
                let query = format!(
                    "{metric}{{{labels}}}",
                    metric = ::footprint_provider_api::consts::METRIC_ERROR_M,
                );
                self.get_raw_one_by_query(query)
            },
            {
                let query = format!(
                    "{metric}{{{labels}}}",
                    metric = ::footprint_provider_api::consts::METRIC_LATITUDE,
                );
                self.get_raw_one_by_query(query)
            },
            {
                let query = format!(
                    "{metric}{{{labels}}}",
                    metric = ::footprint_provider_api::consts::METRIC_LONGITUDE,
                );
                self.get_raw_one_by_query(query)
            },
        )? {
            (Some(error_m), Some(latitude), Some(longitude)) => Ok(Some(Location {
                error_m,
                latitude,
                longitude,
            })),
            _ => Ok(None),
        }
    }

    pub async fn get_raw_vec_all_by_query(
        &self,
        query: impl AsRef<str>,
    ) -> Result<Vec<QueryData<f64>>> {
        match self.get_by_query_with(query).await? {
            QueryResponse::Success { data } => match data {
                QueryResult::Vector(data) => Ok(data),
            },
        }
    }

    pub async fn get_raw_one_by_query(&self, query: impl AsRef<str>) -> Result<Option<f64>> {
        self.get_raw_vec_all_by_query(query)
            .await
            .map(|mut values| values.pop().and_then(|data| data.value.get(1).copied()))
    }

    async fn get_by_query_with<T>(&self, query: impl AsRef<str>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let time = Utc::now().timestamp();
        let url = {
            let mut url = self.url.clone();
            url.set_path(&format!("{path}/api/v1/query", path = url.path()));
            url.set_query(Some(&format!(
                "query={query}&time={time}",
                query = query.as_ref(),
            )));
            url
        };

        let response = self.inner.get(url).send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let reason = status.canonical_reason().unwrap_or_default();
            bail!("{reason}")
        }
    }

    pub async fn put(&self, location: &Location) -> Result<()> {
        let url = self.url.clone();

        let response = self.inner.put(url).json(location).send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            let reason = status.canonical_reason().unwrap_or_default();
            bail!("{reason}")
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum QueryResponse<Data = QueryResult> {
    Success { data: Data },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "resultType", content = "result")]
pub enum QueryResult<Data = QueryData> {
    Vector(Vec<Data>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryData<Value = f64>
where
    Value: FromStr + DeserializeOwned,
    <Value as FromStr>::Err: ::std::error::Error,
{
    pub metric: QueryMetric,
    #[serde(deserialize_with = "deserialize_value_vec")]
    pub value: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryMetric {
    pub kind: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "is_empty")]
    pub namespace: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ValueOrString<T>(#[serde(deserialize_with = "deserialize_value")] T)
where
    T: FromStr + DeserializeOwned,
    <T as FromStr>::Err: ::std::error::Error;

fn deserialize_value_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + DeserializeOwned,
    <T as FromStr>::Err: ::std::error::Error,
{
    Deserialize::deserialize(deserializer).map(|values: Vec<ValueOrString<T>>| {
        values
            .into_iter()
            .map(|ValueOrString(value)| value)
            .collect()
    })
}

fn deserialize_value<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: ::std::error::Error,
{
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum ValueOrString<T> {
        String(String),
        Value(T),
    }

    Deserialize::deserialize(deserializer).and_then(|value| match value {
        ValueOrString::String(value) => value.parse().map_err(::serde::de::Error::custom),
        ValueOrString::Value(value) => Ok(value),
    })
}

fn is_empty(value: &Option<String>) -> bool {
    match value.as_ref() {
        Some(value) => value.is_empty(),
        None => true,
    }
}

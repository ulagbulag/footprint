use anyhow::Result;
use clap::{Parser, Subcommand};
use footprint_api::{DataRef, Location};
use footprint_client::Client;
use reqwest::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl Args {
    async fn run(self) -> Result<()> {
        self.command.run().await
    }
}

#[derive(Subcommand)]
enum Commands {
    Get(CommandGet),
    Query(CommandQuery),
    Update(CommandUpdate),
}

impl Commands {
    async fn run(self) -> Result<()> {
        match self {
            Self::Get(command) => command.run().await,
            Self::Query(command) => command.run().await,
            Self::Update(command) => command.run().await,
        }
    }
}

/// Create a resource from a file or from stdin.
#[derive(Parser)]
struct CommandGet {
    #[command(flatten)]
    client: ArgsClient,

    /// Search by name
    #[arg(long, value_name = "KIND")]
    kind: String,

    /// Search by name
    #[arg(long, value_name = "NAME")]
    name: String,

    /// Search by name
    #[arg(long, value_name = "NAMESPACE")]
    namespace: Option<String>,
}

impl CommandGet {
    async fn run(self) -> Result<()> {
        let data = DataRef {
            kind: self.kind,
            name: self.name,
            namespace: self.namespace,
        };

        // Push metrics
        let writer = Client::new(self.client.url)?;
        writer
            .get_raw(&data)
            .await
            .map_err(Into::into)
            .map(|value| {
                if let Some(value) = value {
                    println!("{value:?}")
                }
            })
    }
}

/// Create a resource from a file or from stdin.
#[derive(Parser)]
struct CommandQuery {
    #[command(flatten)]
    client: ArgsClient,

    /// Raw query
    #[arg(long, value_name = "QUERY")]
    query: String,
}

impl CommandQuery {
    async fn run(self) -> Result<()> {
        // Push metrics
        let writer = Client::new(self.client.url)?;
        writer
            .get_raw_vec_all_by_query(&self.query)
            .await
            .map_err(Into::into)
            .map(|value| value.into_iter().for_each(|value| println!("{value:?}")))
    }
}

/// Create a resource from a file or from stdin.
#[derive(Parser)]
struct CommandUpdate {
    #[command(flatten)]
    client: ArgsClient,

    /// Set an error as meter
    #[arg(long, value_name = "ERROR_M")]
    error_m: f64,

    /// Set a latitude
    #[arg(long, value_name = "LATITUDE")]
    latitude: f64,

    /// Set a longitude
    #[arg(long, value_name = "LONGITUDE")]
    longitude: f64,
}

impl CommandUpdate {
    async fn run(self) -> Result<()> {
        let location = Location {
            error_m: self.error_m,
            latitude: self.latitude,
            longitude: self.longitude,
        };

        // Push metrics
        let writer = Client::new(self.client.url)?;
        writer.put(&location).await.map_err(Into::into)
    }
}

/// Create a resource from a file or from stdin.
#[derive(Parser)]
struct ArgsClient {
    /// Prometheus URL
    #[arg(long, env = "FOOTPRINT_URL", value_name = "URL")]
    url: Url,
}

#[tokio::main]
async fn main() -> Result<()> {
    Args::parse().run().await
}

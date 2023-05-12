use anyhow::Result;
use clap::{Parser, Subcommand};
use footprint_api::Location;
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
    Update(CommandUpdate),
}

impl Commands {
    async fn run(self) -> Result<()> {
        match self {
            Self::Update(command) => command.run().await,
        }
    }
}

/// Create a resource from a file or from stdin.
#[derive(Parser)]
struct CommandUpdate {
    /// Set an error as meter
    #[arg(long, value_name = "ERROR_M")]
    error_m: f64,

    /// Set a latitude
    #[arg(long, value_name = "LATITUDE")]
    latitude: f64,

    /// Set a longitude
    #[arg(long, value_name = "LONGITUDE")]
    longitude: f64,

    #[command(flatten)]
    client: ArgsClient,
}

/// Create a resource from a file or from stdin.
#[derive(Parser)]
struct ArgsClient {
    /// Prometheus URL
    #[arg(long, env = "FOOTPRINT_URL", value_name = "URL")]
    url: Url,
}

impl CommandUpdate {
    async fn run(self) -> Result<()> {
        let location = Location {
            error_m: self.error_m,
            latitude: self.latitude,
            longitude: self.longitude,
        };

        // Push metrics
        let client = Client::new(self.client.url)?;
        client.put(&location).await.map_err(Into::into)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Args::parse().run().await
}

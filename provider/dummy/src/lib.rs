use std::{
    env::{self, VarError},
    time::Duration,
};

use anyhow::{anyhow, Result};
use footprint_api::Location;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::Normal;
use tokio::time::sleep;

pub fn spawn() -> Result<()> {
    let metrics = Metrics::new()?;
    let tick = Tick::new()?;

    ::tokio::task::spawn(run(metrics, tick));
    Ok(())
}

async fn run(mut metrics: Metrics, tick: Tick) {
    loop {
        ::footprint_provider_api::update(metrics.next());
        tick.sleep().await;
    }
}

struct Metrics {
    error_m: Metric,
    latitude: Metric,
    longitude: Metric,
}

impl Metrics {
    fn new() -> Result<Self> {
        Ok(Self {
            error_m: Metric::new("ERROR_M", true)?,
            latitude: Metric::new("LATITUDE", false)?,
            longitude: Metric::new("LONGITUDE", false)?,
        })
    }

    fn next(&mut self) -> Location {
        Location {
            error_m: self.error_m.next(),
            latitude: self.latitude.next(),
            longitude: self.longitude.next(),
        }
    }
}

struct Metric {
    base: f64,
    dist: Normal<f64>,
    positive: bool,
    radius: f64,

    last: f64,
}

impl Metric {
    fn new(key: &str, positive: bool) -> Result<Self> {
        let base = env_var(&format!("FOOTPRINT_DUMMY_BASE_{key}"))?;

        Ok(Self {
            base,
            dist: Normal::new(0.0f64, env_var(&format!("FOOTPRINT_DUMMY_STEP_VAR_{key}"))?)?,
            positive,
            radius: env_var(&format!("FOOTPRINT_DUMMY_RADIUS_{key}"))?,

            last: base,
        })
    }

    fn next(&mut self) -> f64 {
        let step = StdRng::from_entropy().sample::<f64, _>(&self.dist);
        let mut now = (self.last + step)
            .max(self.base - self.radius)
            .min(self.base + self.radius);

        if self.positive {
            now = now.max(0.0);
        }

        self.last = now;
        now
    }
}

struct Tick(Duration);

impl Tick {
    fn new() -> Result<Self> {
        let tick_sec = env_var("FOOTPRINT_DUMMY_TICK_SEC")?;
        Ok(Self(Duration::from_secs_f64(tick_sec)))
    }

    async fn sleep(&self) {
        sleep(self.0).await
    }
}

fn env_var(key: &str) -> Result<f64> {
    env::var(key)
        .map_err(|error| match error {
            VarError::NotPresent => anyhow!("no such environment variable: {key}"),
            error => anyhow!("failed to get environment variable: {key}: {error}"),
        })
        .and_then(|value| {
            value
                .parse()
                .map_err(|error| anyhow!("failed to parse environment variable: {key}: {error}"))
        })
}

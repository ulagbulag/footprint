use anyhow::Result;
use footprint_api::{Location, ObjectLocation};
use footprint_provider_api::env::{env_var, Tick};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::Normal;
use tokio::sync::Mutex;

pub async fn spawn() -> Result<()> {
    let metrics = ::std::sync::Arc::new(Metrics::new().await?);
    let tick = Tick::new()?;

    tick.spawn_async(move || {
        let metrics = metrics.clone();
        async move { metrics.next().await.map(::footprint_provider_api::update) }
    });
    Ok(())
}

struct Metrics {
    error_m: Mutex<Metric>,
    latitude: Mutex<Metric>,
    longitude: Mutex<Metric>,
}

impl Metrics {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            error_m: Mutex::new(Metric::new("ERROR_M", true)?),
            latitude: Mutex::new(Metric::new("LATITUDE", false)?),
            longitude: Mutex::new(Metric::new("LONGITUDE", false)?),
        })
    }

    pub async fn next(&self) -> Result<ObjectLocation> {
        Ok(ObjectLocation {
            id: 0,
            location: Location {
                error_m: self.error_m.lock().await.next(),
                latitude: self.latitude.lock().await.next(),
                longitude: self.longitude.lock().await.next(),
            },
        })
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
        let base = env_var(&format!("FOOTPRINT_BASE_{key}"))?;

        Ok(Self {
            base,
            dist: Normal::new(0.0f64, env_var(&format!("FOOTPRINT_STEP_VAR_{key}"))?)?,
            positive,
            radius: env_var(&format!("FOOTPRINT_RADIUS_{key}"))?,

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

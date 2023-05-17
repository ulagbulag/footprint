use anyhow::Result;
use footprint_api::Location;
use footprint_provider_api::env::{env_var, Tick};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::Normal;

pub fn spawn() -> Result<()> {
    let mut metrics = Metrics::new()?;
    let tick = Tick::new()?;

    tick.spawn(move || {
        ::footprint_provider_api::update(metrics.next());
        Ok(())
    });
    Ok(())
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

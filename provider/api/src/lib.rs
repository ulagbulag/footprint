#[cfg(feature = "env")]
pub mod env {
    use std::{
        env::{self, VarError},
        error::Error,
        future::Future,
        str::FromStr,
        time::Duration,
    };

    use anyhow::{anyhow, Result};
    use tokio::time::sleep;

    pub struct Tick {
        interval: Duration,
    }

    impl Tick {
        pub fn new() -> Result<Self> {
            let tick_sec = env_var("FOOTPRINT_TICK_SEC")?;
            Ok(Self {
                interval: Duration::from_secs_f64(tick_sec),
            })
        }

        pub fn spawn<F>(self, mut f: F)
        where
            F: 'static + Send + FnMut() -> Result<()>,
        {
            ::tokio::task::spawn(async move {
                loop {
                    if let Err(error) = f() {
                        eprintln!("failed to update data: {error}");
                    }
                    sleep(self.interval).await;
                }
            });
        }

        pub fn spawn_async<F, Fut>(self, f: F)
        where
            F: 'static + Send + Sync + Fn() -> Fut,
            Fut: 'static + Send + Future<Output = Result<()>>,
        {
            ::tokio::task::spawn(async move {
                loop {
                    if let Err(error) = f().await {
                        eprintln!("failed to update data: {error}");
                    }
                    sleep(self.interval).await;
                }
            });
        }
    }

    pub fn env_var<T>(key: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Error,
    {
        env::var(key)
            .map_err(|error| match error {
                VarError::NotPresent => anyhow!("no such environment variable: {key}"),
                error => anyhow!("failed to get environment variable: {key}: {error}"),
            })
            .and_then(|value| {
                value.parse().map_err(|error| {
                    anyhow!("failed to parse environment variable: {key}: {error}")
                })
            })
    }
}

#[cfg(feature = "metrics")]
pub fn register(registry: &::prometheus::Registry) -> ::prometheus::Result<()> {
    registry.register(Box::new(self::metrics::GAUGE_ERROR_M.clone()))?;
    registry.register(Box::new(self::metrics::GAUGE_LATITUDE.clone()))?;
    registry.register(Box::new(self::metrics::GAUGE_LONGITUDE.clone()))?;
    Ok(())
}

#[cfg(feature = "metrics")]
pub fn update(
    ::footprint_api::ObjectLocation {
        id: _,
        location:
            ::footprint_api::Location {
                error_m,
                latitude,
                longitude,
            },
    }: ::footprint_api::ObjectLocation,
) {
    self::metrics::GAUGE_ERROR_M.set(error_m);
    self::metrics::GAUGE_LATITUDE.set(latitude);
    self::metrics::GAUGE_LONGITUDE.set(longitude);
}

pub mod consts {
    pub const METRIC_ERROR_M: &str = "ulagbulag_footprint_error_m";
    pub const METRIC_LATITUDE: &str = "ulagbulag_footprint_latitude";
    pub const METRIC_LONGITUDE: &str = "ulagbulag_footprint_longitude";

    pub const LABEL_KIND: &str = "footprint_kind";
    pub const LABEL_NAME: &str = "footprint_name";
    pub const LABEL_NAMESPACE: &str = "footprint_namespace";
}

#[cfg(feature = "metrics")]
mod metrics {
    use std::env::{self, VarError};

    use prometheus::{
        core::{AtomicF64, GenericGauge},
        default_registry, Gauge, Opts,
    };

    ::lazy_static::lazy_static! {
        static ref LABEL_KIND: String = get_env_var("FOOTPRINT_KIND");
        static ref LABEL_NAME: String = get_env_var("FOOTPRINT_NAME");
        static ref LABEL_NAMESPACE: String = env::var("FOOTPRINT_NAMESPACE").ok().unwrap_or_default();

        pub(crate) static ref GAUGE_ERROR_M: GenericGauge<AtomicF64> = new_gauge(
            super::consts::METRIC_ERROR_M,
            "Geolocational Data: Error as Meter",
        );

        pub(crate) static ref GAUGE_LATITUDE: GenericGauge<AtomicF64> = new_gauge(
            super::consts::METRIC_LATITUDE,
            "Geolocational Data: Latitude",
        );

        pub(crate) static ref GAUGE_LONGITUDE: GenericGauge<AtomicF64> = new_gauge(
            super::consts::METRIC_LONGITUDE,
            "Geolocational Data: Longitude",
        );
    }

    fn get_env_var(key: &str) -> String {
        env::var(key).unwrap_or_else(|error| match error {
            VarError::NotPresent => panic!("environment variable {key} not set"),
            error => panic!("{error}"),
        })
    }

    fn get_opt(name: &str, help: &str) -> Opts {
        Opts::new(name, help)
            .const_label(super::consts::LABEL_KIND, LABEL_KIND.to_owned())
            .const_label(super::consts::LABEL_NAME, LABEL_NAME.to_owned())
            .const_label(super::consts::LABEL_NAMESPACE, LABEL_NAMESPACE.to_owned())
    }

    fn new_gauge(name: &str, help: &str) -> GenericGauge<AtomicF64> {
        let gauge = Gauge::with_opts(get_opt(name, help)).unwrap();

        // Register the gauge
        default_registry()
            .register(Box::new(gauge.clone()))
            .unwrap();

        gauge
    }
}

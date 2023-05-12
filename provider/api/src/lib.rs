use footprint_api::Location;
use prometheus::{Registry, Result};

pub fn register(registry: &Registry) -> Result<()> {
    registry.register(Box::new(self::metrics::GAUGE_ERROR_M.clone()))?;
    registry.register(Box::new(self::metrics::GAUGE_LATITUDE.clone()))?;
    registry.register(Box::new(self::metrics::GAUGE_LONGITUDE.clone()))?;
    Ok(())
}

pub fn update(
    Location {
        error_m,
        latitude,
        longitude,
    }: Location,
) {
    self::metrics::GAUGE_ERROR_M.set(error_m);
    self::metrics::GAUGE_LATITUDE.set(latitude);
    self::metrics::GAUGE_LONGITUDE.set(longitude);
}

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
            "ulagbulag_footprint_error_m",
            "Geolocational Data: Error as Meter",
        );

        pub(crate) static ref GAUGE_LATITUDE: GenericGauge<AtomicF64> = new_gauge(
            "ulagbulag_footprint_latitude",
            "Geolocational Data: Latitude",
        );

        pub(crate) static ref GAUGE_LONGITUDE: GenericGauge<AtomicF64> = new_gauge(
            "ulagbulag_footprint_longitude",
            "Geolocational Data: Longitude",
        );
    }

    fn get_env_var(key: &str) -> String {
        env::var("FOOTPRINT_KIND").unwrap_or_else(|error| match error {
            VarError::NotPresent => panic!("environment variable {key} not set"),
            error => panic!("{error}"),
        })
    }

    fn get_opt(name: &str, help: &str) -> Opts {
        Opts::new(name, help)
            .const_label("kind", LABEL_KIND.to_owned())
            .const_label("name", LABEL_NAME.to_owned())
            .const_label("namespace", LABEL_NAMESPACE.to_owned())
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

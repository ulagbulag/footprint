use std::{
    f64,
    ops::{Add, Mul},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LocationData {
    pub data: DataRef,
    pub location: Location,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DataRef {
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Base {
    pub location: GlobalLocation,
    pub rotation: f64,
}

impl Add<LocalLocation> for Base {
    type Output = Location;

    fn add(self, local: LocalLocation) -> Self::Output {
        let length = (local.x * local.x + local.y * local.y).sqrt();
        let rotation = self.rotation + local.x.atan2(local.y);

        let latitude = rotation.sin() * length;
        let longitude = rotation.cos() * length;

        Location {
            global: GlobalLocation {
                error_m: if local.error_m > 0.0 {
                    local.error_m
                } else {
                    self.location.error_m
                },
                latitude: self.location.latitude + latitude,
                longitude: self.location.longitude + longitude,
            },
            local,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObjectLocation {
    pub id: usize,
    #[serde(flatten)]
    pub location: Location,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Location {
    #[serde(flatten)]
    pub global: GlobalLocation,
    #[serde(flatten)]
    pub local: LocalLocation,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GlobalLocation {
    pub error_m: f64,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LocalLocation {
    #[serde(rename = "local_x")]
    pub x: f64,
    #[serde(rename = "local_y")]
    pub y: f64,
    #[serde(rename = "local_error_m")]
    pub error_m: f64,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LocationMetric {
    pub x_m: f64,
    pub y_m: f64,
    pub error_m: f64,
}

impl Add<LocationMetric> for Base {
    type Output = Location;

    fn add(self, metric: LocationMetric) -> Self::Output {
        const RADIUS_EARTH_KM: f64 = 6_378.137;
        const DEGREE_M: f64 = (1.0 / ((2.0 * f64::consts::PI / 360.0) * RADIUS_EARTH_KM)) / 1000.0;

        let local = LocalLocation {
            error_m: metric.error_m,
            x: metric.x_m * DEGREE_M,
            y: metric.y_m * DEGREE_M,
        };
        self.add(local)
    }
}

impl Mul<LocationVectorScale> for LocationMetric {
    type Output = Self;

    fn mul(self, scale: LocationVectorScale) -> Self::Output {
        Self {
            error_m: self.error_m,
            x_m: self.x_m * scale.latitude,
            y_m: self.y_m * scale.longitude,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LocationVectorScale {
    pub latitude: f64,
    pub longitude: f64,
}

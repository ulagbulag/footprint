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
    pub location: Location,
    pub rotation: f64,
}

impl Add<Location> for Base {
    type Output = Location;

    fn add(self, local: Location) -> Self::Output {
        let length = local.latitude * local.latitude + local.longitude * local.longitude;
        let rotation = self.rotation + local.latitude.atan2(local.longitude);

        let latitude = rotation.sin() * length;
        let longitude = rotation.cos() * length;

        Location {
            error_m: if local.error_m > 0.0 {
                local.error_m
            } else {
                self.location.error_m
            },
            latitude: self.location.latitude + latitude,
            longitude: self.location.longitude + longitude,
        }
    }
}

impl Add<LocationVector> for Base {
    type Output = Location;

    fn add(self, local: LocationVector) -> Self::Output {
        const RADIUS_EARTH_KM: f64 = 6_378.137;
        const DEGREE_M: f64 = (1.0 / ((2.0 * f64::consts::PI / 360.0) * RADIUS_EARTH_KM)) / 1000.0;

        let local = Location {
            error_m: local.error_m,
            latitude: local.latitude_m * DEGREE_M,
            longitude: local.longitude_m * DEGREE_M,
        };
        self.add(local)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Location {
    pub error_m: f64,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LocationVector {
    pub error_m: f64,
    pub latitude_m: f64,
    pub longitude_m: f64,
}

impl Mul<LocationVectorScale> for LocationVector {
    type Output = Self;

    fn mul(self, scale: LocationVectorScale) -> Self::Output {
        Self {
            error_m: self.error_m,
            latitude_m: self.latitude_m * scale.latitude,
            longitude_m: self.longitude_m * scale.longitude,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LocationVectorScale {
    pub latitude: f64,
    pub longitude: f64,
}

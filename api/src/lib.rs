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

impl Add<Location> for Base {
    type Output = Location;

    fn add(self, location: Location) -> Self::Output {
        let Location { global, local } = location;

        let length =
            (global.latitude * global.latitude + global.longitude * global.longitude).sqrt();
        let rotation = self.rotation + global.latitude.atan2(global.longitude);

        let latitude = rotation.sin() * length;
        let longitude = rotation.cos() * length;

        Location {
            global: GlobalLocation {
                error_m: if global.error_m > 0.0 {
                    global.error_m
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

impl Mul<LocationVectorScale> for LocalLocation {
    type Output = Location;

    fn mul(self, scale: LocationVectorScale) -> Self::Output {
        Location {
            global: GlobalLocation {
                error_m: self.error_m,
                latitude: self.y * scale.latitude,
                longitude: self.x * scale.longitude,
            },
            local: self,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LocationVectorScale {
    pub latitude: f64,
    pub longitude: f64,
}

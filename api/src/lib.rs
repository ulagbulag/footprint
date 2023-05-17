use std::ops::Add;

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
        let latitude = self.rotation.cos() * length;
        let longitude = self.rotation.sin() * length;

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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Location {
    pub error_m: f64,
    pub latitude: f64,
    pub longitude: f64,
}

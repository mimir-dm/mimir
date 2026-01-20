use super::types::Entry;
use crate::schema::catalog_vehicles;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub name: String,
    pub source: String,

    #[serde(rename = "vehicleType")]
    pub vehicle_type: Option<String>,

    pub size: Option<String>,
    pub page: Option<i32>,

    // Capacity
    #[serde(rename = "capCrew")]
    pub cap_crew: Option<i32>,
    #[serde(rename = "capPassenger")]
    pub cap_passenger: Option<i32>,
    #[serde(rename = "capCargo")]
    pub cap_cargo: Option<f32>,

    // Stats
    pub ac: Option<i32>,
    pub hp: Option<i32>,
    pub speed: Option<Speed>,
    pub pace: Option<i32>,

    // Dimensions [length, width]
    pub dimensions: Option<Vec<String>>,

    // Damage immunities, resistances, etc.
    pub immune: Option<Vec<String>>,
    pub resist: Option<Vec<String>>,
    pub vulnerable: Option<Vec<String>>,

    // Terrain types
    pub terrain: Option<Vec<String>>,

    // Weapons
    pub weapon: Option<Vec<VehicleWeapon>>,

    // Description entries
    #[serde(default)]
    pub entries: Vec<Entry>,

    // SRD name
    pub srd: Option<String>,

    // Fluff flags
    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,
    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
    #[serde(rename = "hasToken")]
    pub has_token: Option<bool>,

    #[serde(flatten)]
    pub other_fields: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speed {
    pub walk: Option<i32>,
    pub swim: Option<i32>,
    pub fly: Option<i32>,
    pub burrow: Option<i32>,
    pub climb: Option<i32>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleWeapon {
    pub name: String,
    pub count: Option<i32>,
    #[serde(default)]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleData {
    pub vehicle: Option<Vec<Vehicle>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VehicleSummary {
    pub name: String,
    pub source: String,
    pub vehicle_type: Option<String>,
    pub size: Option<String>,
    pub cap_crew: Option<i32>,
    pub cap_passenger: Option<i32>,
    pub capacity: String,
    pub terrain: Option<Vec<String>>,
    pub pace: Option<i32>,
    pub speed: Option<String>,
}

impl From<&Vehicle> for VehicleSummary {
    fn from(vehicle: &Vehicle) -> Self {
        let speed_str = vehicle
            .speed
            .as_ref()
            .map(|s| {
                let mut speeds = Vec::new();
                if let Some(walk) = s.walk {
                    speeds.push(format!("Walk {}", walk));
                }
                if let Some(swim) = s.swim {
                    speeds.push(format!("Swim {}", swim));
                }
                if let Some(fly) = s.fly {
                    speeds.push(format!("Fly {}", fly));
                }
                if speeds.is_empty() && vehicle.pace.is_some() {
                    format!("Pace {}", vehicle.pace.unwrap())
                } else {
                    speeds.join(", ")
                }
            })
            .or_else(|| vehicle.pace.map(|p| format!("Pace {}", p)));

        VehicleSummary {
            name: vehicle.name.clone(),
            source: vehicle.source.clone(),
            vehicle_type: vehicle.vehicle_type.clone(),
            size: vehicle.size.clone(),
            cap_crew: vehicle.cap_crew,
            cap_passenger: vehicle.cap_passenger,
            capacity: {
                let crew_str = vehicle
                    .cap_crew
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "—".to_string());
                let pass_str = vehicle
                    .cap_passenger
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "—".to_string());
                format!("{} / {}", crew_str, pass_str)
            },
            terrain: vehicle.terrain.clone(),
            pace: vehicle.pace,
            speed: speed_str,
        }
    }
}

// Database models
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = catalog_vehicles)]
pub struct CatalogVehicle {
    pub id: i32,
    pub name: String,
    pub vehicle_type: Option<String>,
    pub size: Option<String>,
    pub cap_crew: Option<i32>,
    pub cap_passenger: Option<i32>,
    pub pace: Option<i32>,
    pub speed_text: Option<String>,
    pub terrain_text: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_vehicle_json: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = catalog_vehicles)]
pub struct NewCatalogVehicle {
    pub name: String,
    pub vehicle_type: Option<String>,
    pub size: Option<String>,
    pub cap_crew: Option<i32>,
    pub cap_passenger: Option<i32>,
    pub pace: Option<i32>,
    pub speed_text: Option<String>,
    pub terrain_text: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_vehicle_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleFilters {
    pub name: Option<String>,
    pub sources: Option<Vec<String>>,
    pub vehicle_types: Option<Vec<String>>,
    pub sizes: Option<Vec<String>>,
    pub terrains: Option<Vec<String>>,
}

impl From<&CatalogVehicle> for VehicleSummary {
    fn from(catalog: &CatalogVehicle) -> Self {
        VehicleSummary {
            name: catalog.name.clone(),
            source: catalog.source.clone(),
            vehicle_type: catalog.vehicle_type.clone(),
            size: catalog.size.clone(),
            cap_crew: catalog.cap_crew,
            cap_passenger: catalog.cap_passenger,
            capacity: {
                let crew_str = catalog
                    .cap_crew
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "—".to_string());
                let pass_str = catalog
                    .cap_passenger
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "—".to_string());
                format!("{} / {}", crew_str, pass_str)
            },
            terrain: catalog
                .terrain_text
                .as_ref()
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
            pace: catalog.pace,
            speed: catalog.speed_text.clone(),
        }
    }
}

impl From<&Vehicle> for NewCatalogVehicle {
    fn from(vehicle: &Vehicle) -> Self {
        // Format speed text
        let speed_text = vehicle
            .speed
            .as_ref()
            .map(|s| {
                let mut speeds = Vec::new();
                if let Some(walk) = s.walk {
                    speeds.push(format!("Walk {}", walk));
                }
                if let Some(swim) = s.swim {
                    speeds.push(format!("Swim {}", swim));
                }
                if let Some(fly) = s.fly {
                    speeds.push(format!("Fly {}", fly));
                }
                if speeds.is_empty() && vehicle.pace.is_some() {
                    format!("Pace {}", vehicle.pace.unwrap())
                } else {
                    speeds.join(", ")
                }
            })
            .or_else(|| vehicle.pace.map(|p| format!("Pace {}", p)));

        // Format terrain text
        let terrain_text = vehicle
            .terrain
            .as_ref()
            .map(|t| t.join(", "))
            .filter(|s| !s.is_empty());

        Self {
            name: vehicle.name.clone(),
            vehicle_type: vehicle.vehicle_type.clone(),
            size: vehicle.size.clone(),
            cap_crew: vehicle.cap_crew,
            cap_passenger: vehicle.cap_passenger,
            pace: vehicle.pace,
            speed_text,
            terrain_text,
            source: vehicle.source.clone(),
            page: vehicle.page,
            full_vehicle_json: serde_json::to_string(vehicle).unwrap_or_default(),
        }
    }
}

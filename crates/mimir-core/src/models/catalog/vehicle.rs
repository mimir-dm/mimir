//! Vehicle Model
//!
//! Represents a vehicle (ships, airships, infernal war machines, etc.).

use crate::schema::vehicles;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A vehicle from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = vehicles)]
#[diesel(primary_key(id))]
pub struct Vehicle {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub vehicle_type: Option<String>,
    pub data: String,
    pub fluff: Option<String>,
}

impl Vehicle {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Get the human-readable type name.
    pub fn type_name(&self) -> &str {
        self.vehicle_type.as_ref().map_or("Unknown", |t| match t.as_str() {
            "SHIP" => "Ship",
            "INFERNAL" => "Infernal War Machine",
            "CREATURE" => "Creature",
            "OBJECT" => "Object",
            "SPELLJAMMER" => "Spelljammer",
            _ => "Unknown",
        })
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = vehicles)]
pub struct NewVehicle<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub vehicle_type: Option<&'a str>,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewVehicle<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, vehicle_type: None, data, fluff: None }
    }

    pub fn with_type(mut self, vehicle_type: &'a str) -> Self {
        self.vehicle_type = Some(vehicle_type);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vehicle() {
        let vehicle = NewVehicle::new("Galley", "GoS", r#"{"name":"Galley"}"#)
            .with_type("SHIP");
        assert_eq!(vehicle.name, "Galley");
        assert_eq!(vehicle.vehicle_type, Some("SHIP"));
    }
}

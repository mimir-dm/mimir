//! Vehicle catalog extraction types
//!
//! Types for deserializing 5etools vehicle JSON data.

use serde::{Deserialize, Serialize};

/// Vehicle speed information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleSpeed {
    #[serde(default)]
    pub walk: Option<i32>,
    #[serde(default)]
    pub swim: Option<i32>,
    #[serde(default)]
    pub fly: Option<i32>,
    #[serde(default)]
    pub burrow: Option<i32>,
    #[serde(default)]
    pub climb: Option<i32>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Vehicle weapon entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleWeapon {
    pub name: String,
    #[serde(default)]
    pub count: Option<i32>,
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,
}

/// A D&D 5e vehicle from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Vehicle type: SHIP, INFWAR, CREATURE, OBJECT, SHP, AIR, etc.
    #[serde(default)]
    pub vehicle_type: Option<String>,

    /// Size code
    #[serde(default)]
    pub size: Option<String>,

    // Capacity
    #[serde(default)]
    pub cap_crew: Option<i32>,
    #[serde(default)]
    pub cap_passenger: Option<i32>,
    #[serde(default)]
    pub cap_cargo: Option<f32>,

    // Stats
    #[serde(default)]
    pub ac: Option<i32>,
    #[serde(default)]
    pub hp: Option<i32>,
    #[serde(default)]
    pub speed: Option<VehicleSpeed>,
    #[serde(default)]
    pub pace: Option<i32>,

    /// Dimensions [length, width]
    #[serde(default)]
    pub dimensions: Option<Vec<String>>,

    // Damage types
    #[serde(default)]
    pub immune: Option<Vec<String>>,
    #[serde(default)]
    pub resist: Option<Vec<String>>,
    #[serde(default)]
    pub vulnerable: Option<Vec<String>>,

    /// Terrain types the vehicle can traverse
    #[serde(default)]
    pub terrain: Option<Vec<String>>,

    /// Weapons mounted on the vehicle
    #[serde(default)]
    pub weapon: Option<Vec<VehicleWeapon>>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    /// SRD name (if different)
    #[serde(default)]
    pub srd: Option<String>,

    // Flags
    #[serde(default)]
    pub has_fluff: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,
    #[serde(default)]
    pub has_token: Option<bool>,
}

/// Container for vehicle data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleData {
    #[serde(default)]
    pub vehicle: Vec<Vehicle>,
}

/// Vehicle fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub images: Option<Vec<serde_json::Value>>,
}

/// Container for vehicle fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleFluffData {
    #[serde(default)]
    pub vehicle_fluff: Vec<VehicleFluff>,
}

/// Get human-readable name for vehicle type code.
pub fn get_vehicle_type_name(code: &str) -> &'static str {
    match code {
        "SHIP" => "Ship",
        "SHP" => "Ship",
        "INFWAR" => "Infernal War Machine",
        "CREATURE" => "Creature",
        "OBJECT" => "Object",
        "AIR" => "Airship",
        "SPELLJAMMER" => "Spelljammer",
        _ => "Vehicle",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_ship() {
        let json = json!({
            "name": "Galley",
            "source": "DMG",
            "vehicleType": "SHIP",
            "size": "G",
            "capCrew": 80,
            "capPassenger": 40,
            "capCargo": 150.0,
            "pace": 4
        });
        let vehicle: Vehicle = serde_json::from_value(json).unwrap();
        assert_eq!(vehicle.name, "Galley");
        assert_eq!(vehicle.vehicle_type, Some("SHIP".to_string()));
        assert_eq!(vehicle.cap_crew, Some(80));
    }

    #[test]
    fn test_infernal_war_machine() {
        let json = json!({
            "name": "Demon Grinder",
            "source": "BGDIA",
            "vehicleType": "INFWAR",
            "size": "H",
            "ac": 19,
            "hp": 200,
            "speed": {"walk": 100}
        });
        let vehicle: Vehicle = serde_json::from_value(json).unwrap();
        assert_eq!(vehicle.vehicle_type, Some("INFWAR".to_string()));
        assert_eq!(vehicle.ac, Some(19));
        assert_eq!(vehicle.speed.unwrap().walk, Some(100));
    }

    #[test]
    fn test_vehicle_with_weapons() {
        let json = json!({
            "name": "Warship",
            "source": "DMG",
            "weapon": [
                {"name": "Ballista", "count": 3},
                {"name": "Mangonel", "count": 1}
            ]
        });
        let vehicle: Vehicle = serde_json::from_value(json).unwrap();
        let weapons = vehicle.weapon.unwrap();
        assert_eq!(weapons.len(), 2);
        assert_eq!(weapons[0].count, Some(3));
    }

    #[test]
    fn test_vehicle_data() {
        let json = json!({
            "vehicle": [
                {"name": "Galley", "source": "DMG"},
                {"name": "Longship", "source": "DMG"}
            ]
        });
        let data: VehicleData = serde_json::from_value(json).unwrap();
        assert_eq!(data.vehicle.len(), 2);
    }

    #[test]
    fn test_vehicle_type_name() {
        assert_eq!(get_vehicle_type_name("SHIP"), "Ship");
        assert_eq!(get_vehicle_type_name("INFWAR"), "Infernal War Machine");
        assert_eq!(get_vehicle_type_name("AIR"), "Airship");
    }
}

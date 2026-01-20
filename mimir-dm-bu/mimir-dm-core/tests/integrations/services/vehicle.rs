//! Integration tests for VehicleService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::vehicle::VehicleFilters;
use mimir_dm_core::services::VehicleService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_vehicle_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_vehicle_data(conn: &mut SqliteConnection) {
    // Schema: name, vehicle_type, size, cap_crew, cap_passenger, pace, speed_text, terrain_text, source, full_vehicle_json
    let vehicles = vec![
        (
            "Rowboat",
            "WATER",
            "L",
            1,
            3,
            3,
            "3 mph",
            "water",
            "DMG",
            r#"{"name":"Rowboat","source":"DMG","vehicleType":"WATER","size":"L","terrain":["water"]}"#,
        ),
        (
            "Keelboat",
            "WATER",
            "G",
            3,
            6,
            1,
            "1 mph",
            "water",
            "DMG",
            r#"{"name":"Keelboat","source":"DMG","vehicleType":"WATER","size":"G","terrain":["water"]}"#,
        ),
        (
            "Longship",
            "WATER",
            "G",
            40,
            150,
            3,
            "3 mph",
            "water",
            "DMG",
            r#"{"name":"Longship","source":"DMG","vehicleType":"WATER","size":"G","terrain":["water"]}"#,
        ),
        (
            "Galley",
            "WATER",
            "G",
            80,
            40,
            4,
            "4 mph",
            "water",
            "DMG",
            r#"{"name":"Galley","source":"DMG","vehicleType":"WATER","size":"G","terrain":["water"]}"#,
        ),
        (
            "Warship",
            "WATER",
            "G",
            60,
            60,
            2,
            "2 mph",
            "water",
            "DMG",
            r#"{"name":"Warship","source":"DMG","vehicleType":"WATER","size":"G","terrain":["water"]}"#,
        ),
        (
            "Sailing Ship",
            "WATER",
            "G",
            20,
            20,
            2,
            "2 mph",
            "water",
            "DMG",
            r#"{"name":"Sailing Ship","source":"DMG","vehicleType":"WATER","size":"G","terrain":["water"]}"#,
        ),
        (
            "Cart",
            "LAND",
            "L",
            1,
            2,
            1,
            "1 mph",
            "land",
            "PHB",
            r#"{"name":"Cart","source":"PHB","vehicleType":"LAND","size":"L","terrain":["land"]}"#,
        ),
        (
            "Wagon",
            "LAND",
            "L",
            1,
            4,
            1,
            "1 mph",
            "land",
            "PHB",
            r#"{"name":"Wagon","source":"PHB","vehicleType":"LAND","size":"L","terrain":["land"]}"#,
        ),
        (
            "Carriage",
            "LAND",
            "L",
            1,
            4,
            2,
            "2 mph",
            "land",
            "PHB",
            r#"{"name":"Carriage","source":"PHB","vehicleType":"LAND","size":"L","terrain":["land"]}"#,
        ),
        (
            "Chariot",
            "LAND",
            "M",
            1,
            0,
            4,
            "4 mph",
            "land",
            "PHB",
            r#"{"name":"Chariot","source":"PHB","vehicleType":"LAND","size":"M","terrain":["land"]}"#,
        ),
        (
            "Airship",
            "AIR",
            "G",
            10,
            20,
            8,
            "8 mph",
            "air",
            "DMG",
            r#"{"name":"Airship","source":"DMG","vehicleType":"AIR","size":"G","terrain":["air"]}"#,
        ),
        (
            "Apparatus of Kwalish",
            "INFERNAL",
            "L",
            1,
            1,
            3,
            "3 mph",
            "land, water",
            "DMG",
            r#"{"name":"Apparatus of Kwalish","source":"DMG","vehicleType":"INFERNAL","size":"L","terrain":["land","water"]}"#,
        ),
    ];

    for (
        name,
        vehicle_type,
        size,
        cap_crew,
        cap_passenger,
        pace,
        speed_text,
        terrain_text,
        source,
        json,
    ) in vehicles
    {
        diesel::sql_query(
            "INSERT INTO catalog_vehicles (name, vehicle_type, size, cap_crew, cap_passenger, pace, speed_text, terrain_text, source, full_vehicle_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(vehicle_type)
        .bind::<diesel::sql_types::Text, _>(size)
        .bind::<diesel::sql_types::Integer, _>(cap_crew)
        .bind::<diesel::sql_types::Integer, _>(cap_passenger)
        .bind::<diesel::sql_types::Integer, _>(pace)
        .bind::<diesel::sql_types::Text, _>(speed_text)
        .bind::<diesel::sql_types::Text, _>(terrain_text)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_vehicles_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let filters = VehicleFilters {
        name: None,
        sources: None,
        vehicle_types: None,
        sizes: None,
        terrains: None,
    };
    let results = service
        .search_vehicles(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 12, "Should return all 12 seeded vehicles");
}

#[test]
fn test_search_vehicles_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let filters = VehicleFilters {
        name: Some("ship".to_string()),
        sources: None,
        vehicle_types: None,
        sizes: None,
        terrains: None,
    };
    let results = service
        .search_vehicles(filters)
        .expect("Search should succeed");

    // Longship, Warship, Sailing Ship, Airship
    assert_eq!(results.len(), 4, "Should return 4 vehicles matching 'ship'");
}

#[test]
fn test_search_vehicles_by_vehicle_type() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let filters = VehicleFilters {
        name: None,
        sources: None,
        vehicle_types: Some(vec!["WATER".to_string()]),
        sizes: None,
        terrains: None,
    };
    let results = service
        .search_vehicles(filters)
        .expect("Search should succeed");

    // Rowboat, Keelboat, Longship, Galley, Warship, Sailing Ship
    assert_eq!(results.len(), 6, "Should return 6 water vehicles");
}

#[test]
fn test_search_vehicles_by_size() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let filters = VehicleFilters {
        name: None,
        sources: None,
        vehicle_types: None,
        sizes: Some(vec!["G".to_string()]),
        terrains: None,
    };
    let results = service
        .search_vehicles(filters)
        .expect("Search should succeed");

    // Keelboat, Longship, Galley, Warship, Sailing Ship, Airship
    assert_eq!(results.len(), 6, "Should return 6 Gargantuan vehicles");
}

#[test]
fn test_search_vehicles_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let filters = VehicleFilters {
        name: None,
        sources: Some(vec!["PHB".to_string()]),
        vehicle_types: None,
        sizes: None,
        terrains: None,
    };
    let results = service
        .search_vehicles(filters)
        .expect("Search should succeed");

    // Cart, Wagon, Carriage, Chariot
    assert_eq!(results.len(), 4, "Should return 4 PHB vehicles");
}

#[test]
fn test_search_vehicles_by_terrain() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let filters = VehicleFilters {
        name: None,
        sources: None,
        vehicle_types: None,
        sizes: None,
        terrains: Some(vec!["land".to_string()]),
    };
    let results = service
        .search_vehicles(filters)
        .expect("Search should succeed");

    // Cart, Wagon, Carriage, Chariot, Apparatus of Kwalish (has land, water)
    assert_eq!(results.len(), 5, "Should return 5 land vehicles");
}

#[test]
fn test_search_vehicles_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let filters = VehicleFilters {
        name: None,
        sources: Some(vec!["DMG".to_string()]),
        vehicle_types: Some(vec!["WATER".to_string()]),
        sizes: Some(vec!["G".to_string()]),
        terrains: None,
    };
    let results = service
        .search_vehicles(filters)
        .expect("Search should succeed");

    // Keelboat, Longship, Galley, Warship, Sailing Ship (all DMG, WATER, Gargantuan)
    assert_eq!(
        results.len(),
        5,
        "Should return 5 DMG Gargantuan water vehicles"
    );
}

#[test]
fn test_search_vehicles_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let filters = VehicleFilters {
        name: Some("Nonexistent".to_string()),
        sources: None,
        vehicle_types: None,
        sizes: None,
        terrains: None,
    };
    let results = service
        .search_vehicles(filters)
        .expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_vehicle_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let vehicle = service
        .get_vehicle_by_name_and_source("Rowboat", "DMG")
        .expect("Should get vehicle");

    assert!(vehicle.is_some());
    let vehicle = vehicle.unwrap();
    assert_eq!(vehicle.name, "Rowboat");
    assert_eq!(vehicle.source, "DMG");
}

#[test]
fn test_get_vehicle_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let vehicle = service
        .get_vehicle_by_name_and_source("Nonexistent", "DMG")
        .expect("Should not error");

    assert!(vehicle.is_none());
}

#[test]
fn test_get_all_vehicle_types() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let types = service
        .get_all_vehicle_types()
        .expect("Should get vehicle types");

    // AIR, INFERNAL, LAND, WATER
    assert_eq!(types.len(), 4, "Should have 4 vehicle types");
    assert!(types.contains(&"WATER".to_string()));
    assert!(types.contains(&"LAND".to_string()));
    assert!(types.contains(&"AIR".to_string()));
}

#[test]
fn test_get_all_sizes() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let sizes = service.get_all_sizes().expect("Should get sizes");

    // G, L, M
    assert_eq!(sizes.len(), 3, "Should have 3 sizes");
    assert!(sizes.contains(&"G".to_string()));
    assert!(sizes.contains(&"L".to_string()));
    assert!(sizes.contains(&"M".to_string()));
}

#[test]
fn test_get_all_terrains() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let terrains = service.get_all_terrains().expect("Should get terrains");

    // air, land, water (and possibly "land, water" parsed)
    assert!(terrains.len() >= 3, "Should have at least 3 terrains");
    assert!(terrains.contains(&"water".to_string()));
    assert!(terrains.contains(&"land".to_string()));
}

#[test]
fn test_get_vehicle_count_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VehicleService::new(&mut conn);

    let counts = service
        .get_vehicle_count_by_source()
        .expect("Should get counts");

    let dmg_count = counts.iter().find(|(s, _)| s == "DMG").map(|(_, c)| *c);
    let phb_count = counts.iter().find(|(s, _)| s == "PHB").map(|(_, c)| *c);

    assert_eq!(dmg_count, Some(8), "DMG should have 8 vehicles");
    assert_eq!(phb_count, Some(4), "PHB should have 4 vehicles");
}

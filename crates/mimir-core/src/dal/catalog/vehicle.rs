//! Vehicle Data Access Layer
//!
//! Database operations for vehicles.

use crate::models::catalog::{NewVehicle, Vehicle};
use crate::schema::vehicles;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new vehicle.
pub fn insert_vehicle(conn: &mut SqliteConnection, vehicle: &NewVehicle) -> QueryResult<i32> {
    diesel::insert_into(vehicles::table)
        .values(vehicle)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple vehicles in a batch.
pub fn insert_vehicles(
    conn: &mut SqliteConnection,
    vehicles: &[NewVehicle],
) -> QueryResult<usize> {
    diesel::insert_into(vehicles::table)
        .values(vehicles)
        .execute(conn)
}

/// Get a vehicle by its ID.
pub fn get_vehicle(conn: &mut SqliteConnection, id: i32) -> QueryResult<Vehicle> {
    vehicles::table
        .filter(vehicles::id.eq(id))
        .first(conn)
}

/// Get a vehicle by name and source.
pub fn get_vehicle_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Vehicle>> {
    vehicles::table
        .filter(vehicles::name.eq(name))
        .filter(vehicles::source.eq(source))
        .first(conn)
        .optional()
}

/// List all vehicles, ordered by name.
pub fn list_vehicles(conn: &mut SqliteConnection) -> QueryResult<Vec<Vehicle>> {
    vehicles::table.order(vehicles::name.asc()).load(conn)
}

/// List vehicles from a specific source.
pub fn list_vehicles_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Vehicle>> {
    vehicles::table
        .filter(vehicles::source.eq(source))
        .order(vehicles::name.asc())
        .load(conn)
}

/// List vehicles by type (e.g., "land", "water", "air").
pub fn list_vehicles_by_type(
    conn: &mut SqliteConnection,
    vehicle_type: &str,
) -> QueryResult<Vec<Vehicle>> {
    vehicles::table
        .filter(vehicles::vehicle_type.eq(vehicle_type))
        .order(vehicles::name.asc())
        .load(conn)
}

/// Delete a vehicle by its ID.
pub fn delete_vehicle(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(vehicles::table.filter(vehicles::id.eq(id))).execute(conn)
}

/// Delete all vehicles from a specific source.
pub fn delete_vehicles_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(vehicles::table.filter(vehicles::source.eq(source))).execute(conn)
}

/// Count all vehicles.
pub fn count_vehicles(conn: &mut SqliteConnection) -> QueryResult<i64> {
    vehicles::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_vehicle_crud() {
        let mut conn = setup_test_db_with_sources();

        let vehicle = NewVehicle::new("Rowboat", "DMG", r#"{"name":"Rowboat"}"#)
            .with_type("water");
        let id = insert_vehicle(&mut conn, &vehicle).expect("Failed to insert");

        let retrieved = get_vehicle(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Rowboat");
        assert_eq!(retrieved.vehicle_type, Some("water".to_string()));

        let by_name = get_vehicle_by_name(&mut conn, "Rowboat", "DMG")
            .expect("Failed to query")
            .expect("Vehicle not found");
        assert_eq!(by_name.name, "Rowboat");

        delete_vehicle(&mut conn, id).expect("Failed to delete");
        assert_eq!(count_vehicles(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_list_vehicles() {
        let mut conn = setup_test_db_with_sources();

        let vehicles = vec![
            NewVehicle::new("Carriage", "DMG", r#"{}"#).with_type("land"),
            NewVehicle::new("Galley", "DMG", r#"{}"#).with_type("water"),
            NewVehicle::new("Rowboat", "DMG", r#"{}"#).with_type("water"),
        ];
        insert_vehicles(&mut conn, &vehicles).expect("Failed to insert");

        let list = list_vehicles(&mut conn).expect("Failed to list");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name, "Carriage"); // Alphabetical

        let water = list_vehicles_by_type(&mut conn, "water").expect("Failed to list");
        assert_eq!(water.len(), 2);
    }
}

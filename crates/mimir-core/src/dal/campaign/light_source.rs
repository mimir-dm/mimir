//! LightSource Data Access Layer
//!
//! Database operations for dynamic light sources on maps.

use crate::models::campaign::{LightSource, NewLightSource, UpdateLightSource};
use crate::schema::light_sources;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new light source.
pub fn insert_light_source(
    conn: &mut SqliteConnection,
    light: &NewLightSource,
) -> QueryResult<String> {
    diesel::insert_into(light_sources::table)
        .values(light)
        .execute(conn)?;

    Ok(light.id.to_string())
}

/// Get a light source by ID.
pub fn get_light_source(conn: &mut SqliteConnection, id: &str) -> QueryResult<LightSource> {
    light_sources::table.find(id).first(conn)
}

/// Get a light source by ID, returning None if not found.
pub fn get_light_source_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<LightSource>> {
    light_sources::table.find(id).first(conn).optional()
}

/// List all light sources for a map.
pub fn list_light_sources(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<Vec<LightSource>> {
    light_sources::table
        .filter(light_sources::map_id.eq(map_id))
        .load(conn)
}

/// List active light sources for a map.
pub fn list_active_light_sources(
    conn: &mut SqliteConnection,
    map_id: &str,
) -> QueryResult<Vec<LightSource>> {
    light_sources::table
        .filter(light_sources::map_id.eq(map_id))
        .filter(light_sources::active.eq(1))
        .load(conn)
}

/// Update a light source.
pub fn update_light_source(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateLightSource,
) -> QueryResult<usize> {
    diesel::update(light_sources::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a light source by ID.
pub fn delete_light_source(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(light_sources::table.find(id)).execute(conn)
}

/// Delete all light sources for a map.
pub fn delete_all_light_sources(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<usize> {
    diesel::delete(light_sources::table.filter(light_sources::map_id.eq(map_id))).execute(conn)
}

/// Check if a light source exists.
pub fn light_source_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(light_sources::table.find(id))).get_result(conn)
}

/// Count light sources for a map.
pub fn count_light_sources(conn: &mut SqliteConnection, map_id: &str) -> QueryResult<i64> {
    light_sources::table
        .filter(light_sources::map_id.eq(map_id))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::campaign::light_presets as presets;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        conn.batch_execute(
            r#"
            CREATE TABLE campaigns (id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL);
            CREATE TABLE maps (
                id TEXT PRIMARY KEY NOT NULL,
                campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
                name TEXT NOT NULL
            );
            CREATE TABLE light_sources (
                id TEXT PRIMARY KEY NOT NULL,
                map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
                grid_x INTEGER NOT NULL,
                grid_y INTEGER NOT NULL,
                name TEXT,
                bright_radius INTEGER NOT NULL,
                dim_radius INTEGER NOT NULL,
                color TEXT,
                active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO campaigns (id, name) VALUES ('camp-1', 'Test Campaign');
            INSERT INTO maps (id, campaign_id, name) VALUES ('map-1', 'camp-1', 'Dungeon');
            PRAGMA foreign_keys = ON;
            "#,
        )
        .expect("Failed to create tables");

        conn
    }

    #[test]
    fn test_insert_and_get_light_source() {
        let mut conn = setup_test_db();

        let light = NewLightSource::new("ls-1", "map-1", 5, 10, 20, 40);
        let id = insert_light_source(&mut conn, &light).expect("Failed to insert");
        assert_eq!(id, "ls-1");

        let retrieved = get_light_source(&mut conn, "ls-1").expect("Failed to get");
        assert_eq!(retrieved.bright_radius, 20);
        assert_eq!(retrieved.dim_radius, 40);
        assert!(retrieved.is_active());
    }

    #[test]
    fn test_insert_torch() {
        let mut conn = setup_test_db();

        let torch = NewLightSource::torch("ls-1", "map-1", 5, 10);
        insert_light_source(&mut conn, &torch).expect("Failed to insert");

        let retrieved = get_light_source(&mut conn, "ls-1").expect("Failed to get");
        assert_eq!(retrieved.name, Some("Torch".to_string()));
        assert_eq!(retrieved.bright_radius, presets::TORCH_BRIGHT);
        assert_eq!(retrieved.dim_radius, presets::TORCH_DIM);
        assert_eq!(retrieved.color, Some(presets::TORCH_COLOR.to_string()));
    }

    #[test]
    fn test_insert_lantern() {
        let mut conn = setup_test_db();

        let lantern = NewLightSource::lantern("ls-1", "map-1", 0, 0);
        insert_light_source(&mut conn, &lantern).expect("Failed to insert");

        let retrieved = get_light_source(&mut conn, "ls-1").expect("Failed to get");
        assert_eq!(retrieved.name, Some("Lantern".to_string()));
        assert_eq!(retrieved.bright_radius, presets::LANTERN_BRIGHT);
    }

    #[test]
    fn test_insert_inactive() {
        let mut conn = setup_test_db();

        let light = NewLightSource::new("ls-1", "map-1", 0, 0, 20, 40).inactive();
        insert_light_source(&mut conn, &light).expect("Failed to insert");

        let retrieved = get_light_source(&mut conn, "ls-1").expect("Failed to get");
        assert!(!retrieved.is_active());
    }

    #[test]
    fn test_list_light_sources() {
        let mut conn = setup_test_db();

        let light1 = NewLightSource::torch("ls-1", "map-1", 1, 1);
        let light2 = NewLightSource::lantern("ls-2", "map-1", 2, 2);
        insert_light_source(&mut conn, &light1).expect("Failed to insert");
        insert_light_source(&mut conn, &light2).expect("Failed to insert");

        let lights = list_light_sources(&mut conn, "map-1").expect("Failed to list");
        assert_eq!(lights.len(), 2);
    }

    #[test]
    fn test_list_active_light_sources() {
        let mut conn = setup_test_db();

        let active = NewLightSource::torch("ls-1", "map-1", 1, 1);
        let inactive = NewLightSource::torch("ls-2", "map-1", 2, 2).inactive();
        insert_light_source(&mut conn, &active).expect("Failed to insert");
        insert_light_source(&mut conn, &inactive).expect("Failed to insert");

        let lights = list_active_light_sources(&mut conn, "map-1").expect("Failed to list");
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].id, "ls-1");
    }

    #[test]
    fn test_turn_on_off() {
        let mut conn = setup_test_db();

        let light = NewLightSource::torch("ls-1", "map-1", 0, 0);
        insert_light_source(&mut conn, &light).expect("Failed to insert");

        // Turn off
        let update = UpdateLightSource::turn_off("2024-01-20T12:00:00Z");
        update_light_source(&mut conn, "ls-1", &update).expect("Failed to update");

        let retrieved = get_light_source(&mut conn, "ls-1").expect("Failed to get");
        assert!(!retrieved.is_active());

        // Turn on
        let update = UpdateLightSource::turn_on("2024-01-20T12:01:00Z");
        update_light_source(&mut conn, "ls-1", &update).expect("Failed to update");

        let retrieved = get_light_source(&mut conn, "ls-1").expect("Failed to get");
        assert!(retrieved.is_active());
    }

    #[test]
    fn test_update_position() {
        let mut conn = setup_test_db();

        let light = NewLightSource::torch("ls-1", "map-1", 0, 0);
        insert_light_source(&mut conn, &light).expect("Failed to insert");

        let update = UpdateLightSource::set_position(15, 20, "2024-01-20T12:00:00Z");
        update_light_source(&mut conn, "ls-1", &update).expect("Failed to update");

        let retrieved = get_light_source(&mut conn, "ls-1").expect("Failed to get");
        assert_eq!(retrieved.grid_x, 15);
        assert_eq!(retrieved.grid_y, 20);
    }

    #[test]
    fn test_update_radii() {
        let mut conn = setup_test_db();

        let light = NewLightSource::new("ls-1", "map-1", 0, 0, 10, 20);
        insert_light_source(&mut conn, &light).expect("Failed to insert");

        let update = UpdateLightSource::set_radii(60, 120, "2024-01-20T12:00:00Z");
        update_light_source(&mut conn, "ls-1", &update).expect("Failed to update");

        let retrieved = get_light_source(&mut conn, "ls-1").expect("Failed to get");
        assert_eq!(retrieved.bright_radius, 60);
        assert_eq!(retrieved.dim_radius, 120);
    }

    #[test]
    fn test_delete_light_source() {
        let mut conn = setup_test_db();

        let light = NewLightSource::torch("ls-1", "map-1", 0, 0);
        insert_light_source(&mut conn, &light).expect("Failed to insert");

        assert!(light_source_exists(&mut conn, "ls-1").expect("Failed to check"));

        delete_light_source(&mut conn, "ls-1").expect("Failed to delete");

        assert!(!light_source_exists(&mut conn, "ls-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_light_sources() {
        let mut conn = setup_test_db();

        assert_eq!(
            count_light_sources(&mut conn, "map-1").expect("Failed to count"),
            0
        );

        let light1 = NewLightSource::torch("ls-1", "map-1", 1, 1);
        let light2 = NewLightSource::lantern("ls-2", "map-1", 2, 2);
        insert_light_source(&mut conn, &light1).expect("Failed to insert");
        insert_light_source(&mut conn, &light2).expect("Failed to insert");

        assert_eq!(
            count_light_sources(&mut conn, "map-1").expect("Failed to count"),
            2
        );
    }
}

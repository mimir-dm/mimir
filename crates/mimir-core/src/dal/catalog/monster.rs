//! Monster Data Access Layer
//!
//! Database operations for monsters.

use crate::models::catalog::{Monster, MonsterFilter, NewMonster};
use crate::schema::monsters;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new monster.
///
/// Returns the ID of the inserted monster on success.
pub fn insert_monster(conn: &mut SqliteConnection, monster: &NewMonster) -> QueryResult<i32> {
    diesel::insert_into(monsters::table)
        .values(monster)
        .execute(conn)?;

    // Get the last inserted rowid
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple monsters in a batch.
pub fn insert_monsters(conn: &mut SqliteConnection, monsters: &[NewMonster]) -> QueryResult<usize> {
    diesel::insert_into(monsters::table)
        .values(monsters)
        .execute(conn)
}

/// Get a monster by its ID.
pub fn get_monster(conn: &mut SqliteConnection, id: i32) -> QueryResult<Monster> {
    monsters::table
        .filter(monsters::id.eq(id))
        .first(conn)
}

/// Get a monster by its ID, returning None if not found.
pub fn get_monster_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Monster>> {
    monsters::table
        .filter(monsters::id.eq(id))
        .first(conn)
        .optional()
}

/// Get a monster by name and source.
pub fn get_monster_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Monster>> {
    monsters::table
        .filter(monsters::name.eq(name))
        .filter(monsters::source.eq(source))
        .first(conn)
        .optional()
}

/// List all monsters, ordered by name.
pub fn list_monsters(conn: &mut SqliteConnection) -> QueryResult<Vec<Monster>> {
    monsters::table.order(monsters::name.asc()).load(conn)
}

/// List monsters from a specific source.
pub fn list_monsters_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Monster>> {
    monsters::table
        .filter(monsters::source.eq(source))
        .order(monsters::name.asc())
        .load(conn)
}

/// Search monsters with filters.
pub fn search_monsters(
    conn: &mut SqliteConnection,
    filter: &MonsterFilter,
) -> QueryResult<Vec<Monster>> {
    let mut query = monsters::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(monsters::name.like(pattern));
    }

    if let Some(ref source) = filter.source {
        query = query.filter(monsters::source.eq(source));
    }

    if let Some(ref cr) = filter.cr {
        query = query.filter(monsters::cr.eq(cr));
    }

    if let Some(ref creature_type) = filter.creature_type {
        query = query.filter(monsters::creature_type.eq(creature_type));
    }

    if let Some(ref size) = filter.size {
        query = query.filter(monsters::size.eq(size));
    }

    query.order(monsters::name.asc()).load(conn)
}

/// Search monsters with pagination.
pub fn search_monsters_paginated(
    conn: &mut SqliteConnection,
    filter: &MonsterFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Monster>> {
    let mut query = monsters::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(monsters::name.like(pattern));
    }

    if let Some(ref source) = filter.source {
        query = query.filter(monsters::source.eq(source));
    }

    if let Some(ref cr) = filter.cr {
        query = query.filter(monsters::cr.eq(cr));
    }

    if let Some(ref creature_type) = filter.creature_type {
        query = query.filter(monsters::creature_type.eq(creature_type));
    }

    if let Some(ref size) = filter.size {
        query = query.filter(monsters::size.eq(size));
    }

    query
        .order(monsters::name.asc())
        .limit(limit)
        .offset(offset)
        .load(conn)
}

/// Delete a monster by its ID.
pub fn delete_monster(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(monsters::table.filter(monsters::id.eq(id))).execute(conn)
}

/// Delete all monsters from a specific source.
pub fn delete_monsters_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(monsters::table.filter(monsters::source.eq(source))).execute(conn)
}

/// Count all monsters.
pub fn count_monsters(conn: &mut SqliteConnection) -> QueryResult<i64> {
    monsters::table.count().get_result(conn)
}

/// Count monsters from a specific source.
pub fn count_monsters_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    monsters::table
        .filter(monsters::source.eq(source))
        .count()
        .get_result(conn)
}

/// Update a monster's token image path.
pub fn set_token_image_path(
    conn: &mut SqliteConnection,
    id: i32,
    path: Option<&str>,
) -> QueryResult<usize> {
    diesel::update(monsters::table.filter(monsters::id.eq(id)))
        .set(monsters::token_image_path.eq(path))
        .execute(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_source;
    use crate::models::catalog::NewCatalogSource;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        // Create the tables
        conn.batch_execute(
            "CREATE TABLE catalog_sources (
                code TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                imported_at TEXT NOT NULL
            );
            CREATE TABLE monsters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                source TEXT NOT NULL REFERENCES catalog_sources(code),
                cr TEXT,
                creature_type TEXT,
                size TEXT,
                token_image_path TEXT,
                data TEXT NOT NULL,
                UNIQUE(name, source)
            );
            CREATE INDEX idx_monsters_name ON monsters(name);
            CREATE INDEX idx_monsters_source ON monsters(source);
            CREATE INDEX idx_monsters_cr ON monsters(cr);
            CREATE INDEX idx_monsters_creature_type ON monsters(creature_type);
            CREATE INDEX idx_monsters_size ON monsters(size);",
        )
        .expect("Failed to create tables");

        // Insert a test source
        let source = NewCatalogSource::new("MM", "Monster Manual", true, "2024-01-20T12:00:00Z");
        insert_source(&mut conn, &source).expect("Failed to insert source");

        conn
    }

    #[test]
    fn test_insert_and_get_monster() {
        let mut conn = setup_test_db();

        let data = r#"{"name":"Goblin","source":"MM","cr":"1/4","type":"humanoid","size":["S"]}"#;
        let monster = NewMonster::new("Goblin", "MM", data)
            .with_cr("1/4")
            .with_creature_type("humanoid")
            .with_size("S");

        let id = insert_monster(&mut conn, &monster).expect("Failed to insert");
        assert!(id > 0);

        let retrieved = get_monster(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Goblin");
        assert_eq!(retrieved.source, "MM");
        assert_eq!(retrieved.cr, Some("1/4".to_string()));
        assert_eq!(retrieved.creature_type, Some("humanoid".to_string()));
        assert_eq!(retrieved.size, Some("S".to_string()));
    }

    #[test]
    fn test_get_monster_by_name() {
        let mut conn = setup_test_db();

        let data = r#"{"name":"Goblin"}"#;
        let monster = NewMonster::new("Goblin", "MM", data);
        insert_monster(&mut conn, &monster).expect("Failed to insert");

        let found = get_monster_by_name(&mut conn, "Goblin", "MM")
            .expect("Failed to query")
            .expect("Monster not found");
        assert_eq!(found.name, "Goblin");

        let not_found = get_monster_by_name(&mut conn, "Dragon", "MM").expect("Failed to query");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_list_monsters_by_source() {
        let mut conn = setup_test_db();

        let monsters = vec![
            NewMonster::new("Goblin", "MM", r#"{"name":"Goblin"}"#),
            NewMonster::new("Orc", "MM", r#"{"name":"Orc"}"#),
        ];
        insert_monsters(&mut conn, &monsters).expect("Failed to insert");

        let list = list_monsters_by_source(&mut conn, "MM").expect("Failed to list");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_search_monsters() {
        let mut conn = setup_test_db();

        let monsters = vec![
            NewMonster::new("Goblin", "MM", r#"{"name":"Goblin"}"#)
                .with_cr("1/4")
                .with_creature_type("humanoid")
                .with_size("S"),
            NewMonster::new("Orc", "MM", r#"{"name":"Orc"}"#)
                .with_cr("1/2")
                .with_creature_type("humanoid")
                .with_size("M"),
            NewMonster::new("Adult Red Dragon", "MM", r#"{"name":"Adult Red Dragon"}"#)
                .with_cr("17")
                .with_creature_type("dragon")
                .with_size("H"),
        ];
        insert_monsters(&mut conn, &monsters).expect("Failed to insert");

        // Search by creature type
        let filter = MonsterFilter::new().with_creature_type("humanoid");
        let results = search_monsters(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 2);

        // Search by CR
        let filter = MonsterFilter::new().with_cr("1/4");
        let results = search_monsters(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Goblin");

        // Search by name
        let filter = MonsterFilter::new().with_name_contains("dragon");
        let results = search_monsters(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Adult Red Dragon");
    }

    #[test]
    fn test_search_monsters_paginated() {
        let mut conn = setup_test_db();

        // Create owned strings for the monster names
        let names: Vec<String> = (1..=10).map(|i| format!("Monster {}", i)).collect();
        let monsters: Vec<_> = names
            .iter()
            .map(|name| NewMonster::new(name, "MM", r#"{"name":"test"}"#))
            .collect();
        insert_monsters(&mut conn, &monsters).expect("Failed to insert");

        let filter = MonsterFilter::new();
        let page1 = search_monsters_paginated(&mut conn, &filter, 3, 0).expect("Failed to search");
        assert_eq!(page1.len(), 3);

        let page2 = search_monsters_paginated(&mut conn, &filter, 3, 3).expect("Failed to search");
        assert_eq!(page2.len(), 3);
    }

    #[test]
    fn test_delete_monster() {
        let mut conn = setup_test_db();

        let monster = NewMonster::new("Goblin", "MM", r#"{"name":"Goblin"}"#);
        let id = insert_monster(&mut conn, &monster).expect("Failed to insert");

        assert_eq!(count_monsters(&mut conn).expect("Failed to count"), 1);

        delete_monster(&mut conn, id).expect("Failed to delete");

        assert_eq!(count_monsters(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_delete_monsters_by_source() {
        let mut conn = setup_test_db();

        let monsters = vec![
            NewMonster::new("Goblin", "MM", r#"{"name":"Goblin"}"#),
            NewMonster::new("Orc", "MM", r#"{"name":"Orc"}"#),
        ];
        insert_monsters(&mut conn, &monsters).expect("Failed to insert");

        assert_eq!(count_monsters(&mut conn).expect("Failed to count"), 2);

        delete_monsters_by_source(&mut conn, "MM").expect("Failed to delete");

        assert_eq!(count_monsters(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_set_token_image_path() {
        let mut conn = setup_test_db();

        let monster = NewMonster::new("Goblin", "MM", r#"{"name":"Goblin"}"#);
        let id = insert_monster(&mut conn, &monster).expect("Failed to insert");

        set_token_image_path(&mut conn, id, Some("/tokens/goblin.png")).expect("Failed to update");

        let retrieved = get_monster(&mut conn, id).expect("Failed to get");
        assert_eq!(
            retrieved.token_image_path,
            Some("/tokens/goblin.png".to_string())
        );
    }

    #[test]
    fn test_count_monsters() {
        let mut conn = setup_test_db();

        assert_eq!(count_monsters(&mut conn).expect("Failed to count"), 0);

        let monsters = vec![
            NewMonster::new("Goblin", "MM", r#"{"name":"Goblin"}"#),
            NewMonster::new("Orc", "MM", r#"{"name":"Orc"}"#),
        ];
        insert_monsters(&mut conn, &monsters).expect("Failed to insert");

        assert_eq!(count_monsters(&mut conn).expect("Failed to count"), 2);
        assert_eq!(
            count_monsters_by_source(&mut conn, "MM").expect("Failed to count"),
            2
        );
    }
}

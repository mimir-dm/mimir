//! Spell Data Access Layer
//!
//! Database operations for spells.

use crate::models::catalog::{NewSpell, Spell, SpellFilter};
use crate::schema::spells;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new spell.
///
/// Returns the ID of the inserted spell on success.
pub fn insert_spell(conn: &mut SqliteConnection, spell: &NewSpell) -> QueryResult<i32> {
    diesel::insert_into(spells::table)
        .values(spell)
        .execute(conn)?;

    // Get the last inserted rowid
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple spells in a batch.
pub fn insert_spells(conn: &mut SqliteConnection, spells: &[NewSpell]) -> QueryResult<usize> {
    diesel::insert_into(spells::table)
        .values(spells)
        .execute(conn)
}

/// Get a spell by its ID.
pub fn get_spell(conn: &mut SqliteConnection, id: i32) -> QueryResult<Spell> {
    spells::table
        .filter(spells::id.eq(id))
        .first(conn)
}

/// Get a spell by its ID, returning None if not found.
pub fn get_spell_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Spell>> {
    spells::table
        .filter(spells::id.eq(id))
        .first(conn)
        .optional()
}

// Define the LOWER SQL function for case-insensitive matching
diesel::define_sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

/// Get a spell by name and source (case-insensitive name matching).
pub fn get_spell_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Spell>> {
    let name_lower = name.to_lowercase();
    spells::table
        .filter(lower(spells::name).eq(&name_lower))
        .filter(spells::source.eq(source))
        .first(conn)
        .optional()
}

/// List all spells, ordered by level then name.
pub fn list_spells(conn: &mut SqliteConnection) -> QueryResult<Vec<Spell>> {
    spells::table
        .order((spells::level.asc(), spells::name.asc()))
        .load(conn)
}

/// List spells from a specific source.
pub fn list_spells_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Spell>> {
    spells::table
        .filter(spells::source.eq(source))
        .order((spells::level.asc(), spells::name.asc()))
        .load(conn)
}

/// List spells of a specific level.
pub fn list_spells_by_level(conn: &mut SqliteConnection, level: i32) -> QueryResult<Vec<Spell>> {
    spells::table
        .filter(spells::level.eq(level))
        .order(spells::name.asc())
        .load(conn)
}

/// List all cantrips.
pub fn list_cantrips(conn: &mut SqliteConnection) -> QueryResult<Vec<Spell>> {
    list_spells_by_level(conn, 0)
}

/// List all ritual spells.
pub fn list_ritual_spells(conn: &mut SqliteConnection) -> QueryResult<Vec<Spell>> {
    spells::table
        .filter(spells::ritual.eq(1))
        .order((spells::level.asc(), spells::name.asc()))
        .load(conn)
}

/// Search spells with filters.
pub fn search_spells(
    conn: &mut SqliteConnection,
    filter: &SpellFilter,
) -> QueryResult<Vec<Spell>> {
    // If sources filter is explicitly empty, return no results
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = spells::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(spells::name.like(pattern));
    }

    // Use effective_sources() to support both single source and sources array
    if let Some(sources) = filter.effective_sources() {
        query = query.filter(spells::source.eq_any(sources));
    }

    if let Some(level) = filter.level {
        query = query.filter(spells::level.eq(level));
    }

    if let Some(ref school) = filter.school {
        query = query.filter(spells::school.eq(school));
    }

    if let Some(ritual) = filter.ritual {
        let ritual_val = if ritual { 1 } else { 0 };
        query = query.filter(spells::ritual.eq(ritual_val));
    }

    if let Some(concentration) = filter.concentration {
        let conc_val = if concentration { 1 } else { 0 };
        query = query.filter(spells::concentration.eq(conc_val));
    }

    query
        .order((spells::level.asc(), spells::name.asc()))
        .load(conn)
}

/// Search spells with pagination.
pub fn search_spells_paginated(
    conn: &mut SqliteConnection,
    filter: &SpellFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Spell>> {
    // If sources filter is explicitly empty, return no results
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = spells::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(spells::name.like(pattern));
    }

    // Use effective_sources() to support both single source and sources array
    if let Some(sources) = filter.effective_sources() {
        query = query.filter(spells::source.eq_any(sources));
    }

    if let Some(level) = filter.level {
        query = query.filter(spells::level.eq(level));
    }

    if let Some(ref school) = filter.school {
        query = query.filter(spells::school.eq(school));
    }

    if let Some(ritual) = filter.ritual {
        let ritual_val = if ritual { 1 } else { 0 };
        query = query.filter(spells::ritual.eq(ritual_val));
    }

    if let Some(concentration) = filter.concentration {
        let conc_val = if concentration { 1 } else { 0 };
        query = query.filter(spells::concentration.eq(conc_val));
    }

    query
        .order((spells::level.asc(), spells::name.asc()))
        .limit(limit)
        .offset(offset)
        .load(conn)
}

/// Delete a spell by its ID.
pub fn delete_spell(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(spells::table.filter(spells::id.eq(id))).execute(conn)
}

/// Delete all spells from a specific source.
pub fn delete_spells_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(spells::table.filter(spells::source.eq(source))).execute(conn)
}

/// Count all spells.
pub fn count_spells(conn: &mut SqliteConnection) -> QueryResult<i64> {
    spells::table.count().get_result(conn)
}

/// Count spells from a specific source.
pub fn count_spells_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    spells::table
        .filter(spells::source.eq(source))
        .count()
        .get_result(conn)
}

/// Count spells by level.
pub fn count_spells_by_level(conn: &mut SqliteConnection, level: i32) -> QueryResult<i64> {
    spells::table
        .filter(spells::level.eq(level))
        .count()
        .get_result(conn)
}

/// List all distinct sources that have spells.
pub fn list_spell_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    spells::table
        .select(spells::source)
        .distinct()
        .order(spells::source.asc())
        .load(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_insert_and_get_spell() {
        let mut conn = setup_test_db_with_sources();

        let data = r#"{"name":"Fireball","source":"PHB","level":3,"school":"V"}"#;
        let spell = NewSpell::new("Fireball", "PHB", 3, data)
            .with_school("V")
            .with_ritual(false)
            .with_concentration(false);

        let id = insert_spell(&mut conn, &spell).expect("Failed to insert");
        assert!(id > 0);

        let retrieved = get_spell(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Fireball");
        assert_eq!(retrieved.source, "PHB");
        assert_eq!(retrieved.level, 3);
        assert_eq!(retrieved.school, Some("V".to_string()));
        assert_eq!(retrieved.ritual, 0);
        assert_eq!(retrieved.concentration, 0);
    }

    #[test]
    fn test_get_spell_by_name() {
        let mut conn = setup_test_db_with_sources();

        let data = r#"{"name":"Fireball"}"#;
        let spell = NewSpell::new("Fireball", "PHB", 3, data);
        insert_spell(&mut conn, &spell).expect("Failed to insert");

        let found = get_spell_by_name(&mut conn, "Fireball", "PHB")
            .expect("Failed to query")
            .expect("Spell not found");
        assert_eq!(found.name, "Fireball");

        let not_found = get_spell_by_name(&mut conn, "Magic Missile", "PHB").expect("Failed to query");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_list_spells_by_source() {
        let mut conn = setup_test_db_with_sources();

        let spells = vec![
            NewSpell::new("Fireball", "PHB", 3, r#"{"name":"Fireball"}"#),
            NewSpell::new("Magic Missile", "PHB", 1, r#"{"name":"Magic Missile"}"#),
        ];
        insert_spells(&mut conn, &spells).expect("Failed to insert");

        let list = list_spells_by_source(&mut conn, "PHB").expect("Failed to list");
        assert_eq!(list.len(), 2);
        // Should be ordered by level then name
        assert_eq!(list[0].name, "Magic Missile"); // Level 1
        assert_eq!(list[1].name, "Fireball"); // Level 3
    }

    #[test]
    fn test_list_spells_by_level() {
        let mut conn = setup_test_db_with_sources();

        let spells = vec![
            NewSpell::new("Fire Bolt", "PHB", 0, r#"{"name":"Fire Bolt"}"#),
            NewSpell::new("Prestidigitation", "PHB", 0, r#"{"name":"Prestidigitation"}"#),
            NewSpell::new("Fireball", "PHB", 3, r#"{"name":"Fireball"}"#),
        ];
        insert_spells(&mut conn, &spells).expect("Failed to insert");

        let cantrips = list_cantrips(&mut conn).expect("Failed to list");
        assert_eq!(cantrips.len(), 2);

        let level3 = list_spells_by_level(&mut conn, 3).expect("Failed to list");
        assert_eq!(level3.len(), 1);
        assert_eq!(level3[0].name, "Fireball");
    }

    #[test]
    fn test_list_ritual_spells() {
        let mut conn = setup_test_db_with_sources();

        let spells = vec![
            NewSpell::new("Detect Magic", "PHB", 1, r#"{"name":"Detect Magic"}"#)
                .with_ritual(true)
                .with_concentration(true),
            NewSpell::new("Find Familiar", "PHB", 1, r#"{"name":"Find Familiar"}"#)
                .with_ritual(true),
            NewSpell::new("Magic Missile", "PHB", 1, r#"{"name":"Magic Missile"}"#),
        ];
        insert_spells(&mut conn, &spells).expect("Failed to insert");

        let rituals = list_ritual_spells(&mut conn).expect("Failed to list");
        assert_eq!(rituals.len(), 2);
    }

    #[test]
    fn test_search_spells() {
        let mut conn = setup_test_db_with_sources();

        let spells = vec![
            NewSpell::new("Fireball", "PHB", 3, r#"{"name":"Fireball"}"#)
                .with_school("V"),
            NewSpell::new("Fire Bolt", "PHB", 0, r#"{"name":"Fire Bolt"}"#)
                .with_school("V"),
            NewSpell::new("Detect Magic", "PHB", 1, r#"{"name":"Detect Magic"}"#)
                .with_school("D")
                .with_ritual(true)
                .with_concentration(true),
        ];
        insert_spells(&mut conn, &spells).expect("Failed to insert");

        // Search by school
        let filter = SpellFilter::new().with_school("V");
        let results = search_spells(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 2);

        // Search by level
        let filter = SpellFilter::new().with_level(0);
        let results = search_spells(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Fire Bolt");

        // Search by name
        let filter = SpellFilter::new().with_name_contains("fire");
        let results = search_spells(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 2);

        // Search by ritual
        let filter = SpellFilter::new().with_ritual(true);
        let results = search_spells(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Detect Magic");

        // Search by concentration
        let filter = SpellFilter::new().with_concentration(true);
        let results = search_spells(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Detect Magic");
    }

    #[test]
    fn test_search_spells_paginated() {
        let mut conn = setup_test_db_with_sources();

        // Create owned strings for the spell names
        let names: Vec<String> = (1..=10).map(|i| format!("Spell {}", i)).collect();
        let spells: Vec<_> = names
            .iter()
            .enumerate()
            .map(|(i, name)| NewSpell::new(name, "PHB", (i % 3) as i32, r#"{"name":"test"}"#))
            .collect();
        insert_spells(&mut conn, &spells).expect("Failed to insert");

        let filter = SpellFilter::new();
        let page1 = search_spells_paginated(&mut conn, &filter, 3, 0).expect("Failed to search");
        assert_eq!(page1.len(), 3);

        let page2 = search_spells_paginated(&mut conn, &filter, 3, 3).expect("Failed to search");
        assert_eq!(page2.len(), 3);
    }

    #[test]
    fn test_delete_spell() {
        let mut conn = setup_test_db_with_sources();

        let spell = NewSpell::new("Fireball", "PHB", 3, r#"{"name":"Fireball"}"#);
        let id = insert_spell(&mut conn, &spell).expect("Failed to insert");

        assert_eq!(count_spells(&mut conn).expect("Failed to count"), 1);

        delete_spell(&mut conn, id).expect("Failed to delete");

        assert_eq!(count_spells(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_delete_spells_by_source() {
        let mut conn = setup_test_db_with_sources();

        let spells = vec![
            NewSpell::new("Fireball", "PHB", 3, r#"{"name":"Fireball"}"#),
            NewSpell::new("Magic Missile", "PHB", 1, r#"{"name":"Magic Missile"}"#),
        ];
        insert_spells(&mut conn, &spells).expect("Failed to insert");

        assert_eq!(count_spells(&mut conn).expect("Failed to count"), 2);

        delete_spells_by_source(&mut conn, "PHB").expect("Failed to delete");

        assert_eq!(count_spells(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_count_spells() {
        let mut conn = setup_test_db_with_sources();

        assert_eq!(count_spells(&mut conn).expect("Failed to count"), 0);

        let spells = vec![
            NewSpell::new("Fireball", "PHB", 3, r#"{"name":"Fireball"}"#),
            NewSpell::new("Fire Bolt", "PHB", 0, r#"{"name":"Fire Bolt"}"#),
            NewSpell::new("Magic Missile", "PHB", 1, r#"{"name":"Magic Missile"}"#),
        ];
        insert_spells(&mut conn, &spells).expect("Failed to insert");

        assert_eq!(count_spells(&mut conn).expect("Failed to count"), 3);
        assert_eq!(
            count_spells_by_source(&mut conn, "PHB").expect("Failed to count"),
            3
        );
        assert_eq!(
            count_spells_by_level(&mut conn, 0).expect("Failed to count"),
            1
        );
    }
}

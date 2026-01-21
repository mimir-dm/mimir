//! Full-Text Search Operations
//!
//! Database operations for searching the unified FTS5 catalog_fts table.

use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text};
use diesel::SqliteConnection;

/// Content type for FTS entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// Rules content - mechanical descriptions, stats, abilities
    Rules,
    /// Fluff content - lore, flavor text, background
    Fluff,
}

impl ContentType {
    /// Convert to string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Rules => "rules",
            ContentType::Fluff => "fluff",
        }
    }
}

/// A search result from the FTS table.
#[derive(Debug, Clone, QueryableByName)]
pub struct FtsSearchResult {
    /// The type of entity (monster, spell, item, etc.)
    #[diesel(sql_type = Text)]
    pub entity_type: String,

    /// The entity's primary key in its respective table
    #[diesel(sql_type = Text)]
    pub entity_id: String,

    /// Whether this is rules or fluff content
    #[diesel(sql_type = Text)]
    pub content_type: String,

    /// The entity's name
    #[diesel(sql_type = Text)]
    pub name: String,

    /// FTS5 rank score (lower is better match)
    #[diesel(sql_type = Nullable<diesel::sql_types::Double>)]
    pub rank: Option<f64>,
}

/// Index a catalog entity in the FTS table.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `entity_type` - Type of entity (e.g., "monster", "spell")
/// * `entity_id` - Primary key of the entity
/// * `content_type` - Whether this is rules or fluff content
/// * `name` - Entity name
/// * `text_content` - Flattened text content to index
pub fn index_entity(
    conn: &mut SqliteConnection,
    entity_type: &str,
    entity_id: i32,
    content_type: ContentType,
    name: &str,
    text_content: &str,
) -> QueryResult<usize> {
    diesel::sql_query(
        "INSERT INTO catalog_fts (entity_type, entity_id, content_type, name, text_content) VALUES (?, ?, ?, ?, ?)"
    )
    .bind::<Text, _>(entity_type)
    .bind::<Text, _>(entity_id.to_string())
    .bind::<Text, _>(content_type.as_str())
    .bind::<Text, _>(name)
    .bind::<Text, _>(text_content)
    .execute(conn)
}

/// Remove all FTS entries for an entity.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `entity_type` - Type of entity
/// * `entity_id` - Primary key of the entity
pub fn remove_entity_from_index(
    conn: &mut SqliteConnection,
    entity_type: &str,
    entity_id: i32,
) -> QueryResult<usize> {
    diesel::sql_query("DELETE FROM catalog_fts WHERE entity_type = ? AND entity_id = ?")
        .bind::<Text, _>(entity_type)
        .bind::<Text, _>(entity_id.to_string())
        .execute(conn)
}

/// Remove all FTS entries of a specific type.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `entity_type` - Type of entity to remove
pub fn clear_entity_type_from_index(
    conn: &mut SqliteConnection,
    entity_type: &str,
) -> QueryResult<usize> {
    diesel::sql_query("DELETE FROM catalog_fts WHERE entity_type = ?")
        .bind::<Text, _>(entity_type)
        .execute(conn)
}

/// Clear the entire FTS index.
pub fn clear_index(conn: &mut SqliteConnection) -> QueryResult<usize> {
    diesel::sql_query("DELETE FROM catalog_fts").execute(conn)
}

/// Search the catalog using full-text search.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `query` - FTS5 search query (supports boolean operators, phrases, etc.)
/// * `limit` - Maximum number of results
///
/// # Returns
///
/// Vector of search results ordered by relevance.
pub fn search(
    conn: &mut SqliteConnection,
    query: &str,
    limit: i32,
) -> QueryResult<Vec<FtsSearchResult>> {
    diesel::sql_query(
        "SELECT entity_type, entity_id, content_type, name, rank
         FROM catalog_fts
         WHERE catalog_fts MATCH ?
         ORDER BY rank
         LIMIT ?"
    )
    .bind::<Text, _>(query)
    .bind::<Integer, _>(limit)
    .load(conn)
}

/// Search the catalog filtering by entity type.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `query` - FTS5 search query
/// * `entity_type` - Entity type to filter by
/// * `limit` - Maximum number of results
pub fn search_by_entity_type(
    conn: &mut SqliteConnection,
    query: &str,
    entity_type: &str,
    limit: i32,
) -> QueryResult<Vec<FtsSearchResult>> {
    // Use WHERE clause for entity_type filter, combined with FTS MATCH
    diesel::sql_query(
        "SELECT entity_type, entity_id, content_type, name, rank
         FROM catalog_fts
         WHERE catalog_fts MATCH ? AND entity_type = ?
         ORDER BY rank
         LIMIT ?"
    )
    .bind::<Text, _>(query)
    .bind::<Text, _>(entity_type)
    .bind::<Integer, _>(limit)
    .load(conn)
}

/// Search the catalog filtering by content type.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `query` - FTS5 search query
/// * `content_type` - Content type to filter by (rules or fluff)
/// * `limit` - Maximum number of results
pub fn search_by_content_type(
    conn: &mut SqliteConnection,
    query: &str,
    content_type: ContentType,
    limit: i32,
) -> QueryResult<Vec<FtsSearchResult>> {
    diesel::sql_query(
        "SELECT entity_type, entity_id, content_type, name, rank
         FROM catalog_fts
         WHERE catalog_fts MATCH ? AND content_type = ?
         ORDER BY rank
         LIMIT ?"
    )
    .bind::<Text, _>(query)
    .bind::<Text, _>(content_type.as_str())
    .bind::<Integer, _>(limit)
    .load(conn)
}

/// Search with both entity type and content type filters.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `query` - FTS5 search query
/// * `entity_type` - Entity type to filter by
/// * `content_type` - Content type to filter by
/// * `limit` - Maximum number of results
pub fn search_filtered(
    conn: &mut SqliteConnection,
    query: &str,
    entity_type: &str,
    content_type: ContentType,
    limit: i32,
) -> QueryResult<Vec<FtsSearchResult>> {
    diesel::sql_query(
        "SELECT entity_type, entity_id, content_type, name, rank
         FROM catalog_fts
         WHERE catalog_fts MATCH ? AND entity_type = ? AND content_type = ?
         ORDER BY rank
         LIMIT ?"
    )
    .bind::<Text, _>(query)
    .bind::<Text, _>(entity_type)
    .bind::<Text, _>(content_type.as_str())
    .bind::<Integer, _>(limit)
    .load(conn)
}

/// Count total indexed entries.
pub fn count_indexed(conn: &mut SqliteConnection) -> QueryResult<i64> {
    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let result: CountResult =
        diesel::sql_query("SELECT COUNT(*) as count FROM catalog_fts").get_result(conn)?;
    Ok(result.count)
}

/// Count indexed entries by entity type.
pub fn count_indexed_by_type(conn: &mut SqliteConnection, entity_type: &str) -> QueryResult<i64> {
    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let result: CountResult =
        diesel::sql_query("SELECT COUNT(*) as count FROM catalog_fts WHERE entity_type = ?")
            .bind::<Text, _>(entity_type)
            .get_result(conn)?;
    Ok(result.count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        conn.batch_execute(
            "CREATE VIRTUAL TABLE catalog_fts USING fts5(
                entity_type,
                entity_id UNINDEXED,
                content_type,
                name,
                text_content,
                tokenize='porter unicode61'
            );",
        )
        .expect("Failed to create FTS table");

        conn
    }

    #[test]
    fn test_index_and_search() {
        let mut conn = setup_test_db();

        index_entity(
            &mut conn,
            "monster",
            1,
            ContentType::Rules,
            "Goblin",
            "Small humanoid goblinoid. Nimble Escape. The goblin can take the Disengage or Hide action.",
        )
        .expect("Failed to index");

        let results = search(&mut conn, "goblin", 10).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Goblin");
        assert_eq!(results[0].entity_type, "monster");
    }

    #[test]
    fn test_search_by_entity_type() {
        let mut conn = setup_test_db();

        index_entity(&mut conn, "monster", 1, ContentType::Rules, "Goblin", "A small goblinoid")
            .expect("Failed to index monster");
        index_entity(&mut conn, "spell", 1, ContentType::Rules, "Fireball", "A ball of fire explodes")
            .expect("Failed to index spell");

        let monster_results =
            search_by_entity_type(&mut conn, "fire OR goblin", "monster", 10).expect("Failed to search");
        assert_eq!(monster_results.len(), 1);
        assert_eq!(monster_results[0].entity_type, "monster");

        let spell_results =
            search_by_entity_type(&mut conn, "fire OR goblin", "spell", 10).expect("Failed to search");
        assert_eq!(spell_results.len(), 1);
        assert_eq!(spell_results[0].entity_type, "spell");
    }

    #[test]
    fn test_search_by_content_type() {
        let mut conn = setup_test_db();

        index_entity(&mut conn, "monster", 1, ContentType::Rules, "Dragon", "Breath weapon attack")
            .expect("Failed to index rules");
        index_entity(&mut conn, "monster", 1, ContentType::Fluff, "Dragon", "Ancient and wise creatures")
            .expect("Failed to index fluff");

        let rules_results =
            search_by_content_type(&mut conn, "dragon", ContentType::Rules, 10).expect("Failed to search");
        assert_eq!(rules_results.len(), 1);
        assert_eq!(rules_results[0].content_type, "rules");

        let fluff_results =
            search_by_content_type(&mut conn, "dragon", ContentType::Fluff, 10).expect("Failed to search");
        assert_eq!(fluff_results.len(), 1);
        assert_eq!(fluff_results[0].content_type, "fluff");
    }

    #[test]
    fn test_remove_entity() {
        let mut conn = setup_test_db();

        index_entity(&mut conn, "monster", 1, ContentType::Rules, "Goblin", "Small goblinoid")
            .expect("Failed to index");
        index_entity(&mut conn, "monster", 1, ContentType::Fluff, "Goblin", "Goblins are greedy")
            .expect("Failed to index fluff");

        assert_eq!(count_indexed(&mut conn).expect("Failed to count"), 2);

        remove_entity_from_index(&mut conn, "monster", 1).expect("Failed to remove");

        assert_eq!(count_indexed(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_clear_entity_type() {
        let mut conn = setup_test_db();

        index_entity(&mut conn, "monster", 1, ContentType::Rules, "Goblin", "text")
            .expect("Failed to index");
        index_entity(&mut conn, "monster", 2, ContentType::Rules, "Orc", "text")
            .expect("Failed to index");
        index_entity(&mut conn, "spell", 1, ContentType::Rules, "Fireball", "text")
            .expect("Failed to index");

        clear_entity_type_from_index(&mut conn, "monster").expect("Failed to clear");

        assert_eq!(count_indexed(&mut conn).expect("Failed to count"), 1);
        assert_eq!(
            count_indexed_by_type(&mut conn, "spell").expect("Failed to count"),
            1
        );
    }

    #[test]
    fn test_phrase_search() {
        let mut conn = setup_test_db();

        index_entity(
            &mut conn,
            "monster",
            1,
            ContentType::Rules,
            "Ancient Red Dragon",
            "The dragon breathes fire in a 60-foot cone.",
        )
        .expect("Failed to index");

        // Phrase search with quotes
        let results = search(&mut conn, "\"breathes fire\"", 10).expect("Failed to search");
        assert_eq!(results.len(), 1);

        // Words that don't appear together as phrase
        let results = search(&mut conn, "\"fire dragon\"", 10).expect("Failed to search");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_porter_stemming() {
        let mut conn = setup_test_db();

        index_entity(
            &mut conn,
            "monster",
            1,
            ContentType::Rules,
            "Zombie",
            "The zombie is a shambling undead creature.",
        )
        .expect("Failed to index");

        // Porter stemming should match "shambling" when searching "shamble"
        let results = search(&mut conn, "shamble", 10).expect("Failed to search");
        assert_eq!(results.len(), 1);

        // And "undead" when searching "undead"
        let results = search(&mut conn, "undead", 10).expect("Failed to search");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_count_functions() {
        let mut conn = setup_test_db();

        index_entity(&mut conn, "monster", 1, ContentType::Rules, "Goblin", "text")
            .expect("Failed to index");
        index_entity(&mut conn, "monster", 2, ContentType::Rules, "Orc", "text")
            .expect("Failed to index");
        index_entity(&mut conn, "spell", 1, ContentType::Rules, "Fireball", "text")
            .expect("Failed to index");

        assert_eq!(count_indexed(&mut conn).expect("Failed to count"), 3);
        assert_eq!(
            count_indexed_by_type(&mut conn, "monster").expect("Failed to count"),
            2
        );
        assert_eq!(
            count_indexed_by_type(&mut conn, "spell").expect("Failed to count"),
            1
        );
    }
}

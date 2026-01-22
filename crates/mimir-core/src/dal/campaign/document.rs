//! Document Data Access Layer
//!
//! Database operations for documents (markdown content).

use crate::models::campaign::{Document, NewDocument, UpdateDocument};
use crate::schema::documents;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel::SqliteConnection;

/// Insert a new document.
pub fn insert_document(conn: &mut SqliteConnection, doc: &NewDocument) -> QueryResult<String> {
    diesel::insert_into(documents::table)
        .values(doc)
        .execute(conn)?;

    Ok(doc.id.to_string())
}

/// Get a document by ID.
pub fn get_document(conn: &mut SqliteConnection, id: &str) -> QueryResult<Document> {
    documents::table.find(id).first(conn)
}

/// Get a document by ID, returning None if not found.
pub fn get_document_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<Document>> {
    documents::table.find(id).first(conn).optional()
}

/// List all documents for a campaign (including module documents).
pub fn list_campaign_documents(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<Document>> {
    documents::table
        .filter(documents::campaign_id.eq(campaign_id))
        .order(documents::title.asc())
        .load(conn)
}

/// List only campaign-level documents (not in any module).
pub fn list_campaign_level_documents(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<Document>> {
    documents::table
        .filter(documents::campaign_id.eq(campaign_id))
        .filter(documents::module_id.is_null())
        .order(documents::title.asc())
        .load(conn)
}

/// List all documents for a module.
pub fn list_module_documents(
    conn: &mut SqliteConnection,
    module_id: &str,
) -> QueryResult<Vec<Document>> {
    documents::table
        .filter(documents::module_id.eq(module_id))
        .order(documents::title.asc())
        .load(conn)
}

/// List documents by type for a campaign.
pub fn list_documents_by_type(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    doc_type: &str,
) -> QueryResult<Vec<Document>> {
    documents::table
        .filter(documents::campaign_id.eq(campaign_id))
        .filter(documents::doc_type.eq(doc_type))
        .order(documents::title.asc())
        .load(conn)
}

/// Update a document.
pub fn update_document(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateDocument,
) -> QueryResult<usize> {
    diesel::update(documents::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a document by ID.
pub fn delete_document(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(documents::table.find(id)).execute(conn)
}

/// Check if a document exists.
pub fn document_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(documents::table.find(id))).get_result(conn)
}

/// Count documents for a campaign.
pub fn count_campaign_documents(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<i64> {
    documents::table
        .filter(documents::campaign_id.eq(campaign_id))
        .count()
        .get_result(conn)
}

/// Count documents for a module.
pub fn count_module_documents(conn: &mut SqliteConnection, module_id: &str) -> QueryResult<i64> {
    documents::table
        .filter(documents::module_id.eq(module_id))
        .count()
        .get_result(conn)
}

/// FTS search result with relevance ranking.
#[derive(Debug, Clone, QueryableByName, serde::Serialize)]
pub struct DocumentSearchResult {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub campaign_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    pub module_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub title: String,
    #[diesel(sql_type = Text)]
    pub content: String,
    #[diesel(sql_type = Text)]
    pub doc_type: String,
    #[diesel(sql_type = Text)]
    pub created_at: String,
    #[diesel(sql_type = Text)]
    pub updated_at: String,
}

/// Search documents using FTS5 full-text search.
///
/// Searches both title and content. Results are ranked by relevance.
pub fn search_documents(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    query: &str,
) -> QueryResult<Vec<DocumentSearchResult>> {
    // Use FTS5 MATCH syntax with bm25 ranking
    sql_query(
        r#"
        SELECT d.id, d.campaign_id, d.module_id, d.title, d.content, d.doc_type, d.created_at, d.updated_at
        FROM documents d
        JOIN documents_fts fts ON d.rowid = fts.rowid
        WHERE d.campaign_id = ?
          AND documents_fts MATCH ?
        ORDER BY bm25(documents_fts) ASC
        LIMIT 50
        "#,
    )
    .bind::<Text, _>(campaign_id)
    .bind::<Text, _>(query)
    .load(conn)
}

/// Search documents within a specific module using FTS5.
pub fn search_module_documents(
    conn: &mut SqliteConnection,
    module_id: &str,
    query: &str,
) -> QueryResult<Vec<DocumentSearchResult>> {
    sql_query(
        r#"
        SELECT d.id, d.campaign_id, d.module_id, d.title, d.content, d.doc_type, d.created_at, d.updated_at
        FROM documents d
        JOIN documents_fts fts ON d.rowid = fts.rowid
        WHERE d.module_id = ?
          AND documents_fts MATCH ?
        ORDER BY bm25(documents_fts) ASC
        LIMIT 50
        "#,
    )
    .bind::<Text, _>(module_id)
    .bind::<Text, _>(query)
    .load(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        conn.batch_execute(
            r#"
            CREATE TABLE campaigns (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                archived_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE modules (
                id TEXT PRIMARY KEY NOT NULL,
                campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT,
                module_number INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(campaign_id, module_number)
            );
            CREATE TABLE documents (
                id TEXT PRIMARY KEY NOT NULL,
                campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
                module_id TEXT REFERENCES modules(id) ON DELETE CASCADE,
                title TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                doc_type TEXT NOT NULL DEFAULT 'note',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_documents_campaign ON documents(campaign_id);
            CREATE INDEX idx_documents_module ON documents(module_id);
            CREATE INDEX idx_documents_type ON documents(doc_type);

            -- FTS5 for search
            CREATE VIRTUAL TABLE documents_fts USING fts5(
                title,
                content,
                content='documents',
                content_rowid='rowid',
                tokenize='porter unicode61'
            );

            CREATE TRIGGER documents_fts_insert AFTER INSERT ON documents BEGIN
                INSERT INTO documents_fts(rowid, title, content)
                VALUES (new.rowid, new.title, new.content);
            END;

            CREATE TRIGGER documents_fts_delete AFTER DELETE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, title, content)
                VALUES ('delete', old.rowid, old.title, old.content);
            END;

            CREATE TRIGGER documents_fts_update AFTER UPDATE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, title, content)
                VALUES ('delete', old.rowid, old.title, old.content);
                INSERT INTO documents_fts(rowid, title, content)
                VALUES (new.rowid, new.title, new.content);
            END;

            -- Insert test data
            INSERT INTO campaigns (id, name) VALUES ('camp-1', 'Test Campaign');
            INSERT INTO modules (id, campaign_id, name, module_number) VALUES ('mod-1', 'camp-1', 'Chapter 1', 1);
            PRAGMA foreign_keys = ON;
            "#,
        )
        .expect("Failed to create tables");

        conn
    }

    #[test]
    fn test_insert_and_get_document() {
        let mut conn = setup_test_db();

        let doc = NewDocument::for_campaign("doc-1", "camp-1", "Session Notes", "session")
            .with_content("The party met in a tavern...");
        let id = insert_document(&mut conn, &doc).expect("Failed to insert");
        assert_eq!(id, "doc-1");

        let retrieved = get_document(&mut conn, "doc-1").expect("Failed to get");
        assert_eq!(retrieved.id, "doc-1");
        assert_eq!(retrieved.campaign_id, "camp-1");
        assert!(retrieved.module_id.is_none());
        assert_eq!(retrieved.title, "Session Notes");
        assert_eq!(retrieved.doc_type, "session");
        assert!(retrieved.content.contains("tavern"));
    }

    #[test]
    fn test_insert_module_document() {
        let mut conn = setup_test_db();

        let doc =
            NewDocument::for_module("doc-1", "camp-1", "mod-1", "Dungeon Description", "location");
        insert_document(&mut conn, &doc).expect("Failed to insert");

        let retrieved = get_document(&mut conn, "doc-1").expect("Failed to get");
        assert_eq!(retrieved.module_id, Some("mod-1".to_string()));
    }

    #[test]
    fn test_list_campaign_documents() {
        let mut conn = setup_test_db();

        let doc1 = NewDocument::for_campaign("doc-1", "camp-1", "A Note", "note");
        let doc2 = NewDocument::for_campaign("doc-2", "camp-1", "B Note", "note");
        let doc3 = NewDocument::for_module("doc-3", "camp-1", "mod-1", "Module Doc", "location");
        insert_document(&mut conn, &doc1).expect("Failed to insert");
        insert_document(&mut conn, &doc2).expect("Failed to insert");
        insert_document(&mut conn, &doc3).expect("Failed to insert");

        // List all campaign documents (includes module documents)
        let all = list_campaign_documents(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(all.len(), 3);

        // List only campaign-level documents
        let campaign_level =
            list_campaign_level_documents(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(campaign_level.len(), 2);
    }

    #[test]
    fn test_list_module_documents() {
        let mut conn = setup_test_db();

        let doc1 = NewDocument::for_module("doc-1", "camp-1", "mod-1", "Room 1", "location");
        let doc2 = NewDocument::for_module("doc-2", "camp-1", "mod-1", "Room 2", "location");
        let doc3 = NewDocument::for_campaign("doc-3", "camp-1", "Campaign Note", "note");
        insert_document(&mut conn, &doc1).expect("Failed to insert");
        insert_document(&mut conn, &doc2).expect("Failed to insert");
        insert_document(&mut conn, &doc3).expect("Failed to insert");

        let module_docs = list_module_documents(&mut conn, "mod-1").expect("Failed to list");
        assert_eq!(module_docs.len(), 2);
    }

    #[test]
    fn test_list_documents_by_type() {
        let mut conn = setup_test_db();

        let doc1 = NewDocument::for_campaign("doc-1", "camp-1", "Session 1", "session");
        let doc2 = NewDocument::for_campaign("doc-2", "camp-1", "Session 2", "session");
        let doc3 = NewDocument::for_campaign("doc-3", "camp-1", "NPC List", "npc");
        insert_document(&mut conn, &doc1).expect("Failed to insert");
        insert_document(&mut conn, &doc2).expect("Failed to insert");
        insert_document(&mut conn, &doc3).expect("Failed to insert");

        let sessions =
            list_documents_by_type(&mut conn, "camp-1", "session").expect("Failed to list");
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_update_document_content() {
        let mut conn = setup_test_db();

        let doc = NewDocument::for_campaign("doc-1", "camp-1", "Notes", "note")
            .with_content("Original content");
        insert_document(&mut conn, &doc).expect("Failed to insert");

        let update = UpdateDocument::set_content("Updated content", "2024-01-20T12:00:00Z");
        update_document(&mut conn, "doc-1", &update).expect("Failed to update");

        let retrieved = get_document(&mut conn, "doc-1").expect("Failed to get");
        assert_eq!(retrieved.content, "Updated content");
    }

    #[test]
    fn test_update_document_move_to_module() {
        let mut conn = setup_test_db();

        let doc = NewDocument::for_campaign("doc-1", "camp-1", "Notes", "note");
        insert_document(&mut conn, &doc).expect("Failed to insert");

        let update = UpdateDocument::move_to_module("mod-1", "2024-01-20T12:00:00Z");
        update_document(&mut conn, "doc-1", &update).expect("Failed to update");

        let retrieved = get_document(&mut conn, "doc-1").expect("Failed to get");
        assert_eq!(retrieved.module_id, Some("mod-1".to_string()));
    }

    #[test]
    fn test_delete_document() {
        let mut conn = setup_test_db();

        let doc = NewDocument::for_campaign("doc-1", "camp-1", "Notes", "note");
        insert_document(&mut conn, &doc).expect("Failed to insert");

        assert!(document_exists(&mut conn, "doc-1").expect("Failed to check"));

        delete_document(&mut conn, "doc-1").expect("Failed to delete");

        assert!(!document_exists(&mut conn, "doc-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_documents() {
        let mut conn = setup_test_db();

        assert_eq!(
            count_campaign_documents(&mut conn, "camp-1").expect("Failed to count"),
            0
        );

        let doc1 = NewDocument::for_campaign("doc-1", "camp-1", "Note 1", "note");
        let doc2 = NewDocument::for_module("doc-2", "camp-1", "mod-1", "Note 2", "note");
        insert_document(&mut conn, &doc1).expect("Failed to insert");
        insert_document(&mut conn, &doc2).expect("Failed to insert");

        assert_eq!(
            count_campaign_documents(&mut conn, "camp-1").expect("Failed to count"),
            2
        );
        assert_eq!(
            count_module_documents(&mut conn, "mod-1").expect("Failed to count"),
            1
        );
    }

    #[test]
    fn test_fts_search() {
        let mut conn = setup_test_db();

        let doc1 = NewDocument::for_campaign("doc-1", "camp-1", "Dragon Encounter", "session")
            .with_content("The party fought a fearsome red dragon in the mountain cave.");
        let doc2 = NewDocument::for_campaign("doc-2", "camp-1", "Town Visit", "session")
            .with_content("The party visited the town of Waterdeep to resupply.");
        let doc3 = NewDocument::for_campaign("doc-3", "camp-1", "Dragon Lore", "note")
            .with_content("Dragons are ancient creatures of immense power.");
        insert_document(&mut conn, &doc1).expect("Failed to insert");
        insert_document(&mut conn, &doc2).expect("Failed to insert");
        insert_document(&mut conn, &doc3).expect("Failed to insert");

        // Search for "dragon" - should find 2 documents
        let results = search_documents(&mut conn, "camp-1", "dragon").expect("Failed to search");
        assert_eq!(results.len(), 2);

        // Search for "waterdeep" - should find 1 document
        let results = search_documents(&mut conn, "camp-1", "waterdeep").expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Town Visit");
    }

    #[test]
    fn test_fts_search_module() {
        let mut conn = setup_test_db();

        let doc1 = NewDocument::for_module("doc-1", "camp-1", "mod-1", "Goblin Cave", "location")
            .with_content("A dark cave inhabited by goblins.");
        let doc2 = NewDocument::for_campaign("doc-2", "camp-1", "Goblin Notes", "note")
            .with_content("Notes about goblin behavior.");
        insert_document(&mut conn, &doc1).expect("Failed to insert");
        insert_document(&mut conn, &doc2).expect("Failed to insert");

        // Search within module only
        let results =
            search_module_documents(&mut conn, "mod-1", "goblin").expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Goblin Cave");
    }
}

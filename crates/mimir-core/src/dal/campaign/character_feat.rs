//! CharacterFeat Data Access Layer
//!
//! Database operations for character feats.

use crate::models::campaign::{CharacterFeat, NewCharacterFeat};
use crate::schema::character_feats;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new character feat.
pub fn insert_character_feat(
    conn: &mut SqliteConnection,
    feat: &NewCharacterFeat,
) -> QueryResult<String> {
    diesel::insert_into(character_feats::table)
        .values(feat)
        .execute(conn)?;

    Ok(feat.id.to_string())
}

/// Get a character feat by ID.
pub fn get_character_feat(conn: &mut SqliteConnection, id: &str) -> QueryResult<CharacterFeat> {
    character_feats::table.find(id).first(conn)
}

/// Get a character feat by ID, returning None if not found.
pub fn get_character_feat_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<CharacterFeat>> {
    character_feats::table.find(id).first(conn).optional()
}

/// List all feats for a character.
pub fn list_character_feats(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterFeat>> {
    character_feats::table
        .filter(character_feats::character_id.eq(character_id))
        .order(character_feats::feat_name.asc())
        .load(conn)
}

/// List feats by source type (asi, race, class, bonus).
pub fn list_feats_by_source_type(
    conn: &mut SqliteConnection,
    character_id: &str,
    source_type: &str,
) -> QueryResult<Vec<CharacterFeat>> {
    character_feats::table
        .filter(character_feats::character_id.eq(character_id))
        .filter(character_feats::source_type.eq(source_type))
        .order(character_feats::feat_name.asc())
        .load(conn)
}

/// Check if a character has a specific feat (by name).
pub fn character_has_feat(
    conn: &mut SqliteConnection,
    character_id: &str,
    feat_name: &str,
) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(
        character_feats::table
            .filter(character_feats::character_id.eq(character_id))
            .filter(character_feats::feat_name.eq(feat_name)),
    ))
    .get_result(conn)
}

/// Delete a character feat by ID.
pub fn delete_character_feat(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(character_feats::table.find(id)).execute(conn)
}

/// Delete all feats for a character.
pub fn delete_character_feats(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        character_feats::table.filter(character_feats::character_id.eq(character_id)),
    )
    .execute(conn)
}

/// Check if a character feat exists.
pub fn character_feat_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(character_feats::table.find(id))).get_result(conn)
}

/// Count feats for a character.
pub fn count_character_feats(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<i64> {
    character_feats::table
        .filter(character_feats::character_id.eq(character_id))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::campaign::FeatSourceType;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        conn.batch_execute(
            r#"
            CREATE TABLE campaigns (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL
            );
            CREATE TABLE characters (
                id TEXT PRIMARY KEY NOT NULL,
                campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                is_npc INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE character_feats (
                id TEXT PRIMARY KEY NOT NULL,
                character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
                feat_name TEXT NOT NULL,
                feat_source TEXT NOT NULL,
                source_type TEXT NOT NULL DEFAULT 'asi'
            );
            INSERT INTO campaigns (id, name) VALUES ('camp-1', 'Test Campaign');
            INSERT INTO characters (id, campaign_id, name) VALUES ('char-1', 'camp-1', 'Test Hero');
            PRAGMA foreign_keys = ON;
            "#,
        )
        .expect("Failed to create tables");

        conn
    }

    #[test]
    fn test_insert_and_get_feat() {
        let mut conn = setup_test_db();

        let feat = NewCharacterFeat::from_asi("feat-1", "char-1", "Great Weapon Master", "PHB");
        let id = insert_character_feat(&mut conn, &feat).expect("Failed to insert");
        assert_eq!(id, "feat-1");

        let retrieved = get_character_feat(&mut conn, "feat-1").expect("Failed to get");
        assert_eq!(retrieved.feat_name, "Great Weapon Master");
        assert_eq!(retrieved.feat_source, "PHB");
        assert_eq!(retrieved.source_type, "asi");
    }

    #[test]
    fn test_list_character_feats() {
        let mut conn = setup_test_db();

        let feat1 = NewCharacterFeat::from_asi("feat-1", "char-1", "Alert", "PHB");
        let feat2 = NewCharacterFeat::from_asi("feat-2", "char-1", "Sentinel", "PHB");
        let feat3 = NewCharacterFeat::from_race("feat-3", "char-1", "Prodigy", "XGE");
        insert_character_feat(&mut conn, &feat1).expect("Failed to insert");
        insert_character_feat(&mut conn, &feat2).expect("Failed to insert");
        insert_character_feat(&mut conn, &feat3).expect("Failed to insert");

        let feats = list_character_feats(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(feats.len(), 3);
        // Should be sorted by name
        assert_eq!(feats[0].feat_name, "Alert");
        assert_eq!(feats[1].feat_name, "Prodigy");
        assert_eq!(feats[2].feat_name, "Sentinel");
    }

    #[test]
    fn test_list_feats_by_source_type() {
        let mut conn = setup_test_db();

        let feat1 = NewCharacterFeat::from_asi("feat-1", "char-1", "GWM", "PHB");
        let feat2 = NewCharacterFeat::from_asi("feat-2", "char-1", "PAM", "PHB");
        let feat3 = NewCharacterFeat::from_race("feat-3", "char-1", "Prodigy", "XGE");
        insert_character_feat(&mut conn, &feat1).expect("Failed to insert");
        insert_character_feat(&mut conn, &feat2).expect("Failed to insert");
        insert_character_feat(&mut conn, &feat3).expect("Failed to insert");

        let asi_feats =
            list_feats_by_source_type(&mut conn, "char-1", "asi").expect("Failed to list");
        assert_eq!(asi_feats.len(), 2);

        let race_feats =
            list_feats_by_source_type(&mut conn, "char-1", "race").expect("Failed to list");
        assert_eq!(race_feats.len(), 1);
    }

    #[test]
    fn test_character_has_feat() {
        let mut conn = setup_test_db();

        assert!(!character_has_feat(&mut conn, "char-1", "Lucky").expect("Failed to check"));

        let feat = NewCharacterFeat::bonus("feat-1", "char-1", "Lucky", "PHB");
        insert_character_feat(&mut conn, &feat).expect("Failed to insert");

        assert!(character_has_feat(&mut conn, "char-1", "Lucky").expect("Failed to check"));
    }

    #[test]
    fn test_different_source_types() {
        let mut conn = setup_test_db();

        let asi = NewCharacterFeat::new(
            "feat-1",
            "char-1",
            "Sentinel",
            "PHB",
            FeatSourceType::Asi,
        );
        let race = NewCharacterFeat::new(
            "feat-2",
            "char-1",
            "Prodigy",
            "XGE",
            FeatSourceType::Race,
        );
        let class = NewCharacterFeat::new(
            "feat-3",
            "char-1",
            "Martial Adept",
            "PHB",
            FeatSourceType::Class,
        );
        let bonus = NewCharacterFeat::new(
            "feat-4",
            "char-1",
            "Lucky",
            "PHB",
            FeatSourceType::Bonus,
        );

        insert_character_feat(&mut conn, &asi).expect("Failed to insert");
        insert_character_feat(&mut conn, &race).expect("Failed to insert");
        insert_character_feat(&mut conn, &class).expect("Failed to insert");
        insert_character_feat(&mut conn, &bonus).expect("Failed to insert");

        let feats = list_character_feats(&mut conn, "char-1").expect("Failed to list");
        assert_eq!(feats.len(), 4);
    }

    #[test]
    fn test_delete_character_feat() {
        let mut conn = setup_test_db();

        let feat = NewCharacterFeat::from_asi("feat-1", "char-1", "Tough", "PHB");
        insert_character_feat(&mut conn, &feat).expect("Failed to insert");

        assert!(character_feat_exists(&mut conn, "feat-1").expect("Failed to check"));

        delete_character_feat(&mut conn, "feat-1").expect("Failed to delete");

        assert!(!character_feat_exists(&mut conn, "feat-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_character_feats() {
        let mut conn = setup_test_db();

        assert_eq!(
            count_character_feats(&mut conn, "char-1").expect("Failed to count"),
            0
        );

        let feat1 = NewCharacterFeat::from_asi("feat-1", "char-1", "Alert", "PHB");
        let feat2 = NewCharacterFeat::from_asi("feat-2", "char-1", "Sentinel", "PHB");
        insert_character_feat(&mut conn, &feat1).expect("Failed to insert");
        insert_character_feat(&mut conn, &feat2).expect("Failed to insert");

        assert_eq!(
            count_character_feats(&mut conn, "char-1").expect("Failed to count"),
            2
        );
    }
}

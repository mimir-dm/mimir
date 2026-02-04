//! CharacterFeature Data Access Layer
//!
//! Database operations for character class feature choices (fighting styles, metamagic, etc).

use crate::models::campaign::{CharacterFeature, NewCharacterFeature};
use crate::schema::character_features;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new character feature.
pub fn insert_character_feature(
    conn: &mut SqliteConnection,
    feature: &NewCharacterFeature,
) -> QueryResult<String> {
    diesel::insert_into(character_features::table)
        .values(feature)
        .execute(conn)?;

    Ok(feature.id.to_string())
}

/// Get a character feature by ID.
pub fn get_character_feature(conn: &mut SqliteConnection, id: &str) -> QueryResult<CharacterFeature> {
    character_features::table.find(id).first(conn)
}

/// Get a character feature by ID, returning None if not found.
pub fn get_character_feature_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<CharacterFeature>> {
    character_features::table.find(id).first(conn).optional()
}

/// List all features for a character.
pub fn list_character_features(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<Vec<CharacterFeature>> {
    character_features::table
        .filter(character_features::character_id.eq(character_id))
        .order(character_features::feature_name.asc())
        .load(conn)
}

/// List features by type (fighting_style, metamagic, maneuver, invocation, pact_boon).
pub fn list_features_by_type(
    conn: &mut SqliteConnection,
    character_id: &str,
    feature_type: &str,
) -> QueryResult<Vec<CharacterFeature>> {
    character_features::table
        .filter(character_features::character_id.eq(character_id))
        .filter(character_features::feature_type.eq(feature_type))
        .order(character_features::feature_name.asc())
        .load(conn)
}

/// List features by source class.
pub fn list_features_by_class(
    conn: &mut SqliteConnection,
    character_id: &str,
    source_class: &str,
) -> QueryResult<Vec<CharacterFeature>> {
    character_features::table
        .filter(character_features::character_id.eq(character_id))
        .filter(character_features::source_class.eq(source_class))
        .order(character_features::feature_name.asc())
        .load(conn)
}

/// Check if a character has a specific feature (by type and name).
pub fn character_has_feature(
    conn: &mut SqliteConnection,
    character_id: &str,
    feature_type: &str,
    feature_name: &str,
) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(
        character_features::table
            .filter(character_features::character_id.eq(character_id))
            .filter(character_features::feature_type.eq(feature_type))
            .filter(character_features::feature_name.eq(feature_name)),
    ))
    .get_result(conn)
}

/// Find a feature by type and name (for swap operations).
pub fn find_feature_by_name(
    conn: &mut SqliteConnection,
    character_id: &str,
    feature_type: &str,
    feature_name: &str,
) -> QueryResult<Option<CharacterFeature>> {
    character_features::table
        .filter(character_features::character_id.eq(character_id))
        .filter(character_features::feature_type.eq(feature_type))
        .filter(character_features::feature_name.eq(feature_name))
        .first(conn)
        .optional()
}

/// Delete a character feature by ID.
pub fn delete_character_feature(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(character_features::table.find(id)).execute(conn)
}

/// Delete all features for a character.
pub fn delete_all_character_features(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        character_features::table.filter(character_features::character_id.eq(character_id)),
    )
    .execute(conn)
}

/// Delete features by type for a character.
pub fn delete_features_by_type(
    conn: &mut SqliteConnection,
    character_id: &str,
    feature_type: &str,
) -> QueryResult<usize> {
    diesel::delete(
        character_features::table
            .filter(character_features::character_id.eq(character_id))
            .filter(character_features::feature_type.eq(feature_type)),
    )
    .execute(conn)
}

/// Check if a character feature exists.
pub fn character_feature_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(character_features::table.find(id))).get_result(conn)
}

/// Count features for a character.
pub fn count_character_features(conn: &mut SqliteConnection, character_id: &str) -> QueryResult<i64> {
    character_features::table
        .filter(character_features::character_id.eq(character_id))
        .count()
        .get_result(conn)
}

/// Count features by type for a character.
pub fn count_features_by_type(
    conn: &mut SqliteConnection,
    character_id: &str,
    feature_type: &str,
) -> QueryResult<i64> {
    character_features::table
        .filter(character_features::character_id.eq(character_id))
        .filter(character_features::feature_type.eq(feature_type))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::{insert_campaign, insert_character};
    use crate::models::campaign::{NewCampaign, NewCharacter};

    fn setup_test_data(conn: &mut SqliteConnection) {
        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");

        let character = NewCharacter::new_pc("char-1", Some("camp-1"), "Test Fighter", "Player");
        insert_character(conn, &character).expect("Failed to create character");
    }

    #[test]
    fn test_insert_and_get_feature() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let feature = NewCharacterFeature::fighting_style(
            "feat-1", "char-1", "Defense", "PHB", "Fighter"
        );
        let id = insert_character_feature(&mut conn, &feature).expect("Failed to insert");
        assert_eq!(id, "feat-1");

        let retrieved = get_character_feature(&mut conn, "feat-1").expect("Failed to get");
        assert_eq!(retrieved.feature_name, "Defense");
        assert_eq!(retrieved.feature_type, "fighting_style");
        assert_eq!(retrieved.source_class, "Fighter");
    }

    #[test]
    fn test_list_features_by_type() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let fs = NewCharacterFeature::fighting_style("f1", "char-1", "Defense", "PHB", "Fighter");
        let mm1 = NewCharacterFeature::metamagic("f2", "char-1", "Quickened Spell", "PHB");
        let mm2 = NewCharacterFeature::metamagic("f3", "char-1", "Twinned Spell", "PHB");
        insert_character_feature(&mut conn, &fs).expect("Failed to insert");
        insert_character_feature(&mut conn, &mm1).expect("Failed to insert");
        insert_character_feature(&mut conn, &mm2).expect("Failed to insert");

        let fighting_styles = list_features_by_type(&mut conn, "char-1", "fighting_style")
            .expect("Failed to list");
        assert_eq!(fighting_styles.len(), 1);

        let metamagic = list_features_by_type(&mut conn, "char-1", "metamagic")
            .expect("Failed to list");
        assert_eq!(metamagic.len(), 2);
    }

    #[test]
    fn test_character_has_feature() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert!(!character_has_feature(&mut conn, "char-1", "fighting_style", "Defense")
            .expect("Failed to check"));

        let feature = NewCharacterFeature::fighting_style(
            "feat-1", "char-1", "Defense", "PHB", "Fighter"
        );
        insert_character_feature(&mut conn, &feature).expect("Failed to insert");

        assert!(character_has_feature(&mut conn, "char-1", "fighting_style", "Defense")
            .expect("Failed to check"));
    }

    #[test]
    fn test_find_feature_by_name() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let feature = NewCharacterFeature::invocation(
            "feat-1", "char-1", "Agonizing Blast", "PHB"
        );
        insert_character_feature(&mut conn, &feature).expect("Failed to insert");

        let found = find_feature_by_name(&mut conn, "char-1", "invocation", "Agonizing Blast")
            .expect("Failed to find");
        assert!(found.is_some());
        assert_eq!(found.unwrap().feature_name, "Agonizing Blast");

        let not_found = find_feature_by_name(&mut conn, "char-1", "invocation", "Devil's Sight")
            .expect("Failed to find");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_delete_feature() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let feature = NewCharacterFeature::maneuver("feat-1", "char-1", "Riposte", "PHB");
        insert_character_feature(&mut conn, &feature).expect("Failed to insert");

        assert!(character_feature_exists(&mut conn, "feat-1").expect("Failed to check"));

        delete_character_feature(&mut conn, "feat-1").expect("Failed to delete");

        assert!(!character_feature_exists(&mut conn, "feat-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_features_by_type() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let m1 = NewCharacterFeature::maneuver("f1", "char-1", "Riposte", "PHB");
        let m2 = NewCharacterFeature::maneuver("f2", "char-1", "Precision Attack", "PHB");
        let m3 = NewCharacterFeature::maneuver("f3", "char-1", "Trip Attack", "PHB");
        insert_character_feature(&mut conn, &m1).expect("Failed to insert");
        insert_character_feature(&mut conn, &m2).expect("Failed to insert");
        insert_character_feature(&mut conn, &m3).expect("Failed to insert");

        let count = count_features_by_type(&mut conn, "char-1", "maneuver")
            .expect("Failed to count");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_pact_boon() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let boon = NewCharacterFeature::pact_boon("feat-1", "char-1", "Pact of the Blade", "PHB");
        insert_character_feature(&mut conn, &boon).expect("Failed to insert");

        let retrieved = get_character_feature(&mut conn, "feat-1").expect("Failed to get");
        assert_eq!(retrieved.feature_type, "pact_boon");
        assert_eq!(retrieved.feature_name, "Pact of the Blade");
        assert_eq!(retrieved.source_class, "Warlock");
    }
}

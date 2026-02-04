//! Character Data Access Layer
//!
//! Database operations for characters (PCs and NPCs).

use crate::models::campaign::{Character, NewCharacter, UpdateCharacter};
use crate::schema::characters;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new character.
pub fn insert_character(conn: &mut SqliteConnection, character: &NewCharacter) -> QueryResult<String> {
    diesel::insert_into(characters::table)
        .values(character)
        .execute(conn)?;

    Ok(character.id.to_string())
}

/// Get a character by ID.
pub fn get_character(conn: &mut SqliteConnection, id: &str) -> QueryResult<Character> {
    characters::table.find(id).first(conn)
}

/// Get a character by ID, returning None if not found.
pub fn get_character_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<Character>> {
    characters::table.find(id).first(conn).optional()
}

/// List all characters for a campaign.
pub fn list_campaign_characters(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<Character>> {
    characters::table
        .filter(characters::campaign_id.eq(campaign_id))
        .order(characters::name.asc())
        .load(conn)
}

/// List all player characters for a campaign.
pub fn list_pcs(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<Vec<Character>> {
    characters::table
        .filter(characters::campaign_id.eq(campaign_id))
        .filter(characters::is_npc.eq(0))
        .order(characters::name.asc())
        .load(conn)
}

/// List all unassigned player characters (no campaign).
pub fn list_unassigned_pcs(conn: &mut SqliteConnection) -> QueryResult<Vec<Character>> {
    characters::table
        .filter(characters::campaign_id.is_null())
        .filter(characters::is_npc.eq(0))
        .order(characters::name.asc())
        .load(conn)
}

/// List all NPCs for a campaign.
pub fn list_npcs(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<Vec<Character>> {
    characters::table
        .filter(characters::campaign_id.eq(campaign_id))
        .filter(characters::is_npc.eq(1))
        .order(characters::name.asc())
        .load(conn)
}

/// List NPCs by location.
pub fn list_npcs_by_location(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    location: &str,
) -> QueryResult<Vec<Character>> {
    characters::table
        .filter(characters::campaign_id.eq(campaign_id))
        .filter(characters::is_npc.eq(1))
        .filter(characters::location.eq(location))
        .order(characters::name.asc())
        .load(conn)
}

/// List NPCs by faction.
pub fn list_npcs_by_faction(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    faction: &str,
) -> QueryResult<Vec<Character>> {
    characters::table
        .filter(characters::campaign_id.eq(campaign_id))
        .filter(characters::is_npc.eq(1))
        .filter(characters::faction.eq(faction))
        .order(characters::name.asc())
        .load(conn)
}

/// Update a character.
pub fn update_character(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateCharacter,
) -> QueryResult<usize> {
    diesel::update(characters::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a character by ID.
pub fn delete_character(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(characters::table.find(id)).execute(conn)
}

/// Assign a character to a campaign.
pub fn assign_character_to_campaign(
    conn: &mut SqliteConnection,
    character_id: &str,
    campaign_id: &str,
) -> QueryResult<usize> {
    diesel::update(characters::table.find(character_id))
        .set(characters::campaign_id.eq(Some(campaign_id)))
        .execute(conn)
}

/// Check if a character exists.
pub fn character_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(characters::table.find(id))).get_result(conn)
}

/// Count all characters for a campaign.
pub fn count_campaign_characters(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<i64> {
    characters::table
        .filter(characters::campaign_id.eq(campaign_id))
        .count()
        .get_result(conn)
}

/// Count PCs for a campaign.
pub fn count_pcs(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<i64> {
    characters::table
        .filter(characters::campaign_id.eq(campaign_id))
        .filter(characters::is_npc.eq(0))
        .count()
        .get_result(conn)
}

/// Count NPCs for a campaign.
pub fn count_npcs(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<i64> {
    characters::table
        .filter(characters::campaign_id.eq(campaign_id))
        .filter(characters::is_npc.eq(1))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::insert_campaign;
    use crate::models::campaign::NewCampaign;

    fn setup_test_data(conn: &mut SqliteConnection) {
        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");
    }

    #[test]
    fn test_insert_and_get_character() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let pc = NewCharacter::new_pc("char-1", Some("camp-1"), "Gandalf", "John");
        let id = insert_character(&mut conn, &pc).expect("Failed to insert");
        assert_eq!(id, "char-1");

        let retrieved = get_character(&mut conn, "char-1").expect("Failed to get");
        assert_eq!(retrieved.id, "char-1");
        assert_eq!(retrieved.name, "Gandalf");
        assert_eq!(retrieved.is_npc, 0);
        assert_eq!(retrieved.player_name, Some("John".to_string()));
    }

    #[test]
    fn test_insert_npc() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let npc = NewCharacter::new_npc("char-1", Some("camp-1"), "Shopkeeper")
            .with_npc_info(Some("merchant"), Some("Waterdeep"), Some("Merchants Guild"));
        insert_character(&mut conn, &npc).expect("Failed to insert");

        let retrieved = get_character(&mut conn, "char-1").expect("Failed to get");
        assert_eq!(retrieved.is_npc, 1);
        assert_eq!(retrieved.role, Some("merchant".to_string()));
        assert_eq!(retrieved.location, Some("Waterdeep".to_string()));
        assert_eq!(retrieved.faction, Some("Merchants Guild".to_string()));
    }

    #[test]
    fn test_list_campaign_characters() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let pc1 = NewCharacter::new_pc("char-1", Some("camp-1"), "Aragorn", "John");
        let pc2 = NewCharacter::new_pc("char-2", Some("camp-1"), "Legolas", "Jane");
        let npc = NewCharacter::new_npc("char-3", Some("camp-1"), "Innkeeper");
        insert_character(&mut conn, &pc1).expect("Failed to insert");
        insert_character(&mut conn, &pc2).expect("Failed to insert");
        insert_character(&mut conn, &npc).expect("Failed to insert");

        let all = list_campaign_characters(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_list_pcs_and_npcs() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let pc1 = NewCharacter::new_pc("char-1", Some("camp-1"), "Fighter", "John");
        let pc2 = NewCharacter::new_pc("char-2", Some("camp-1"), "Wizard", "Jane");
        let npc1 = NewCharacter::new_npc("char-3", Some("camp-1"), "Guard");
        let npc2 = NewCharacter::new_npc("char-4", Some("camp-1"), "Merchant");
        let npc3 = NewCharacter::new_npc("char-5", Some("camp-1"), "Bard");
        insert_character(&mut conn, &pc1).expect("Failed to insert");
        insert_character(&mut conn, &pc2).expect("Failed to insert");
        insert_character(&mut conn, &npc1).expect("Failed to insert");
        insert_character(&mut conn, &npc2).expect("Failed to insert");
        insert_character(&mut conn, &npc3).expect("Failed to insert");

        let pcs = list_pcs(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(pcs.len(), 2);

        let npcs = list_npcs(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(npcs.len(), 3);
    }

    #[test]
    fn test_list_npcs_by_location() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let npc1 = NewCharacter::new_npc("char-1", Some("camp-1"), "Guard")
            .with_npc_info(Some("guard"), Some("Waterdeep"), None);
        let npc2 = NewCharacter::new_npc("char-2", Some("camp-1"), "Merchant")
            .with_npc_info(Some("merchant"), Some("Waterdeep"), None);
        let npc3 = NewCharacter::new_npc("char-3", Some("camp-1"), "Farmer")
            .with_npc_info(Some("commoner"), Some("Phandalin"), None);
        insert_character(&mut conn, &npc1).expect("Failed to insert");
        insert_character(&mut conn, &npc2).expect("Failed to insert");
        insert_character(&mut conn, &npc3).expect("Failed to insert");

        let waterdeep_npcs =
            list_npcs_by_location(&mut conn, "camp-1", "Waterdeep").expect("Failed to list");
        assert_eq!(waterdeep_npcs.len(), 2);

        let phandalin_npcs =
            list_npcs_by_location(&mut conn, "camp-1", "Phandalin").expect("Failed to list");
        assert_eq!(phandalin_npcs.len(), 1);
    }

    #[test]
    fn test_list_npcs_by_faction() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let npc1 = NewCharacter::new_npc("char-1", Some("camp-1"), "Agent")
            .with_npc_info(None, None, Some("Zhentarim"));
        let npc2 = NewCharacter::new_npc("char-2", Some("camp-1"), "Spy")
            .with_npc_info(None, None, Some("Zhentarim"));
        let npc3 = NewCharacter::new_npc("char-3", Some("camp-1"), "Paladin")
            .with_npc_info(None, None, Some("Order of the Gauntlet"));
        insert_character(&mut conn, &npc1).expect("Failed to insert");
        insert_character(&mut conn, &npc2).expect("Failed to insert");
        insert_character(&mut conn, &npc3).expect("Failed to insert");

        let zhentarim =
            list_npcs_by_faction(&mut conn, "camp-1", "Zhentarim").expect("Failed to list");
        assert_eq!(zhentarim.len(), 2);
    }

    #[test]
    fn test_update_character_name() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let pc = NewCharacter::new_pc("char-1", Some("camp-1"), "Original", "John");
        insert_character(&mut conn, &pc).expect("Failed to insert");

        let update = UpdateCharacter::set_name("Updated", "2024-01-20T12:00:00Z");
        update_character(&mut conn, "char-1", &update).expect("Failed to update");

        let retrieved = get_character(&mut conn, "char-1").expect("Failed to get");
        assert_eq!(retrieved.name, "Updated");
    }

    #[test]
    fn test_update_ability_scores() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let pc = NewCharacter::new_pc("char-1", Some("camp-1"), "Fighter", "John");
        insert_character(&mut conn, &pc).expect("Failed to insert");

        let update =
            UpdateCharacter::set_ability_scores(18, 14, 16, 10, 12, 8, "2024-01-20T12:00:00Z");
        update_character(&mut conn, "char-1", &update).expect("Failed to update");

        let retrieved = get_character(&mut conn, "char-1").expect("Failed to get");
        assert_eq!(retrieved.strength, 18);
        assert_eq!(retrieved.dexterity, 14);
        assert_eq!(retrieved.constitution, 16);
        assert_eq!(retrieved.intelligence, 10);
        assert_eq!(retrieved.wisdom, 12);
        assert_eq!(retrieved.charisma, 8);
    }

    #[test]
    fn test_update_currency() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let pc = NewCharacter::new_pc("char-1", Some("camp-1"), "Rogue", "Jane");
        insert_character(&mut conn, &pc).expect("Failed to insert");

        let update = UpdateCharacter::set_currency(500, 100, 50, 25, 10, "2024-01-20T12:00:00Z");
        update_character(&mut conn, "char-1", &update).expect("Failed to update");

        let retrieved = get_character(&mut conn, "char-1").expect("Failed to get");
        assert_eq!(retrieved.cp, 500);
        assert_eq!(retrieved.sp, 100);
        assert_eq!(retrieved.ep, 50);
        assert_eq!(retrieved.gp, 25);
        assert_eq!(retrieved.pp, 10);
    }

    #[test]
    fn test_delete_character() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let pc = NewCharacter::new_pc("char-1", Some("camp-1"), "Doomed", "John");
        insert_character(&mut conn, &pc).expect("Failed to insert");

        assert!(character_exists(&mut conn, "char-1").expect("Failed to check"));

        delete_character(&mut conn, "char-1").expect("Failed to delete");

        assert!(!character_exists(&mut conn, "char-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_characters() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert_eq!(
            count_campaign_characters(&mut conn, "camp-1").expect("Failed to count"),
            0
        );

        let pc = NewCharacter::new_pc("char-1", Some("camp-1"), "PC", "John");
        let npc1 = NewCharacter::new_npc("char-2", Some("camp-1"), "NPC1");
        let npc2 = NewCharacter::new_npc("char-3", Some("camp-1"), "NPC2");
        insert_character(&mut conn, &pc).expect("Failed to insert");
        insert_character(&mut conn, &npc1).expect("Failed to insert");
        insert_character(&mut conn, &npc2).expect("Failed to insert");

        assert_eq!(
            count_campaign_characters(&mut conn, "camp-1").expect("Failed to count"),
            3
        );
        assert_eq!(count_pcs(&mut conn, "camp-1").expect("Failed to count"), 1);
        assert_eq!(count_npcs(&mut conn, "camp-1").expect("Failed to count"), 2);
    }

    #[test]
    fn test_get_character_optional() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let result = get_character_optional(&mut conn, "nonexistent").expect("Failed to query");
        assert!(result.is_none());

        let pc = NewCharacter::new_pc("char-1", Some("camp-1"), "Hero", "John");
        insert_character(&mut conn, &pc).expect("Failed to insert");

        let result = get_character_optional(&mut conn, "char-1").expect("Failed to query");
        assert!(result.is_some());
    }
}

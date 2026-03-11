//! ModuleMonster Data Access Layer
//!
//! Database operations for module monsters (catalog monster instances).

use crate::models::campaign::{ModuleMonster, NewModuleMonster, UpdateModuleMonster};
use crate::schema::module_monsters;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new module monster.
pub fn insert_module_monster(
    conn: &mut SqliteConnection,
    monster: &NewModuleMonster,
) -> QueryResult<String> {
    diesel::insert_into(module_monsters::table)
        .values(monster)
        .execute(conn)?;

    Ok(monster.id.to_string())
}

/// Get a module monster by ID.
pub fn get_module_monster(conn: &mut SqliteConnection, id: &str) -> QueryResult<ModuleMonster> {
    module_monsters::table.find(id).first(conn)
}

/// Get a module monster by ID, returning None if not found.
pub fn get_module_monster_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<ModuleMonster>> {
    module_monsters::table.find(id).first(conn).optional()
}

/// List all monsters for a module.
pub fn list_module_monsters(
    conn: &mut SqliteConnection,
    module_id: &str,
) -> QueryResult<Vec<ModuleMonster>> {
    module_monsters::table
        .filter(module_monsters::module_id.eq(module_id))
        .order((module_monsters::monster_name.asc(), module_monsters::created_at.asc()))
        .load(conn)
}

/// Update a module monster.
pub fn update_module_monster(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateModuleMonster,
) -> QueryResult<usize> {
    diesel::update(module_monsters::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a module monster by ID.
pub fn delete_module_monster(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(module_monsters::table.find(id)).execute(conn)
}

/// Delete all monsters for a module.
pub fn delete_all_module_monsters(
    conn: &mut SqliteConnection,
    module_id: &str,
) -> QueryResult<usize> {
    diesel::delete(module_monsters::table.filter(module_monsters::module_id.eq(module_id)))
        .execute(conn)
}

/// Check if a module monster exists.
pub fn module_monster_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(module_monsters::table.find(id))).get_result(conn)
}

/// Count monsters for a module.
pub fn count_module_monsters(conn: &mut SqliteConnection, module_id: &str) -> QueryResult<i64> {
    module_monsters::table
        .filter(module_monsters::module_id.eq(module_id))
        .count()
        .get_result(conn)
}

/// Get total monster count for a module (sum of quantities).
pub fn get_total_monster_count(conn: &mut SqliteConnection, module_id: &str) -> QueryResult<i64> {
    use diesel::dsl::sum;

    module_monsters::table
        .filter(module_monsters::module_id.eq(module_id))
        .select(sum(module_monsters::quantity))
        .first::<Option<i64>>(conn)
        .map(|opt| opt.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::{insert_campaign, insert_module};
    use crate::models::campaign::{NewCampaign, NewModule};

    fn setup_test_data(conn: &mut SqliteConnection) {
        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");

        let module = NewModule::new("mod-1", "camp-1", "Dungeon", 1);
        insert_module(conn, &module).expect("Failed to create module");
    }

    #[test]
    fn test_insert_and_get_module_monster() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let monster = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM");
        let id = insert_module_monster(&mut conn, &monster).expect("Failed to insert");
        assert_eq!(id, "mm-1");

        let retrieved = get_module_monster(&mut conn, "mm-1").expect("Failed to get");
        assert_eq!(retrieved.monster_name, Some("Goblin".to_string()));
        assert_eq!(retrieved.monster_source, Some("MM".to_string()));
        assert!(retrieved.homebrew_monster_id.is_none());
        assert!(retrieved.display_name.is_none());
    }

    #[test]
    fn test_insert_with_customizations() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let monster = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM")
            .with_display_name("Goblin Chief")
            .with_notes("Leader of the goblins")
            .with_quantity(1);
        insert_module_monster(&mut conn, &monster).expect("Failed to insert");

        let retrieved = get_module_monster(&mut conn, "mm-1").expect("Failed to get");
        assert_eq!(retrieved.display_name, Some("Goblin Chief".to_string()));
        assert!(retrieved.notes.unwrap().contains("Leader"));
    }

    #[test]
    fn test_list_module_monsters() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let goblin = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM");
        let wolf = NewModuleMonster::new("mm-2", "mod-1", "Wolf", "MM");
        let bugbear = NewModuleMonster::new("mm-3", "mod-1", "Bugbear", "MM");
        insert_module_monster(&mut conn, &goblin).expect("Failed to insert");
        insert_module_monster(&mut conn, &wolf).expect("Failed to insert");
        insert_module_monster(&mut conn, &bugbear).expect("Failed to insert");

        let monsters = list_module_monsters(&mut conn, "mod-1").expect("Failed to list");
        assert_eq!(monsters.len(), 3);
        // Sorted alphabetically by monster_name
        assert_eq!(monsters[0].monster_name, Some("Bugbear".to_string()));
        assert_eq!(monsters[1].monster_name, Some("Goblin".to_string()));
        assert_eq!(monsters[2].monster_name, Some("Wolf".to_string()));
    }

    #[test]
    fn test_update_module_monster() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let monster = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM");
        insert_module_monster(&mut conn, &monster).expect("Failed to insert");

        let update = UpdateModuleMonster::set_quantity(6, "2024-01-20T12:00:00Z");
        update_module_monster(&mut conn, "mm-1", &update).expect("Failed to update");

        let retrieved = get_module_monster(&mut conn, "mm-1").expect("Failed to get");
        assert_eq!(retrieved.quantity, 6);
    }

    #[test]
    fn test_delete_module_monster() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let monster = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM");
        insert_module_monster(&mut conn, &monster).expect("Failed to insert");

        assert!(module_monster_exists(&mut conn, "mm-1").expect("Failed to check"));

        delete_module_monster(&mut conn, "mm-1").expect("Failed to delete");

        assert!(!module_monster_exists(&mut conn, "mm-1").expect("Failed to check"));
    }

    #[test]
    fn test_count_and_total() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let goblin = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM")
            .with_quantity(6);
        let wolf = NewModuleMonster::new("mm-2", "mod-1", "Wolf", "MM")
            .with_quantity(2);
        insert_module_monster(&mut conn, &goblin).expect("Failed to insert");
        insert_module_monster(&mut conn, &wolf).expect("Failed to insert");

        // 2 distinct monster types
        assert_eq!(
            count_module_monsters(&mut conn, "mod-1").expect("Failed to count"),
            2
        );

        // 8 total monsters (6 goblins + 2 wolves)
        assert_eq!(
            get_total_monster_count(&mut conn, "mod-1").expect("Failed to count"),
            8
        );
    }

    #[test]
    fn test_insert_homebrew_module_monster() {
        use crate::dal::campaign::insert_campaign_homebrew_monster;
        use crate::models::campaign::NewCampaignHomebrewMonster;

        let mut conn = test_connection();
        setup_test_data(&mut conn);

        // Create a homebrew monster in the campaign first (FK target)
        let hb = NewCampaignHomebrewMonster::new("hb-1", "camp-1", "Flame Skulker", "{}");
        insert_campaign_homebrew_monster(&mut conn, &hb).expect("Failed to create homebrew monster");

        // Add it to the module via homebrew reference
        let monster = NewModuleMonster::from_homebrew("mm-hb-1", "mod-1", "hb-1")
            .with_display_name("Flame Skulker Alpha")
            .with_quantity(2);
        insert_module_monster(&mut conn, &monster).expect("Failed to insert homebrew module monster");

        let retrieved = get_module_monster(&mut conn, "mm-hb-1").expect("Failed to get");
        assert!(retrieved.monster_name.is_none());
        assert!(retrieved.monster_source.is_none());
        assert_eq!(retrieved.homebrew_monster_id, Some("hb-1".to_string()));
        assert_eq!(retrieved.display_name, Some("Flame Skulker Alpha".to_string()));
        assert_eq!(retrieved.quantity, 2);
        assert!(retrieved.is_homebrew());
        assert!(!retrieved.is_catalog());
    }

    #[test]
    fn test_list_mixed_catalog_and_homebrew() {
        use crate::dal::campaign::insert_campaign_homebrew_monster;
        use crate::models::campaign::NewCampaignHomebrewMonster;

        let mut conn = test_connection();
        setup_test_data(&mut conn);

        // Create homebrew monster
        let hb = NewCampaignHomebrewMonster::new("hb-1", "camp-1", "Void Wyrm", "{}");
        insert_campaign_homebrew_monster(&mut conn, &hb).expect("Failed to create homebrew");

        // Add catalog monster
        let catalog = NewModuleMonster::new("mm-1", "mod-1", "Goblin", "MM");
        insert_module_monster(&mut conn, &catalog).expect("Failed to insert catalog");

        // Add homebrew monster
        let homebrew = NewModuleMonster::from_homebrew("mm-2", "mod-1", "hb-1");
        insert_module_monster(&mut conn, &homebrew).expect("Failed to insert homebrew");

        let monsters = list_module_monsters(&mut conn, "mod-1").expect("Failed to list");
        assert_eq!(monsters.len(), 2);

        // Catalog monster has name, homebrew doesn't — sorted by name then created_at
        // NULL names sort before non-NULL in SQLite ASC order
        let catalog_m = monsters.iter().find(|m| m.is_catalog()).unwrap();
        let homebrew_m = monsters.iter().find(|m| m.is_homebrew()).unwrap();

        assert_eq!(catalog_m.monster_name, Some("Goblin".to_string()));
        assert_eq!(homebrew_m.homebrew_monster_id, Some("hb-1".to_string()));
    }

    #[test]
    fn test_delete_homebrew_module_monster() {
        use crate::dal::campaign::insert_campaign_homebrew_monster;
        use crate::models::campaign::NewCampaignHomebrewMonster;

        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let hb = NewCampaignHomebrewMonster::new("hb-1", "camp-1", "Shadow Beast", "{}");
        insert_campaign_homebrew_monster(&mut conn, &hb).expect("Failed to create homebrew");

        let monster = NewModuleMonster::from_homebrew("mm-hb-1", "mod-1", "hb-1");
        insert_module_monster(&mut conn, &monster).expect("Failed to insert");

        assert!(module_monster_exists(&mut conn, "mm-hb-1").expect("Failed to check"));
        delete_module_monster(&mut conn, "mm-hb-1").expect("Failed to delete");
        assert!(!module_monster_exists(&mut conn, "mm-hb-1").expect("Failed to check"));
    }
}

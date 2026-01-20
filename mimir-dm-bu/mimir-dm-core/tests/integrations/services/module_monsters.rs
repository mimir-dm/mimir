//! Integration tests for module monster service

use mimir_dm_core::establish_connection;
use mimir_dm_core::run_migrations;
use mimir_dm_core::services::{CampaignService, ModuleMonsterService, ModuleService};
use tempfile::TempDir;

fn setup_test_db() -> mimir_dm_core::connection::DbConnection {
    let mut conn = establish_connection(":memory:").unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");

    // Seed templates
    mimir_dm_core::seed::template_seeder::seed_templates(&mut conn).unwrap();

    conn
}

fn create_test_module(conn: &mut mimir_dm_core::connection::DbConnection) -> (i32, i32) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir_path = temp_dir.path().to_string_lossy().to_string();

    let mut campaign_service = CampaignService::new(conn);
    let campaign = campaign_service
        .create_campaign(
            "Test Campaign",
            Some("Test campaign for module monster tests".to_string()),
            &dir_path,
        )
        .unwrap();

    let mut module_service = ModuleService::new(conn);
    let module = module_service
        .create_module(campaign.id, "Test Module".to_string(), 4)
        .unwrap();

    // Keep temp_dir alive by leaking it - in tests this is okay
    std::mem::forget(temp_dir);

    (campaign.id, module.id)
}

#[test]
fn test_add_monster() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    let monster = service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            3,
            None,
        )
        .unwrap();

    assert_eq!(monster.module_id, module_id);
    assert_eq!(monster.monster_name, "Goblin");
    assert_eq!(monster.monster_source, "MM");
    assert_eq!(monster.quantity, 3);
    assert!(monster.encounter_tag.is_none());
}

#[test]
fn test_add_monster_with_encounter_tag() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    let monster = service
        .add_monster(
            module_id,
            "Wolf".to_string(),
            "MM".to_string(),
            2,
            Some("Cave Entrance".to_string()),
        )
        .unwrap();

    assert_eq!(monster.monster_name, "Wolf");
    assert_eq!(monster.encounter_tag, Some("Cave Entrance".to_string()));
}

#[test]
fn test_add_duplicate_monster_combines_quantity() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    // Add 2 goblins
    service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            2,
            None,
        )
        .unwrap();

    // Add 3 more goblins - should combine
    let monster = service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            3,
            None,
        )
        .unwrap();

    assert_eq!(monster.quantity, 5); // 2 + 3

    // Verify only one entry exists
    let monsters = service.get_monsters_for_module(module_id).unwrap();
    assert_eq!(monsters.len(), 1);
}

#[test]
fn test_add_same_monster_different_encounters_separate() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    // Add goblins to different encounters
    service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            2,
            Some("Encounter A".to_string()),
        )
        .unwrap();

    service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            3,
            Some("Encounter B".to_string()),
        )
        .unwrap();

    // Should be separate entries
    let monsters = service.get_monsters_for_module(module_id).unwrap();
    assert_eq!(monsters.len(), 2);
}

#[test]
fn test_remove_monster() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    let monster = service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            3,
            None,
        )
        .unwrap();

    // Verify it exists
    let monsters = service.get_monsters_for_module(module_id).unwrap();
    assert_eq!(monsters.len(), 1);

    // Remove it
    service.remove_monster(monster.id).unwrap();

    // Verify it's gone
    let monsters = service.get_monsters_for_module(module_id).unwrap();
    assert_eq!(monsters.len(), 0);
}

#[test]
fn test_update_monster_quantity() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    let monster = service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            3,
            None,
        )
        .unwrap();

    // Update quantity
    let updated = service
        .update_monster(monster.id, Some(5), None)
        .unwrap();

    assert_eq!(updated.quantity, 5);
}

#[test]
fn test_update_monster_encounter_tag() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    let monster = service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            3,
            None,
        )
        .unwrap();

    assert!(monster.encounter_tag.is_none());

    // Add encounter tag
    let updated = service
        .update_monster(monster.id, None, Some(Some("Boss Fight".to_string())))
        .unwrap();

    assert_eq!(updated.encounter_tag, Some("Boss Fight".to_string()));

    // Remove encounter tag
    let updated = service
        .update_monster(monster.id, None, Some(None))
        .unwrap();

    assert!(updated.encounter_tag.is_none());
}

#[test]
fn test_get_monsters_for_module() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            3,
            None,
        )
        .unwrap();

    service
        .add_monster(
            module_id,
            "Wolf".to_string(),
            "MM".to_string(),
            2,
            Some("Cave Entrance".to_string()),
        )
        .unwrap();

    service
        .add_monster(
            module_id,
            "Bugbear".to_string(),
            "MM".to_string(),
            1,
            Some("Boss Room".to_string()),
        )
        .unwrap();

    let monsters = service.get_monsters_for_module(module_id).unwrap();
    assert_eq!(monsters.len(), 3);
}

#[test]
fn test_get_monsters_grouped_by_encounter() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    // Add monsters to different encounters
    service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            2,
            Some("Cave Entrance".to_string()),
        )
        .unwrap();

    service
        .add_monster(
            module_id,
            "Wolf".to_string(),
            "MM".to_string(),
            3,
            Some("Cave Entrance".to_string()),
        )
        .unwrap();

    service
        .add_monster(
            module_id,
            "Bugbear".to_string(),
            "MM".to_string(),
            1,
            Some("Boss Room".to_string()),
        )
        .unwrap();

    service
        .add_monster(
            module_id,
            "Rat".to_string(),
            "MM".to_string(),
            5,
            None, // No encounter tag
        )
        .unwrap();

    let groups = service.get_monsters_grouped_by_encounter(module_id).unwrap();

    // Should have 3 groups: Cave Entrance, Boss Room, and None
    assert_eq!(groups.len(), 3);

    // Find each group and verify
    let cave_entrance = groups.iter().find(|g| g.encounter_tag == Some("Cave Entrance".to_string()));
    assert!(cave_entrance.is_some());
    assert_eq!(cave_entrance.unwrap().monsters.len(), 2);

    let boss_room = groups.iter().find(|g| g.encounter_tag == Some("Boss Room".to_string()));
    assert!(boss_room.is_some());
    assert_eq!(boss_room.unwrap().monsters.len(), 1);

    let untagged = groups.iter().find(|g| g.encounter_tag.is_none());
    assert!(untagged.is_some());
    assert_eq!(untagged.unwrap().monsters.len(), 1);
}

#[test]
fn test_get_encounter_tags() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    service
        .add_monster(
            module_id,
            "Goblin".to_string(),
            "MM".to_string(),
            2,
            Some("Cave Entrance".to_string()),
        )
        .unwrap();

    service
        .add_monster(
            module_id,
            "Bugbear".to_string(),
            "MM".to_string(),
            1,
            Some("Boss Room".to_string()),
        )
        .unwrap();

    service
        .add_monster(
            module_id,
            "Rat".to_string(),
            "MM".to_string(),
            5,
            None,
        )
        .unwrap();

    let tags = service.get_encounter_tags(module_id).unwrap();

    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&Some("Cave Entrance".to_string())));
    assert!(tags.contains(&Some("Boss Room".to_string())));
    assert!(tags.contains(&None));
}

#[test]
fn test_clear_module_monsters() {
    let mut conn = setup_test_db();
    let (_, module_id) = create_test_module(&mut conn);

    let mut service = ModuleMonsterService::new(&mut conn);

    // Add several monsters
    service
        .add_monster(module_id, "Goblin".to_string(), "MM".to_string(), 3, None)
        .unwrap();
    service
        .add_monster(module_id, "Wolf".to_string(), "MM".to_string(), 2, None)
        .unwrap();
    service
        .add_monster(module_id, "Bugbear".to_string(), "MM".to_string(), 1, None)
        .unwrap();

    // Verify they exist
    let monsters = service.get_monsters_for_module(module_id).unwrap();
    assert_eq!(monsters.len(), 3);

    // Clear all
    let deleted = service.clear_module_monsters(module_id).unwrap();
    assert_eq!(deleted, 3);

    // Verify all gone
    let monsters = service.get_monsters_for_module(module_id).unwrap();
    assert_eq!(monsters.len(), 0);
}

#[test]
fn test_monsters_isolated_between_modules() {
    let mut conn = setup_test_db();

    // Create two modules
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir_path = temp_dir.path().to_string_lossy().to_string();

    let mut campaign_service = CampaignService::new(&mut conn);
    let campaign = campaign_service
        .create_campaign("Test Campaign", None, &dir_path)
        .unwrap();

    let mut module_service = ModuleService::new(&mut conn);
    let module1 = module_service
        .create_module(campaign.id, "Module 1".to_string(), 4)
        .unwrap();
    let module2 = module_service
        .create_module(campaign.id, "Module 2".to_string(), 4)
        .unwrap();

    std::mem::forget(temp_dir);

    let mut service = ModuleMonsterService::new(&mut conn);

    // Add monsters to module 1
    service
        .add_monster(module1.id, "Goblin".to_string(), "MM".to_string(), 3, None)
        .unwrap();

    // Add monsters to module 2
    service
        .add_monster(module2.id, "Dragon".to_string(), "MM".to_string(), 1, None)
        .unwrap();

    // Verify isolation
    let module1_monsters = service.get_monsters_for_module(module1.id).unwrap();
    assert_eq!(module1_monsters.len(), 1);
    assert_eq!(module1_monsters[0].monster_name, "Goblin");

    let module2_monsters = service.get_monsters_for_module(module2.id).unwrap();
    assert_eq!(module2_monsters.len(), 1);
    assert_eq!(module2_monsters[0].monster_name, "Dragon");
}

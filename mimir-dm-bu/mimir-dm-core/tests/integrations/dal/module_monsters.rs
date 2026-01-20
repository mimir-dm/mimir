//! Integration tests for module monster DAL

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::dal::campaign::module_monsters::ModuleMonsterRepository;
use mimir_dm_core::dal::campaign::modules::ModuleRepository;
use mimir_dm_core::models::campaign::campaigns::NewCampaign;
use mimir_dm_core::models::campaign::module_monsters::{NewModuleMonster, UpdateModuleMonster};
use mimir_dm_core::models::campaign::modules::NewModule;
use tempfile::TempDir;

/// Helper to create test campaign and module
fn setup_campaign_and_module(
    conn: &mut diesel::SqliteConnection,
    temp_dir: &TempDir,
) -> (i32, i32) {
    let mut campaign_repo = CampaignRepository::new(conn);
    let campaign = campaign_repo
        .create(NewCampaign {
            name: "Test Campaign".to_string(),
            status: "active".to_string(),
            directory_path: temp_dir.path().to_string_lossy().to_string(),
        })
        .unwrap();

    let mut module_repo = ModuleRepository::new(conn);
    let module = module_repo
        .create(NewModule {
            campaign_id: campaign.id,
            name: "Test Module".to_string(),
            module_number: 1,
            status: "planning".to_string(),
            expected_sessions: 4,
        })
        .unwrap();

    (campaign.id, module.id)
}

#[test]
fn test_create_module_monster() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);
    let monster = repo
        .create(NewModuleMonster {
            module_id,
            monster_name: "Goblin".to_string(),
            monster_source: "PHB".to_string(),
            quantity: 4,
            encounter_tag: Some("ambush".to_string()),
        })
        .unwrap();

    assert_eq!(monster.monster_name, "Goblin");
    assert_eq!(monster.monster_source, "PHB");
    assert_eq!(monster.quantity, 4);
    assert_eq!(monster.encounter_tag, Some("ambush".to_string()));
    assert_eq!(monster.module_id, module_id);
}

#[test]
fn test_find_monster_by_id() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);
    let created = repo
        .create(NewModuleMonster {
            module_id,
            monster_name: "Orc".to_string(),
            monster_source: "MM".to_string(),
            quantity: 2,
            encounter_tag: None,
        })
        .unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.monster_name, "Orc");
}

#[test]
fn test_find_nonexistent_monster() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = ModuleMonsterRepository::new(&mut conn);
    let result = repo.find_by_id(99999).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_update_module_monster() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);
    let created = repo
        .create(NewModuleMonster {
            module_id,
            monster_name: "Skeleton".to_string(),
            monster_source: "MM".to_string(),
            quantity: 3,
            encounter_tag: None,
        })
        .unwrap();

    let updated = repo
        .update(
            created.id,
            UpdateModuleMonster {
                quantity: Some(6),
                encounter_tag: Some(Some("crypt_battle".to_string())),
            },
        )
        .unwrap();

    assert_eq!(updated.quantity, 6);
    assert_eq!(updated.encounter_tag, Some("crypt_battle".to_string()));
}

#[test]
fn test_delete_module_monster() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);
    let created = repo
        .create(NewModuleMonster {
            module_id,
            monster_name: "Zombie".to_string(),
            monster_source: "MM".to_string(),
            quantity: 5,
            encounter_tag: None,
        })
        .unwrap();

    repo.delete(created.id).unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_none());
}

#[test]
fn test_list_monsters_by_module() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);

    // Create multiple monsters
    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Wolf".to_string(),
        monster_source: "MM".to_string(),
        quantity: 4,
        encounter_tag: Some("forest".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Bear".to_string(),
        monster_source: "MM".to_string(),
        quantity: 1,
        encounter_tag: Some("forest".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Bandit".to_string(),
        monster_source: "MM".to_string(),
        quantity: 3,
        encounter_tag: Some("road".to_string()),
    })
    .unwrap();

    let monsters = repo.list_by_module(module_id).unwrap();
    assert_eq!(monsters.len(), 3);
}

#[test]
fn test_list_monsters_grouped_by_encounter() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);

    // Create monsters in different encounters
    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Wolf".to_string(),
        monster_source: "MM".to_string(),
        quantity: 4,
        encounter_tag: Some("forest".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Bear".to_string(),
        monster_source: "MM".to_string(),
        quantity: 1,
        encounter_tag: Some("forest".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Bandit".to_string(),
        monster_source: "MM".to_string(),
        quantity: 3,
        encounter_tag: Some("road".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Rat".to_string(),
        monster_source: "MM".to_string(),
        quantity: 8,
        encounter_tag: None,
    })
    .unwrap();

    let groups = repo.list_by_module_grouped(module_id).unwrap();

    // Should have 3 groups: forest, road, and None
    assert_eq!(groups.len(), 3);

    // Check forest group has 2 monsters
    let forest_group = groups.iter().find(|(tag, _)| *tag == Some("forest".to_string()));
    assert!(forest_group.is_some());
    assert_eq!(forest_group.unwrap().1.len(), 2);
}

#[test]
fn test_find_monsters_by_encounter() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);

    // Create monsters in different encounters
    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Goblin".to_string(),
        monster_source: "MM".to_string(),
        quantity: 6,
        encounter_tag: Some("cave".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Bugbear".to_string(),
        monster_source: "MM".to_string(),
        quantity: 1,
        encounter_tag: Some("cave".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Bandit".to_string(),
        monster_source: "MM".to_string(),
        quantity: 3,
        encounter_tag: Some("camp".to_string()),
    })
    .unwrap();

    // Find cave encounter monsters
    let cave_monsters = repo.find_by_encounter(module_id, Some("cave")).unwrap();
    assert_eq!(cave_monsters.len(), 2);

    // Find monsters with no encounter tag
    let untagged = repo.find_by_encounter(module_id, None).unwrap();
    assert_eq!(untagged.len(), 0);
}

#[test]
fn test_find_existing_monster() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Dragon, Red".to_string(),
        monster_source: "MM".to_string(),
        quantity: 1,
        encounter_tag: Some("boss".to_string()),
    })
    .unwrap();

    // Should find existing
    let existing = repo
        .find_existing(module_id, "Dragon, Red", "MM", Some("boss"))
        .unwrap();
    assert!(existing.is_some());

    // Should not find with different encounter tag
    let not_found = repo
        .find_existing(module_id, "Dragon, Red", "MM", Some("miniboss"))
        .unwrap();
    assert!(not_found.is_none());

    // Should not find with different source
    let not_found = repo
        .find_existing(module_id, "Dragon, Red", "XGE", Some("boss"))
        .unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_delete_monsters_by_module() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);

    // Create multiple monsters
    for name in &["Kobold", "Goblin", "Orc"] {
        repo.create(NewModuleMonster {
            module_id,
            monster_name: name.to_string(),
            monster_source: "MM".to_string(),
            quantity: 2,
            encounter_tag: None,
        })
        .unwrap();
    }

    let monsters = repo.list_by_module(module_id).unwrap();
    assert_eq!(monsters.len(), 3);

    // Delete all
    let deleted = repo.delete_by_module(module_id).unwrap();
    assert_eq!(deleted, 3);

    let monsters = repo.list_by_module(module_id).unwrap();
    assert_eq!(monsters.len(), 0);
}

#[test]
fn test_get_encounter_tags() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleMonsterRepository::new(&mut conn);

    // Create monsters with various tags
    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Goblin".to_string(),
        monster_source: "MM".to_string(),
        quantity: 4,
        encounter_tag: Some("ambush".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Wolf".to_string(),
        monster_source: "MM".to_string(),
        quantity: 2,
        encounter_tag: Some("ambush".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Dragon".to_string(),
        monster_source: "MM".to_string(),
        quantity: 1,
        encounter_tag: Some("boss".to_string()),
    })
    .unwrap();

    repo.create(NewModuleMonster {
        module_id,
        monster_name: "Rat".to_string(),
        monster_source: "MM".to_string(),
        quantity: 6,
        encounter_tag: None,
    })
    .unwrap();

    let tags = repo.get_encounter_tags(module_id).unwrap();

    // Should have 3 distinct tags: ambush, boss, None
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&Some("ambush".to_string())));
    assert!(tags.contains(&Some("boss".to_string())));
    assert!(tags.contains(&None));
}

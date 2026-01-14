//! Integration tests for module NPC DAL

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::dal::campaign::module_npcs::ModuleNpcRepository;
use mimir_dm_core::dal::campaign::modules::ModuleRepository;
use mimir_dm_core::dal::character::CharacterRepository;
use mimir_dm_core::models::campaign::campaigns::NewCampaign;
use mimir_dm_core::models::campaign::module_npcs::{NewModuleNpc, UpdateModuleNpc};
use mimir_dm_core::models::campaign::modules::NewModule;
use mimir_dm_core::models::character::NewCharacter;
use tempfile::TempDir;

/// Helper to create test campaign, module, and NPC character
fn setup_campaign_module_and_npc(
    conn: &mut diesel::SqliteConnection,
    temp_dir: &TempDir,
) -> (i32, i32, i32) {
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

    let mut char_repo = CharacterRepository::new(conn);
    let npc = char_repo
        .create(NewCharacter {
            campaign_id: Some(campaign.id),
            player_id: None,
            character_name: "Test NPC".to_string(),
            is_npc: Some(true),
            directory_path: temp_dir.path().join("npcs").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap();

    (campaign.id, module.id, npc.id)
}

/// Helper to create additional NPC characters
fn create_npc(
    conn: &mut diesel::SqliteConnection,
    campaign_id: i32,
    name: &str,
    temp_dir: &TempDir,
) -> i32 {
    let mut char_repo = CharacterRepository::new(conn);
    char_repo
        .create(NewCharacter {
            campaign_id: Some(campaign_id),
            player_id: None,
            character_name: name.to_string(),
            is_npc: Some(true),
            directory_path: temp_dir.path().join("npcs").to_string_lossy().to_string(),
            class: None,
            race: None,
        })
        .unwrap()
        .id
}

#[test]
fn test_create_module_npc() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);
    let module_npc = repo
        .create(NewModuleNpc {
            module_id,
            character_id: npc_id,
            role: Some("Quest Giver".to_string()),
            encounter_tag: Some("town_square".to_string()),
            notes: Some("Gives the party their initial quest".to_string()),
        })
        .unwrap();

    assert_eq!(module_npc.module_id, module_id);
    assert_eq!(module_npc.character_id, npc_id);
    assert_eq!(module_npc.role, Some("Quest Giver".to_string()));
    assert_eq!(module_npc.encounter_tag, Some("town_square".to_string()));
    assert!(module_npc.notes.is_some());
}

#[test]
fn test_find_npc_by_id() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);
    let created = repo
        .create(NewModuleNpc {
            module_id,
            character_id: npc_id,
            role: Some("Merchant".to_string()),
            encounter_tag: None,
            notes: None,
        })
        .unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.role, Some("Merchant".to_string()));
}

#[test]
fn test_find_nonexistent_npc() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = ModuleNpcRepository::new(&mut conn);
    let result = repo.find_by_id(99999).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_update_module_npc() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);
    let created = repo
        .create(NewModuleNpc {
            module_id,
            character_id: npc_id,
            role: Some("Guard".to_string()),
            encounter_tag: None,
            notes: None,
        })
        .unwrap();

    let updated = repo
        .update(
            created.id,
            UpdateModuleNpc {
                role: Some(Some("Captain of the Guard".to_string())),
                encounter_tag: Some(Some("castle_gate".to_string())),
                notes: Some(Some("Promoted during session 3".to_string())),
            },
        )
        .unwrap();

    assert_eq!(updated.role, Some("Captain of the Guard".to_string()));
    assert_eq!(updated.encounter_tag, Some("castle_gate".to_string()));
    assert_eq!(updated.notes, Some("Promoted during session 3".to_string()));
}

#[test]
fn test_delete_module_npc() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);
    let created = repo
        .create(NewModuleNpc {
            module_id,
            character_id: npc_id,
            role: None,
            encounter_tag: None,
            notes: None,
        })
        .unwrap();

    repo.delete(created.id).unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_none());
}

#[test]
fn test_list_npcs_by_module() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (campaign_id, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    // Create additional NPCs
    let npc2_id = create_npc(&mut conn, campaign_id, "Blacksmith", &temp_dir);
    let npc3_id = create_npc(&mut conn, campaign_id, "Innkeeper", &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);

    // Link NPCs to module
    repo.create(NewModuleNpc {
        module_id,
        character_id: npc_id,
        role: Some("Quest Giver".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc2_id,
        role: Some("Merchant".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc3_id,
        role: Some("Information".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    let npcs = repo.list_by_module(module_id).unwrap();
    assert_eq!(npcs.len(), 3);
}

#[test]
fn test_list_npcs_grouped_by_role() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (campaign_id, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    // Create additional NPCs
    let npc2_id = create_npc(&mut conn, campaign_id, "Guard 1", &temp_dir);
    let npc3_id = create_npc(&mut conn, campaign_id, "Guard 2", &temp_dir);
    let npc4_id = create_npc(&mut conn, campaign_id, "Merchant", &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);

    // Link NPCs with different roles
    repo.create(NewModuleNpc {
        module_id,
        character_id: npc_id,
        role: Some("Quest Giver".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc2_id,
        role: Some("Guard".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc3_id,
        role: Some("Guard".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc4_id,
        role: None,
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    let groups = repo.list_by_module_grouped(module_id).unwrap();

    // Should have 3 groups: Guard, Quest Giver, None
    assert_eq!(groups.len(), 3);

    // Check Guard group has 2 NPCs
    let guard_group = groups
        .iter()
        .find(|(role, _)| *role == Some("Guard".to_string()));
    assert!(guard_group.is_some());
    assert_eq!(guard_group.unwrap().1.len(), 2);
}

#[test]
fn test_find_npcs_by_role() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (campaign_id, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    // Create additional NPCs
    let npc2_id = create_npc(&mut conn, campaign_id, "Scholar 1", &temp_dir);
    let npc3_id = create_npc(&mut conn, campaign_id, "Scholar 2", &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc_id,
        role: Some("Villain".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc2_id,
        role: Some("Scholar".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc3_id,
        role: Some("Scholar".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    // Find scholars
    let scholars = repo.find_by_role(module_id, Some("Scholar")).unwrap();
    assert_eq!(scholars.len(), 2);

    // Find NPCs with no role
    let no_role = repo.find_by_role(module_id, None).unwrap();
    assert_eq!(no_role.len(), 0);
}

#[test]
fn test_find_npcs_by_encounter_tag() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (campaign_id, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    // Create additional NPCs
    let npc2_id = create_npc(&mut conn, campaign_id, "Bandit Leader", &temp_dir);
    let npc3_id = create_npc(&mut conn, campaign_id, "Bandit", &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc_id,
        role: None,
        encounter_tag: Some("town".to_string()),
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc2_id,
        role: None,
        encounter_tag: Some("ambush".to_string()),
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc3_id,
        role: None,
        encounter_tag: Some("ambush".to_string()),
        notes: None,
    })
    .unwrap();

    // Find ambush NPCs
    let ambush_npcs = repo.find_by_encounter_tag(module_id, Some("ambush")).unwrap();
    assert_eq!(ambush_npcs.len(), 2);

    // Find NPCs with no encounter tag
    let no_tag = repo.find_by_encounter_tag(module_id, None).unwrap();
    assert_eq!(no_tag.len(), 0);
}

#[test]
fn test_find_existing_npc_link() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc_id,
        role: Some("Important".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    // Should find existing link
    let existing = repo.find_existing(module_id, npc_id).unwrap();
    assert!(existing.is_some());

    // Should not find non-existent link
    let not_found = repo.find_existing(module_id, 99999).unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_delete_npcs_by_module() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (campaign_id, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    // Create additional NPCs
    let npc2_id = create_npc(&mut conn, campaign_id, "NPC 2", &temp_dir);
    let npc3_id = create_npc(&mut conn, campaign_id, "NPC 3", &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);

    // Link multiple NPCs
    for id in &[npc_id, npc2_id, npc3_id] {
        repo.create(NewModuleNpc {
            module_id,
            character_id: *id,
            role: None,
            encounter_tag: None,
            notes: None,
        })
        .unwrap();
    }

    let npcs = repo.list_by_module(module_id).unwrap();
    assert_eq!(npcs.len(), 3);

    // Delete all
    let deleted = repo.delete_by_module(module_id).unwrap();
    assert_eq!(deleted, 3);

    let npcs = repo.list_by_module(module_id).unwrap();
    assert_eq!(npcs.len(), 0);
}

#[test]
fn test_get_roles() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (campaign_id, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    // Create additional NPCs
    let npc2_id = create_npc(&mut conn, campaign_id, "NPC 2", &temp_dir);
    let npc3_id = create_npc(&mut conn, campaign_id, "NPC 3", &temp_dir);
    let npc4_id = create_npc(&mut conn, campaign_id, "NPC 4", &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc_id,
        role: Some("Ally".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc2_id,
        role: Some("Ally".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc3_id,
        role: Some("Enemy".to_string()),
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc4_id,
        role: None,
        encounter_tag: None,
        notes: None,
    })
    .unwrap();

    let roles = repo.get_roles(module_id).unwrap();

    // Should have 3 distinct roles: Ally, Enemy, None
    assert_eq!(roles.len(), 3);
    assert!(roles.contains(&Some("Ally".to_string())));
    assert!(roles.contains(&Some("Enemy".to_string())));
    assert!(roles.contains(&None));
}

#[test]
fn test_get_encounter_tags() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (campaign_id, module_id, npc_id) = setup_campaign_module_and_npc(&mut conn, &temp_dir);

    // Create additional NPCs
    let npc2_id = create_npc(&mut conn, campaign_id, "NPC 2", &temp_dir);
    let npc3_id = create_npc(&mut conn, campaign_id, "NPC 3", &temp_dir);

    let mut repo = ModuleNpcRepository::new(&mut conn);

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc_id,
        role: None,
        encounter_tag: Some("scene_1".to_string()),
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc2_id,
        role: None,
        encounter_tag: Some("scene_1".to_string()),
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleNpc {
        module_id,
        character_id: npc3_id,
        role: None,
        encounter_tag: Some("scene_2".to_string()),
        notes: None,
    })
    .unwrap();

    let tags = repo.get_encounter_tags(module_id).unwrap();

    // Should have 2 distinct tags: scene_1, scene_2
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&Some("scene_1".to_string())));
    assert!(tags.contains(&Some("scene_2".to_string())));
}

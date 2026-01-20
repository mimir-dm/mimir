//! Integration tests for module item DAL

use crate::common::TestDatabase;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::dal::campaign::module_items::ModuleItemRepository;
use mimir_dm_core::dal::campaign::modules::ModuleRepository;
use mimir_dm_core::models::campaign::campaigns::NewCampaign;
use mimir_dm_core::models::campaign::module_items::{NewModuleItem, UpdateModuleItem};
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
fn test_create_module_item() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);
    let item = repo
        .create(NewModuleItem {
            module_id,
            location: Some("Treasure Chest".to_string()),
            name: "Longsword +1".to_string(),
            source: "DMG".to_string(),
            quantity: 1,
            notes: Some("Hidden compartment".to_string()),
        })
        .unwrap();

    assert_eq!(item.name, "Longsword +1");
    assert_eq!(item.source, "DMG");
    assert_eq!(item.quantity, 1);
    assert_eq!(item.location, Some("Treasure Chest".to_string()));
    assert_eq!(item.notes, Some("Hidden compartment".to_string()));
    assert_eq!(item.module_id, module_id);
}

#[test]
fn test_find_item_by_id() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);
    let created = repo
        .create(NewModuleItem {
            module_id,
            location: None,
            name: "Potion of Healing".to_string(),
            source: "PHB".to_string(),
            quantity: 3,
            notes: None,
        })
        .unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Potion of Healing");
}

#[test]
fn test_find_nonexistent_item() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");

    let mut repo = ModuleItemRepository::new(&mut conn);
    let result = repo.find_by_id(99999).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_update_module_item() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);
    let created = repo
        .create(NewModuleItem {
            module_id,
            location: Some("Room 1".to_string()),
            name: "Gold Coins".to_string(),
            source: "PHB".to_string(),
            quantity: 50,
            notes: None,
        })
        .unwrap();

    let updated = repo
        .update(
            created.id,
            UpdateModuleItem {
                location: Some(Some("Room 2".to_string())),
                quantity: Some(100),
                notes: Some(Some("Discovered in hidden alcove".to_string())),
            },
        )
        .unwrap();

    assert_eq!(updated.quantity, 100);
    assert_eq!(updated.location, Some("Room 2".to_string()));
    assert_eq!(
        updated.notes,
        Some("Discovered in hidden alcove".to_string())
    );
}

#[test]
fn test_delete_module_item() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);
    let created = repo
        .create(NewModuleItem {
            module_id,
            location: None,
            name: "Scroll of Fireball".to_string(),
            source: "PHB".to_string(),
            quantity: 1,
            notes: None,
        })
        .unwrap();

    repo.delete(created.id).unwrap();

    let found = repo.find_by_id(created.id).unwrap();
    assert!(found.is_none());
}

#[test]
fn test_list_items_by_module() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);

    // Create multiple items
    repo.create(NewModuleItem {
        module_id,
        location: Some("Altar".to_string()),
        name: "Holy Symbol".to_string(),
        source: "PHB".to_string(),
        quantity: 1,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: Some("Altar".to_string()),
        name: "Candles".to_string(),
        source: "PHB".to_string(),
        quantity: 6,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: Some("Treasury".to_string()),
        name: "Gems".to_string(),
        source: "PHB".to_string(),
        quantity: 10,
        notes: Some("Assorted gems".to_string()),
    })
    .unwrap();

    let items = repo.list_by_module(module_id).unwrap();
    assert_eq!(items.len(), 3);
}

#[test]
fn test_list_items_grouped_by_location() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);

    // Create items in different locations
    repo.create(NewModuleItem {
        module_id,
        location: Some("Chest A".to_string()),
        name: "Gold".to_string(),
        source: "PHB".to_string(),
        quantity: 100,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: Some("Chest A".to_string()),
        name: "Silver".to_string(),
        source: "PHB".to_string(),
        quantity: 500,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: Some("Chest B".to_string()),
        name: "Ring of Protection".to_string(),
        source: "DMG".to_string(),
        quantity: 1,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: None,
        name: "Misc Loot".to_string(),
        source: "PHB".to_string(),
        quantity: 1,
        notes: None,
    })
    .unwrap();

    let groups = repo.list_by_module_grouped(module_id).unwrap();

    // Should have 3 groups: Chest A, Chest B, and None
    assert_eq!(groups.len(), 3);

    // Check Chest A group has 2 items
    let chest_a = groups
        .iter()
        .find(|(loc, _)| *loc == Some("Chest A".to_string()));
    assert!(chest_a.is_some());
    assert_eq!(chest_a.unwrap().1.len(), 2);
}

#[test]
fn test_find_items_by_location() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);

    // Create items in different locations
    repo.create(NewModuleItem {
        module_id,
        location: Some("Armory".to_string()),
        name: "Longsword".to_string(),
        source: "PHB".to_string(),
        quantity: 4,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: Some("Armory".to_string()),
        name: "Shield".to_string(),
        source: "PHB".to_string(),
        quantity: 4,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: Some("Kitchen".to_string()),
        name: "Rations".to_string(),
        source: "PHB".to_string(),
        quantity: 20,
        notes: None,
    })
    .unwrap();

    // Find armory items
    let armory_items = repo.find_by_location(module_id, Some("Armory")).unwrap();
    assert_eq!(armory_items.len(), 2);

    // Find items with no location
    let unlocated = repo.find_by_location(module_id, None).unwrap();
    assert_eq!(unlocated.len(), 0);
}

#[test]
fn test_find_existing_item() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);

    repo.create(NewModuleItem {
        module_id,
        location: Some("Boss Room".to_string()),
        name: "Vorpal Sword".to_string(),
        source: "DMG".to_string(),
        quantity: 1,
        notes: None,
    })
    .unwrap();

    // Should find existing
    let existing = repo
        .find_existing(module_id, "Vorpal Sword", "DMG", Some("Boss Room"))
        .unwrap();
    assert!(existing.is_some());

    // Should not find with different location
    let not_found = repo
        .find_existing(module_id, "Vorpal Sword", "DMG", Some("Entrance"))
        .unwrap();
    assert!(not_found.is_none());

    // Should not find with different source
    let not_found = repo
        .find_existing(module_id, "Vorpal Sword", "XGE", Some("Boss Room"))
        .unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_delete_items_by_module() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);

    // Create multiple items
    for name in &["Torch", "Rope", "Lantern"] {
        repo.create(NewModuleItem {
            module_id,
            location: None,
            name: name.to_string(),
            source: "PHB".to_string(),
            quantity: 1,
            notes: None,
        })
        .unwrap();
    }

    let items = repo.list_by_module(module_id).unwrap();
    assert_eq!(items.len(), 3);

    // Delete all
    let deleted = repo.delete_by_module(module_id).unwrap();
    assert_eq!(deleted, 3);

    let items = repo.list_by_module(module_id).unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_get_locations() {
    let test_db = TestDatabase::file_based().expect("Failed to create test database");
    let mut conn = test_db.connection().expect("Failed to get database connection");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let (_, module_id) = setup_campaign_and_module(&mut conn, &temp_dir);

    let mut repo = ModuleItemRepository::new(&mut conn);

    // Create items with various locations
    repo.create(NewModuleItem {
        module_id,
        location: Some("Room 1".to_string()),
        name: "Item A".to_string(),
        source: "PHB".to_string(),
        quantity: 1,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: Some("Room 1".to_string()),
        name: "Item B".to_string(),
        source: "PHB".to_string(),
        quantity: 1,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: Some("Room 2".to_string()),
        name: "Item C".to_string(),
        source: "PHB".to_string(),
        quantity: 1,
        notes: None,
    })
    .unwrap();

    repo.create(NewModuleItem {
        module_id,
        location: None,
        name: "Item D".to_string(),
        source: "PHB".to_string(),
        quantity: 1,
        notes: None,
    })
    .unwrap();

    let locations = repo.get_locations(module_id).unwrap();

    // Should have 3 distinct locations: Room 1, Room 2, None
    assert_eq!(locations.len(), 3);
    assert!(locations.contains(&Some("Room 1".to_string())));
    assert!(locations.contains(&Some("Room 2".to_string())));
    assert!(locations.contains(&None));
}

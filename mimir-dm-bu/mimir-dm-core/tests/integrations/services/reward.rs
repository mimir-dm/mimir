//! Integration tests for RewardService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::RewardFilters;
use mimir_dm_core::services::RewardService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_reward_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_reward_data(conn: &mut SqliteConnection) {
    // Schema: name, reward_type, description, has_prerequisites, source, full_reward_json
    let rewards = vec![
        ("Blessing of Health", "Blessing", "Your Constitution score increases by 2, up to a maximum of 22.", 0, "DMG", r#"{"name":"Blessing of Health","source":"DMG","type":"Blessing","entries":["Your Constitution score increases by 2, up to a maximum of 22."]}"#),
        ("Blessing of Protection", "Blessing", "You gain a +1 bonus to AC and saving throws.", 0, "DMG", r#"{"name":"Blessing of Protection","source":"DMG","type":"Blessing","entries":["You gain a +1 bonus to AC and saving throws."]}"#),
        ("Blessing of Magic Resistance", "Blessing", "You have advantage on saving throws against spells and other magical effects.", 0, "DMG", r#"{"name":"Blessing of Magic Resistance","source":"DMG","type":"Blessing","entries":["You have advantage on saving throws against spells and other magical effects."]}"#),
        ("Blessing of Valhalla", "Blessing", "This blessing grants you a one-time benefit.", 0, "DMG", r#"{"name":"Blessing of Valhalla","source":"DMG","type":"Blessing","entries":["This blessing grants you a one-time benefit."]}"#),
        ("Blessing of Weapon Enhancement", "Blessing", "One nonmagical weapon becomes a +1 weapon.", 0, "DMG", r#"{"name":"Blessing of Weapon Enhancement","source":"DMG","type":"Blessing","entries":["One nonmagical weapon becomes a +1 weapon."]}"#),
        ("Boon of Combat Prowess", "Boon", "When you miss with a melee attack, you can choose to hit instead.", 1, "DMG", r#"{"name":"Boon of Combat Prowess","source":"DMG","type":"Boon","prerequisite":[{"level":20}],"entries":["When you miss with a melee attack, you can choose to hit instead."]}"#),
        ("Boon of Dimensional Travel", "Boon", "As an action, you can cast the misty step spell.", 1, "DMG", r#"{"name":"Boon of Dimensional Travel","source":"DMG","type":"Boon","prerequisite":[{"level":20}],"entries":["As an action, you can cast the misty step spell."]}"#),
        ("Boon of Fate", "Boon", "When another creature that you can see within 60 feet makes an ability check, you can roll a d10.", 1, "DMG", r#"{"name":"Boon of Fate","source":"DMG","type":"Boon","prerequisite":[{"level":20}],"entries":["When another creature that you can see within 60 feet makes an ability check, you can roll a d10."]}"#),
        ("Boon of Fortitude", "Boon", "Your hit point maximum increases by 40.", 1, "DMG", r#"{"name":"Boon of Fortitude","source":"DMG","type":"Boon","prerequisite":[{"level":20}],"entries":["Your hit point maximum increases by 40."]}"#),
        ("Boon of High Magic", "Boon", "You gain one 9th-level spell slot.", 1, "DMG", r#"{"name":"Boon of High Magic","source":"DMG","type":"Boon","prerequisite":[{"level":20}],"entries":["You gain one 9th-level spell slot."]}"#),
        ("Charm of Animal Conjuring", "Charm", "As an action, you can cast the conjure animals spell.", 0, "DMG", r#"{"name":"Charm of Animal Conjuring","source":"DMG","type":"Charm","entries":["As an action, you can cast the conjure animals spell."]}"#),
        ("Charm of Darkvision", "Charm", "You gain darkvision out to a range of 60 feet.", 0, "DMG", r#"{"name":"Charm of Darkvision","source":"DMG","type":"Charm","entries":["You gain darkvision out to a range of 60 feet."]}"#),
        ("Charm of Feather Falling", "Charm", "You gain the effect of a feather fall spell.", 0, "DMG", r#"{"name":"Charm of Feather Falling","source":"DMG","type":"Charm","entries":["You gain the effect of a feather fall spell."]}"#),
        ("Charm of Heroism", "Charm", "As an action, you can give yourself the benefit of a potion of heroism.", 0, "DMG", r#"{"name":"Charm of Heroism","source":"DMG","type":"Charm","entries":["As an action, you can give yourself the benefit of a potion of heroism."]}"#),
        ("Charm of Restoration", "Charm", "Once you receive this charm, you can cast the greater restoration spell.", 0, "DMG", r#"{"name":"Charm of Restoration","source":"DMG","type":"Charm","entries":["Once you receive this charm, you can cast the greater restoration spell."]}"#),
        ("Charm of the Slayer", "Charm", "One sword you own becomes a dragon slayer or giant slayer.", 0, "XGE", r#"{"name":"Charm of the Slayer","source":"XGE","type":"Charm","entries":["One sword you own becomes a dragon slayer or giant slayer."]}"#),
    ];

    for (name, reward_type, description, has_prerequisites, source, json) in rewards {
        diesel::sql_query(
            "INSERT INTO catalog_rewards (name, reward_type, description, has_prerequisites, source, full_reward_json) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(reward_type)
        .bind::<diesel::sql_types::Text, _>(description)
        .bind::<diesel::sql_types::Integer, _>(has_prerequisites)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_rewards_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: None,
        search: None,
        reward_types: None,
        sources: None,
        has_prerequisites: None,
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 16, "Should return all 16 seeded rewards");
}

#[test]
fn test_search_rewards_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: Some("Blessing of Health".to_string()),
        search: None,
        reward_types: None,
        sources: None,
        has_prerequisites: None,
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    assert_eq!(
        results.len(),
        1,
        "Should return 1 reward matching exact name"
    );
    assert_eq!(results[0].name, "Blessing of Health");
}

#[test]
fn test_search_rewards_by_search() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: None,
        search: Some("spell".to_string()),
        reward_types: None,
        sources: None,
        has_prerequisites: None,
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    // Multiple rewards mention "spell" in their descriptions
    assert!(
        results.len() >= 2,
        "Should return rewards mentioning 'spell'"
    );
}

#[test]
fn test_search_rewards_by_reward_type_blessing() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: None,
        search: None,
        reward_types: Some(vec!["Blessing".to_string()]),
        sources: None,
        has_prerequisites: None,
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 5, "Should return 5 blessings");
    assert!(results.iter().all(|r| r.reward_type == "Blessing"));
}

#[test]
fn test_search_rewards_by_reward_type_boon() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: None,
        search: None,
        reward_types: Some(vec!["Boon".to_string()]),
        sources: None,
        has_prerequisites: None,
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 5, "Should return 5 boons");
}

#[test]
fn test_search_rewards_by_reward_type_charm() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: None,
        search: None,
        reward_types: Some(vec!["Charm".to_string()]),
        sources: None,
        has_prerequisites: None,
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 6, "Should return 6 charms");
}

#[test]
fn test_search_rewards_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: None,
        search: None,
        reward_types: None,
        sources: Some(vec!["XGE".to_string()]),
        has_prerequisites: None,
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should return 1 XGE reward");
    assert_eq!(results[0].source, "XGE");
}

#[test]
fn test_search_rewards_by_has_prerequisites_true() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: None,
        search: None,
        reward_types: None,
        sources: None,
        has_prerequisites: Some(true),
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    // All 5 boons have prerequisites
    assert_eq!(
        results.len(),
        5,
        "Should return 5 rewards with prerequisites"
    );
    assert!(results.iter().all(|r| r.has_prerequisites));
}

#[test]
fn test_search_rewards_by_has_prerequisites_false() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: None,
        search: None,
        reward_types: None,
        sources: None,
        has_prerequisites: Some(false),
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    // 5 blessings + 6 charms = 11 rewards without prerequisites
    assert_eq!(
        results.len(),
        11,
        "Should return 11 rewards without prerequisites"
    );
    assert!(results.iter().all(|r| !r.has_prerequisites));
}

#[test]
fn test_search_rewards_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: None,
        search: None,
        reward_types: Some(vec!["Boon".to_string()]),
        sources: Some(vec!["DMG".to_string()]),
        has_prerequisites: Some(true),
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    // All 5 DMG boons have prerequisites
    assert_eq!(
        results.len(),
        5,
        "Should return 5 DMG boons with prerequisites"
    );
}

#[test]
fn test_search_rewards_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RewardFilters {
        name: Some("Nonexistent".to_string()),
        search: None,
        reward_types: None,
        sources: None,
        has_prerequisites: None,
    };
    let results = RewardService::search_rewards(&mut conn, filters).expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_reward_by_id() {
    let (mut conn, _temp_dir) = setup_test_db();

    // First reward in fresh DB should be ID 1
    let reward = RewardService::get_reward_by_id(&mut conn, 1).expect("Should get reward");

    assert!(reward.is_some());
}

#[test]
fn test_get_reward_by_id_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let reward = RewardService::get_reward_by_id(&mut conn, 99999).expect("Should not error");

    assert!(reward.is_none());
}

#[test]
fn test_get_reward_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let reward =
        RewardService::get_reward_by_name_and_source(&mut conn, "Blessing of Health", "DMG")
            .expect("Should get reward");

    assert!(reward.is_some());
    let reward = reward.unwrap();
    assert_eq!(reward.name, "Blessing of Health");
    assert_eq!(reward.source, "DMG");
}

#[test]
fn test_get_reward_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let reward = RewardService::get_reward_by_name_and_source(&mut conn, "Nonexistent", "DMG")
        .expect("Should not error");

    assert!(reward.is_none());
}

#[test]
fn test_get_reward_types() {
    let (mut conn, _temp_dir) = setup_test_db();

    let types = RewardService::get_reward_types(&mut conn).expect("Should get reward types");

    assert_eq!(types.len(), 3, "Should have 3 reward types");
    assert!(types.contains(&"Blessing".to_string()));
    assert!(types.contains(&"Boon".to_string()));
    assert!(types.contains(&"Charm".to_string()));
}

#[test]
fn test_get_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = RewardService::get_sources(&mut conn).expect("Should get sources");

    assert_eq!(sources.len(), 2, "Should have 2 sources");
    assert!(sources.contains(&"DMG".to_string()));
    assert!(sources.contains(&"XGE".to_string()));
}

#[test]
fn test_get_reward_count() {
    let (mut conn, _temp_dir) = setup_test_db();

    let count = RewardService::get_reward_count(&mut conn).expect("Should get count");

    assert_eq!(count, 16, "Should have 16 rewards");
}

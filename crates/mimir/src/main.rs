//! Mimir Application Entry Point
//!
//! Initializes the Tauri application with database connection and runs the event loop.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use mimir_core::db::init_database;
use mimir_lib::commands::{asset, campaign, catalog, character, document, map, module, source};
use mimir_lib::{AppPaths, AppState};
use tauri::Manager;
use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("mimir=info,mimir_core=info"))
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Get Tauri's app data directory
            let tauri_app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Initialize paths (creates directories, handles dev/prod separation)
            let paths = AppPaths::from_tauri_path(tauri_app_data_dir)
                .expect("Failed to initialize application paths");

            // Initialize database with migrations
            let conn = init_database(&paths.database_url())
                .expect("Failed to initialize database");

            // Create and manage app state
            let state = AppState::new(conn, paths);
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Campaign commands
            campaign::list_campaigns,
            campaign::list_archived_campaigns,
            campaign::get_campaign,
            campaign::create_campaign,
            campaign::update_campaign,
            campaign::archive_campaign,
            campaign::unarchive_campaign,
            campaign::delete_campaign,
            // Module commands
            module::list_modules,
            module::get_module,
            module::get_module_by_number,
            module::create_module,
            module::update_module,
            module::delete_module,
            // Character commands - list
            character::list_characters,
            character::list_pcs,
            character::list_npcs,
            // Character commands - CRUD
            character::get_character,
            character::create_pc,
            character::create_npc,
            character::update_character,
            character::delete_character,
            // Character commands - inventory
            character::get_character_inventory,
            character::get_equipped_items,
            character::get_attuned_items,
            character::add_inventory_item,
            character::remove_inventory_item,
            character::update_inventory_item,
            // Document commands - list
            document::list_campaign_documents,
            document::list_module_documents,
            // Document commands - CRUD
            document::get_document,
            document::create_document,
            document::update_document,
            document::delete_document,
            // Document commands - search
            document::search_documents,
            document::search_module_documents,
            // Map commands - list
            map::list_campaign_maps,
            map::list_campaign_level_maps,
            map::list_module_maps,
            // Map commands - CRUD
            map::get_map,
            map::create_map,
            map::update_map,
            map::delete_map,
            // Map commands - UVTT data
            map::read_map_uvtt,
            // Asset commands - list
            asset::list_campaign_assets,
            asset::list_module_assets,
            // Asset commands - CRUD
            asset::get_asset,
            asset::upload_asset,
            asset::delete_asset,
            // Asset commands - file data
            asset::read_asset_file,
            // Catalog commands - monsters
            catalog::search_monsters,
            catalog::get_monster,
            catalog::get_monster_by_name,
            catalog::list_monster_sources,
            catalog::count_monsters,
            // Catalog commands - spells
            catalog::search_spells,
            catalog::get_spell,
            catalog::get_spell_by_name,
            catalog::list_spell_sources,
            catalog::count_spells,
            // Catalog commands - items
            catalog::search_items,
            catalog::get_item,
            catalog::get_item_by_name,
            catalog::list_item_sources,
            catalog::count_items,
            // Catalog commands - races
            catalog::search_races,
            catalog::get_race,
            catalog::get_race_by_name,
            catalog::list_race_sources,
            catalog::count_races,
            // Catalog commands - backgrounds
            catalog::search_backgrounds,
            catalog::get_background,
            catalog::get_background_by_name,
            catalog::list_background_sources,
            catalog::count_backgrounds,
            // Catalog commands - classes
            catalog::search_classes,
            catalog::get_class,
            catalog::get_class_by_name,
            catalog::list_class_sources,
            catalog::count_classes,
            // Catalog commands - feats
            catalog::search_feats,
            catalog::get_feat,
            catalog::get_feat_by_name,
            catalog::list_feat_sources,
            catalog::count_feats,
            // Catalog commands - conditions
            catalog::search_conditions,
            catalog::get_condition,
            catalog::get_condition_by_name,
            catalog::list_condition_sources,
            catalog::count_conditions,
            // Catalog commands - languages
            catalog::search_languages,
            catalog::get_language,
            catalog::get_language_by_name,
            catalog::list_language_sources,
            catalog::count_languages,
            // Catalog commands - traps
            catalog::search_traps,
            catalog::get_trap,
            catalog::get_trap_by_name,
            catalog::list_trap_sources,
            catalog::count_traps,
            // Catalog commands - hazards
            catalog::search_hazards,
            catalog::get_hazard,
            catalog::get_hazard_by_name,
            catalog::list_hazard_sources,
            catalog::count_hazards,
            // Catalog commands - actions
            catalog::search_actions,
            catalog::get_action,
            catalog::get_action_by_name,
            catalog::list_action_sources,
            catalog::count_actions,
            // Catalog commands - deities
            catalog::search_deities,
            catalog::get_deity,
            catalog::get_deity_by_name,
            catalog::list_deity_sources,
            catalog::count_deities,
            // Catalog commands - optional features
            catalog::search_optional_features,
            catalog::get_optional_feature,
            catalog::get_optional_feature_by_name,
            catalog::list_optional_feature_sources,
            catalog::count_optional_features,
            // Catalog commands - tables
            catalog::search_tables,
            catalog::get_table,
            catalog::get_table_by_name,
            catalog::list_table_sources,
            catalog::count_tables,
            // Catalog commands - variant rules
            catalog::search_variant_rules,
            catalog::get_variant_rule,
            catalog::get_variant_rule_by_name,
            catalog::list_variant_rule_sources,
            catalog::count_variant_rules,
            // Catalog commands - vehicles
            catalog::search_vehicles,
            catalog::get_vehicle,
            catalog::get_vehicle_by_name,
            catalog::list_vehicle_sources,
            catalog::count_vehicles,
            // Catalog commands - cults
            catalog::search_cults,
            catalog::get_cult,
            catalog::get_cult_by_name,
            catalog::list_cult_sources,
            catalog::count_cults,
            // Catalog commands - psionics
            catalog::search_psionics,
            catalog::get_psionic,
            catalog::get_psionic_by_name,
            catalog::list_psionic_sources,
            catalog::count_psionics,
            // Catalog commands - rewards
            catalog::search_rewards,
            catalog::get_reward,
            catalog::get_reward_by_name,
            catalog::list_reward_sources,
            catalog::count_rewards,
            // Catalog commands - objects
            catalog::search_objects,
            catalog::get_object,
            catalog::get_object_by_name,
            catalog::list_object_sources,
            catalog::count_objects,
            // Source management commands
            source::list_catalog_sources,
            source::import_catalog_from_zip,
            source::import_catalog_images,
            source::set_source_enabled,
            source::delete_catalog_source,
            // Book content commands (Reading mode)
            source::list_library_books,
            source::get_book_content,
            source::serve_book_image,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Mimir application");
}

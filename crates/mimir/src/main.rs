//! Mimir Application Entry Point
//!
//! Initializes the Tauri application with database connection and runs the event loop.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use mimir_core::db::init_database;
use mimir_lib::commands::{archive, asset, campaign, catalog, character, dev, dm_map, document, homebrew, homebrew_monster, homebrew_spell, map, module, player_display, print, source};
use mimir_lib::{AppPaths, AppState};
use mimir_print::PrintState;
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
            let state = AppState::new(conn, paths.clone());
            app.manage(state);

            // Create and manage print state
            let print_state = PrintState::new(
                paths.app_dir.join("templates"),
                paths.assets_dir.clone(),
            );
            app.manage(print_state);

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
            campaign::list_campaign_sources,
            campaign::add_campaign_source,
            campaign::remove_campaign_source,
            campaign::set_campaign_sources,
            // Archive commands (campaign export/import)
            archive::export_campaign,
            archive::preview_archive,
            archive::import_campaign,
            // Homebrew item commands
            homebrew::list_homebrew_items,
            homebrew::get_homebrew_item,
            homebrew::get_homebrew_item_by_name,
            homebrew::create_homebrew_item,
            homebrew::update_homebrew_item,
            homebrew::delete_homebrew_item,
            // Homebrew monster commands
            homebrew_monster::list_homebrew_monsters,
            homebrew_monster::get_homebrew_monster,
            homebrew_monster::create_homebrew_monster,
            homebrew_monster::update_homebrew_monster,
            homebrew_monster::delete_homebrew_monster,
            // Homebrew spell commands
            homebrew_spell::list_homebrew_spells,
            homebrew_spell::get_homebrew_spell,
            homebrew_spell::create_homebrew_spell,
            homebrew_spell::update_homebrew_spell,
            homebrew_spell::delete_homebrew_spell,
            // Module commands
            module::list_modules,
            module::get_module,
            module::get_module_by_number,
            module::create_module,
            module::update_module,
            module::delete_module,
            module::reorder_module,
            // Module monster commands
            module::list_module_monsters_with_data,
            module::add_module_monster,
            module::update_module_monster,
            // Module NPC commands
            module::list_module_npcs,
            // Token commands
            module::list_tokens,
            module::list_token_summaries,
            module::create_token,
            module::update_token,
            module::update_token_position,
            module::update_token_vision,
            module::toggle_token_visibility,
            module::delete_token,
            module::serve_token_image,
            // Character commands - list
            character::list_characters,
            character::list_pcs,
            character::list_npcs,
            character::list_unassigned_pcs,
            // Character commands - CRUD
            character::get_character,
            character::create_pc,
            character::create_npc,
            character::update_character,
            character::delete_character,
            character::assign_character_to_campaign,
            // Character commands - level up
            character::level_up_character,
            // Character commands - inventory
            character::get_character_inventory,
            character::get_equipped_items,
            character::get_attuned_items,
            character::add_inventory_item,
            character::remove_inventory_item,
            character::update_inventory_item,
            // Character commands - sources
            character::list_character_sources,
            character::add_character_source,
            character::remove_character_source,
            character::set_character_sources,
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
            map::get_uvtt_map,
            map::serve_map_image,
            // Map commands - light sources
            map::list_light_sources,
            map::create_light_source,
            map::create_torch,
            map::create_lantern,
            map::update_light_source,
            map::move_light_source,
            map::toggle_light_source,
            map::delete_light_source,
            map::delete_all_light_sources,
            // Map commands - fog of war
            map::get_fog_state,
            map::toggle_fog,
            map::enable_fog,
            map::disable_fog,
            map::reveal_rect,
            map::reveal_circle,
            map::reveal_all,
            map::delete_revealed_area,
            map::reset_fog,
            // Map commands - traps
            map::list_map_traps,
            map::get_map_trap,
            map::create_map_trap,
            map::update_map_trap,
            map::move_map_trap,
            map::toggle_map_trap_visibility,
            map::trigger_map_trap,
            map::reset_map_trap,
            map::delete_map_trap,
            // Map commands - POIs (Points of Interest)
            map::list_map_pois,
            map::get_map_poi,
            map::create_map_poi,
            map::update_map_poi,
            map::move_map_poi,
            map::toggle_map_poi_visibility,
            map::delete_map_poi,
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
            catalog::get_spells_by_class,
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
            // Catalog commands - class features
            catalog::get_class_feature,
            catalog::list_class_features,
            // Catalog commands - subclasses
            catalog::get_subclass,
            catalog::get_subclass_by_name,
            catalog::list_subclasses_by_class,
            catalog::count_subclasses,
            // Catalog commands - subclass features
            catalog::get_subclass_feature,
            catalog::list_subclass_features,
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
            // Catalog commands - level-up helpers
            catalog::get_class_info,
            catalog::get_class_spellcasting,
            catalog::list_fighting_styles,
            catalog::list_metamagic,
            catalog::list_maneuvers,
            catalog::list_invocations,
            catalog::list_feats_with_prereqs,
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
            // Player display commands
            player_display::is_player_display_open,
            player_display::open_player_display_window,
            player_display::close_player_display_window,
            player_display::toggle_player_display_fullscreen,
            player_display::send_map_to_display,
            player_display::update_display_viewport,
            player_display::toggle_display_blackout,
            // DM map window commands
            dm_map::is_dm_map_open,
            dm_map::open_dm_map_window,
            dm_map::close_dm_map_window,
            dm_map::toggle_dm_map_fullscreen,
            // App info commands
            dev::get_app_info,
            // Dev tools commands (dev mode only)
            dev::is_dev_mode,
            dev::is_dev_seeded,
            dev::seed_dev_data,
            dev::reseed_dev_data,
            dev::clear_dev_data,
            // Print/PDF export commands
            print::list_print_templates,
            print::export_character,
            print::export_campaign_document,
            print::export_campaign_documents,
            print::export_module_documents,
            print::print_map,
            print::generate_character_sheet,
            print::save_pdf,
            print::export_module_monsters,
            print::export_monster_card,
            print::export_trap_card,
            print::export_trap_cards,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Mimir application");
}

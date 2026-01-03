#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_init;
mod commands;
mod embedded_test_book;
mod seed_templates;
mod services;
mod state;
mod types;

use app_init::initialize_app;
use commands::catalog::action::{
    get_action, get_action_count, get_action_sources, get_action_time_types, search_actions,
};
use commands::catalog::background::{
    get_background_count, get_background_details, get_background_sources, search_backgrounds,
};
use commands::catalog::condition::{
    get_condition, get_condition_count, get_condition_item_types, get_condition_sources,
    search_conditions,
};
use commands::catalog::feat::{get_feat_count, get_feat_details, get_feat_sources, search_feats};
use commands::catalog::language::{
    get_language_count, get_language_details, get_language_scripts, get_language_sources,
    get_language_types, search_languages,
};
use commands::catalog::psionic::{
    get_psionic_details, get_psionic_orders, get_psionic_sources, get_psionic_types,
    search_psionics,
};
use commands::catalog::reward::{
    get_reward_count, get_reward_details, get_reward_sources, get_reward_types, search_rewards,
};
use commands::catalog::vehicle::{
    get_vehicle_details, get_vehicle_sizes, get_vehicle_statistics, get_vehicle_terrains,
    get_vehicle_types, search_vehicles,
};
use commands::{system::logs, *};
use mimir_dm_core::{run_migrations, DatabaseService};
use services::context_service::ContextState;
use services::llm::{self, CancellationTokens, ConfirmationReceivers, LlmService};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tracing::{error, info, warn};

fn main() {
    // Initialize the application first
    let app_paths = match initialize_app() {
        Ok(paths) => {
            info!("Application initialized successfully");
            paths
        }
        Err(e) => {
            error!("Failed to initialize application: {}", e);
            std::process::exit(1);
        }
    };

    let is_new_db = app_paths.is_new_database();

    // Start Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            // Initialize database service from core
            let db_service = match DatabaseService::new(
                &app_paths.database_path_str(),
                app_paths.is_memory_db,
            ) {
                Ok(service) => {
                    info!("Database service initialized successfully");
                    service
                }
                Err(e) => {
                    error!("Failed to initialize database service: {}", e);
                    return Err(Box::new(std::io::Error::other(format!(
                        "Database initialization failed: {}",
                        e
                    ))));
                }
            };

            // Run migrations
            info!("Running database migrations...");
            let mut conn = match db_service.get_connection() {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get database connection for migrations: {}", e);
                    return Err(Box::new(std::io::Error::other(format!(
                        "Database connection failed: {}",
                        e
                    ))));
                }
            };
            match run_migrations(&mut conn) {
                Ok(_) => info!("Database migrations completed successfully"),
                Err(e) => warn!("Database migration warning: {}", e),
            }
            // Seed templates for new databases (reuse migration connection)
            if is_new_db {
                info!("Seeding initial templates...");
                if let Err(e) = seed_templates::seed_templates(&mut conn) {
                    warn!("Failed to seed templates: {}", e);
                }
            }

            // Seed dev data in debug builds
            #[cfg(debug_assertions)]
            {
                use directories::UserDirs;
                if let Some(user_dirs) = UserDirs::new() {
                    let campaigns_dir = user_dirs
                        .document_dir()
                        .unwrap_or_else(|| user_dirs.home_dir())
                        .join("Mimir Campaigns");

                    // Ensure campaigns directory exists
                    if let Err(e) = std::fs::create_dir_all(&campaigns_dir) {
                        warn!("Failed to create campaigns directory: {}", e);
                    } else {
                        let campaigns_path = campaigns_dir.to_string_lossy().to_string();
                        let data_path = app_paths.data_dir.to_string_lossy().to_string();
                        match mimir_dm_core::seed::seed_dev_data(&mut conn, &campaigns_path, &data_path) {
                            Ok(true) => info!("Dev seed data created successfully"),
                            Ok(false) => info!("Dev seed data already exists, skipped"),
                            Err(e) => warn!("Failed to seed dev data: {}", e),
                        }
                    }
                }
            }

            drop(conn); // Release connection after seeding

            // Build all state components
            let db_service = Arc::new(db_service);
            let app_paths_state = Arc::new(app_paths);
            let context_state = ContextState::new();

            let session_manager = commands::chat::chat_sessions::init_session_manager(
                &app_paths_state,
            )
            .map_err(|e| {
                error!("Failed to initialize session manager: {}", e);
                e
            })?;

            let confirmation_receivers: ConfirmationReceivers =
                Arc::new(tokio::sync::Mutex::new(HashMap::new()));
            let cancellation_tokens: CancellationTokens =
                Arc::new(tokio::sync::Mutex::new(HashMap::new()));
            let llm_service = Arc::new(tokio::sync::Mutex::new(None::<LlmService>));

            // Clone references needed for async LLM initialization
            let db_service_clone = Arc::clone(&db_service);
            let app_paths_clone = Arc::clone(&app_paths_state);
            let confirmation_receivers_clone = Arc::clone(&confirmation_receivers);
            let llm_service_clone = Arc::clone(&llm_service);

            // Create consolidated AppState
            let app_state = state::AppState::new(
                db_service,
                app_paths_state,
                context_state,
                session_manager,
                confirmation_receivers,
                cancellation_tokens,
                llm_service,
            );

            // Single state registration
            app.manage(app_state);

            // Spawn async task to initialize LLM
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                info!("Starting LLM service initialization...");
                match llm::initialize_llm(
                    app_handle,
                    db_service_clone,
                    confirmation_receivers_clone,
                    app_paths_clone,
                )
                .await
                {
                    Ok(service) => {
                        info!("LLM service initialized successfully");
                        let mut llm = llm_service_clone.lock().await;
                        *llm = Some(service);
                    }
                    Err(e) => {
                        error!("Failed to initialize LLM service: {}", e);
                        warn!("Application will continue without LLM functionality");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_info,
            get_default_campaigns_directory,
            list_campaigns,
            create_campaign,
            get_campaign,
            generate_campaign_document,
            list_templates,
            get_campaign_documents,
            get_documents_by_level,
            create_document,
            update_document,
            complete_document,
            delete_document,
            get_incomplete_documents,
            get_completed_documents,
            create_document_from_template,
            read_document_file,
            save_document_file,
            check_campaign_stage_completion,
            transition_campaign_stage,
            archive_campaign,
            unarchive_campaign,
            delete_campaign,
            list_archived_campaigns,
            get_campaign_summary,
            refresh_campaign_summary,
            initialize_stage_documents,
            get_board_configuration,
            get_next_stage,
            // Module commands
            create_module,
            get_module,
            list_campaign_modules,
            update_module,
            transition_module_stage,
            initialize_module_documents,
            get_module_documents,
            check_module_completion,
            find_modules_needing_next,
            increment_module_sessions,
            delete_module,
            // Module monster commands
            add_module_monster,
            remove_module_monster,
            update_module_monster,
            list_module_monsters,
            list_module_monsters_with_data,
            list_module_monsters_by_encounter,
            get_module_encounter_tags,
            clear_module_monsters,
            sync_module_monsters_to_file,
            // Map commands
            upload_map,
            get_map,
            list_maps,
            list_map_summaries,
            update_map_grid,
            delete_map,
            serve_map_image,
            get_uvtt_map,
            // Token commands
            create_token,
            get_token,
            list_tokens,
            list_visible_tokens,
            list_token_summaries,
            update_token,
            update_token_position,
            bulk_update_token_positions,
            toggle_token_visibility,
            set_token_visibility,
            delete_token,
            delete_tokens_for_map,
            serve_token_image,
            // Fog of war commands
            toggle_fog,
            enable_fog,
            disable_fog,
            get_fog_state,
            reveal_rect,
            reveal_circle,
            reveal_all,
            get_revealed_areas,
            delete_revealed_area,
            reset_fog,
            // Light source commands
            create_light_source,
            create_torch,
            create_lantern,
            get_light_source,
            list_light_sources,
            list_active_light_sources,
            update_light_source,
            move_light_source,
            toggle_light_source,
            delete_light_source,
            delete_all_light_sources,
            // Display control commands
            send_map_to_display,
            update_display_viewport,
            toggle_display_blackout,
            is_player_display_open,
            // Book library commands
            upload_book_archive,
            list_library_books,
            remove_book_from_library,
            get_book_content,
            serve_book_image,
            lookup_reference,
            // Dev tools
            is_dev_mode,
            remove_dev_test_book,
            // Catalog commands
            search_spells,
            get_spell_details,
            get_spell_sources,
            get_spell_schools,
            get_spell_statistics,
            get_spell_count,
            // Action catalog commands
            search_actions,
            get_action,
            get_action_time_types,
            get_action_sources,
            get_action_count,
            // Feat catalog commands
            search_feats,
            get_feat_details,
            get_feat_sources,
            get_feat_count,
            // Race catalog commands
            search_races,
            get_race_details,
            get_race_sources,
            get_race_count,
            get_race_sizes,
            // Background catalog commands
            search_backgrounds,
            get_background_details,
            get_background_sources,
            get_background_count,
            // Condition catalog commands
            search_conditions,
            get_condition,
            get_condition_item_types,
            get_condition_sources,
            get_condition_count,
            // Optional feature catalog commands (database-backed)
            search_optional_features,
            get_optional_feature,
            get_optional_feature_details,
            get_optional_feature_types,
            get_optional_feature_sources,
            // Item catalog commands (database-backed)
            search_items,
            get_item,
            get_item_details,
            get_item_types,
            get_item_rarities,
            get_item_sources,
            // Monster catalog commands (database-backed)
            search_monsters,
            get_monster_details,
            get_monster_sizes,
            get_monster_types,
            get_monster_alignments,
            get_monster_cr_range,
            get_monster_statistics,
            // Object catalog commands
            search_objects,
            get_object_details,
            get_object_sources,
            get_object_count,
            get_object_types,
            get_object_sizes,
            // Trap catalog commands
            search_traps,
            get_trap_details,
            get_trap_sources,
            get_trap_count,
            get_trap_types,
            get_trap_categories,
            // Cult catalog commands
            search_cults,
            get_cult_details,
            get_cult_sources,
            get_cult_count,
            get_cult_types,
            get_cult_categories,
            // Language catalog commands
            search_languages,
            get_language_details,
            get_language_types,
            get_language_scripts,
            get_language_sources,
            get_language_count,
            // Reward catalog commands
            search_rewards,
            get_reward_details,
            get_reward_types,
            get_reward_sources,
            get_reward_count,
            // Variant Rule catalog commands
            commands::catalog::variant_rule::search_variant_rules,
            commands::catalog::variant_rule::get_variant_rule,
            commands::catalog::variant_rule::get_variant_rule_details,
            commands::catalog::variant_rule::get_variant_rule_types,
            commands::catalog::variant_rule::get_variant_rule_sources,
            // Psionic catalog commands
            search_psionics,
            get_psionic_details,
            get_psionic_types,
            get_psionic_orders,
            get_psionic_sources,
            // Deity catalog commands (database-backed)
            search_deities,
            get_deity_details,
            get_deity_pantheons,
            get_deity_domains,
            get_deity_alignments,
            get_deity_statistics,
            // Vehicle catalog commands (database-backed)
            search_vehicles,
            get_vehicle_details,
            get_vehicle_types,
            get_vehicle_sizes,
            get_vehicle_terrains,
            get_vehicle_statistics,
            // Class catalog commands
            commands::catalog::class::search_classes,
            commands::catalog::class::get_class_details,
            commands::catalog::class::get_subclass_details,
            commands::catalog::class::get_class_subclasses,
            commands::catalog::class::get_class_sources,
            commands::catalog::class::get_class_primary_abilities,
            commands::catalog::class::get_class_statistics,
            // Table catalog commands
            commands::catalog::table::search_tables,
            commands::catalog::table::get_table,
            commands::catalog::table::get_table_details,
            commands::catalog::table::get_table_categories,
            commands::catalog::table::get_table_sources,
            // Context commands
            update_context,
            get_full_context,
            register_window,
            unregister_window,
            clear_shared_context,
            get_context_for_llm,
            update_context_usage,
            // Window management commands
            open_context_debug_window,
            open_chat_window,
            open_log_viewer_window,
            open_player_display_window,
            close_player_display_window,
            toggle_player_display_fullscreen,
            // LLM commands
            llm::commands::check_llm_status,
            llm::commands::get_llm_model_info,
            llm::commands::send_chat_message,
            llm::commands::cancel_chat_message,
            llm::commands::get_model_context_info,
            llm::commands::confirm_tool_action,
            llm::commands::list_available_models,
            llm::commands::get_provider_settings,
            llm::commands::save_provider_settings,
            llm::commands::reload_llm_service,
            // Chat session commands
            list_chat_sessions,
            load_chat_session,
            save_chat_session,
            create_chat_session,
            delete_chat_session,
            // Session todo commands
            get_session_todos,
            configure_todo_storage,
            // Log management commands
            logs::list_log_files,
            logs::read_log_file,
            logs::tail_log_file,
            logs::open_logs_folder,
            // Player management commands
            create_player,
            get_player,
            list_players,
            update_player,
            delete_player,
            // Character management commands
            create_minimal_character,
            create_character,
            store_character,
            get_character,
            get_character_spell_slots,
            list_all_characters,
            list_characters_for_campaign,
            list_npcs_for_campaign,
            list_pcs_for_campaign,
            create_npc,
            get_character_versions,
            get_character_version,
            update_character,
            delete_character,
            assign_character_to_campaign,
            level_up_character,
            add_spell_to_known,
            prepare_spells,
            cast_spell,
            take_rest,
            add_item_to_inventory,
            remove_item_from_inventory,
            update_character_currency,
            update_character_equipped,
            render_character_sheet,
            write_text_file,
            get_feature_details,
            // Print commands
            list_print_templates,
            generate_character_sheet,
            export_character,
            save_pdf,
            print_map,
            export_campaign_document,
            export_campaign_documents,
            export_module_documents
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            error!("Failed to run Tauri application: {}", e);
            std::process::exit(1);
        });
}

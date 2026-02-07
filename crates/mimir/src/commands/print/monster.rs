//! Monster Card Export Commands
//!
//! Tauri commands for exporting monster cards to PDF.

use base64::Engine;
use mimir_core::dal::campaign as dal;
use mimir_core::dal::catalog as catalog_dal;
use mimir_print::sections::MonsterCardSection;
use mimir_print::{DocumentBuilder, PrintState};
use serde::Deserialize;
use serde_json::Value;
use tauri::State;
use tracing::{error, info};

use crate::state::AppState;

use super::{ApiResponse, PrintResult};

/// Export options for monster cards
#[derive(Debug, Deserialize, Default)]
pub struct MonsterExportOptions {
    pub show_cut_lines: Option<bool>,
}

/// Export module monsters as cards to PDF
#[tauri::command]
pub fn export_module_monsters(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    module_id: String,
    options: Option<MonsterExportOptions>,
) -> ApiResponse<PrintResult> {
    info!("=== export_module_monsters called ===");
    info!("  module_id: {}", module_id);

    let opts = options.unwrap_or_default();

    // Log options
    info!("=== Options ===");
    info!("  show_cut_lines: {:?}", opts.show_cut_lines);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the module
    use mimir_core::services::ModuleService;
    let module = match ModuleService::new(&mut db).get(&module_id) {
        Ok(Some(m)) => m,
        Ok(None) => return ApiResponse::err(format!("Module not found: {}", module_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get module: {}", e)),
    };

    // Get module monsters
    let module_monsters = match dal::list_module_monsters(&mut db, &module_id) {
        Ok(monsters) => monsters,
        Err(e) => return ApiResponse::err(format!("Failed to list module monsters: {}", e)),
    };

    info!("=== Module Data ===");
    info!("  module_name: {}", module.name);
    info!("  module_monsters_count: {}", module_monsters.len());

    if module_monsters.is_empty() {
        error!("No monsters found for this module");
        return ApiResponse::err("No monsters found for this module");
    }

    for mm in &module_monsters {
        info!(
            "  - {} ({}) x{}",
            mm.monster_name, mm.monster_source, mm.quantity
        );
    }

    // Look up each monster's full data from the catalog
    let mut monster_data: Vec<Value> = Vec::new();
    for mm in &module_monsters {
        match catalog_dal::get_monster_by_name(&mut db, &mm.monster_name, &mm.monster_source) {
            Ok(Some(catalog_monster)) => {
                // Parse the monster's data JSON
                match catalog_monster.parse_data() {
                    Ok(mut data) => {
                        // Apply display name override if set
                        if let Some(ref display_name) = mm.display_name {
                            if let Some(obj) = data.as_object_mut() {
                                obj.insert(
                                    "name".to_string(),
                                    Value::String(display_name.clone()),
                                );
                            }
                        }
                        // Add quantity copies if > 1
                        for _ in 0..mm.quantity {
                            monster_data.push(data.clone());
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to parse monster data for {}: {}",
                            mm.monster_name, e
                        );
                    }
                }
            }
            Ok(None) => {
                error!(
                    "Catalog monster not found: {} ({})",
                    mm.monster_name, mm.monster_source
                );
            }
            Err(e) => {
                error!("Failed to look up monster {}: {}", mm.monster_name, e);
            }
        }
    }

    if monster_data.is_empty() {
        error!("No valid monster data found for this module");
        return ApiResponse::err("No valid monster data found for this module");
    }

    info!("=== Section ===");
    info!(
        "[SECTION] Adding MonsterCardSection with {} monsters",
        monster_data.len()
    );

    // Create monster cards section
    let mut section = MonsterCardSection::new(monster_data);
    if let Some(show_cut) = opts.show_cut_lines {
        section = section.with_cut_lines(show_cut);
    }

    // Build PDF
    info!("=== Building PDF ===");
    let title = format!("{} - Monster Cards", module.name);
    let pdf_result = DocumentBuilder::new(&title)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false)
        .append(section)
        .to_pdf();

    match pdf_result {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Monster cards PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate monster cards PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

/// Export a single monster as a card to PDF
#[tauri::command]
pub fn export_monster_card(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    monster_name: String,
    monster_source: String,
    options: Option<MonsterExportOptions>,
) -> ApiResponse<PrintResult> {
    info!("=== export_monster_card called ===");
    info!("  monster_name: {}", monster_name);
    info!("  monster_source: {}", monster_source);

    let opts = options.unwrap_or_default();

    // Log options
    info!("=== Options ===");
    info!("  show_cut_lines: {:?}", opts.show_cut_lines);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Look up the monster from catalog
    let catalog_monster =
        match catalog_dal::get_monster_by_name(&mut db, &monster_name, &monster_source) {
            Ok(Some(m)) => m,
            Ok(None) => {
                return ApiResponse::err(format!(
                    "Monster not found: {} ({})",
                    monster_name, monster_source
                ))
            }
            Err(e) => return ApiResponse::err(format!("Failed to look up monster: {}", e)),
        };

    // Parse monster data
    let monster_data = match catalog_monster.parse_data() {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse monster data: {}", e);
            return ApiResponse::err(format!("Failed to parse monster data: {}", e));
        }
    };

    info!("=== Section ===");
    info!("[SECTION] Adding MonsterCardSection for single monster");

    // Create monster cards section with single monster
    let mut section = MonsterCardSection::new(vec![monster_data]);
    if let Some(show_cut) = opts.show_cut_lines {
        section = section.with_cut_lines(show_cut);
    }

    // Build PDF
    info!("=== Building PDF ===");
    let title = format!("{} - Monster Card", monster_name);
    let pdf_result = DocumentBuilder::new(&title)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false)
        .append(section)
        .to_pdf();

    match pdf_result {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Monster card PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate monster card PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

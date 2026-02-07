//! Trap Card Export Commands
//!
//! Tauri commands for exporting trap cards to PDF.

use base64::Engine;
use mimir_core::dal::catalog as catalog_dal;
use mimir_print::sections::TrapCardSection;
use mimir_print::{DocumentBuilder, PrintState};
use serde::Deserialize;
use serde_json::Value;
use tauri::State;
use tracing::{error, info};

use crate::state::AppState;

use super::{ApiResponse, PrintResult};

/// Export options for trap cards
#[derive(Debug, Deserialize, Default)]
pub struct TrapExportOptions {
    pub show_cut_lines: Option<bool>,
}

/// Export a single trap as a card to PDF
#[tauri::command]
pub fn export_trap_card(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    trap_name: String,
    trap_source: String,
    options: Option<TrapExportOptions>,
) -> ApiResponse<PrintResult> {
    info!("=== export_trap_card called ===");
    info!("  trap_name: {}", trap_name);
    info!("  trap_source: {}", trap_source);

    let opts = options.unwrap_or_default();

    // Log options
    info!("=== Options ===");
    info!("  show_cut_lines: {:?}", opts.show_cut_lines);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Look up the trap from catalog
    let catalog_trap = match catalog_dal::get_trap_by_name(&mut db, &trap_name, &trap_source) {
        Ok(Some(t)) => t,
        Ok(None) => {
            return ApiResponse::err(format!(
                "Trap not found: {} ({})",
                trap_name, trap_source
            ))
        }
        Err(e) => return ApiResponse::err(format!("Failed to look up trap: {}", e)),
    };

    // Parse trap data
    let trap_data = match catalog_trap.parse_data() {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse trap data: {}", e);
            return ApiResponse::err(format!("Failed to parse trap data: {}", e));
        }
    };

    info!("=== Section ===");
    info!("[SECTION] Adding TrapCardSection for single trap");

    // Create trap cards section with single trap
    let mut section = TrapCardSection::new(vec![trap_data]);
    if let Some(show_cut) = opts.show_cut_lines {
        section = section.with_cut_lines(show_cut);
    }

    // Build PDF
    info!("=== Building PDF ===");
    let title = format!("{} - Trap Card", trap_name);
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
                "Trap card PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate trap card PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

/// Export multiple traps as cards to PDF
#[tauri::command]
pub fn export_trap_cards(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    traps: Vec<(String, String)>, // Vec of (name, source) tuples
    options: Option<TrapExportOptions>,
) -> ApiResponse<PrintResult> {
    info!("=== export_trap_cards called ===");
    info!("  traps_count: {}", traps.len());

    let opts = options.unwrap_or_default();

    // Log options
    info!("=== Options ===");
    info!("  show_cut_lines: {:?}", opts.show_cut_lines);

    if traps.is_empty() {
        error!("No traps specified");
        return ApiResponse::err("No traps specified");
    }

    // Log requested traps
    info!("=== Requested Traps ===");
    for (name, source) in &traps {
        info!("  - {} ({})", name, source);
    }

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Look up each trap from catalog
    let mut trap_data: Vec<Value> = Vec::new();
    for (name, source) in &traps {
        match catalog_dal::get_trap_by_name(&mut db, name, source) {
            Ok(Some(catalog_trap)) => match catalog_trap.parse_data() {
                Ok(data) => {
                    trap_data.push(data);
                }
                Err(e) => {
                    error!("Failed to parse trap data for {}: {}", name, e);
                }
            },
            Ok(None) => {
                error!("Catalog trap not found: {} ({})", name, source);
            }
            Err(e) => {
                error!("Failed to look up trap {}: {}", name, e);
            }
        }
    }

    if trap_data.is_empty() {
        error!("No valid trap data found");
        return ApiResponse::err("No valid trap data found");
    }

    info!("=== Section ===");
    info!(
        "[SECTION] Adding TrapCardSection with {} traps",
        trap_data.len()
    );

    // Create trap cards section
    let mut section = TrapCardSection::new(trap_data);
    if let Some(show_cut) = opts.show_cut_lines {
        section = section.with_cut_lines(show_cut);
    }

    // Build PDF
    info!("=== Building PDF ===");
    let pdf_result = DocumentBuilder::new("Trap Cards")
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
                "Trap cards PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate trap cards PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

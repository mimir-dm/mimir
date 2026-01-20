//! Board configuration commands.
//!
//! Provides Tauri commands for retrieving board configuration data including
//! stages, transitions, and stage metadata for the workflow board UI.

use crate::types::ApiResponse;
use mimir_dm_core::domain::BoardRegistry;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct BoardConfiguration {
    pub board_type: String,
    pub stages: Vec<StageInfo>,
    pub transitions: Vec<TransitionRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StageInfo {
    pub key: String,
    pub display_name: String,
    pub description: String,
    pub required_documents: Vec<String>,
    pub optional_documents: Vec<String>,
    pub no_completion_required_documents: Vec<String>,
    pub completion_message: Option<String>,
    pub transition_prompt: Option<String>,
    pub help_text: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitionRule {
    pub from: String,
    pub to: String,
    pub allowed: bool,
}

/// Get the complete configuration for a specific board type.
///
/// Returns all stages, their metadata, document requirements, and valid transitions.
/// Used to render the workflow board UI and determine available actions.
///
/// # Parameters
/// - `board_type` - The board type identifier (e.g., "session", "campaign")
///
/// # Returns
/// `ApiResponse` containing `BoardConfiguration` with stages and transitions.
///
/// # Errors
/// Returns error response if the board type is not found.
#[tauri::command]
pub async fn get_board_configuration(
    board_type: String,
) -> Result<ApiResponse<BoardConfiguration>, String> {
    info!("Getting configuration for board type: {}", board_type);

    let board_registry = BoardRegistry::new();
    let board = match board_registry.get(&board_type) {
        Some(b) => b,
        None => {
            error!("Board type '{}' not found", board_type);
            return Ok(ApiResponse::error(format!(
                "Board type '{}' not found",
                board_type
            )));
        }
    };

    // Get all stages and their metadata
    let stages: Vec<StageInfo> = board
        .stages()
        .into_iter()
        .map(|stage| {
            let metadata = board.stage_metadata(stage);
            StageInfo {
                key: stage.to_string(),
                display_name: metadata.display_name,
                description: metadata.description,
                required_documents: board
                    .required_documents(stage)
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                optional_documents: board
                    .optional_documents(stage)
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                no_completion_required_documents: board
                    .no_completion_required_documents(stage)
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                completion_message: metadata.completion_message,
                transition_prompt: metadata.transition_prompt,
                help_text: metadata.help_text,
                content: metadata.content,
            }
        })
        .collect();

    // Generate all possible transition rules
    let mut transitions = Vec::new();
    for from_stage in board.stages() {
        for to_stage in board.stages() {
            if from_stage != to_stage {
                transitions.push(TransitionRule {
                    from: from_stage.to_string(),
                    to: to_stage.to_string(),
                    allowed: board.can_transition(from_stage, to_stage),
                });
            }
        }
    }

    let configuration = BoardConfiguration {
        board_type: board.board_type().to_string(),
        stages,
        transitions,
    };

    info!(
        "Returning configuration with {} stages",
        configuration.stages.len()
    );
    Ok(ApiResponse::success(configuration))
}

/// Get the next valid stage for a given board and current stage.
///
/// Returns the default next stage in the workflow, if one exists.
/// Used to determine the primary advancement path.
///
/// # Parameters
/// - `board_type` - The board type identifier
/// - `current_stage` - The current stage key
///
/// # Returns
/// `ApiResponse` containing the next stage key, or `None` if at the end.
///
/// # Errors
/// Returns error response if the board type is not found.
#[tauri::command]
pub async fn get_next_stage(
    board_type: String,
    current_stage: String,
) -> Result<ApiResponse<Option<String>>, String> {
    info!(
        "Getting next stage for board '{}' from stage '{}'",
        board_type, current_stage
    );

    let board_registry = BoardRegistry::new();
    let board = match board_registry.get(&board_type) {
        Some(b) => b,
        None => {
            error!("Board type '{}' not found", board_type);
            return Ok(ApiResponse::error(format!(
                "Board type '{}' not found",
                board_type
            )));
        }
    };

    let next_stage = board.next_stage(&current_stage).map(|s| s.to_string());

    info!("Next stage: {:?}", next_stage);
    Ok(ApiResponse::success(next_stage))
}

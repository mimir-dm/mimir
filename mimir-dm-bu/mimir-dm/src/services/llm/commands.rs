//! Tauri command handlers for LLM service
//!
//! This module contains all Tauri commands that expose LLM functionality
//! to the frontend application.

use crate::services::app_settings::AppSettings;
use crate::services::llm::chat_processor::ChatProcessor;
use crate::services::provider_settings::ProviderSettings;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Chat message structure for Tauri commands
#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    /// Tool call ID - required for messages with role="tool" (tool results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Chat response with token usage
#[derive(Clone, Serialize, Deserialize)]
pub struct ChatResponseWithUsage {
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Tauri command to check LLM status
#[tauri::command]
pub async fn check_llm_status(state: State<'_, AppState>) -> Result<bool, String> {
    let service = state.llm.lock().await;

    if let Some(llm) = service.as_ref() {
        llm.check_service().await.map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

/// Tauri command to get model info
#[tauri::command]
pub async fn get_llm_model_info(state: State<'_, AppState>) -> Result<String, String> {
    let service = state.llm.lock().await;

    if let Some(llm) = service.as_ref() {
        Ok(llm.model_name().to_string())
    } else {
        Err("LLM service not initialized".to_string())
    }
}

/// Tauri command to send a chat message (with optional tool support)
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn send_chat_message(
    state: State<'_, AppState>,
    messages: Vec<ChatMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    enable_tools: Option<bool>,
    session_id: Option<String>,
    _model_name: Option<String>,
    ollama_url: Option<String>,
    campaign_directory_path: Option<String>,
    campaign_id: Option<i32>,
) -> Result<ChatResponseWithUsage, String> {
    let service = state.llm.lock().await;

    let llm = service
        .as_ref()
        .ok_or_else(|| "LLM service not initialized".to_string())?;

    // Require a valid session ID for all chat messages
    let session_id =
        session_id.ok_or_else(|| "Session ID is required for chat messages".to_string())?;

    // Create and register cancellation token for this session
    let cancellation_token = CancellationToken::new();

    {
        let mut tokens = state.cancellations.lock().await;
        tokens.insert(session_id.clone(), cancellation_token.clone());
    }

    // Convert to provider messages
    let provider_messages: Vec<mimir_dm_llm::Message> = messages
        .into_iter()
        .map(|msg| mimir_dm_llm::Message {
            role: msg.role,
            content: msg.content,
            tool_call_id: msg.tool_call_id,
        })
        .collect();

    // Use the ChatProcessor to handle the complex message processing
    let processor = ChatProcessor::new(llm);
    let result = processor
        .process_chat(
            provider_messages,
            max_tokens,
            temperature,
            enable_tools.unwrap_or(false),
            &session_id,
            ollama_url.as_deref(),
            campaign_directory_path.as_deref(),
            campaign_id,
            cancellation_token,
        )
        .await;

    // Clean up cancellation token
    {
        let mut tokens = state.cancellations.lock().await;
        tokens.remove(&session_id);
    }

    // Convert result
    result.map(|response| ChatResponseWithUsage {
        content: response.content,
        prompt_tokens: response.prompt_tokens,
        completion_tokens: response.completion_tokens,
        total_tokens: response.total_tokens,
    })
}

/// Tauri command to get model context info
#[tauri::command]
pub async fn get_model_context_info(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let service = state.llm.lock().await;

    let llm = service
        .as_ref()
        .ok_or_else(|| "LLM service not initialized".to_string())?;

    // For now, return hardcoded info for gpt-oss:20b
    // In the future, we could query this from Ollama
    Ok(serde_json::json!({
        "model": llm.model_name(),
        "context_length": 262144,  // From our curl query
        "default_max_tokens": 16384, // Increased for thinking models (thinking section gets discarded)
        "architecture": "qwen3moe"
    }))
}

/// Tauri command to confirm or reject a tool action
#[tauri::command]
pub async fn confirm_tool_action(
    state: State<'_, AppState>,
    confirmation_id: String,
    confirmed: bool,
) -> Result<(), String> {
    info!(
        "Received confirmation request: ID={}, confirmed={}",
        confirmation_id, confirmed
    );

    let id =
        Uuid::parse_str(&confirmation_id).map_err(|e| format!("Invalid confirmation ID: {}", e))?;

    // Find and remove the sender for this confirmation
    let sender = {
        let mut receivers = state.confirmations.lock().await;
        info!("Current receivers in map: {}", receivers.len());
        for (key, _) in receivers.iter() {
            info!("  - Receiver ID: {}", key);
        }
        receivers.remove(&id)
    };

    if let Some(tx) = sender {
        // Send the response back to the waiting tool execution
        tx.send(confirmed)
            .map_err(|_| "Failed to send confirmation - receiver dropped".to_string())?;
        info!(
            "Confirmation {} sent successfully: {}",
            confirmation_id, confirmed
        );
    } else {
        error!(
            "Confirmation ID {} not found in receivers map",
            confirmation_id
        );
        return Err("Confirmation ID not found or already processed".to_string());
    }

    Ok(())
}

/// Tauri command to list available models from the current provider
#[tauri::command]
pub async fn list_available_models(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    use mimir_dm_llm::LlmProvider;

    let service = state.llm.lock().await;

    let llm = service
        .as_ref()
        .ok_or_else(|| "LLM service not initialized".to_string())?;

    // Use the provider's list_models method
    let models = llm
        .provider
        .list_models()
        .await
        .map_err(|e| format!("Failed to list models: {}", e))?;

    // Convert ModelInfo to JSON values
    let model_list: Vec<serde_json::Value> = models
        .into_iter()
        .map(|model| {
            serde_json::json!({
                "name": model.name
            })
        })
        .collect();

    Ok(model_list)
}

/// Tauri command to cancel an ongoing chat message
#[tauri::command]
pub async fn cancel_chat_message(
    state: State<'_, AppState>,
    session_id: Option<String>,
) -> Result<(), String> {
    info!(
        "cancel_chat_message called with session_id: {:?}",
        session_id
    );
    if let Some(session_id) = session_id {
        let mut tokens = state.cancellations.lock().await;
        info!(
            "Current active tokens: {:?}",
            tokens.keys().collect::<Vec<_>>()
        );
        if let Some(token) = tokens.remove(&session_id) {
            token.cancel();
            info!(
                "Successfully cancelled chat message for session: {}",
                session_id
            );
            Ok(())
        } else {
            warn!(
                "No active request found for session: {} (available: {:?})",
                session_id,
                tokens.keys().collect::<Vec<_>>()
            );
            Err("No active request found for this session".to_string())
        }
    } else {
        error!("Session ID is required for cancellation but was None");
        Err("Session ID is required for cancellation".to_string())
    }
}

/// Tauri command to get provider settings
#[tauri::command]
pub async fn get_provider_settings(state: State<'_, AppState>) -> Result<ProviderSettings, String> {
    info!("Loading provider settings");

    ProviderSettings::load(&state.paths.config_dir).map_err(|e| {
        error!("Failed to load provider settings: {}", e);
        format!("Failed to load provider settings: {}", e)
    })
}

/// Tauri command to save provider settings
#[tauri::command]
pub async fn save_provider_settings(
    state: State<'_, AppState>,
    settings: ProviderSettings,
) -> Result<(), String> {
    info!("Saving provider settings: {:?}", settings.provider_type);

    // Validate settings before saving
    settings.validate().map_err(|e| {
        error!("Invalid provider settings: {}", e);
        format!("Invalid provider settings: {}", e)
    })?;

    settings.save(&state.paths.config_dir).map_err(|e| {
        error!("Failed to save provider settings: {}", e);
        format!("Failed to save provider settings: {}", e)
    })?;

    info!("Provider settings saved successfully");
    Ok(())
}

/// Tauri command to reload LLM service with new provider settings
#[tauri::command]
pub async fn reload_llm_service(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    info!("Reloading LLM service with new provider settings");

    // Create new LLM service with updated settings
    let new_service = super::initialize_llm(
        app_handle,
        state.db.clone(),
        state.confirmations.clone(),
        state.paths.clone(),
    )
    .await
    .map_err(|e| {
        error!("Failed to reload LLM service: {}", e);
        format!("Failed to reload LLM service: {}", e)
    })?;

    // Replace the old service with the new one
    let mut service_guard = state.llm.lock().await;
    *service_guard = Some(new_service);

    info!("LLM service reloaded successfully");
    Ok(())
}

/// Tauri command to get app settings
#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    info!("Loading app settings");

    AppSettings::load(&state.paths.config_dir).map_err(|e| {
        error!("Failed to load app settings: {}", e);
        format!("Failed to load app settings: {}", e)
    })
}

/// Tauri command to save app settings
#[tauri::command]
pub async fn save_app_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    info!("Saving app settings: ai_assistant_enabled={}", settings.ai_assistant_enabled);

    settings.save(&state.paths.config_dir).map_err(|e| {
        error!("Failed to save app settings: {}", e);
        format!("Failed to save app settings: {}", e)
    })?;

    info!("App settings saved successfully");
    Ok(())
}

//! Task executor - runs tasks through the production ChatProcessor

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use mimir_dm::app_init::AppPaths;
use mimir_dm::commands::content::books::catalog_import::import_all_catalogs_from_book;
use mimir_dm::services::llm::chat_processor::{ChatProcessor, ToolCallRecord};
use mimir_dm::services::llm::{ConfirmationReceivers, LlmService};
use mimir_dm::services::provider_settings::{GroqConfig, OllamaConfig, ProviderSettings, ProviderType};
use mimir_dm_core::seed::seed_dev_data;
use mimir_dm_core::services::CharacterService;
use mimir_dm_core::DatabaseService;
use mimir_dm_llm::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tar::Archive;
use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::tasks::{AgentTask, SetupAction};
use crate::verification::{run_verifications, VerificationContext, VerificationResult};

/// Helper to create a minimal CharacterData for testing
fn create_test_character_data(
    name: &str,
    race: &str,
    class: &str,
    level: i32,
    max_hp: i32,
) -> mimir_dm_core::models::character::CharacterData {
    use mimir_dm_core::models::character::{
        AbilityScores, Appearance, CharacterData, ClassLevel, Currency, EquippedItems, Personality,
        Proficiencies, RoleplayNotes, SpellData,
    };

    CharacterData {
        character_name: name.to_string(),
        player_id: None,
        level,
        experience_points: 0,
        version: 1,
        snapshot_reason: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        race: race.to_string(),
        subrace: None,
        classes: vec![ClassLevel {
            class_name: class.to_string(),
            subclass: None,
            level,
            hit_dice_type: "d10".to_string(),
            hit_dice_remaining: level,
        }],
        background: "Custom".to_string(),
        alignment: None,
        abilities: AbilityScores {
            strength: 14,
            dexterity: 12,
            constitution: 14,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        },
        max_hp,
        current_hp: max_hp,
        speed: 30,
        proficiencies: Proficiencies {
            skills: vec![],
            saves: vec![],
            armor: vec![],
            weapons: vec![],
            tools: vec![],
            languages: vec!["Common".to_string()],
        },
        class_features: vec![],
        feats: vec![],
        spells: SpellData::default(),
        inventory: vec![],
        currency: Currency::default(),
        equipped: EquippedItems::default(),
        personality: Personality::default(),
        player_name: None,
        appearance: Appearance::default(),
        backstory: None,
        background_feature: None,
        roleplay_notes: RoleplayNotes::default(),
        npc_role: None,
        npc_location: None,
        npc_faction: None,
        npc_notes: None,
        legendary_actions: vec![],
        legendary_action_count: None,
    }
}

/// Result of running a single task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub duration_ms: u64,
    pub response: String,
    pub tools_called: Vec<String>,
    pub tool_results: Vec<ToolResultRecord>,
    pub verification_results: Vec<VerificationResult>,
    /// Results for each turn in a multi-turn conversation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub turn_results: Vec<TurnResult>,
    pub error: Option<String>,
}

/// Result of a single turn in a multi-turn conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub turn_number: usize,
    pub prompt: String,
    pub response: String,
    pub tools_called: Vec<String>,
    pub verification_results: Vec<VerificationResult>,
    pub success: bool,
}

/// Record of a tool call result for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultRecord {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: String,
    pub success: bool,
}

impl From<&ToolCallRecord> for ToolResultRecord {
    fn from(record: &ToolCallRecord) -> Self {
        Self {
            tool_name: record.name.clone(),
            arguments: record.arguments.clone(),
            result: record.result.clone(),
            success: record.success,
        }
    }
}

/// Executor for running agent tasks using production ChatProcessor
pub struct AgentTestExecutor {
    llm_service: LlmService,
    db_service: Arc<DatabaseService>,
    _temp_dir: TempDir,
    system_prompt: String,
    ollama_url: String,
    campaign_id: i32,
}

impl AgentTestExecutor {
    /// Create a new executor with a fresh test database seeded with dev data
    pub async fn new(
        provider_name: String,
        model: Option<String>,
        ollama_url: String,
        api_key: Option<String>,
        _keep_db: bool,
    ) -> Result<Self> {
        // Create temp directory for test environment
        let temp_dir = TempDir::new().context("Failed to create temp directory")?;
        let base_path = temp_dir.path();

        // Create required subdirectories (matching TestEnv pattern)
        let app_dir = base_path.join("app");
        let config_dir = base_path.join("config");
        let data_dir = base_path.join("data");
        let logs_dir = base_path.join("logs");
        let campaigns_dir = data_dir.join("campaigns");

        std::fs::create_dir_all(&app_dir)?;
        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(&logs_dir)?;
        std::fs::create_dir_all(&campaigns_dir)?;
        std::fs::create_dir_all(logs_dir.join("chat_sessions"))?;
        std::fs::create_dir_all(data_dir.join("chat_sessions"))?;
        std::fs::create_dir_all(data_dir.join("books"))?;

        // Create AppPaths
        let paths = Arc::new(AppPaths {
            app_dir,
            config_dir: config_dir.clone(),
            data_dir: data_dir.clone(),
            logs_dir,
            database_path: data_dir.join("test.db"),
            is_memory_db: false,
        });

        // Initialize database
        let db_path = paths.database_path.to_str().unwrap();
        let db_service = Arc::new(
            DatabaseService::new(db_path, false).context("Failed to create database service")?,
        );

        // Run migrations
        {
            let mut conn = db_service.get_connection()?;
            mimir_dm_core::run_migrations(&mut conn)?;
        }

        // Seed templates first
        {
            let mut conn = db_service.get_connection()?;
            mimir_dm_core::seed::template_seeder::seed_templates(&mut conn)?;
        }

        // Seed dev data (creates campaign, players, characters)
        {
            let mut conn = db_service.get_connection()?;
            seed_dev_data(&mut conn, campaigns_dir.to_str().unwrap(), data_dir.to_str().unwrap())?;
        }

        // Load PHB catalog data for character creation tests
        Self::load_phb_catalog_data(&db_service, &data_dir)?;

        tracing::info!("Test database seeded at: {:?}", db_path);

        // Create provider settings for the test
        let provider_settings =
            Self::create_provider_settings(&provider_name, model.clone(), &ollama_url, api_key)?;

        // Save provider settings so LlmService can load them
        provider_settings.save(&config_dir)?;

        // Create confirmation receivers (for tool confirmations - auto-approve in tests)
        let confirmations: ConfirmationReceivers = Arc::new(Mutex::new(HashMap::new()));

        // Create LlmService without AppHandle (test mode)
        let llm_service = LlmService::new(
            db_service.clone(),
            confirmations,
            None, // No AppHandle for tests
            paths.clone(),
        )
        .context("Failed to create LlmService")?;

        // Build system prompt from database
        let system_prompt = crate::prompts::build_system_prompt(&db_service, 1)?;

        tracing::info!(
            "Agent test executor initialized with {} provider, model: {:?}",
            provider_name,
            model
        );

        Ok(Self {
            llm_service,
            db_service,
            _temp_dir: temp_dir,
            system_prompt,
            ollama_url,
            campaign_id: 1, // Test campaign from dev seed
        })
    }

    fn create_provider_settings(
        provider_name: &str,
        model: Option<String>,
        ollama_url: &str,
        api_key: Option<String>,
    ) -> Result<ProviderSettings> {
        match provider_name {
            "ollama" => {
                let model_name = model.unwrap_or_else(|| "gpt-oss:20b".to_string());
                Ok(ProviderSettings {
                    provider_type: ProviderType::Ollama,
                    ollama_config: Some(OllamaConfig {
                        base_url: ollama_url.to_string(),
                        model: Some(model_name),
                    }),
                    groq_config: None,
                    tool_confirmation_timeout_secs: 30,
                })
            }
            "groq" => {
                let model_name = model.unwrap_or_else(|| "qwen/qwen3-32b".to_string());
                let key = api_key
                    .or_else(|| std::env::var("GROQ_API_KEY").ok())
                    .context("Groq requires API key (--api-key or GROQ_API_KEY env)")?;

                Ok(ProviderSettings {
                    provider_type: ProviderType::Groq,
                    ollama_config: None,
                    groq_config: Some(GroqConfig {
                        api_key: key,
                        model: Some(model_name),
                    }),
                    tool_confirmation_timeout_secs: 30,
                })
            }
            other => anyhow::bail!("Unknown provider: {}. Supported: ollama, groq", other),
        }
    }

    /// Load PHB catalog data (races, classes, backgrounds) for character creation tests
    ///
    /// Looks for PHB archive in the standard data/books-output location.
    /// If not found, logs a warning but continues (some tests may fail).
    fn load_phb_catalog_data(db_service: &Arc<DatabaseService>, data_dir: &Path) -> Result<()> {
        // Look for PHB archive relative to cargo workspace root
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = crate_dir.parent().unwrap().parent().unwrap();
        let phb_archive = workspace_root.join("data/books-output/phb.tar.gz");

        if !phb_archive.exists() {
            tracing::warn!(
                "PHB archive not found at {:?} - character creation tests may fail",
                phb_archive
            );
            return Ok(());
        }

        tracing::info!("Loading PHB catalog data from {:?}", phb_archive);

        // Extract PHB to books directory
        let books_dir = data_dir.join("books");
        let tar_gz = std::fs::File::open(&phb_archive)
            .context("Failed to open PHB archive")?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(&books_dir)
            .context("Failed to extract PHB archive")?;

        // Import catalog data from extracted PHB
        let phb_dir = books_dir.join("PHB");
        if phb_dir.exists() {
            let mut conn = db_service.get_connection()?;
            import_all_catalogs_from_book(&mut conn, &phb_dir, "PHB");
            tracing::info!("PHB catalog data loaded successfully");
        } else {
            tracing::warn!("PHB directory not found after extraction");
        }

        Ok(())
    }

    /// Get campaign directory path for the test campaign
    fn get_campaign_directory(&self) -> Option<String> {
        let mut conn = self.db_service.get_connection().ok()?;
        let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);
        service
            .get_campaign(self.campaign_id)
            .ok()?
            .map(|c| c.directory_path)
    }

    /// Execute setup actions before running a task
    fn run_setup_actions(&self, setup_actions: &[SetupAction]) -> Result<()> {
        if setup_actions.is_empty() {
            return Ok(());
        }

        let mut conn = self.db_service.get_connection()?;

        for action in setup_actions {
            match action {
                SetupAction::CreateCharacter {
                    name,
                    class,
                    level,
                    race,
                } => {
                    tracing::info!(
                        "Setup: Creating character '{}' ({} {} level {})",
                        name,
                        race.as_deref().unwrap_or("Human"),
                        class,
                        level
                    );

                    let mut service = CharacterService::new(&mut conn);
                    let campaign_id = 1;
                    let player_id = 1;
                    let max_hp = *level * 8 + 2;
                    let race_str = race.as_deref().unwrap_or("Human");
                    let character_data =
                        create_test_character_data(name, race_str, class, *level, max_hp);

                    match service.create_character(
                        Some(campaign_id),
                        Some(player_id),
                        false, // is_npc
                        "/tmp",
                        character_data,
                    ) {
                        Ok(character) => {
                            tracing::info!(
                                "Setup: Created character '{}' with ID {}",
                                name,
                                character.id
                            );
                        }
                        Err(e) => {
                            tracing::warn!("Setup: Failed to create character '{}': {}", name, e);
                        }
                    }
                }
                SetupAction::CreateNpc {
                    name,
                    class,
                    race,
                } => {
                    tracing::info!(
                        "Setup: Creating NPC '{}' ({} {})",
                        name,
                        race.as_deref().unwrap_or("Human"),
                        class.as_deref().unwrap_or("Commoner")
                    );

                    let mut service = CharacterService::new(&mut conn);
                    let campaign_id = 1;
                    let race_str = race.as_deref().unwrap_or("Human");
                    let class_str = class.as_deref().unwrap_or("Commoner");
                    let character_data =
                        create_test_character_data(name, race_str, class_str, 1, 8);

                    match service.create_character(
                        Some(campaign_id),
                        None, // No player_id for NPCs
                        true, // is_npc
                        "/tmp",
                        character_data,
                    ) {
                        Ok(character) => {
                            tracing::info!(
                                "Setup: Created NPC '{}' with ID {}",
                                name,
                                character.id
                            );
                        }
                        Err(e) => {
                            tracing::warn!("Setup: Failed to create NPC '{}': {}", name, e);
                        }
                    }
                }
                SetupAction::CreateCampaign { name } => {
                    tracing::info!("Setup: Creating campaign '{}' (not implemented)", name);
                }
                SetupAction::Sql { statement } => {
                    tracing::info!(
                        "Setup: Executing SQL: {}",
                        &statement[..statement.len().min(50)]
                    );
                    use diesel::prelude::*;
                    if let Err(e) = diesel::sql_query(statement).execute(&mut conn) {
                        tracing::warn!("Setup: SQL failed: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Run a single task using the production ChatProcessor
    pub async fn run_task(&self, task: &AgentTask) -> TaskResult {
        let start = Instant::now();

        // Run setup actions first
        if let Err(e) = self.run_setup_actions(&task.setup) {
            tracing::warn!("Setup actions failed for task {}: {}", task.id, e);
        }

        // Check if this is a multi-turn task
        if !task.turns.is_empty() {
            return self.run_multi_turn_task(task, start).await;
        }

        // Single-turn task execution using ChatProcessor
        self.run_single_turn_task(task, start).await
    }

    /// Run a single-turn task
    async fn run_single_turn_task(&self, task: &AgentTask, start: Instant) -> TaskResult {
        tracing::info!("=== Task: {} ===", task.id);
        tracing::info!("User prompt: {}", task.prompt);

        // Build messages
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: self.system_prompt.clone(),
                tool_call_id: None,
            },
            Message {
                role: "user".to_string(),
                content: task.prompt.clone(),
                tool_call_id: None,
            },
        ];

        // Create ChatProcessor and process
        let processor = ChatProcessor::new(&self.llm_service);
        let session_id = format!("test-{}", task.id);
        let cancellation_token = CancellationToken::new();

        let result = processor
            .process_chat(
                messages,
                Some(8192),                              // max_tokens
                Some(0.3),                               // temperature
                true,                                    // enable_tools
                &session_id,
                Some(self.ollama_url.as_str()),         // ollama_url
                self.get_campaign_directory().as_deref(), // campaign_directory_path
                Some(self.campaign_id),                 // campaign_id
                cancellation_token,
            )
            .await;

        match result {
            Ok(response) => {
                let tools_called: Vec<String> =
                    response.tools_called.iter().map(|t| t.name.clone()).collect();
                let tool_results: Vec<ToolResultRecord> =
                    response.tools_called.iter().map(|t| t.into()).collect();
                let had_errors = response.tools_called.iter().any(|t| !t.success);

                tracing::info!("LLM response: {}", response.content);

                // Run verifications
                let verification_results = if let Some(ref verifications) = task.verify {
                    let context = VerificationContext {
                        db_service: self.db_service.clone(),
                        response_content: response.content.clone(),
                        tools_called: response.tools_called.clone(),
                        had_errors,
                        llm_service: Some(&self.llm_service),
                        original_prompt: task.prompt.clone(),
                    };
                    run_verifications(verifications, &context).await
                } else {
                    vec![]
                };

                // Success is verification-based when verifications exist,
                // otherwise fall back to error-based (for simple tests without verifications)
                let success = if verification_results.is_empty() {
                    !had_errors
                } else {
                    verification_results.iter().all(|v| v.passed)
                };

                // Log verification results
                for vr in &verification_results {
                    if vr.passed {
                        tracing::info!(
                            "✓ {}: {}",
                            vr.check_type,
                            vr.message.as_deref().unwrap_or("passed")
                        );
                    } else {
                        tracing::warn!(
                            "✗ {}: {}",
                            vr.check_type,
                            vr.message.as_deref().unwrap_or("failed")
                        );
                    }
                }
                tracing::info!(
                    "=== Task {} {} ===\n",
                    task.id,
                    if success { "PASSED" } else { "FAILED" }
                );

                TaskResult {
                    task_id: task.id.clone(),
                    success,
                    duration_ms: start.elapsed().as_millis() as u64,
                    response: response.content,
                    tools_called,
                    tool_results,
                    verification_results,
                    turn_results: vec![],
                    error: None,
                }
            }
            Err(e) => TaskResult {
                task_id: task.id.clone(),
                success: false,
                duration_ms: start.elapsed().as_millis() as u64,
                response: String::new(),
                tools_called: vec![],
                tool_results: vec![],
                verification_results: vec![],
                turn_results: vec![],
                error: Some(format!("ChatProcessor error: {}", e)),
            },
        }
    }

    /// Run a multi-turn conversation task
    async fn run_multi_turn_task(&self, task: &AgentTask, start: Instant) -> TaskResult {
        tracing::info!(
            "=== Multi-Turn Task: {} ({} turns) ===",
            task.id,
            task.turns.len()
        );

        let mut all_tools_called = Vec::new();
        let mut all_tool_results: Vec<ToolResultRecord> = Vec::new();
        let mut turn_results: Vec<TurnResult> = Vec::new();
        let mut had_errors = false;
        let mut final_response = String::new();

        // Start with system message
        let mut messages = vec![Message {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
            tool_call_id: None,
        }];

        let session_id = format!("test-{}", task.id);

        // Process each turn
        for (turn_idx, turn) in task.turns.iter().enumerate() {
            tracing::info!("--- Turn {} ---", turn_idx + 1);
            tracing::info!("User: {}", turn.prompt);

            // Add user message
            messages.push(Message {
                role: "user".to_string(),
                content: turn.prompt.clone(),
                tool_call_id: None,
            });

            // Process this turn using ChatProcessor
            let processor = ChatProcessor::new(&self.llm_service);
            let cancellation_token = CancellationToken::new();

            let result = processor
                .process_chat(
                    messages.clone(),
                    Some(8192),
                    Some(0.3),
                    true,
                    &session_id,
                    Some(self.ollama_url.as_str()),
                    self.get_campaign_directory().as_deref(),
                    Some(self.campaign_id),
                    cancellation_token,
                )
                .await;

            match result {
                Ok(response) => {
                    let turn_tools_called: Vec<String> =
                        response.tools_called.iter().map(|t| t.name.clone()).collect();
                    let turn_tool_records = response.tools_called.clone();

                    all_tools_called.extend(turn_tools_called.clone());
                    all_tool_results
                        .extend(response.tools_called.iter().map(|t| t.into()));

                    if response.tools_called.iter().any(|t| !t.success) {
                        had_errors = true;
                    }

                    tracing::info!("Assistant: {}", response.content);
                    final_response = response.content.clone();

                    // Add assistant response to conversation for next turn
                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: response.content.clone(),
                        tool_call_id: None,
                    });

                    // Run turn-specific verifications
                    let turn_verification_results = if let Some(ref verifications) = turn.verify {
                        let context = VerificationContext {
                            db_service: self.db_service.clone(),
                            response_content: response.content,
                            tools_called: turn_tool_records,
                            had_errors,
                            llm_service: Some(&self.llm_service),
                            original_prompt: turn.prompt.clone(),
                        };
                        run_verifications(verifications, &context).await
                    } else {
                        vec![]
                    };

                    // Turn success is verification-based when verifications exist,
                    // otherwise fall back to error-based
                    let turn_had_errors = response.tools_called.iter().any(|t| !t.success);
                    let turn_success = if turn_verification_results.is_empty() {
                        !turn_had_errors
                    } else {
                        turn_verification_results.iter().all(|v| v.passed)
                    };

                    // Log turn verification results
                    for vr in &turn_verification_results {
                        if vr.passed {
                            tracing::info!(
                                "✓ Turn {} - {}: {}",
                                turn_idx + 1,
                                vr.check_type,
                                vr.message.as_deref().unwrap_or("passed")
                            );
                        } else {
                            tracing::warn!(
                                "✗ Turn {} - {}: {}",
                                turn_idx + 1,
                                vr.check_type,
                                vr.message.as_deref().unwrap_or("failed")
                            );
                        }
                    }

                    turn_results.push(TurnResult {
                        turn_number: turn_idx + 1,
                        prompt: turn.prompt.clone(),
                        response: final_response.clone(),
                        tools_called: turn_tools_called,
                        verification_results: turn_verification_results,
                        success: turn_success,
                    });

                    if !turn_success {
                        tracing::warn!(
                            "Turn {} failed, stopping multi-turn execution",
                            turn_idx + 1
                        );
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("LLM error in turn {}: {}", turn_idx + 1, e);

                    turn_results.push(TurnResult {
                        turn_number: turn_idx + 1,
                        prompt: turn.prompt.clone(),
                        response: String::new(),
                        tools_called: vec![],
                        verification_results: vec![],
                        success: false,
                    });
                    break;
                }
            }
        }

        // Overall success is all turns passing their verifications
        // (tool errors don't fail the test if verifications pass)
        let success = turn_results.iter().all(|t| t.success);
        tracing::info!(
            "=== Task {} {} ({} turns) ===\n",
            task.id,
            if success { "PASSED" } else { "FAILED" },
            turn_results.len()
        );

        TaskResult {
            task_id: task.id.clone(),
            success,
            duration_ms: start.elapsed().as_millis() as u64,
            response: final_response,
            tools_called: all_tools_called,
            tool_results: all_tool_results,
            verification_results: vec![],
            turn_results,
            error: None,
        }
    }
}

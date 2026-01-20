//! Tool system for LLM function calling
//!
//! This module provides tools that can be called by the LLM to fetch data
//! and perform actions within the application.

use anyhow::Result;
use mimir_dm_llm::traits::ToolCallContext as ToolCall;
use mimir_dm_llm::{Tool as LlmTool, ToolTrait};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{info, warn};

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolTrait>>,
    recent_calls: Arc<Mutex<VecDeque<ToolCall>>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            recent_calls: Arc::new(Mutex::new(VecDeque::with_capacity(10))),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn ToolTrait>) {
        let name = tool.name().to_string();
        info!("Registering tool: {}", name);
        self.tools.insert(name, tool);
    }

    /// Get all tool definitions for the LLM
    pub fn get_tool_definitions(&self) -> Vec<LlmTool> {
        self.tools.values().map(|tool| tool.to_llm_tool()).collect()
    }

    /// Execute a tool by name with the given arguments
    pub async fn execute_tool(&self, name: &str, arguments: Value) -> Result<String> {
        // Record the tool call before execution
        self.record_tool_call(name, &arguments);

        match self.tools.get(name) {
            Some(tool) => {
                let result = tool
                    .execute_with_context(arguments, self.recent_calls.clone())
                    .await
                    .map_err(|e| anyhow::anyhow!("Tool execution failed: {}", e))?;
                Ok(result)
            }
            None => {
                warn!("Tool not found: {}", name);
                Err(anyhow::anyhow!("Tool not found: {}", name))
            }
        }
    }

    /// Record a tool call in the recent calls history
    fn record_tool_call(&self, name: &str, arguments: &Value) {
        let mut calls = self.recent_calls.lock().unwrap();

        // Extract file_path from arguments if present
        let file_path = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        calls.push_back(ToolCall {
            name: name.to_string(),
            timestamp: Instant::now(),
            file_path,
        });

        // Keep only the last 10 calls
        while calls.len() > 10 {
            calls.pop_front();
        }
    }

    /// Check if a tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Check if a tool requires confirmation
    pub fn requires_confirmation(&self, name: &str) -> bool {
        self.tools
            .get(name)
            .map(|tool| tool.requires_confirmation())
            .unwrap_or(false)
    }

    /// Get action description for a tool
    pub fn get_action_description(
        &self,
        name: &str,
        arguments: &Value,
    ) -> Option<mimir_dm_llm::traits::ActionDescription> {
        self.tools
            .get(name)
            .and_then(|tool| tool.describe_action(arguments))
    }

    /// Generate system prompt rules based on registered tools
    ///
    /// This method examines the available tools and generates guidance rules
    /// that help the LLM understand tool dependencies and proper usage patterns.
    pub fn generate_system_rules(&self, session_id: Option<&str>) -> Vec<String> {
        self.generate_system_rules_with_directory(session_id, None)
    }

    /// Generate system prompt rules with optional custom directory override
    ///
    /// This method allows overriding the directory path for campaign-specific operations
    pub fn generate_system_rules_with_directory(
        &self,
        session_id: Option<&str>,
        custom_directory: Option<&str>,
    ) -> Vec<String> {
        let mut rules = Vec::new();

        // When campaign directory is provided, we work exclusively with that directory
        if let Some(campaign_dir) = custom_directory {
            info!(
                "Using campaign directory for file operations: {}",
                campaign_dir
            );
        }

        // Add general context information
        rules.push(self.generate_context_information(session_id, custom_directory));

        // Check if both get_document and update_document are available
        if self.has_tool("get_document") && self.has_tool("update_document") {
            rules.push(
                "TOOL USAGE RULE: When asked to update, create, or work on a document:\n\
                1. ALWAYS call get_document first to read the current content\n\
                2. THEN call update_document with your changes\n\
                3. NEVER just show content without saving it - if you generate content, you MUST use update_document to save it\n\
                4. Don't just explain what you would do - actually make the tool calls to complete the user's request\n\
                5. If you create new content for a document, use update_document to save it immediately".to_string()
            );
        }

        // Add file tool usage rules if file tools are available
        if self.has_tool("read_file") && self.has_tool("write_file") && self.has_tool("list_files")
        {
            if let Some(custom_dir) = custom_directory {
                // Use campaign directory exclusively - no fallback to data directory
                rules.push(format!(
                    "## FILE PATH REQUIREMENTS\n\
                    \n\
                    **CAMPAIGN DIRECTORY**: {}\n\
                    \n\
                    ### Usage Guidelines\n\
                    - ALL file operations must use this campaign directory\n\
                    - Always use the complete path: `{}/your_filename.txt`\n\
                    - If uncertain about structure, run list_files first\n\
                    \n\
                    ### Campaign Structure\n\
                    - You are working within a campaign directory structure\n\
                    - Standard subdirectories: session_zero/, world/, modules/, sessions/, characters/, npcs/, resources/, templates/\n\
                    - Templates are in the templates/ subdirectory\n\
                    \n\
                    ### When Taking Action\n\
                    - Use the exact campaign path for ALL file operations\n\
                    - Take direct action when given clear file operation instructions",
                    custom_dir,
                    custom_dir
                ));
            } else {
                // When no campaign directory is provided, require discovery
                rules.push(
                    "## FILE OPERATIONS - CAMPAIGN REQUIRED\n\
                    \n\
                    ### Required Workflow\n\
                    1. A campaign must be selected before file operations can be performed\n\
                    2. File operations are only available within campaign context\n\
                    3. Use list_files() only after campaign context is established\n"
                        .to_string(),
                );
                warn!("No campaign directory provided - file operations limited");
            }
        }

        // Add tool awareness and relationship guidance
        rules.push(self.generate_tool_awareness_guidance());

        // Debug: Log what rules we generated
        info!("Generated {} system rules total", rules.len());
        for (i, rule) in rules.iter().enumerate() {
            if rule.contains("FILE TOOL USAGE RULES") {
                info!(
                    "Rule {}: FILE TOOL USAGE RULES (first 200 chars): {}",
                    i,
                    &rule.chars().take(200).collect::<String>()
                );
            }
        }

        rules
    }

    /// Generate context information for the LLM session
    fn generate_context_information(
        &self,
        session_id: Option<&str>,
        custom_directory: Option<&str>,
    ) -> String {
        let mut context = String::from("## Session Context\n");

        // Add campaign directory information - this is critical for LLM to know
        if let Some(custom_dir) = custom_directory {
            context.push_str(&format!(
                "**CAMPAIGN DIRECTORY**\n\
                - **PATH**: {}\n\
                - **ALL file operations must use this campaign directory**\n\
                - **CONTEXT**: Active campaign - files will be created in organized campaign structure\n\n",
                custom_dir
            ));
        } else {
            context.push_str(
                "**FILE OPERATIONS**\n\
                - **STATUS**: No campaign selected\n\
                - **REQUIREMENT**: Campaign must be selected for file operations\n\n",
            );
        }

        // Add session ID if available
        if let Some(session_id) = session_id {
            context.push_str(&format!("- Session ID: {}\n", session_id));
        }

        // Add campaign directory structure information
        if let Some(custom_dir) = custom_directory {
            // Get available files in the campaign directory
            let available_files = self.get_campaign_files(custom_dir);

            context.push_str(&format!(
                "- **Campaign Path**: {}\n\
                - **Structure**: Organized campaign directory with standard subdirectories\n\
                - **Templates**: Available in templates/ subdirectory for structured content creation\n\
                - **Path Requirement**: ALL file operations must use paths starting with: {}\n",
                custom_dir,
                custom_dir
            ));

            if !available_files.is_empty() {
                context.push_str("- **Available Files**:\n");
                for file in available_files.iter().take(20) {
                    // Limit to 20 files to avoid overwhelming
                    context.push_str(&format!("  - {}\n", file));
                }
                if available_files.len() > 20 {
                    context.push_str(&format!(
                        "  - ... and {} more files\n",
                        available_files.len() - 20
                    ));
                }
            } else {
                context.push_str(
                    "- **Available Files**: No files found - this appears to be a new campaign\n",
                );
            }
        } else {
            context.push_str(
                "- **Campaign Status**: No active campaign\n\
                - **File Operations**: Unavailable - campaign selection required\n\
                - **Next Step**: Select or create a campaign to enable file operations\n",
            );
        }

        // Add tool availability context
        let tool_count = self.tools.len();
        let tool_names: Vec<&str> = self.tools.keys().map(|s| s.as_str()).collect();
        context.push_str(&format!(
            "- Available Tools ({}): {}\n",
            tool_count,
            tool_names.join(", ")
        ));

        // Add session management notes
        context.push_str(
            "- **Session Management**: Persistent session where previous context and actions may influence current behavior\n\
            - **Tool State**: Some tools maintain state across calls (e.g., todo_write tracks progress per session)\n\
            - **File Operations**: All file operations are sandboxed to the application directory for security\n\
            - **Path Requirements**: ALWAYS use fully qualified (absolute) paths for all file operations\n"
        );

        context
    }

    /// Generate tool awareness and relationship guidance
    fn generate_tool_awareness_guidance(&self) -> String {
        let mut guidance = String::from("## TOOL AWARENESS\n\n");

        // Comprehensive tool categories with descriptions
        guidance.push_str("### Tool Categories and Usage\n\n");

        // Character Management Tools
        guidance.push_str("#### Character Management\n");
        guidance.push_str("Use these tools to manage player characters (PCs) and NPCs:\n\n");
        if self.has_tool("get_character") {
            guidance.push_str("- **get_character**: Retrieve full character data by ID (stats, inventory, spells, equipment)\n");
        }
        if self.has_tool("list_campaign_characters") {
            guidance.push_str("- **list_campaign_characters**: List all characters in a campaign (both PCs and NPCs)\n");
        }
        if self.has_tool("list_npcs") {
            guidance.push_str("- **list_npcs**: List only NPCs in a campaign\n");
        }
        if self.has_tool("list_player_characters") {
            guidance.push_str("- **list_player_characters**: List only player characters (PCs) in a campaign\n");
        }
        if self.has_tool("get_character_stats") {
            guidance.push_str("- **get_character_stats**: Get character's ability scores, saves, skills, and derived values\n");
        }
        if self.has_tool("list_players") {
            guidance.push_str("- **list_players**: List all players registered in the database\n");
        }
        if self.has_tool("create_character") {
            guidance.push_str("- **create_character**: Create a new player character (PC) with full stats and details\n");
        }
        if self.has_tool("create_npc") {
            guidance.push_str("- **create_npc**: Create a new NPC - USE THIS for all NPC creation, NOT file templates\n");
        }
        if self.has_tool("update_character") {
            guidance.push_str("- **update_character**: Update character attributes, background, or other details\n");
        }
        if self.has_tool("level_up") {
            guidance.push_str("- **level_up**: Level up a character to a target level (calculates HP based on class hit dice + CON)\n");
        }
        guidance.push_str("\n");

        // Combat & Health Tools
        guidance.push_str("#### Combat & Health\n");
        guidance.push_str("Use these for combat tracking and health management:\n\n");
        if self.has_tool("update_character_hp") {
            guidance.push_str("- **update_character_hp**: Apply damage or healing to a character (use negative for damage)\n");
        }
        if self.has_tool("take_rest") {
            guidance.push_str("- **take_rest**: Apply short or long rest mechanics (restore HP, spell slots, hit dice)\n");
        }
        guidance.push_str("\n");

        // Spellcasting Tools
        guidance.push_str("#### Spellcasting\n");
        guidance.push_str("Use these for spell management:\n\n");
        if self.has_tool("check_spell_slots") {
            guidance.push_str("- **check_spell_slots**: Check available spell slots for spellcasting characters\n");
        }
        if self.has_tool("cast_spell") {
            guidance.push_str("- **cast_spell**: Cast a spell, consuming the appropriate spell slot\n");
        }
        guidance.push_str("\n");

        // Inventory & Equipment Tools
        guidance.push_str("#### Inventory & Equipment\n");
        guidance.push_str("Use these to manage character possessions:\n\n");
        if self.has_tool("add_inventory_item") {
            guidance.push_str("- **add_inventory_item**: Add an item to a character's inventory\n");
        }
        if self.has_tool("remove_inventory_item") {
            guidance.push_str("- **remove_inventory_item**: Remove an item from inventory\n");
        }
        if self.has_tool("update_equipped") {
            guidance.push_str("- **update_equipped**: Change what items are equipped (armor, weapons, shield)\n");
        }
        if self.has_tool("update_currency") {
            guidance.push_str("- **update_currency**: Add or remove gold, silver, copper, electrum, or platinum\n");
        }
        guidance.push_str("\n");

        // D&D Catalog/Reference Tools
        guidance.push_str("#### D&D Reference Catalog\n");
        guidance.push_str("Use these to search the D&D 5e rules catalog:\n\n");
        if self.has_tool("search_monsters") {
            guidance.push_str("- **search_monsters**: Search monster catalog by name, CR, type, size, or alignment\n");
        }
        if self.has_tool("search_spells") {
            guidance.push_str("- **search_spells**: Search spells by name, level, school, or class availability\n");
        }
        if self.has_tool("search_items") {
            guidance.push_str("- **search_items**: Search equipment, weapons, armor, and magic items\n");
        }
        guidance.push_str("\n");

        // Module/Adventure Tools
        guidance.push_str("#### Adventure Modules\n");
        guidance.push_str("Use these to manage adventure modules within campaigns:\n\n");
        if self.has_tool("create_module") {
            guidance.push_str("- **create_module**: Create a new adventure module from templates\n");
        }
        if self.has_tool("list_modules") {
            guidance.push_str("- **list_modules**: List all modules for a campaign\n");
        }
        if self.has_tool("get_module") {
            guidance.push_str("- **get_module**: Get details of a specific module\n");
        }
        if self.has_tool("update_module_status") {
            guidance.push_str("- **update_module_status**: Update module progress status\n");
        }
        guidance.push_str("\n");

        // File Tools
        guidance.push_str("#### File Operations\n");
        guidance.push_str("Use these to read/write campaign files (session notes, world building, etc.):\n\n");
        if self.has_tool("read_file") {
            guidance.push_str("- **read_file**: Read contents of a file in the campaign directory\n");
        }
        if self.has_tool("write_file") {
            guidance.push_str("- **write_file**: Create or overwrite a file (use for new content)\n");
        }
        if self.has_tool("edit_file") {
            guidance.push_str("- **edit_file**: Edit specific lines in an existing file (use for modifications)\n");
        }
        if self.has_tool("list_files") {
            guidance.push_str("- **list_files**: List files in the campaign directory structure\n");
        }
        guidance.push_str("\n");

        // Task Management
        guidance.push_str("#### Task Management\n");
        if self.has_tool("todo_write") {
            guidance.push_str("- **todo_write**: Track tasks and progress for complex multi-step operations\n");
        }
        guidance.push_str("\n");

        // Collect any additional workflow guidance from tools
        guidance.push_str("### Tool-Specific Workflow Guidance\n\n");
        let mut has_guidance = false;
        for (name, tool) in &self.tools {
            if let Some(tool_guidance) = tool.workflow_guidance() {
                guidance.push_str(&format!("**{}:**\n{}\n\n", name, tool_guidance));
                has_guidance = true;
            }
        }
        if !has_guidance {
            guidance.push_str("Follow tool descriptions for specific usage patterns.\n\n");
        }

        // ReAct Pattern
        guidance.push_str("### ReAct Pattern for Multi-Step Tasks\n\n");
        guidance.push_str("For tasks requiring multiple tool calls, use explicit reasoning:\n\n");
        guidance.push_str("1. **THOUGHT**: Analyze what needs to be done and plan your approach\n");
        guidance.push_str("   - Use `<thought>` blocks to show your reasoning\n");
        guidance.push_str("   - Break complex tasks into clear steps\n");
        guidance.push_str("2. **ACTION**: Execute the appropriate tool(s)\n");
        guidance.push_str("3. **OBSERVATION**: Examine tool results and determine next steps\n");
        guidance.push_str("4. **REPEAT**: Continue until task is complete\n\n");

        guidance.push_str("### Example Workflows\n\n");

        guidance.push_str("**Combat HP Update:**\n");
        guidance.push_str("```\n");
        guidance.push_str("<thought>The goblin hit Thorin for 8 damage. I'll update his HP.</thought>\n");
        guidance.push_str("[use update_character_hp with character_id and hp_change=-8]\n");
        guidance.push_str("```\n\n");

        guidance.push_str("**Looking up a Monster:**\n");
        guidance.push_str("```\n");
        guidance.push_str("<thought>DM needs stats for a CR 3 monster. I'll search the catalog.</thought>\n");
        guidance.push_str("[use search_monsters with cr_min=3, cr_max=3]\n");
        guidance.push_str("```\n\n");

        guidance.push_str("**Adding Loot to Inventory:**\n");
        guidance.push_str("```\n");
        guidance.push_str("<thought>Party found a Longsword. I'll add it to the fighter's inventory.</thought>\n");
        guidance.push_str("[use add_inventory_item with character_id, item_name=\"Longsword\", quantity=1]\n");
        guidance.push_str("```\n\n");

        guidance.push_str("### General Action Patterns\n");
        guidance.push_str("- **Take direct action** when user requests clear operations\n");
        guidance.push_str("- **Use tools in logical sequence** based on their guidance\n");
        guidance.push_str("- **Always complete requested actions** rather than just explaining them\n");
        guidance.push_str("- **Show reasoning** in `<thought>` blocks for complex tasks\n");
        guidance.push_str("- **Use catalog search tools** to look up D&D rules, monsters, spells, and items\n");
        guidance.push_str("- **Prefer character tools over file editing** for character data changes\n");
        guidance.push_str("- **NEVER assume or guess** - always use tools to verify character info, party composition, HP, spell slots, etc. Use list_campaign_characters, get_character, or get_character_stats to get accurate current state\n");

        guidance
    }

    /// Get list of files in a campaign directory
    fn get_campaign_files(&self, campaign_dir: &str) -> Vec<String> {
        let campaign_path = Path::new(campaign_dir);
        let mut files = Vec::new();

        if !campaign_path.exists() {
            warn!("Campaign directory does not exist: {}", campaign_dir);
            return files;
        }

        // Recursively walk the directory and collect file paths
        if let Ok(entries) = self.walk_directory(campaign_path, campaign_path) {
            files = entries;
        }

        // Sort files for consistent display
        files.sort();
        files
    }

    /// Recursively walk a directory and return relative file paths
    #[allow(clippy::only_used_in_recursion)]
    fn walk_directory(&self, dir: &Path, base_path: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut files = Vec::new();

        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and common ignore patterns
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with('.')
                        || dir_name == "node_modules"
                        || dir_name == "target"
                    {
                        continue;
                    }
                }

                // Recursively process subdirectory
                let mut sub_files = self.walk_directory(&path, base_path)?;
                files.append(&mut sub_files);
            } else {
                // Add file with relative path
                if let Ok(relative_path) = path.strip_prefix(base_path) {
                    if let Some(path_str) = relative_path.to_str() {
                        files.push(path_str.to_string());
                    }
                }
            }
        }

        Ok(files)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Character tools modules
pub mod character_tools;
pub mod character_write_tools;

// Catalog query tools
pub mod catalog_tools;

// Module management tools
pub mod module_tools;

// Note: campaign_summary_tools module removed - story summaries are now
// auto-generated during context building in chat_processor.rs

#[cfg(test)]
mod character_tools_test;

// Re-exports for convenience
pub use catalog_tools::{SearchItemsTool, SearchMonstersTool, SearchSpellsTool};
pub use character_tools::{
    CheckSpellSlotsTool, GetCharacterStatsTool, GetCharacterTool, ListCampaignCharactersTool,
    ListNpcsTool, ListPcsTool, ListPlayersTool,
};
pub use character_write_tools::{
    AddInventoryItemTool, CastSpellTool, CreateCharacterTool, CreateNpcTool, LevelUpTool,
    RemoveInventoryItemTool, TakeRestTool, UpdateCharacterHpTool, UpdateCharacterTool,
    UpdateCurrencyTool, UpdateEquippedTool,
};
pub use module_tools::{CreateModuleTool, GetModuleTool, ListModulesTool, UpdateModuleStatusTool};

use mimir_dm_core::DatabaseService;
use mimir_dm_llm::tools::{EditFileTool, ListFilesTool, ReadFileTool, WriteFileTool};
use mimir_dm_llm::FileToolsConfig;
use mimir_dm_llm::{TodoListTool, TodoStateManager};
use std::path::PathBuf;

/// Register all standard tools in the tool registry
///
/// This is the single source of truth for what tools are available.
/// Both production and tests should use this function to ensure consistency.
///
/// If `campaign_dir` is provided, file tools will be registered with that directory as root.
pub fn register_all_tools(
    registry: &mut ToolRegistry,
    db_service: Arc<DatabaseService>,
    todo_state_manager: TodoStateManager,
) {
    register_all_tools_with_file_config(registry, db_service, todo_state_manager, None);
}

/// Register all tools with optional campaign-specific file tools
///
/// This is the single source of truth for what tools are available.
pub fn register_all_tools_with_file_config(
    registry: &mut ToolRegistry,
    db_service: Arc<DatabaseService>,
    todo_state_manager: TodoStateManager,
    campaign_dir: Option<&str>,
) {
    // File tools (only if campaign directory is provided)
    if let Some(dir) = campaign_dir {
        let file_config = Arc::new(FileToolsConfig::with_root(PathBuf::from(dir)));
        registry.register(Arc::new(ReadFileTool::new(file_config.clone())));
        registry.register(Arc::new(WriteFileTool::new(file_config.clone())));
        registry.register(Arc::new(ListFilesTool::new(file_config.clone())));
        registry.register(Arc::new(EditFileTool::new(file_config)));
    }

    // Task management
    let todo_tool = TodoListTool::new(todo_state_manager);
    registry.register(Arc::new(todo_tool));

    // Character read tools
    registry.register(Arc::new(ListPlayersTool::new(db_service.clone())));
    registry.register(Arc::new(GetCharacterTool::new(db_service.clone())));
    registry.register(Arc::new(ListCampaignCharactersTool::new(db_service.clone())));
    registry.register(Arc::new(ListNpcsTool::new(db_service.clone())));
    registry.register(Arc::new(ListPcsTool::new(db_service.clone())));
    registry.register(Arc::new(GetCharacterStatsTool::new(db_service.clone())));
    registry.register(Arc::new(CheckSpellSlotsTool::new(db_service.clone())));

    // Character write tools
    registry.register(Arc::new(CreateCharacterTool::new(db_service.clone())));
    registry.register(Arc::new(CreateNpcTool::new(db_service.clone())));
    registry.register(Arc::new(UpdateCharacterHpTool::new(db_service.clone())));
    registry.register(Arc::new(AddInventoryItemTool::new(db_service.clone())));
    registry.register(Arc::new(RemoveInventoryItemTool::new(db_service.clone())));
    registry.register(Arc::new(UpdateCharacterTool::new(db_service.clone())));
    registry.register(Arc::new(LevelUpTool::new(db_service.clone())));
    registry.register(Arc::new(CastSpellTool::new(db_service.clone())));
    registry.register(Arc::new(TakeRestTool::new(db_service.clone())));
    registry.register(Arc::new(UpdateEquippedTool::new(db_service.clone())));
    registry.register(Arc::new(UpdateCurrencyTool::new(db_service.clone())));

    // Module management tools
    registry.register(Arc::new(CreateModuleTool::new(db_service.clone())));
    registry.register(Arc::new(ListModulesTool::new(db_service.clone())));
    registry.register(Arc::new(GetModuleTool::new(db_service.clone())));
    registry.register(Arc::new(UpdateModuleStatusTool::new(db_service.clone())));

    // Catalog search tools
    registry.register(Arc::new(SearchMonstersTool::new(db_service.clone())));
    registry.register(Arc::new(SearchSpellsTool::new(db_service.clone())));
    registry.register(Arc::new(SearchItemsTool::new(db_service)));

    // Note: Campaign summary is NOT registered as an LLM tool.
    // Story summaries are auto-generated and injected as context during chat processing.
    // See chat_processor.rs build_campaign_context() for the auto-regeneration logic.
}

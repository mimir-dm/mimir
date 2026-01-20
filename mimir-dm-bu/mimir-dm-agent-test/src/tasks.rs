//! Task definitions for agent testing
//!
//! Extends the basic eval task format with verification rules

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// An agent task with verification rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Unique task identifier
    pub id: String,

    /// Description of what the task tests
    pub description: String,

    /// The prompt to send to the agent (for single-turn tasks)
    #[serde(default)]
    pub prompt: String,

    /// Multi-turn conversation (for multi-turn tasks)
    /// Each turn contains a user prompt and optional verification
    #[serde(default)]
    pub turns: Vec<ConversationTurn>,

    /// Expected tools to be called (optional - for checking tool selection)
    #[serde(default)]
    pub expected_tools: Vec<String>,

    /// Setup actions to run before the task (e.g., create test data)
    #[serde(default)]
    pub setup: Vec<SetupAction>,

    /// Verification rules to check after task completion (for single-turn)
    #[serde(default)]
    pub verify: Option<Vec<Verification>>,

    /// Maximum time allowed for task completion (seconds)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Tags for filtering/categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A single turn in a multi-turn conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// The user's message for this turn
    pub prompt: String,

    /// Optional verification to run after this turn
    #[serde(default)]
    pub verify: Option<Vec<Verification>>,

    /// Expected tools for this turn
    #[serde(default)]
    pub expected_tools: Vec<String>,
}

fn default_timeout() -> u64 {
    120
}

/// Setup action to run before a task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SetupAction {
    /// Create a character for testing
    CreateCharacter {
        name: String,
        class: String,
        level: i32,
        race: Option<String>,
    },
    /// Create an NPC for testing
    CreateNpc {
        name: String,
        #[serde(default)]
        class: Option<String>,
        #[serde(default)]
        race: Option<String>,
    },
    /// Create a campaign for testing
    CreateCampaign { name: String },
    /// Run a SQL statement
    Sql { statement: String },
}

/// Verification rule to check after task completion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Verification {
    /// Check that a character exists with expected properties
    CharacterExists {
        name: String,
        #[serde(default)]
        expect: Option<CharacterExpectation>,
    },

    /// Check that a character was modified
    CharacterModified {
        name: String,
        field: String,
        expected_value: serde_json::Value,
    },

    /// Check that an item exists in character inventory
    InventoryContains {
        character_name: String,
        item_name: String,
        #[serde(default)]
        quantity: Option<i32>,
    },

    /// Check that a tool was called with specific arguments
    ToolCalled {
        tool_name: String,
        #[serde(default)]
        with_args: Option<serde_json::Value>,
    },

    /// Check response contains certain text
    ResponseContains { text: String },

    /// Check response does NOT contain certain text
    ResponseNotContains { text: String },

    /// Custom SQL query returns expected result
    SqlQuery {
        query: String,
        expect_rows: Option<i32>,
        expect_value: Option<serde_json::Value>,
    },

    /// Check that no errors occurred
    NoErrors,

    /// Use an LLM to judge the response against criteria
    /// This enables semantic evaluation rather than strict text matching
    LlmJudge {
        /// The criteria the LLM should evaluate against
        /// e.g., "Response confirms character creation and summarizes key stats"
        criteria: String,
        /// Optional additional context to provide to the judge
        #[serde(default)]
        context: Option<String>,
    },
}

/// Expected character properties
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterExpectation {
    pub class: Option<String>,
    pub level: Option<i32>,
    pub race: Option<String>,
    pub current_hp: Option<i32>,
    pub max_hp: Option<i32>,
    pub is_npc: Option<bool>,
}

/// A collection of tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTaskSet {
    /// Name of this task set
    pub name: String,
    /// Description
    pub description: String,
    /// The tasks
    pub tasks: Vec<AgentTask>,
}

impl AgentTaskSet {
    /// Load from a JSON file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let task_set: AgentTaskSet = serde_json::from_str(&content)?;
        Ok(task_set)
    }

    /// Load all task sets from a directory
    pub fn load_all(dir: &Path) -> Result<Vec<Self>> {
        let mut task_sets = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                match Self::from_file(&path) {
                    Ok(ts) => task_sets.push(ts),
                    Err(e) => {
                        tracing::warn!("Failed to load task set {:?}: {}", path, e);
                    }
                }
            }
        }
        Ok(task_sets)
    }
}

/// Create default task definitions
pub fn create_default_tasks(output_dir: &Path) -> Result<()> {
    // Character management tasks
    let character_tasks = AgentTaskSet {
        name: "Character Management".to_string(),
        description: "Tasks testing character creation and modification".to_string(),
        tasks: vec![
            AgentTask {
                id: "create_fighter".to_string(),
                description: "Create a basic fighter character and level it up".to_string(),
                prompt: "Create a level 1 human fighter named Gareth Stonewall for player Alice (player_id 1), then level him up to level 3. He should end up with 28 max HP.".to_string(),
                turns: vec![],
                expected_tools: vec!["create_character".to_string(), "level_up".to_string()],
                setup: vec![],
                verify: Some(vec![
                    Verification::CharacterExists {
                        name: "Gareth Stonewall".to_string(),
                        expect: Some(CharacterExpectation {
                            class: Some("Fighter".to_string()),
                            level: Some(3),
                            race: Some("Human".to_string()),
                            max_hp: Some(28),
                            ..Default::default()
                        }),
                    },
                    Verification::ToolCalled {
                        tool_name: "create_character".to_string(),
                        with_args: None,
                    },
                ]),
                timeout_secs: 120,
                tags: vec!["character".to_string(), "creation".to_string()],
            },
            AgentTask {
                id: "update_hp".to_string(),
                description: "Update character HP after taking damage".to_string(),
                prompt: "Bruenor just took 15 damage from a goblin attack. Update his HP.".to_string(),
                turns: vec![],
                expected_tools: vec!["update_character_hp".to_string()],
                setup: vec![SetupAction::CreateCharacter {
                    name: "Bruenor".to_string(),
                    class: "Fighter".to_string(),
                    level: 5,
                    race: Some("Dwarf".to_string()),
                }],
                verify: Some(vec![
                    Verification::ToolCalled {
                        tool_name: "update_character_hp".to_string(),
                        with_args: None,
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 60,
                tags: vec!["character".to_string(), "hp".to_string()],
            },
            AgentTask {
                id: "add_inventory".to_string(),
                description: "Add an item to character inventory".to_string(),
                prompt: "Add a Longsword to Tordek's inventory.".to_string(),
                turns: vec![],
                expected_tools: vec!["add_inventory_item".to_string()],
                setup: vec![SetupAction::CreateCharacter {
                    name: "Tordek".to_string(),
                    class: "Fighter".to_string(),
                    level: 4,
                    race: Some("Dwarf".to_string()),
                }],
                verify: Some(vec![
                    Verification::InventoryContains {
                        character_name: "Tordek".to_string(),
                        item_name: "Longsword".to_string(),
                        quantity: Some(1),
                    },
                ]),
                timeout_secs: 60,
                tags: vec!["character".to_string(), "inventory".to_string()],
            },
        ],
    };

    // Catalog query tasks
    let catalog_tasks = AgentTaskSet {
        name: "Catalog Queries".to_string(),
        description: "Tasks testing catalog search functionality".to_string(),
        tasks: vec![
            AgentTask {
                id: "search_monsters_cr".to_string(),
                description: "Search monsters by challenge rating".to_string(),
                prompt: "Find me some CR 1 monsters that would be good for a forest encounter.".to_string(),
                turns: vec![],
                expected_tools: vec!["search_monsters".to_string()],
                setup: vec![],
                verify: Some(vec![
                    Verification::ToolCalled {
                        tool_name: "search_monsters".to_string(),
                        with_args: None,
                    },
                    Verification::ResponseContains {
                        text: "CR".to_string(),
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 60,
                tags: vec!["catalog".to_string(), "monsters".to_string()],
            },
            AgentTask {
                id: "search_spells".to_string(),
                description: "Search spells by level".to_string(),
                prompt: "What 2nd level evocation spells are available?".to_string(),
                turns: vec![],
                expected_tools: vec!["search_spells".to_string()],
                setup: vec![],
                verify: Some(vec![
                    Verification::ToolCalled {
                        tool_name: "search_spells".to_string(),
                        with_args: None,
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 60,
                tags: vec!["catalog".to_string(), "spells".to_string()],
            },
        ],
    };

    // Edge case tasks
    let edge_cases = AgentTaskSet {
        name: "Edge Cases".to_string(),
        description: "Tasks testing boundary conditions and error handling".to_string(),
        tasks: vec![
            AgentTask {
                id: "no_tool_greeting".to_string(),
                description: "Simple greeting should not trigger tools".to_string(),
                prompt: "Hello! How are you today?".to_string(),
                turns: vec![],
                expected_tools: vec![],
                setup: vec![],
                verify: Some(vec![
                    Verification::ResponseNotContains {
                        text: "error".to_string(),
                    },
                ]),
                timeout_secs: 30,
                tags: vec!["edge_case".to_string()],
            },
            AgentTask {
                id: "ambiguous_character".to_string(),
                description: "Ambiguous request should ask for clarification".to_string(),
                prompt: "Update the character.".to_string(),
                turns: vec![],
                expected_tools: vec![],
                setup: vec![],
                verify: Some(vec![Verification::NoErrors]),
                timeout_secs: 30,
                tags: vec!["edge_case".to_string()],
            },
        ],
    };

    // Multi-step tasks
    let multi_step = AgentTaskSet {
        name: "Multi-Step Tasks".to_string(),
        description: "Tasks requiring multiple tool calls".to_string(),
        tasks: vec![
            AgentTask {
                id: "create_and_equip".to_string(),
                description: "Create character and add equipment".to_string(),
                prompt: "Create a level 1 elf wizard named Lyra Starweaver with the Sage background for player Bob (player_id 2), then add a quarterstaff and a spellbook to her inventory.".to_string(),
                turns: vec![],
                expected_tools: vec![
                    "create_character".to_string(),
                    "add_inventory_item".to_string(),
                ],
                setup: vec![],
                verify: Some(vec![
                    Verification::CharacterExists {
                        name: "Lyra Starweaver".to_string(),
                        expect: Some(CharacterExpectation {
                            class: Some("Wizard".to_string()),
                            level: Some(1),
                            ..Default::default()
                        }),
                    },
                    Verification::InventoryContains {
                        character_name: "Lyra Starweaver".to_string(),
                        item_name: "Quarterstaff".to_string(),
                        quantity: None,
                    },
                ]),
                timeout_secs: 120,
                tags: vec!["multi_step".to_string()],
            },
            // Multi-turn conversation test
            AgentTask {
                id: "multi_turn_character_flow".to_string(),
                description: "Multi-turn: Create character then modify it".to_string(),
                prompt: String::new(),  // Empty for multi-turn
                turns: vec![
                    ConversationTurn {
                        prompt: "Create a level 1 half-orc barbarian named Grok the Mighty for player Charlie (player_id 3), then level it up to 2.".to_string(),
                        verify: Some(vec![
                            Verification::CharacterExists {
                                name: "Grok".to_string(),
                                expect: Some(CharacterExpectation {
                                    class: Some("Barbarian".to_string()),
                                    level: Some(2),
                                    ..Default::default()
                                }),
                            },
                        ]),
                        expected_tools: vec!["create_character".to_string(), "level_up".to_string()],
                    },
                    ConversationTurn {
                        prompt: "Give Grok a greataxe.".to_string(),
                        verify: Some(vec![
                            Verification::InventoryContains {
                                character_name: "Grok".to_string(),
                                item_name: "Greataxe".to_string(),
                                quantity: None,
                            },
                        ]),
                        expected_tools: vec!["add_inventory_item".to_string()],
                    },
                    ConversationTurn {
                        prompt: "Grok takes 8 damage from a trap.".to_string(),
                        verify: Some(vec![
                            Verification::ToolCalled {
                                tool_name: "update_character_hp".to_string(),
                                with_args: None,
                            },
                        ]),
                        expected_tools: vec!["update_character_hp".to_string()],
                    },
                ],
                expected_tools: vec![],
                setup: vec![],
                verify: None,
                timeout_secs: 180,
                tags: vec!["multi_turn".to_string(), "character".to_string()],
            },
            // Multi-turn module planning
            AgentTask {
                id: "multi_turn_module_planning".to_string(),
                description: "Multi-turn: Plan and discuss a module".to_string(),
                prompt: String::new(),
                turns: vec![
                    ConversationTurn {
                        prompt: "I want to create a mystery adventure for the party. Let's call it 'The Missing Merchant'. It should be about 3 sessions.".to_string(),
                        verify: Some(vec![
                            Verification::ToolCalled {
                                tool_name: "create_module".to_string(),
                                with_args: None,
                            },
                        ]),
                        expected_tools: vec!["create_module".to_string()],
                    },
                    ConversationTurn {
                        prompt: "What modules do we have now?".to_string(),
                        verify: Some(vec![
                            Verification::ToolCalled {
                                tool_name: "list_modules".to_string(),
                                with_args: None,
                            },
                        ]),
                        expected_tools: vec!["list_modules".to_string()],
                    },
                ],
                expected_tools: vec![],
                setup: vec![],
                verify: None,
                timeout_secs: 120,
                tags: vec!["multi_turn".to_string(), "module".to_string()],
            },
        ],
    };

    // Module/Adventure planning tasks
    let module_tasks = AgentTaskSet {
        name: "Module Planning".to_string(),
        description: "Tasks testing adventure module creation and management".to_string(),
        tasks: vec![
            AgentTask {
                id: "create_dungeon_module".to_string(),
                description: "Create a new dungeon crawl adventure module".to_string(),
                prompt: "I want to plan a dungeon crawl adventure called 'The Sunken Citadel'. It should take about 4 sessions.".to_string(),
                turns: vec![],
                expected_tools: vec!["create_module".to_string()],
                setup: vec![],
                verify: Some(vec![
                    Verification::ToolCalled {
                        tool_name: "create_module".to_string(),
                        with_args: None,
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 60,
                tags: vec!["module".to_string(), "creation".to_string()],
            },
            AgentTask {
                id: "list_campaign_modules".to_string(),
                description: "List all modules in the campaign".to_string(),
                prompt: "What adventure modules do we have planned for this campaign?".to_string(),
                turns: vec![],
                expected_tools: vec!["list_modules".to_string()],
                setup: vec![],
                verify: Some(vec![
                    Verification::ToolCalled {
                        tool_name: "list_modules".to_string(),
                        with_args: None,
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 60,
                tags: vec!["module".to_string(), "query".to_string()],
            },
        ],
    };

    // Continuity and context questions
    let continuity_tasks = AgentTaskSet {
        name: "Campaign Continuity".to_string(),
        description: "Tasks testing story continuity and campaign context awareness".to_string(),
        tasks: vec![
            AgentTask {
                id: "npc_question".to_string(),
                description: "Ask about an NPC from the campaign".to_string(),
                prompt: "Who is Glasstaff and what do we know about him?".to_string(),
                turns: vec![],
                expected_tools: vec![],  // Should answer from context
                setup: vec![],
                verify: Some(vec![
                    Verification::LlmJudge {
                        criteria: "Response correctly identifies Glasstaff as Iarno Albrek, mentions his connection to the Redbrands gang, and provides relevant context about his role in the campaign.".to_string(),
                        context: Some("Glasstaff is the alias of Iarno Albrek, the leader of the Redbrand ruffians in Lost Mine of Phandelver.".to_string()),
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 30,
                tags: vec!["continuity".to_string(), "npc".to_string()],
            },
            AgentTask {
                id: "story_recap".to_string(),
                description: "Ask for a story recap".to_string(),
                prompt: "Can you remind me what the party's current objectives are?".to_string(),
                turns: vec![],
                expected_tools: vec![],  // Should answer from context
                setup: vec![],
                verify: Some(vec![
                    Verification::LlmJudge {
                        criteria: "Response provides a helpful summary of the party's current objectives, referencing the campaign context appropriately.".to_string(),
                        context: None,
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 30,
                tags: vec!["continuity".to_string(), "story".to_string()],
            },
            AgentTask {
                id: "party_status".to_string(),
                description: "Ask about party composition".to_string(),
                prompt: "Who is our party's main healer?".to_string(),
                turns: vec![],
                expected_tools: vec![],  // Could use list_characters or answer from context
                setup: vec![],
                verify: Some(vec![
                    Verification::LlmJudge {
                        criteria: "Response correctly identifies the party's healer or healing-capable character based on the campaign context.".to_string(),
                        context: None,
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 30,
                tags: vec!["continuity".to_string(), "party".to_string()],
            },
        ],
    };

    // NPC Management tasks
    let npc_tasks = AgentTaskSet {
        name: "NPC Management".to_string(),
        description: "Tasks testing NPC creation and management".to_string(),
        tasks: vec![
            AgentTask {
                id: "list_npcs".to_string(),
                description: "List all NPCs in the campaign".to_string(),
                prompt: "Show me all the NPCs in this campaign.".to_string(),
                turns: vec![],
                expected_tools: vec!["list_npcs".to_string()],
                setup: vec![
                    SetupAction::CreateNpc {
                        name: "Barthen".to_string(),
                        class: None,
                        race: Some("Human".to_string()),
                    },
                    SetupAction::CreateNpc {
                        name: "Sildar Hallwinter".to_string(),
                        class: Some("Fighter".to_string()),
                        race: Some("Human".to_string()),
                    },
                ],
                verify: Some(vec![
                    Verification::ToolCalled {
                        tool_name: "list_npcs".to_string(),
                        with_args: None,
                    },
                    Verification::ResponseContains {
                        text: "Barthen".to_string(),
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 60,
                tags: vec!["npc".to_string(), "query".to_string()],
            },
            AgentTask {
                id: "list_pcs".to_string(),
                description: "List player characters separately from NPCs".to_string(),
                prompt: "Who are the player characters in the party?".to_string(),
                turns: vec![],
                expected_tools: vec!["list_player_characters".to_string()],
                setup: vec![
                    SetupAction::CreateCharacter {
                        name: "Alayna".to_string(),
                        class: "Cleric".to_string(),
                        level: 3,
                        race: Some("Elf".to_string()),
                    },
                    SetupAction::CreateNpc {
                        name: "Gundren Rockseeker".to_string(),
                        class: None,
                        race: Some("Dwarf".to_string()),
                    },
                ],
                verify: Some(vec![
                    Verification::ToolCalled {
                        tool_name: "list_player_characters".to_string(),
                        with_args: None,
                    },
                    Verification::ResponseContains {
                        text: "Alayna".to_string(),
                    },
                    Verification::ResponseNotContains {
                        text: "Gundren".to_string(),
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 60,
                tags: vec!["character".to_string(), "query".to_string()],
            },
            AgentTask {
                id: "create_npc".to_string(),
                description: "Create a new NPC for the campaign".to_string(),
                prompt: "Create a human cleric NPC named 'Sister Garaele' who works at the Shrine of Luck in Phandalin.".to_string(),
                turns: vec![],
                expected_tools: vec!["create_npc".to_string()],
                setup: vec![],
                verify: Some(vec![
                    Verification::CharacterExists {
                        name: "Sister Garaele".to_string(),
                        expect: Some(CharacterExpectation {
                            race: Some("Human".to_string()),
                            is_npc: Some(true),
                            ..Default::default()
                        }),
                    },
                ]),
                timeout_secs: 60,
                tags: vec!["npc".to_string(), "creation".to_string()],
            },
            AgentTask {
                id: "npc_vs_pc_distinction".to_string(),
                description: "Correctly distinguish between NPCs and player characters".to_string(),
                prompt: "Can you list the NPCs and player characters separately? I want to see who's who.".to_string(),
                turns: vec![],
                expected_tools: vec!["list_npcs".to_string(), "list_player_characters".to_string()],
                setup: vec![
                    SetupAction::CreateCharacter {
                        name: "Theron".to_string(),
                        class: "Rogue".to_string(),
                        level: 4,
                        race: Some("Halfling".to_string()),
                    },
                    SetupAction::CreateNpc {
                        name: "Toblen Stonehill".to_string(),
                        class: None,
                        race: Some("Human".to_string()),
                    },
                ],
                verify: Some(vec![
                    Verification::ToolCalled {
                        tool_name: "list_npcs".to_string(),
                        with_args: None,
                    },
                    Verification::ToolCalled {
                        tool_name: "list_player_characters".to_string(),
                        with_args: None,
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 90,
                tags: vec!["npc".to_string(), "character".to_string(), "query".to_string()],
            },
            // Multi-turn NPC scenario
            AgentTask {
                id: "multi_turn_npc_management".to_string(),
                description: "Multi-turn: Create and then query NPCs".to_string(),
                prompt: String::new(),
                turns: vec![
                    ConversationTurn {
                        prompt: "Create an NPC named 'Daran Edermath' who is a retired half-elf adventurer running an orchard.".to_string(),
                        verify: Some(vec![
                            Verification::CharacterExists {
                                name: "Daran Edermath".to_string(),
                                expect: Some(CharacterExpectation {
                                    race: Some("Half-Elf".to_string()),
                                    is_npc: Some(true),
                                    ..Default::default()
                                }),
                            },
                        ]),
                        expected_tools: vec!["create_npc".to_string()],
                    },
                    ConversationTurn {
                        prompt: "Now create another NPC - 'Halia Thornton' who is a human merchant and the guildmaster of the Miner's Exchange.".to_string(),
                        verify: Some(vec![
                            Verification::CharacterExists {
                                name: "Halia Thornton".to_string(),
                                expect: Some(CharacterExpectation {
                                    race: Some("Human".to_string()),
                                    is_npc: Some(true),
                                    ..Default::default()
                                }),
                            },
                        ]),
                        expected_tools: vec!["create_npc".to_string()],
                    },
                    ConversationTurn {
                        prompt: "Great! Now show me all the NPCs we've created.".to_string(),
                        verify: Some(vec![
                            Verification::ToolCalled {
                                tool_name: "list_npcs".to_string(),
                                with_args: None,
                            },
                            Verification::ResponseContains {
                                text: "Daran".to_string(),
                            },
                            Verification::ResponseContains {
                                text: "Halia".to_string(),
                            },
                        ]),
                        expected_tools: vec!["list_npcs".to_string()],
                    },
                ],
                expected_tools: vec![],
                setup: vec![],
                verify: None,
                timeout_secs: 180,
                tags: vec!["multi_turn".to_string(), "npc".to_string()],
            },
        ],
    };

    // D&D Knowledge questions (no tools needed)
    let knowledge_tasks = AgentTaskSet {
        name: "D&D Knowledge".to_string(),
        description: "Tasks testing D&D rules knowledge without requiring tools".to_string(),
        tasks: vec![
            AgentTask {
                id: "spell_slots".to_string(),
                description: "Ask about spell slot rules".to_string(),
                prompt: "How many spell slots does a 5th level wizard have?".to_string(),
                turns: vec![],
                expected_tools: vec![],
                setup: vec![],
                verify: Some(vec![
                    Verification::LlmJudge {
                        criteria: "Response correctly lists the spell slots for a 5th level wizard: 4 first-level, 3 second-level, and 2 third-level slots.".to_string(),
                        context: Some("A 5th level wizard has 4/3/2 spell slots (1st/2nd/3rd level).".to_string()),
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 30,
                tags: vec!["knowledge".to_string(), "spellcasting".to_string()],
            },
            AgentTask {
                id: "action_economy".to_string(),
                description: "Ask about combat actions".to_string(),
                prompt: "What actions can a character take on their turn in combat?".to_string(),
                turns: vec![],
                expected_tools: vec![],
                setup: vec![],
                verify: Some(vec![
                    Verification::LlmJudge {
                        criteria: "Response correctly explains D&D 5e action economy, mentioning action, bonus action, movement, free object interaction, and optionally reaction.".to_string(),
                        context: None,
                    },
                    Verification::NoErrors,
                ]),
                timeout_secs: 30,
                tags: vec!["knowledge".to_string(), "combat".to_string()],
            },
            // NOTE: condition_rules test removed - requires RAG pipeline for accurate D&D rules
            // Don't rely on LLM's baked-in knowledge for official game rules
        ],
    };

    // Save task sets
    std::fs::write(
        output_dir.join("character_tasks.json"),
        serde_json::to_string_pretty(&character_tasks)?,
    )?;
    std::fs::write(
        output_dir.join("catalog_tasks.json"),
        serde_json::to_string_pretty(&catalog_tasks)?,
    )?;
    std::fs::write(
        output_dir.join("edge_cases.json"),
        serde_json::to_string_pretty(&edge_cases)?,
    )?;
    std::fs::write(
        output_dir.join("multi_step.json"),
        serde_json::to_string_pretty(&multi_step)?,
    )?;
    std::fs::write(
        output_dir.join("module_tasks.json"),
        serde_json::to_string_pretty(&module_tasks)?,
    )?;
    std::fs::write(
        output_dir.join("continuity_tasks.json"),
        serde_json::to_string_pretty(&continuity_tasks)?,
    )?;
    std::fs::write(
        output_dir.join("npc_tasks.json"),
        serde_json::to_string_pretty(&npc_tasks)?,
    )?;
    std::fs::write(
        output_dir.join("knowledge_tasks.json"),
        serde_json::to_string_pretty(&knowledge_tasks)?,
    )?;

    let total_tasks = character_tasks.tasks.len() + catalog_tasks.tasks.len() +
        edge_cases.tasks.len() + multi_step.tasks.len() +
        module_tasks.tasks.len() + continuity_tasks.tasks.len() +
        npc_tasks.tasks.len() + knowledge_tasks.tasks.len();

    println!("Created 8 task files with {} total tasks", total_tasks);

    Ok(())
}

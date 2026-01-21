//! Document Templates
//!
//! Embeds campaign and module document templates at compile time.
//! These templates provide initial content for documents created when
//! campaigns and modules are set up.

// =============================================================================
// Campaign Templates
// =============================================================================

/// Campaign pitch template - the initial hook and premise.
pub const CAMPAIGN_PITCH: &str = include_str!("campaign_pitch.md");

/// Starting scenario template - where the adventure begins.
pub const STARTING_SCENARIO: &str = include_str!("starting_scenario.md");

/// World primer template - essential setting information for players.
pub const WORLD_PRIMER: &str = include_str!("world_primer.md");

/// Character guidelines template - rules and options for character creation.
pub const CHARACTER_GUIDELINES: &str = include_str!("character_guidelines.md");

/// Table expectations template - social contract and gameplay norms.
pub const TABLE_EXPECTATIONS: &str = include_str!("table_expectations.md");

/// Character integration template - weaving PCs into the story.
pub const CHARACTER_INTEGRATION: &str = include_str!("character_integration.md");

/// Campaign bible template - the DM's reference for campaign lore.
pub const CAMPAIGN_BIBLE: &str = include_str!("campaign_bible.md");

/// Safety tools template - lines, veils, and safety mechanics.
pub const SAFETY_TOOLS: &str = include_str!("safety_tools.md");

/// House rules template - custom rules and rulings.
pub const HOUSE_RULES: &str = include_str!("house_rules.md");

/// Player secrets template - information known only to specific players.
pub const PLAYER_SECRETS: &str = include_str!("player_secrets.md");

/// Faction overview template - major factions and their relationships.
pub const FACTION_OVERVIEW: &str = include_str!("faction_overview.md");

// =============================================================================
// Module Templates
// =============================================================================

/// General module overview template - works for any adventure type.
pub const MODULE_OVERVIEW: &str = include_str!("module_overview.md");

/// Mystery module template - investigation and clue-based adventures.
pub const MODULE_MYSTERY: &str = include_str!("module_mystery.md");

/// Dungeon crawl template - exploration and combat-focused adventures.
pub const MODULE_DUNGEON: &str = include_str!("module_dungeon.md");

/// Heist module template - planning and execution of complex operations.
pub const MODULE_HEIST: &str = include_str!("module_heist.md");

/// Horror module template - tension, dread, and survival.
pub const MODULE_HORROR: &str = include_str!("module_horror.md");

/// Political intrigue template - factions, diplomacy, and scheming.
pub const MODULE_POLITICAL: &str = include_str!("module_political.md");

/// Play notes template - session-by-session DM notes.
pub const PLAY_NOTES: &str = include_str!("play_notes.md");

// =============================================================================
// Template Access Functions
// =============================================================================

/// Campaign document type with its template and display title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CampaignTemplateInfo {
    /// The document type identifier (e.g., "campaign_pitch").
    pub doc_type: &'static str,
    /// Human-readable title (e.g., "Campaign Pitch").
    pub title: &'static str,
    /// The template content.
    pub content: &'static str,
}

/// Module type with its template and display title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleTemplateInfo {
    /// The module type identifier (e.g., "mystery").
    pub module_type: &'static str,
    /// Human-readable title (e.g., "Mystery Module").
    pub title: &'static str,
    /// The template content.
    pub content: &'static str,
}

/// All campaign templates with metadata.
pub const CAMPAIGN_TEMPLATES: &[CampaignTemplateInfo] = &[
    CampaignTemplateInfo {
        doc_type: "campaign_pitch",
        title: "Campaign Pitch",
        content: CAMPAIGN_PITCH,
    },
    CampaignTemplateInfo {
        doc_type: "starting_scenario",
        title: "Starting Scenario",
        content: STARTING_SCENARIO,
    },
    CampaignTemplateInfo {
        doc_type: "world_primer",
        title: "World Primer",
        content: WORLD_PRIMER,
    },
    CampaignTemplateInfo {
        doc_type: "character_guidelines",
        title: "Character Guidelines",
        content: CHARACTER_GUIDELINES,
    },
    CampaignTemplateInfo {
        doc_type: "table_expectations",
        title: "Table Expectations",
        content: TABLE_EXPECTATIONS,
    },
    CampaignTemplateInfo {
        doc_type: "character_integration",
        title: "Character Integration",
        content: CHARACTER_INTEGRATION,
    },
    CampaignTemplateInfo {
        doc_type: "campaign_bible",
        title: "Campaign Bible",
        content: CAMPAIGN_BIBLE,
    },
    CampaignTemplateInfo {
        doc_type: "safety_tools",
        title: "Safety Tools",
        content: SAFETY_TOOLS,
    },
    CampaignTemplateInfo {
        doc_type: "house_rules",
        title: "House Rules",
        content: HOUSE_RULES,
    },
    CampaignTemplateInfo {
        doc_type: "player_secrets",
        title: "Player Secrets",
        content: PLAYER_SECRETS,
    },
    CampaignTemplateInfo {
        doc_type: "faction_overview",
        title: "Faction Overview",
        content: FACTION_OVERVIEW,
    },
];

/// All module templates with metadata.
pub const MODULE_TEMPLATES: &[ModuleTemplateInfo] = &[
    ModuleTemplateInfo {
        module_type: "general",
        title: "Module Overview",
        content: MODULE_OVERVIEW,
    },
    ModuleTemplateInfo {
        module_type: "mystery",
        title: "Mystery Module",
        content: MODULE_MYSTERY,
    },
    ModuleTemplateInfo {
        module_type: "dungeon",
        title: "Dungeon Crawl",
        content: MODULE_DUNGEON,
    },
    ModuleTemplateInfo {
        module_type: "heist",
        title: "Heist Module",
        content: MODULE_HEIST,
    },
    ModuleTemplateInfo {
        module_type: "horror",
        title: "Horror Module",
        content: MODULE_HORROR,
    },
    ModuleTemplateInfo {
        module_type: "political",
        title: "Political Intrigue",
        content: MODULE_POLITICAL,
    },
];

/// Get a campaign template by document type.
///
/// # Arguments
/// * `doc_type` - The document type identifier (e.g., "campaign_pitch", "world_primer")
///
/// # Returns
/// The template content if found, or `None` if the document type is unknown.
pub fn get_campaign_template(doc_type: &str) -> Option<&'static str> {
    match doc_type {
        "campaign_pitch" => Some(CAMPAIGN_PITCH),
        "starting_scenario" => Some(STARTING_SCENARIO),
        "world_primer" => Some(WORLD_PRIMER),
        "character_guidelines" => Some(CHARACTER_GUIDELINES),
        "table_expectations" => Some(TABLE_EXPECTATIONS),
        "character_integration" => Some(CHARACTER_INTEGRATION),
        "campaign_bible" => Some(CAMPAIGN_BIBLE),
        "safety_tools" => Some(SAFETY_TOOLS),
        "house_rules" => Some(HOUSE_RULES),
        "player_secrets" => Some(PLAYER_SECRETS),
        "faction_overview" => Some(FACTION_OVERVIEW),
        _ => None,
    }
}

/// Get a module template by module type.
///
/// # Arguments
/// * `module_type` - The module type (e.g., "general", "mystery", "dungeon")
///
/// # Returns
/// The template content if found, or `None` if the module type is unknown.
pub fn get_module_template(module_type: &str) -> Option<&'static str> {
    match module_type {
        "general" => Some(MODULE_OVERVIEW),
        "mystery" => Some(MODULE_MYSTERY),
        "dungeon" => Some(MODULE_DUNGEON),
        "heist" => Some(MODULE_HEIST),
        "horror" => Some(MODULE_HORROR),
        "political" => Some(MODULE_POLITICAL),
        _ => None,
    }
}

/// Get the play notes template.
///
/// Play notes are used for session-by-session DM notes within a module.
pub fn get_play_notes_template() -> &'static str {
    PLAY_NOTES
}

/// List all campaign templates with their types and titles.
///
/// # Returns
/// A vector of (doc_type, title) pairs for all campaign templates.
pub fn list_campaign_templates() -> Vec<(&'static str, &'static str)> {
    CAMPAIGN_TEMPLATES
        .iter()
        .map(|t| (t.doc_type, t.title))
        .collect()
}

/// List all module templates with their types and titles.
///
/// # Returns
/// A vector of (module_type, title) pairs for all module templates.
pub fn list_module_templates() -> Vec<(&'static str, &'static str)> {
    MODULE_TEMPLATES
        .iter()
        .map(|t| (t.module_type, t.title))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_templates_count() {
        assert_eq!(CAMPAIGN_TEMPLATES.len(), 11);
    }

    #[test]
    fn test_module_templates_count() {
        assert_eq!(MODULE_TEMPLATES.len(), 6);
    }

    #[test]
    fn test_get_campaign_template() {
        assert!(get_campaign_template("campaign_pitch").is_some());
        assert!(get_campaign_template("world_primer").is_some());
        assert!(get_campaign_template("nonexistent").is_none());
    }

    #[test]
    fn test_get_module_template() {
        assert!(get_module_template("general").is_some());
        assert!(get_module_template("mystery").is_some());
        assert!(get_module_template("dungeon").is_some());
        assert!(get_module_template("nonexistent").is_none());
    }

    #[test]
    fn test_play_notes_template() {
        let template = get_play_notes_template();
        assert!(!template.is_empty());
        assert!(template.contains("Play Notes"));
    }

    #[test]
    fn test_campaign_templates_not_empty() {
        for template in CAMPAIGN_TEMPLATES {
            assert!(
                !template.content.is_empty(),
                "Campaign template '{}' is empty",
                template.doc_type
            );
            assert!(
                template.content.len() > 100,
                "Campaign template '{}' seems too short ({} bytes)",
                template.doc_type,
                template.content.len()
            );
        }
    }

    #[test]
    fn test_module_templates_not_empty() {
        for template in MODULE_TEMPLATES {
            assert!(
                !template.content.is_empty(),
                "Module template '{}' is empty",
                template.module_type
            );
            assert!(
                template.content.len() > 100,
                "Module template '{}' seems too short ({} bytes)",
                template.module_type,
                template.content.len()
            );
        }
    }

    #[test]
    fn test_list_campaign_templates() {
        let templates = list_campaign_templates();
        assert_eq!(templates.len(), 11);
        assert!(templates.contains(&("campaign_pitch", "Campaign Pitch")));
        assert!(templates.contains(&("world_primer", "World Primer")));
    }

    #[test]
    fn test_list_module_templates() {
        let templates = list_module_templates();
        assert_eq!(templates.len(), 6);
        assert!(templates.contains(&("general", "Module Overview")));
        assert!(templates.contains(&("mystery", "Mystery Module")));
    }
}

//! Template seeder for initial campaign templates

use crate::dal::campaign::template_documents::TemplateRepository;
use crate::models::campaign::template_documents::NewTemplateDocument;
use crate::models::campaign::template_frontmatter::TemplateFrontmatter;
use diesel::prelude::*;

/// Macro to define templates and their content in one place
macro_rules! define_templates {
    (
        $(
            $const_name:ident => $file_name:literal
        ),* $(,)?
    ) => {
        // Generate include_str! constants for each template
        $(
            const $const_name: &str = include_str!(
                concat!("../../../../docs/src/campaign-framework/06-templates/templates/", $file_name)
            );
        )*

        /// Get template content by filename
        fn get_template_content(file_name: &str) -> Option<&'static str> {
            match file_name {
                $(
                    $file_name => Some($const_name),
                )*
                _ => None,
            }
        }

        /// Get all template filenames
        fn get_all_template_files() -> Vec<&'static str> {
            vec![
                $(
                    $file_name,
                )*
            ]
        }
    };
}

// Define all templates and their content using the macro
define_templates! {
    // Campaign Board Documents - Core campaign setup
    CAMPAIGN_BIBLE => "campaign_bible.md",
    CAMPAIGN_PITCH => "campaign_pitch.md",
    STARTING_SCENARIO => "starting_scenario.md",
    QUICK_START_KIT => "quick_start_kit.md",
    CHARACTER_INTEGRATION => "character_integration.md",
    CHARACTER_GUIDELINES => "character_guidelines.md",
    TABLE_EXPECTATIONS => "table_expectations.md",
    MAJOR_NPC_TRACKER => "major_npc_tracker.md",
    QUICK_NPC_REFERENCE => "quick_npc_reference.md",
    PC_ARC_TRACKER => "pc_arc_tracker.md",
    WORLD_PRIMER => "world_primer.md",
    REGION_OVERVIEW => "region_overview.md",
    FACTION_TEMPLATE => "faction_template.md",

    // Module Board Documents - Module-specific templates
    MODULE_OVERVIEW => "module_overview.md",
    MODULE_DUNGEON => "module_dungeon.md",
    MODULE_HEIST => "module_heist.md",
    MODULE_HORROR => "module_horror.md",
    MODULE_MYSTERY => "module_mystery.md",
    MODULE_POLITICAL => "module_political.md",

    // Play Documents - Capture session outcomes
    PLAY_NOTES => "play_notes.md",
    CLUE_TRACKER => "clue_tracker.md",
    DOCUMENT_TRACKER => "document_tracker.md",
}

/// Seed the database with initial templates
pub fn seed_templates(conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
    let mut count = 0;

    for file_name in get_all_template_files() {
        // Get the template content using the generated function
        let raw_content = get_template_content(file_name).ok_or_else(|| {
            diesel::result::Error::QueryBuilderError(
                format!("Unknown template file: {}", file_name).into(),
            )
        })?;

        // Parse frontmatter from the file - required
        let frontmatter =
            TemplateFrontmatter::parse_from_markdown(raw_content).ok_or_else(|| {
                diesel::result::Error::QueryBuilderError(
                    format!(
                        "Template file {} is missing required frontmatter",
                        file_name
                    )
                    .into(),
                )
            })?;

        // Extract content without frontmatter
        let content = TemplateFrontmatter::extract_content(raw_content);

        // Create the template document
        let new_template = NewTemplateDocument {
            document_id: frontmatter.id.clone(),
            version_number: None, // Will auto-increment
            document_content: content,
            content_hash: None, // Will be computed by repository
            document_type: Some(frontmatter.template_type.clone()),
            document_level: Some(frontmatter.level.clone()),
            purpose: Some(frontmatter.purpose.clone()),
            variables_schema: Some(frontmatter.variables_schema().map_err(|e| {
                diesel::result::Error::QueryBuilderError(
                    format!("Failed to serialize variables schema: {}", e).into(),
                )
            })?),
            default_values: Some(serde_json::to_string(&frontmatter.defaults_map()).map_err(
                |e| {
                    diesel::result::Error::QueryBuilderError(
                        format!("Failed to serialize default values: {}", e).into(),
                    )
                },
            )?),
            is_active: Some(true),
            metadata: Some(frontmatter.to_json().map_err(|e| {
                diesel::result::Error::QueryBuilderError(
                    format!("Failed to serialize metadata: {}", e).into(),
                )
            })?),
        };

        // Check if template already exists
        let exists = TemplateRepository::get_latest(conn, &new_template.document_id).is_ok();

        if !exists {
            TemplateRepository::create(conn, new_template)?;
            count += 1;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::establish_connection;

    #[test]
    fn test_seed_templates() {
        let mut conn = establish_connection(":memory:").unwrap();

        // Run migrations
        crate::run_migrations(&mut conn).unwrap();

        // Seed templates
        let count = seed_templates(&mut conn).unwrap();
        assert_eq!(count, 22); // We have 22 templates

        // Verify a few templates exist
        let campaign_pitch = TemplateRepository::get_latest(&mut conn, "campaign_pitch").unwrap();
        assert_eq!(campaign_pitch.document_type.unwrap(), "campaign_pitch");
        assert_eq!(campaign_pitch.document_level.unwrap(), "campaign");

        let module_overview = TemplateRepository::get_latest(&mut conn, "module_overview").unwrap();
        assert_eq!(module_overview.document_type.unwrap(), "module_overview");
        assert_eq!(module_overview.document_level.unwrap(), "module");
    }
}

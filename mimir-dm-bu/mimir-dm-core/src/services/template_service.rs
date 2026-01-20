//! Template service for document generation
//!
//! This service handles template rendering and document generation

use crate::{
    connection::DbConnection,
    dal::campaign::campaigns::CampaignRepository,
    dal::campaign::template_documents::TemplateRepository,
    domain::{TemplateInfo, TemplateVariable},
    error::{DbError, Result},
    models::campaign::campaigns::Campaign,
    models::campaign::template_documents::TemplateDocument,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};
use tracing::debug;

/// Service for template rendering and document generation.
pub struct TemplateService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> TemplateService<'a> {
    /// Create a new template service
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Render a template for a campaign and return the rendered content
    pub fn render_template(
        &mut self,
        campaign_id: i32,
        template_id: &str,
        variables: HashMap<String, JsonValue>,
    ) -> Result<String> {
        // Get the campaign
        let mut campaign_repo = CampaignRepository::new(self.conn);
        let campaign = campaign_repo
            .find_by_id(campaign_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Campaign".to_string(),
                id: campaign_id.to_string(),
            })?;

        // Get the template
        let template = TemplateRepository::get_latest(self.conn, template_id)?;

        // Render the template content
        self.render_template_content(&template, &campaign, variables)
    }

    /// Generate and save a document from a template
    /// Returns the file path where the document was saved
    pub fn generate_document(
        &mut self,
        campaign_id: i32,
        template_id: &str,
        variables: HashMap<String, JsonValue>,
    ) -> Result<String> {
        // Get the campaign to find its directory
        let mut campaign_repo = CampaignRepository::new(self.conn);
        let campaign = campaign_repo
            .find_by_id(campaign_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Campaign".to_string(),
                id: campaign_id.to_string(),
            })?;

        // Render the template
        let rendered_content = self.render_template(campaign_id, template_id, variables)?;

        // Determine where to save it
        let file_path = self.determine_template_file_path(&campaign.directory_path, template_id);

        // Save to file
        let full_path = PathBuf::from(&file_path);

        // Create parent directory if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write the file
        fs::write(&full_path, rendered_content)?;

        debug!(path = %full_path.display(), "Generated document");
        Ok(file_path)
    }

    /// Get the file path where a template would be saved for a campaign
    pub fn get_template_file_path(
        &mut self,
        campaign_id: i32,
        template_id: &str,
    ) -> Result<String> {
        let mut campaign_repo = CampaignRepository::new(self.conn);
        let campaign = campaign_repo
            .find_by_id(campaign_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Campaign".to_string(),
                id: campaign_id.to_string(),
            })?;

        Ok(self.determine_template_file_path(&campaign.directory_path, template_id))
    }

    /// List all available templates
    pub fn list_templates(&mut self) -> Result<Vec<TemplateDocument>> {
        TemplateRepository::get_all_active(self.conn).map_err(|e| e.into())
    }

    /// List all available templates with parsed metadata and variables
    pub fn list_templates_with_details(&mut self) -> Result<Vec<TemplateInfo>> {
        let templates = self.list_templates()?;

        Ok(templates
            .into_iter()
            .map(|template| {
                // Parse variables from the variables_schema JSON
                let variables = match &template.variables_schema {
                    Some(schema_str) => serde_json::from_str::<Vec<JsonValue>>(schema_str)
                        .ok()
                        .map(|vars| {
                            vars.into_iter()
                                .filter_map(|v| {
                                    Some(TemplateVariable {
                                        name: v.get("name")?.as_str()?.to_string(),
                                        var_type: v
                                            .get("var_type")
                                            .or(v.get("type"))?
                                            .as_str()?
                                            .to_string(),
                                        description: v.get("description")?.as_str()?.to_string(),
                                        default: v.get("default")?.clone(),
                                        required: v
                                            .get("required")
                                            .and_then(|r| r.as_bool())
                                            .unwrap_or(true),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                    None => vec![],
                };

                // Parse title from metadata
                let title = template
                    .metadata
                    .as_ref()
                    .and_then(|m| serde_json::from_str::<JsonValue>(m).ok())
                    .and_then(|m| m.get("title")?.as_str().map(String::from))
                    .unwrap_or_else(|| "Untitled Template".to_string());

                TemplateInfo {
                    id: template.document_id,
                    title,
                    purpose: template
                        .purpose
                        .unwrap_or_else(|| "No purpose specified".to_string()),
                    level: template
                        .document_level
                        .unwrap_or_else(|| "unknown".to_string()),
                    template_type: template
                        .document_type
                        .unwrap_or_else(|| "unknown".to_string()),
                    variables,
                }
            })
            .collect())
    }

    /// Get a specific template
    pub fn get_template(&mut self, template_id: &str) -> Result<TemplateDocument> {
        TemplateRepository::get_latest(self.conn, template_id).map_err(|e| e.into())
    }

    /// Render template content with variables
    fn render_template_content(
        &self,
        template: &TemplateDocument,
        campaign: &Campaign,
        variables: HashMap<String, JsonValue>,
    ) -> Result<String> {
        // Create Tera context
        let mut context = Context::new();

        // Add campaign-level variables
        context.insert("campaign_name", &campaign.name);
        context.insert("campaign_status", &campaign.status);
        context.insert("campaign_directory", &campaign.directory_path);

        // Add user-provided variables
        for (key, value) in variables {
            context.insert(&key, &value);
        }

        // Render the template
        let mut tera = Tera::default();
        tera.add_raw_template(&template.document_id, &template.document_content)
            .map_err(|e| DbError::InvalidData(format!("Failed to parse template: {}", e)))?;

        let rendered = tera
            .render(&template.document_id, &context)
            .map_err(|e| DbError::InvalidData(format!("Failed to render template: {}", e)))?;

        Ok(rendered)
    }

    /// Determine where to save a template based on its ID
    fn determine_template_file_path(&self, campaign_dir: &str, template_id: &str) -> String {
        let campaign_path = Path::new(campaign_dir);

        let file_path = match template_id {
            // Campaign level documents
            "campaign-bible" => campaign_path.join("campaign_bible.md"),
            "campaign-pitch" => campaign_path.join("pitch.md"),
            "starting-scenario" => campaign_path.join("session_zero/starting_scenario.md"),
            "quick-start-kit" => campaign_path.join("quick_start_kit.md"),

            // World building
            "world-overview" => campaign_path.join("world/overview.md"),
            "region-overview" => campaign_path.join("regions/region_overview.md"),
            "faction-template" => campaign_path.join("world/factions/faction.md"),

            // Characters and NPCs
            "character-integration" => campaign_path.join("characters/character_integration.md"),
            "major-npc-tracker" => campaign_path.join("npcs/major_npcs.md"),
            "quick-npc-reference" => campaign_path.join("npcs/quick_reference.md"),
            "pc-arc-tracker" => campaign_path.join("characters/pc_arc_tracker.md"),

            // Session management
            "session-outline" => campaign_path.join("sessions/session_outline.md"),
            "clue-tracker" => campaign_path.join("sessions/clue_tracker.md"),
            "document-tracker" => campaign_path.join("document_tracker.md"),

            // Module templates
            "module-overview" => campaign_path.join("modules/module_overview.md"),
            template if template.starts_with("module-") => {
                campaign_path.join(format!("modules/{}.md", template))
            }

            // Default fallback
            _ => campaign_path.join(format!("{}.md", template_id)),
        };

        file_path.to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        connection::establish_connection, dal::campaign::campaigns::CampaignRepository,
        dal::campaign::template_documents::TemplateRepository,
        models::campaign::campaigns::NewCampaign,
        models::campaign::template_documents::NewTemplateDocument,
    };
    use tempfile::TempDir;

    #[test]
    fn test_determine_template_file_path() {
        let mut conn = establish_connection(":memory:").unwrap();
        let service = TemplateService::new(&mut conn);

        let path =
            service.determine_template_file_path("/home/user/campaigns/test", "campaign-bible");
        assert_eq!(path, "/home/user/campaigns/test/campaign_bible.md");

        let path =
            service.determine_template_file_path("/home/user/campaigns/test", "world-overview");
        assert_eq!(path, "/home/user/campaigns/test/world/overview.md");

        let path =
            service.determine_template_file_path("/home/user/campaigns/test", "module-custom");
        assert_eq!(path, "/home/user/campaigns/test/modules/module-custom.md");

        let path =
            service.determine_template_file_path("/home/user/campaigns/test", "faction-template");
        assert_eq!(path, "/home/user/campaigns/test/world/factions/faction.md");

        // Test default fallback
        let path =
            service.determine_template_file_path("/home/user/campaigns/test", "unknown-template");
        assert_eq!(path, "/home/user/campaigns/test/unknown-template.md");
    }

    #[test]
    fn test_render_template() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        // Create a test campaign
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "concept".to_string(),
                directory_path: "/test/path".to_string(),
            })
            .unwrap();

        // Create a test template
        let template = NewTemplateDocument {
            document_id: "test-template".to_string(),
            version_number: None,
            document_content:
                "# {{ campaign_name }}\n\nPlayer: {{ player_name }}\nLevel: {{ level }}".to_string(),
            content_hash: None,
            document_type: Some("test".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: Some("Test template".to_string()),
            variables_schema: Some(
                r#"[
                {"name": "player_name", "type": "string", "default": "Unknown"},
                {"name": "level", "type": "number", "default": 1}
            ]"#
                .to_string(),
            ),
            default_values: None,
            is_active: Some(true),
            metadata: None,
        };
        TemplateRepository::create(&mut conn, template).unwrap();

        // Test rendering
        let mut service = TemplateService::new(&mut conn);
        let mut variables = HashMap::new();
        variables.insert("player_name".to_string(), serde_json::json!("Alice"));
        variables.insert("level".to_string(), serde_json::json!(5));

        let rendered = service
            .render_template(campaign.id, "test-template", variables)
            .unwrap();

        assert!(rendered.contains("# Test Campaign"));
        assert!(rendered.contains("Player: Alice"));
        assert!(rendered.contains("Level: 5"));
    }

    #[test]
    fn test_render_template_with_missing_variables() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        // Create a test campaign
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "concept".to_string(),
                directory_path: "/test/path".to_string(),
            })
            .unwrap();

        // Create a template with variables
        let template = NewTemplateDocument {
            document_id: "test-template-2".to_string(),
            version_number: None,
            document_content: "Campaign: {{ campaign_name }}, Status: {{ campaign_status }}, Custom: {{ custom_var | default(value=\"N/A\") }}".to_string(),
            content_hash: None,
            document_type: Some("test".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: Some("Test template".to_string()),
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        };
        TemplateRepository::create(&mut conn, template).unwrap();

        // Test rendering without providing custom_var
        let mut service = TemplateService::new(&mut conn);
        let variables = HashMap::new(); // Empty variables

        let rendered = service
            .render_template(campaign.id, "test-template-2", variables)
            .unwrap();

        assert!(rendered.contains("Campaign: Test Campaign"));
        assert!(rendered.contains("Status: concept"));
        assert!(rendered.contains("Custom: N/A"));
    }

    #[test]
    fn test_render_nonexistent_template() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        // Create a test campaign
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "concept".to_string(),
                directory_path: "/test/path".to_string(),
            })
            .unwrap();

        let mut service = TemplateService::new(&mut conn);
        let result = service.render_template(campaign.id, "nonexistent-template", HashMap::new());

        assert!(result.is_err());
    }

    #[test]
    fn test_render_template_nonexistent_campaign() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = TemplateService::new(&mut conn);
        let result = service.render_template(99999, "some-template", HashMap::new());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::NotFound { .. }));
    }

    #[test]
    fn test_generate_document() {
        let temp_dir = TempDir::new().unwrap();
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        // Create a test campaign with temp directory
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Test Campaign".to_string(),
                status: "concept".to_string(),
                directory_path: temp_dir.path().to_str().unwrap().to_string(),
            })
            .unwrap();

        // Create a test template
        let template = NewTemplateDocument {
            document_id: "test-doc".to_string(),
            version_number: None,
            document_content: "# Test Document\n\nCampaign: {{ campaign_name }}".to_string(),
            content_hash: None,
            document_type: Some("test".to_string()),
            document_level: Some("campaign".to_string()),
            purpose: Some("Test document".to_string()),
            variables_schema: None,
            default_values: None,
            is_active: Some(true),
            metadata: None,
        };
        TemplateRepository::create(&mut conn, template).unwrap();

        // Generate the document
        let mut service = TemplateService::new(&mut conn);
        let file_path = service
            .generate_document(campaign.id, "test-doc", HashMap::new())
            .unwrap();

        // Verify file was created
        let full_path = PathBuf::from(&file_path);
        assert!(full_path.exists());

        // Verify content
        let content = fs::read_to_string(&full_path).unwrap();
        assert!(content.contains("# Test Document"));
        assert!(content.contains("Campaign: Test Campaign"));
    }

    #[test]
    fn test_get_template_file_path() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        // Create a test campaign
        let mut campaign_repo = CampaignRepository::new(&mut conn);
        let campaign = campaign_repo
            .create(NewCampaign {
                name: "Path Test Campaign".to_string(),
                status: "concept".to_string(),
                directory_path: "/campaigns/my-campaign".to_string(),
            })
            .unwrap();

        let mut service = TemplateService::new(&mut conn);

        let path = service
            .get_template_file_path(campaign.id, "campaign-bible")
            .unwrap();

        assert_eq!(path, "/campaigns/my-campaign/campaign_bible.md");

        let path = service
            .get_template_file_path(campaign.id, "session-outline")
            .unwrap();

        assert_eq!(path, "/campaigns/my-campaign/sessions/session_outline.md");
    }

    #[test]
    fn test_get_template_file_path_nonexistent_campaign() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = TemplateService::new(&mut conn);

        let result = service.get_template_file_path(99999, "some-template");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::NotFound { .. }));
    }
}

//! Module Service
//!
//! Business logic for module management including type-based document creation.

use chrono::Utc;
use diesel::SqliteConnection;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{Module, NewDocument, NewModule, UpdateModule as DalUpdateModule};
use crate::services::{ServiceError, ServiceResult};
use crate::templates;

/// Module types that determine which template is used for the overview document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleType {
    /// General-purpose module (default)
    #[default]
    General,
    /// Investigation and clue-based adventures
    Mystery,
    /// Exploration and combat-focused adventures
    Dungeon,
    /// Planning and execution of complex operations
    Heist,
    /// Tension, dread, and survival
    Horror,
    /// Factions, diplomacy, and scheming
    Political,
}

impl ModuleType {
    /// Get the template key for this module type.
    pub fn template_key(&self) -> &'static str {
        match self {
            ModuleType::General => "general",
            ModuleType::Mystery => "mystery",
            ModuleType::Dungeon => "dungeon",
            ModuleType::Heist => "heist",
            ModuleType::Horror => "horror",
            ModuleType::Political => "political",
        }
    }

    /// Get the display name for this module type.
    pub fn display_name(&self) -> &'static str {
        match self {
            ModuleType::General => "General",
            ModuleType::Mystery => "Mystery",
            ModuleType::Dungeon => "Dungeon Crawl",
            ModuleType::Heist => "Heist",
            ModuleType::Horror => "Horror",
            ModuleType::Political => "Political Intrigue",
        }
    }
}

impl std::fmt::Display for ModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Input for creating a new module.
#[derive(Debug, Clone)]
pub struct CreateModuleInput {
    /// Campaign this module belongs to
    pub campaign_id: String,
    /// Module name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Module type (determines which template is used)
    pub module_type: ModuleType,
}

impl CreateModuleInput {
    /// Create a new module input.
    pub fn new(campaign_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            name: name.into(),
            description: None,
            module_type: ModuleType::default(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the module type.
    pub fn with_type(mut self, module_type: ModuleType) -> Self {
        self.module_type = module_type;
        self
    }
}

/// Input for updating a module.
#[derive(Debug, Clone, Default)]
pub struct UpdateModuleInput {
    /// New name (if changing)
    pub name: Option<String>,
    /// New description (if changing). Use Some(None) to clear.
    pub description: Option<Option<String>>,
}

impl UpdateModuleInput {
    /// Create an update to change the name.
    pub fn set_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            description: None,
        }
    }

    /// Create an update to change the description.
    pub fn set_description(description: Option<String>) -> Self {
        Self {
            name: None,
            description: Some(description),
        }
    }
}

/// Service for module management.
///
/// Handles module CRUD operations and automatic creation of type-specific documents.
pub struct ModuleService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> ModuleService<'a> {
    /// Create a new module service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Create a new module with type-specific overview and play notes documents.
    ///
    /// This creates the module and populates it with:
    /// 1. A type-specific overview document (mystery, dungeon, etc.)
    /// 2. A blank play notes document for session tracking
    pub fn create(&mut self, input: CreateModuleInput) -> ServiceResult<Module> {
        use diesel::Connection;

        let module_id = Uuid::new_v4().to_string();

        self.conn.transaction(|conn| {
            // Get next module number for this campaign
            let module_number = dal::next_module_number(conn, &input.campaign_id)?;

            // Create the module
            let mut new_module =
                NewModule::new(&module_id, &input.campaign_id, &input.name, module_number);
            if let Some(ref desc) = input.description {
                new_module = new_module.with_description(desc);
            }
            dal::insert_module(conn, &new_module)?;

            // Get the type-specific template content
            let overview_content = templates::get_module_template(input.module_type.template_key())
                .unwrap_or_else(|| {
                    templates::get_module_template("general")
                        .expect("General template must exist")
                });

            // Create overview document from type-specific template
            let overview_id = Uuid::new_v4().to_string();
            let overview_doc = NewDocument::for_module(
                &overview_id,
                &input.campaign_id,
                &module_id,
                "Module Overview",
                "module_overview",
            )
            .with_content(overview_content);
            dal::insert_document(conn, &overview_doc)?;

            // Create play notes document
            let play_notes_id = Uuid::new_v4().to_string();
            let play_notes_content = templates::get_play_notes_template();
            let play_notes_doc = NewDocument::for_module(
                &play_notes_id,
                &input.campaign_id,
                &module_id,
                "Play Notes",
                "play_notes",
            )
            .with_content(play_notes_content);
            dal::insert_document(conn, &play_notes_doc)?;

            // Fetch and return the created module
            dal::get_module(conn, &module_id).map_err(ServiceError::from)
        })
    }

    /// List all modules for a campaign, ordered by module number.
    pub fn list_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<Vec<Module>> {
        dal::list_modules(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Get a module by ID.
    ///
    /// Returns `None` if the module doesn't exist.
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<Module>> {
        dal::get_module_optional(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a module by campaign ID and module number.
    pub fn get_by_number(
        &mut self,
        campaign_id: &str,
        module_number: i32,
    ) -> ServiceResult<Option<Module>> {
        dal::get_module_by_number(self.conn, campaign_id, module_number).map_err(ServiceError::from)
    }

    /// Update a module.
    ///
    /// Returns the updated module, or an error if not found.
    pub fn update(&mut self, id: &str, input: UpdateModuleInput) -> ServiceResult<Module> {
        let now = Utc::now().to_rfc3339();

        // Build the update changeset
        let name_ref = input.name.as_deref();
        let desc_ref = input.description.as_ref().map(|d| d.as_deref());

        let update = DalUpdateModule {
            name: name_ref,
            description: desc_ref,
            module_number: None,
            updated_at: Some(&now),
        };

        let rows = dal::update_module(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Module", id));
        }

        dal::get_module(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a module permanently.
    ///
    /// This will cascade delete all related data (documents, maps, etc.).
    pub fn delete(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_module(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Module", id));
        }

        Ok(())
    }

    /// Count modules for a campaign.
    pub fn count_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_modules(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Check if a module exists.
    pub fn exists(&mut self, id: &str) -> ServiceResult<bool> {
        dal::module_exists(self.conn, id).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign::{count_module_documents, insert_campaign, list_module_documents};
    use crate::models::campaign::NewCampaign;
    use crate::test_utils::setup_test_db;

    fn create_test_campaign(conn: &mut SqliteConnection) -> String {
        let campaign_id = Uuid::new_v4().to_string();
        let campaign = NewCampaign::new(&campaign_id, "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");
        campaign_id
    }

    #[test]
    fn test_create_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let module = service.create(input).expect("Failed to create module");

        assert_eq!(module.name, "Chapter 1");
        assert_eq!(module.campaign_id, campaign_id);
        assert_eq!(module.module_number, 1);
        assert!(module.description.is_none());
    }

    #[test]
    fn test_create_module_with_description() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1")
            .with_description("The beginning of our story");
        let module = service.create(input).expect("Failed to create module");

        assert_eq!(
            module.description,
            Some("The beginning of our story".to_string())
        );
    }

    #[test]
    fn test_create_module_creates_2_documents() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let module = service.create(input).expect("Failed to create module");

        // Check that 2 documents were created
        let doc_count =
            count_module_documents(&mut conn, &module.id).expect("Failed to count documents");
        assert_eq!(doc_count, 2);

        // Verify document types
        let docs =
            list_module_documents(&mut conn, &module.id).expect("Failed to list documents");
        assert_eq!(docs.len(), 2);

        let doc_types: Vec<&str> = docs.iter().map(|d| d.doc_type.as_str()).collect();
        assert!(doc_types.contains(&"module_overview"));
        assert!(doc_types.contains(&"play_notes"));
    }

    #[test]
    fn test_create_module_with_type() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "The Haunted Manor")
            .with_type(ModuleType::Horror);
        let module = service.create(input).expect("Failed to create module");

        // Verify the overview document has horror template content
        let docs =
            list_module_documents(&mut conn, &module.id).expect("Failed to list documents");
        let overview = docs
            .iter()
            .find(|d| d.doc_type == "module_overview")
            .expect("Overview not found");

        // Horror template should have horror-specific content
        assert!(overview.content.contains("Horror"));
    }

    #[test]
    fn test_create_modules_auto_increment_number() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input1 = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let module1 = service.create(input1).expect("Failed to create module");
        assert_eq!(module1.module_number, 1);

        let input2 = CreateModuleInput::new(&campaign_id, "Chapter 2");
        let module2 = service.create(input2).expect("Failed to create module");
        assert_eq!(module2.module_number, 2);

        let input3 = CreateModuleInput::new(&campaign_id, "Chapter 3");
        let module3 = service.create(input3).expect("Failed to create module");
        assert_eq!(module3.module_number, 3);
    }

    #[test]
    fn test_list_modules_for_campaign() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input1 = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let input2 = CreateModuleInput::new(&campaign_id, "Chapter 2");
        service.create(input1).expect("Failed to create module");
        service.create(input2).expect("Failed to create module");

        let modules = service
            .list_for_campaign(&campaign_id)
            .expect("Failed to list modules");
        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0].module_number, 1);
        assert_eq!(modules[1].module_number, 2);
    }

    #[test]
    fn test_get_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let created = service.create(input).expect("Failed to create module");

        let retrieved = service
            .get(&created.id)
            .expect("Failed to get module")
            .expect("Module not found");

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Chapter 1");
    }

    #[test]
    fn test_get_module_not_found() {
        let mut conn = setup_test_db();

        let mut service = ModuleService::new(&mut conn);

        let result = service.get("nonexistent").expect("Failed to query module");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_module_by_number() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let created = service.create(input).expect("Failed to create module");

        let retrieved = service
            .get_by_number(&campaign_id, 1)
            .expect("Failed to get module")
            .expect("Module not found");

        assert_eq!(retrieved.id, created.id);
    }

    #[test]
    fn test_update_module_name() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Original Name");
        let created = service.create(input).expect("Failed to create module");

        let update = UpdateModuleInput::set_name("New Name");
        let updated = service
            .update(&created.id, update)
            .expect("Failed to update module");

        assert_eq!(updated.name, "New Name");
    }

    #[test]
    fn test_update_module_not_found() {
        let mut conn = setup_test_db();

        let mut service = ModuleService::new(&mut conn);

        let update = UpdateModuleInput::set_name("New Name");
        let result = service.update("nonexistent", update);

        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_delete_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let created = service.create(input).expect("Failed to create module");

        assert!(service.exists(&created.id).expect("Failed to check exists"));

        service.delete(&created.id).expect("Failed to delete module");

        assert!(!service.exists(&created.id).expect("Failed to check exists"));
    }

    #[test]
    fn test_delete_module_not_found() {
        let mut conn = setup_test_db();

        let mut service = ModuleService::new(&mut conn);

        let result = service.delete("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_count_modules() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        assert_eq!(
            service
                .count_for_campaign(&campaign_id)
                .expect("Failed to count"),
            0
        );

        let input1 = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let input2 = CreateModuleInput::new(&campaign_id, "Chapter 2");
        service.create(input1).expect("Failed to create module");
        service.create(input2).expect("Failed to create module");

        assert_eq!(
            service
                .count_for_campaign(&campaign_id)
                .expect("Failed to count"),
            2
        );
    }

    #[test]
    fn test_module_type_display() {
        assert_eq!(ModuleType::General.display_name(), "General");
        assert_eq!(ModuleType::Mystery.display_name(), "Mystery");
        assert_eq!(ModuleType::Dungeon.display_name(), "Dungeon Crawl");
        assert_eq!(ModuleType::Heist.display_name(), "Heist");
        assert_eq!(ModuleType::Horror.display_name(), "Horror");
        assert_eq!(ModuleType::Political.display_name(), "Political Intrigue");
    }

    #[test]
    fn test_all_module_types_have_templates() {
        let types = [
            ModuleType::General,
            ModuleType::Mystery,
            ModuleType::Dungeon,
            ModuleType::Heist,
            ModuleType::Horror,
            ModuleType::Political,
        ];

        for module_type in types {
            let template = templates::get_module_template(module_type.template_key());
            assert!(
                template.is_some(),
                "Template missing for {:?}",
                module_type
            );
            assert!(
                !template.unwrap().is_empty(),
                "Template empty for {:?}",
                module_type
            );
        }
    }
}

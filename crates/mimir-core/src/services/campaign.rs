//! Campaign Service
//!
//! Business logic for campaign management including automatic document creation.

use diesel::SqliteConnection;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{Campaign, NewCampaign, NewDocument, UpdateCampaign};
use crate::services::{ServiceError, ServiceResult};
use crate::templates;
use crate::utils::now_rfc3339;

/// Input for creating a new campaign.
#[derive(Debug, Clone)]
pub struct CreateCampaignInput {
    /// Campaign name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
}

impl CreateCampaignInput {
    /// Create a new campaign input.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Input for updating a campaign.
#[derive(Debug, Clone, Default)]
pub struct UpdateCampaignInput {
    /// New name (if changing)
    pub name: Option<String>,
    /// New description (if changing). Use Some(None) to clear.
    pub description: Option<Option<String>>,
}

impl UpdateCampaignInput {
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

/// Service for campaign management.
///
/// Handles campaign CRUD operations and automatic creation of initial documents.
pub struct CampaignService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> CampaignService<'a> {
    /// Create a new campaign service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Create a new campaign with all 11 initial documents.
    ///
    /// This creates the campaign and populates it with template documents
    /// for campaign management (pitch, world primer, character guidelines, etc.).
    pub fn create(&mut self, input: CreateCampaignInput) -> ServiceResult<Campaign> {
        use diesel::Connection;

        let campaign_id = Uuid::new_v4().to_string();

        self.conn.transaction(|conn| {
            // Create the campaign
            let mut new_campaign = NewCampaign::new(&campaign_id, &input.name);
            if let Some(ref desc) = input.description {
                new_campaign = new_campaign.with_description(desc);
            }
            dal::insert_campaign(conn, &new_campaign)?;

            // Create all 11 campaign documents from templates
            for template_info in templates::CAMPAIGN_TEMPLATES {
                let doc_id = Uuid::new_v4().to_string();
                let doc = NewDocument::for_campaign(
                    &doc_id,
                    &campaign_id,
                    template_info.title,
                    template_info.doc_type,
                )
                .with_content(template_info.content);
                dal::insert_document(conn, &doc)?;
            }

            // Fetch and return the created campaign
            dal::get_campaign(conn, &campaign_id).map_err(ServiceError::from)
        })
    }

    /// List campaigns.
    ///
    /// # Arguments
    /// * `include_archived` - If true, includes archived campaigns in the list.
    pub fn list(&mut self, include_archived: bool) -> ServiceResult<Vec<Campaign>> {
        dal::list_campaigns(self.conn, include_archived).map_err(ServiceError::from)
    }

    /// Get a campaign by ID.
    ///
    /// Returns `None` if the campaign doesn't exist.
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<Campaign>> {
        dal::get_campaign_optional(self.conn, id).map_err(ServiceError::from)
    }

    /// Update a campaign.
    ///
    /// Returns the updated campaign, or an error if not found.
    pub fn update(&mut self, id: &str, input: UpdateCampaignInput) -> ServiceResult<Campaign> {
        let now = now_rfc3339();

        // Build the update changeset
        let name_ref = input.name.as_deref();
        let desc_ref = input.description.as_ref().map(|d| d.as_deref());

        let update = UpdateCampaign {
            name: name_ref,
            description: desc_ref,
            archived_at: None,
            updated_at: Some(&now),
        };

        let rows = dal::update_campaign(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Campaign", id));
        }

        dal::get_campaign(self.conn, id).map_err(ServiceError::from)
    }

    /// Archive a campaign (soft delete).
    ///
    /// Archived campaigns are hidden from default lists but can be restored.
    pub fn archive(&mut self, id: &str) -> ServiceResult<()> {
        let now = now_rfc3339();
        let update = UpdateCampaign::archive(&now);

        let rows = dal::update_campaign(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Campaign", id));
        }

        Ok(())
    }

    /// Unarchive a campaign.
    pub fn unarchive(&mut self, id: &str) -> ServiceResult<()> {
        let now = now_rfc3339();
        let update = UpdateCampaign::unarchive(&now);

        let rows = dal::update_campaign(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Campaign", id));
        }

        Ok(())
    }

    /// Delete a campaign permanently.
    ///
    /// This will cascade delete all related data (modules, documents, etc.).
    pub fn delete(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_campaign(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Campaign", id));
        }

        Ok(())
    }

    /// Count campaigns.
    ///
    /// # Arguments
    /// * `include_archived` - If true, includes archived campaigns in the count.
    pub fn count(&mut self, include_archived: bool) -> ServiceResult<i64> {
        if include_archived {
            dal::count_all_campaigns(self.conn).map_err(ServiceError::from)
        } else {
            dal::count_campaigns(self.conn).map_err(ServiceError::from)
        }
    }

    /// Check if a campaign exists.
    pub fn exists(&mut self, id: &str) -> ServiceResult<bool> {
        dal::campaign_exists(self.conn, id).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign::{count_campaign_documents, list_campaign_level_documents};
    use crate::test_utils::setup_test_db;

    #[test]
    fn test_create_campaign() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let input = CreateCampaignInput::new("Test Campaign");
        let campaign = service.create(input).expect("Failed to create campaign");

        assert_eq!(campaign.name, "Test Campaign");
        assert!(campaign.description.is_none());
        assert!(!campaign.is_archived());
    }

    #[test]
    fn test_create_campaign_with_description() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let input =
            CreateCampaignInput::new("Test Campaign").with_description("A great adventure");
        let campaign = service.create(input).expect("Failed to create campaign");

        assert_eq!(campaign.description, Some("A great adventure".to_string()));
    }

    #[test]
    fn test_create_campaign_creates_11_documents() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let input = CreateCampaignInput::new("Test Campaign");
        let campaign = service.create(input).expect("Failed to create campaign");

        // Check that 11 documents were created
        let doc_count =
            count_campaign_documents(&mut conn, &campaign.id).expect("Failed to count documents");
        assert_eq!(doc_count, 11);

        // Verify documents are campaign-level (not module documents)
        let docs = list_campaign_level_documents(&mut conn, &campaign.id)
            .expect("Failed to list documents");
        assert_eq!(docs.len(), 11);

        // Check some expected document types
        let doc_types: Vec<&str> = docs.iter().map(|d| d.doc_type.as_str()).collect();
        assert!(doc_types.contains(&"campaign_pitch"));
        assert!(doc_types.contains(&"world_primer"));
        assert!(doc_types.contains(&"character_guidelines"));
    }

    #[test]
    fn test_list_campaigns() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let input1 = CreateCampaignInput::new("Campaign 1");
        let input2 = CreateCampaignInput::new("Campaign 2");
        service.create(input1).expect("Failed to create campaign");
        let campaign2 = service.create(input2).expect("Failed to create campaign");

        // Archive one campaign
        service
            .archive(&campaign2.id)
            .expect("Failed to archive campaign");

        // List without archived
        let active = service.list(false).expect("Failed to list campaigns");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Campaign 1");

        // List with archived
        let all = service.list(true).expect("Failed to list campaigns");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_get_campaign() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let input = CreateCampaignInput::new("Test Campaign");
        let created = service.create(input).expect("Failed to create campaign");

        let retrieved = service
            .get(&created.id)
            .expect("Failed to get campaign")
            .expect("Campaign not found");

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Test Campaign");
    }

    #[test]
    fn test_get_campaign_not_found() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let result = service
            .get("nonexistent")
            .expect("Failed to query campaign");
        assert!(result.is_none());
    }

    #[test]
    fn test_update_campaign_name() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let input = CreateCampaignInput::new("Original Name");
        let created = service.create(input).expect("Failed to create campaign");

        let update = UpdateCampaignInput::set_name("New Name");
        let updated = service
            .update(&created.id, update)
            .expect("Failed to update campaign");

        assert_eq!(updated.name, "New Name");
    }

    #[test]
    fn test_update_campaign_not_found() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let update = UpdateCampaignInput::set_name("New Name");
        let result = service.update("nonexistent", update);

        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_archive_and_unarchive() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let input = CreateCampaignInput::new("Test Campaign");
        let created = service.create(input).expect("Failed to create campaign");

        // Archive
        service
            .archive(&created.id)
            .expect("Failed to archive campaign");
        let archived = service
            .get(&created.id)
            .expect("Failed to get campaign")
            .unwrap();
        assert!(archived.is_archived());

        // Unarchive
        service
            .unarchive(&created.id)
            .expect("Failed to unarchive campaign");
        let unarchived = service
            .get(&created.id)
            .expect("Failed to get campaign")
            .unwrap();
        assert!(!unarchived.is_archived());
    }

    #[test]
    fn test_delete_campaign() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let input = CreateCampaignInput::new("Test Campaign");
        let created = service.create(input).expect("Failed to create campaign");

        assert!(service.exists(&created.id).expect("Failed to check exists"));

        service
            .delete(&created.id)
            .expect("Failed to delete campaign");

        assert!(!service.exists(&created.id).expect("Failed to check exists"));
    }

    #[test]
    fn test_delete_campaign_not_found() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        let result = service.delete("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_count_campaigns() {
        let mut conn = setup_test_db();
        let mut service = CampaignService::new(&mut conn);

        assert_eq!(service.count(false).expect("Failed to count"), 0);

        let input1 = CreateCampaignInput::new("Campaign 1");
        let input2 = CreateCampaignInput::new("Campaign 2");
        service.create(input1).expect("Failed to create campaign");
        let campaign2 = service.create(input2).expect("Failed to create campaign");

        assert_eq!(service.count(false).expect("Failed to count"), 2);

        service
            .archive(&campaign2.id)
            .expect("Failed to archive campaign");

        assert_eq!(service.count(false).expect("Failed to count"), 1);
        assert_eq!(service.count(true).expect("Failed to count"), 2);
    }
}

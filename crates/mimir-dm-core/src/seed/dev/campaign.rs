//! Campaign and module seeding for dev data.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::campaigns::Campaign;
use crate::models::campaign::modules::Module;
use crate::services::{CampaignService, ModuleService};

pub const TEST_CAMPAIGN_NAME: &str = "The Lost Mine of Phandelver";

/// Seed the test campaign and transition to active.
pub fn seed_campaign(conn: &mut DbConnection, campaigns_dir: &str) -> Result<Campaign> {
    let mut service = CampaignService::new(conn);
    let campaign = service.create_campaign(
        TEST_CAMPAIGN_NAME,
        Some("A classic D&D adventure for 4-5 characters of levels 1-5".into()),
        campaigns_dir,
    )?;

    // Transition through stages: concept -> session_zero -> integration -> active
    for stage in ["session_zero", "integration", "active"] {
        let mut service = CampaignService::new(conn);
        service.transition_campaign_stage(campaign.id, stage)?;
    }

    // Re-fetch to get updated state
    let mut service = CampaignService::new(conn);
    service.get_campaign(campaign.id)?
        .ok_or_else(|| crate::error::DbError::NotFound {
            entity_type: "Campaign".into(),
            id: campaign.id.to_string(),
        })
}

/// Seed test modules.
pub fn seed_modules(conn: &mut DbConnection, campaign_id: i32) -> Result<Vec<Module>> {
    let mut service = ModuleService::new(conn);
    let module = service.create_module_with_documents(
        campaign_id,
        "Cragmaw Hideout".into(),
        3,
        Some("dungeon".into()),
    )?;
    Ok(vec![module])
}

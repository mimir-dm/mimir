//! Test helpers shared by the character service test modules.

use diesel::SqliteConnection;
use uuid::Uuid;

use crate::dal::campaign::insert_campaign;
use crate::models::campaign::NewCampaign;

pub(super) fn create_test_campaign(conn: &mut SqliteConnection) -> String {
    let campaign_id = Uuid::new_v4().to_string();
    let campaign = NewCampaign::new(&campaign_id, "Test Campaign");
    insert_campaign(conn, &campaign).expect("Failed to create campaign");
    campaign_id
}

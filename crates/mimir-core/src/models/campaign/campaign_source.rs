//! Campaign Source Model
//!
//! Links campaigns to allowed source books.

use crate::schema::campaign_sources;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A campaign source - links a campaign to an allowed source book.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = campaign_sources)]
pub struct CampaignSource {
    /// Unique ID (UUID)
    pub id: String,
    /// Campaign this source belongs to
    pub campaign_id: String,
    /// Source book code (e.g., "PHB", "MM")
    pub source_code: String,
}

/// Data for inserting a new campaign source.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = campaign_sources)]
pub struct NewCampaignSource<'a> {
    pub id: &'a str,
    pub campaign_id: &'a str,
    pub source_code: &'a str,
}

impl<'a> NewCampaignSource<'a> {
    /// Create a new campaign source link.
    pub fn new(id: &'a str, campaign_id: &'a str, source_code: &'a str) -> Self {
        Self {
            id,
            campaign_id,
            source_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_campaign_source() {
        let source = NewCampaignSource::new("source-id", "campaign-id", "PHB");
        assert_eq!(source.id, "source-id");
        assert_eq!(source.campaign_id, "campaign-id");
        assert_eq!(source.source_code, "PHB");
    }
}

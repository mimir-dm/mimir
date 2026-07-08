//! Source Service
//!
//! Business logic for managing allowed source books on campaigns and
//! characters. The `set_*` operations replace the full list inside a single
//! transaction so a failure can never leave sources partially wiped.

use diesel::{Connection, SqliteConnection};
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    CampaignSource, CharacterSource, NewCampaignSource, NewCharacterSource,
};
use crate::services::{ServiceError, ServiceResult};

/// Service for campaign and character source management.
pub struct SourceService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> SourceService<'a> {
    /// Create a new source service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    // -- Campaign sources ----------------------------------------------------

    /// List source codes enabled for a campaign.
    pub fn list_campaign_sources(&mut self, campaign_id: &str) -> ServiceResult<Vec<String>> {
        dal::list_campaign_source_codes(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Add a source to a campaign's allowed sources.
    pub fn add_campaign_source(
        &mut self,
        campaign_id: &str,
        source_code: &str,
    ) -> ServiceResult<CampaignSource> {
        let id = Uuid::new_v4().to_string();
        let source = NewCampaignSource::new(&id, campaign_id, source_code);
        dal::insert_campaign_source(self.conn, &source)?;
        Ok(dal::get_campaign_source(self.conn, &id)?)
    }

    /// Remove a source from a campaign's allowed sources.
    pub fn remove_campaign_source(
        &mut self,
        campaign_id: &str,
        source_code: &str,
    ) -> ServiceResult<()> {
        dal::delete_campaign_source_by_code(self.conn, campaign_id, source_code)?;
        Ok(())
    }

    /// Replace a campaign's source list atomically.
    pub fn set_campaign_sources(
        &mut self,
        campaign_id: &str,
        source_codes: &[String],
    ) -> ServiceResult<Vec<String>> {
        self.conn.transaction::<_, diesel::result::Error, _>(|conn| {
            dal::delete_all_campaign_sources(conn, campaign_id)?;
            for code in source_codes {
                let id = Uuid::new_v4().to_string();
                let source = NewCampaignSource::new(&id, campaign_id, code);
                dal::insert_campaign_source(conn, &source)?;
            }
            Ok(())
        })?;
        self.list_campaign_sources(campaign_id)
    }

    // -- Character sources -----------------------------------------------------

    /// List source codes allowed for a character.
    pub fn list_character_sources(&mut self, character_id: &str) -> ServiceResult<Vec<String>> {
        dal::list_character_source_codes(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Add a source to a character's allowed sources.
    pub fn add_character_source(
        &mut self,
        character_id: &str,
        source_code: &str,
    ) -> ServiceResult<CharacterSource> {
        let id = Uuid::new_v4().to_string();
        let source = NewCharacterSource::new(&id, character_id, source_code);
        dal::insert_character_source(self.conn, &source)?;
        Ok(dal::get_character_source(self.conn, &id)?)
    }

    /// Remove a source from a character's allowed sources.
    pub fn remove_character_source(
        &mut self,
        character_id: &str,
        source_code: &str,
    ) -> ServiceResult<()> {
        dal::delete_character_source_by_code(self.conn, character_id, source_code)?;
        Ok(())
    }

    /// Replace a character's source list atomically.
    pub fn set_character_sources(
        &mut self,
        character_id: &str,
        source_codes: &[String],
    ) -> ServiceResult<Vec<String>> {
        self.conn.transaction::<_, diesel::result::Error, _>(|conn| {
            dal::delete_all_character_sources(conn, character_id)?;
            for code in source_codes {
                let id = Uuid::new_v4().to_string();
                let source = NewCharacterSource::new(&id, character_id, code);
                dal::insert_character_source(conn, &source)?;
            }
            Ok(())
        })?;
        self.list_character_sources(character_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::campaign::insert_campaign;
    use crate::models::campaign::NewCampaign;
    use diesel::RunQueryDsl;

    fn setup_campaign(conn: &mut SqliteConnection) -> String {
        // campaign_sources.source_code has an FK to catalog_sources(code)
        for code in ["PHB", "MM", "XGE"] {
            diesel::sql_query(format!(
                "INSERT INTO catalog_sources (code, name, enabled, imported_at) \
                 VALUES ('{}', '{}', 1, '2026-01-01T00:00:00Z')",
                code, code
            ))
            .execute(conn)
            .expect("insert catalog source");
        }

        let id = Uuid::new_v4().to_string();
        let campaign = NewCampaign::new(&id, "Test Campaign");
        insert_campaign(conn, &campaign).expect("insert campaign");
        id
    }

    #[test]
    fn set_campaign_sources_replaces_atomically() {
        let mut conn = test_connection();
        let campaign_id = setup_campaign(&mut conn);
        let mut service = SourceService::new(&mut conn);

        let codes = vec!["PHB".to_string(), "MM".to_string()];
        let result = service.set_campaign_sources(&campaign_id, &codes).unwrap();
        assert_eq!(result, vec!["MM".to_string(), "PHB".to_string()]);

        // Replacing shrinks the list; no leftovers from the previous set
        let codes = vec!["XGE".to_string()];
        let result = service.set_campaign_sources(&campaign_id, &codes).unwrap();
        assert_eq!(result, vec!["XGE".to_string()]);
    }

    #[test]
    fn add_and_remove_campaign_source() {
        let mut conn = test_connection();
        let campaign_id = setup_campaign(&mut conn);
        let mut service = SourceService::new(&mut conn);

        let source = service.add_campaign_source(&campaign_id, "PHB").unwrap();
        assert_eq!(source.source_code, "PHB");
        assert_eq!(service.list_campaign_sources(&campaign_id).unwrap().len(), 1);

        service.remove_campaign_source(&campaign_id, "PHB").unwrap();
        assert!(service.list_campaign_sources(&campaign_id).unwrap().is_empty());
    }
}

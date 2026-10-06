//! Character feats recorded outside level-up.

use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::dal::catalog as catalog_dal;
use crate::models::campaign::{CharacterFeat, FeatSourceType, NewCharacterFeat};
use crate::services::{ServiceError, ServiceResult};

use super::CharacterService;

impl<'a> CharacterService<'a> {
    /// List the feats a character has, ordered by name.
    pub fn list_feats(&mut self, character_id: &str) -> ServiceResult<Vec<CharacterFeat>> {
        if !dal::character_exists(self.conn, character_id)? {
            return Err(ServiceError::not_found("Character", character_id));
        }
        dal::list_character_feats(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Record a feat for a character outside level-up (e.g. a deferred ASI choice).
    ///
    /// The feat must exist in the catalog under `feat_name` (case-insensitive)
    /// and `feat_source`; it is stored with the catalog's spelling. A character
    /// may hold a feat once, unless the catalog marks it repeatable.
    /// Single implementation shared by the desktop UI and MCP server.
    pub fn add_feat(
        &mut self,
        character_id: &str,
        feat_name: &str,
        feat_source: &str,
        source_type: FeatSourceType,
    ) -> ServiceResult<CharacterFeat> {
        if !dal::character_exists(self.conn, character_id)? {
            return Err(ServiceError::not_found("Character", character_id));
        }

        let catalog_feat = catalog_dal::get_feat_by_name(self.conn, feat_name, feat_source)?
            .ok_or_else(|| {
                ServiceError::Validation(format!(
                    "Feat '{}' from {} is not in the catalog",
                    feat_name, feat_source
                ))
            })?;

        let repeatable = serde_json::from_str::<serde_json::Value>(&catalog_feat.data)
            .ok()
            .and_then(|d| d.get("repeatable").and_then(|r| r.as_bool()))
            .unwrap_or(false);
        // Compare case-insensitively: level-up stores the name as the caller
        // typed it, so an existing row may not use the catalog's spelling.
        let wanted = catalog_feat.name.to_lowercase();
        let already_has = dal::list_character_feats(self.conn, character_id)?
            .iter()
            .any(|f| f.feat_name.to_lowercase() == wanted);
        if !repeatable && already_has {
            return Err(ServiceError::Validation(format!(
                "Character already has {}",
                catalog_feat.name
            )));
        }

        let id = Uuid::new_v4().to_string();
        let new_feat = NewCharacterFeat::new(
            &id,
            character_id,
            &catalog_feat.name,
            &catalog_feat.source,
            source_type,
        );
        dal::insert_character_feat(self.conn, &new_feat)?;

        Ok(CharacterFeat {
            id,
            character_id: character_id.to_string(),
            feat_name: catalog_feat.name,
            feat_source: catalog_feat.source,
            source_type: source_type.as_str().to_string(),
        })
    }

    /// Remove a feat from a character by name (case-insensitive).
    ///
    /// Removes every instance of the feat, so a repeatable feat taken twice is
    /// removed twice. Returns how many were removed.
    pub fn remove_feat(&mut self, character_id: &str, feat_name: &str) -> ServiceResult<usize> {
        let wanted = feat_name.to_lowercase();
        let matching: Vec<_> = self
            .list_feats(character_id)?
            .into_iter()
            .filter(|f| f.feat_name.to_lowercase() == wanted)
            .collect();
        if matching.is_empty() {
            return Err(ServiceError::Validation(format!(
                "Character doesn't have {}",
                feat_name
            )));
        }
        for feat in &matching {
            dal::delete_character_feat(self.conn, &feat.id)?;
        }
        Ok(matching.len())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::create_test_campaign;
    use super::super::CreateCharacterInput;
    use super::*;
    use diesel::SqliteConnection;

    // -- Feats (MIMIR-T-0658) -------------------------------------------------

    fn feat_fixture() -> (SqliteConnection, String) {
        use crate::dal::catalog::insert_feat;
        use crate::models::catalog::NewFeat;

        let mut conn = crate::test_utils::setup_test_db_with_sources();
        insert_feat(
            &mut conn,
            &NewFeat::new("Alert", "PHB", r#"{"name":"Alert"}"#),
        )
        .expect("insert Alert");
        insert_feat(
            &mut conn,
            &NewFeat::new(
                "Elemental Adept",
                "PHB",
                r#"{"name":"Elemental Adept","repeatable":true}"#,
            ),
        )
        .expect("insert Elemental Adept");

        let campaign_id = create_test_campaign(&mut conn);
        let mut service = CharacterService::new(&mut conn);
        let character = service
            .create(CreateCharacterInput::new_pc(
                Some(&campaign_id),
                "Hero",
                "John",
            ))
            .expect("create character");
        (conn, character.id)
    }

    #[test]
    fn test_add_feat_uses_catalog_name_and_source_type() {
        let (mut conn, character_id) = feat_fixture();
        let mut service = CharacterService::new(&mut conn);

        let feat = service
            .add_feat(&character_id, "alert", "PHB", FeatSourceType::Asi)
            .expect("add feat");
        assert_eq!(
            feat.feat_name, "Alert",
            "stored with the catalog's spelling"
        );
        assert_eq!(feat.feat_source, "PHB");
        assert_eq!(feat.source_type, "asi");

        let feats = service.list_feats(&character_id).expect("list feats");
        assert_eq!(feats.len(), 1);
        assert_eq!(feats[0].id, feat.id);
    }

    #[test]
    fn test_add_feat_rejects_duplicate_unless_repeatable() {
        let (mut conn, character_id) = feat_fixture();
        let mut service = CharacterService::new(&mut conn);

        service
            .add_feat(&character_id, "Alert", "PHB", FeatSourceType::Asi)
            .expect("first add");
        let err = service
            .add_feat(&character_id, "ALERT", "PHB", FeatSourceType::Bonus)
            .expect_err("duplicate must be rejected");
        assert!(matches!(err, ServiceError::Validation(ref m) if m.contains("already has")));

        service
            .add_feat(&character_id, "Elemental Adept", "PHB", FeatSourceType::Asi)
            .expect("first Elemental Adept");
        service
            .add_feat(&character_id, "Elemental Adept", "PHB", FeatSourceType::Asi)
            .expect("repeatable feat may be taken again");

        assert_eq!(service.list_feats(&character_id).unwrap().len(), 3);
    }

    #[test]
    fn test_add_feat_duplicate_check_ignores_case_of_existing_rows() {
        let (mut conn, character_id) = feat_fixture();

        // A level-up stores the feat name as typed, e.g. lowercase.
        let id = Uuid::new_v4().to_string();
        dal::insert_character_feat(
            &mut conn,
            &NewCharacterFeat::new(&id, &character_id, "alert", "PHB", FeatSourceType::Asi),
        )
        .unwrap();

        let mut service = CharacterService::new(&mut conn);
        let err = service
            .add_feat(&character_id, "Alert", "PHB", FeatSourceType::Asi)
            .expect_err("lowercase row must count as a duplicate");
        assert!(matches!(err, ServiceError::Validation(ref m) if m.contains("already has")));
    }

    #[test]
    fn test_add_feat_rejects_catalog_miss_and_missing_character() {
        let (mut conn, character_id) = feat_fixture();
        let mut service = CharacterService::new(&mut conn);

        let err = service
            .add_feat(&character_id, "Not A Feat", "PHB", FeatSourceType::Asi)
            .expect_err("unknown feat");
        assert!(matches!(err, ServiceError::Validation(ref m) if m.contains("not in the catalog")));

        let err = service
            .add_feat(&character_id, "Alert", "XGE", FeatSourceType::Asi)
            .expect_err("right name, wrong source");
        assert!(matches!(err, ServiceError::Validation(_)));

        let err = service
            .add_feat("no-such-character", "Alert", "PHB", FeatSourceType::Asi)
            .expect_err("missing character");
        assert!(matches!(err, ServiceError::NotFound { .. }));

        let err = service
            .list_feats("no-such-character")
            .expect_err("list on missing character");
        assert!(matches!(err, ServiceError::NotFound { .. }));

        assert!(service.list_feats(&character_id).unwrap().is_empty());
    }

    #[test]
    fn test_remove_feat_is_case_insensitive_and_reports_misses() {
        let (mut conn, character_id) = feat_fixture();
        let mut service = CharacterService::new(&mut conn);

        service
            .add_feat(&character_id, "Alert", "PHB", FeatSourceType::Asi)
            .unwrap();
        service
            .add_feat(&character_id, "Elemental Adept", "PHB", FeatSourceType::Asi)
            .unwrap();
        service
            .add_feat(&character_id, "Elemental Adept", "PHB", FeatSourceType::Asi)
            .unwrap();

        assert_eq!(service.remove_feat(&character_id, "alert").unwrap(), 1);
        assert_eq!(
            service
                .remove_feat(&character_id, "elemental adept")
                .unwrap(),
            2,
            "every instance of a repeatable feat is removed"
        );
        assert!(service.list_feats(&character_id).unwrap().is_empty());

        let err = service
            .remove_feat(&character_id, "Alert")
            .expect_err("nothing left to remove");
        assert!(matches!(err, ServiceError::Validation(ref m) if m.contains("doesn't have")));
    }
}

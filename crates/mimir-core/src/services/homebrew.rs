//! Homebrew Service
//!
//! Business logic for homebrew items, monsters, and spells.
//! Centralizes UUID generation, timestamp management, and JSON validation
//! that was previously duplicated across Tauri commands and MCP tools.

use diesel::SqliteConnection;
use serde_json::Value;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::dal::catalog as catalog_dal;
use crate::models::campaign::{
    CampaignHomebrewItem, CampaignHomebrewMonster, CampaignHomebrewSpell,
    NewCampaignHomebrewItem, NewCampaignHomebrewMonster, NewCampaignHomebrewSpell,
    UpdateCampaignHomebrewItem, UpdateCampaignHomebrewMonster, UpdateCampaignHomebrewSpell,
};
use crate::services::{ServiceError, ServiceResult};
use crate::utils::now_rfc3339;

// ── Input structs ──────────────────────────────────────────────────────

/// Input for creating a homebrew item.
///
/// When `cloned_from_name` and `cloned_from_source` are provided, the catalog
/// item's data blob is used as the base. Any user-provided `data` fields are
/// deep-merged on top (user overrides win). `data` is optional when cloning.
#[derive(Debug, Clone)]
pub struct CreateHomebrewItemInput {
    pub campaign_id: String,
    pub name: String,
    pub data: Option<String>,
    pub item_type: Option<String>,
    pub rarity: Option<String>,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew item.
#[derive(Debug, Clone, Default)]
pub struct UpdateHomebrewItemInput {
    pub name: Option<String>,
    pub data: Option<String>,
    pub item_type: Option<Option<String>>,
    pub rarity: Option<Option<String>>,
}

/// Input for creating a homebrew monster.
///
/// When `cloned_from_name` and `cloned_from_source` are provided, the catalog
/// monster's data blob is used as the base. Any user-provided `data` fields are
/// deep-merged on top. `data` is optional when cloning.
#[derive(Debug, Clone)]
pub struct CreateHomebrewMonsterInput {
    pub campaign_id: String,
    pub name: String,
    pub data: Option<String>,
    pub cr: Option<String>,
    pub creature_type: Option<String>,
    pub size: Option<String>,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew monster.
#[derive(Debug, Clone, Default)]
pub struct UpdateHomebrewMonsterInput {
    pub name: Option<String>,
    pub data: Option<String>,
    pub cr: Option<Option<String>>,
    pub creature_type: Option<Option<String>>,
    pub size: Option<Option<String>>,
}

/// Input for creating a homebrew spell.
///
/// When `cloned_from_name` and `cloned_from_source` are provided, the catalog
/// spell's data blob is used as the base. Any user-provided `data` fields are
/// deep-merged on top. `data` is optional when cloning.
#[derive(Debug, Clone)]
pub struct CreateHomebrewSpellInput {
    pub campaign_id: String,
    pub name: String,
    pub data: Option<String>,
    pub level: Option<i32>,
    pub school: Option<String>,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew spell.
#[derive(Debug, Clone, Default)]
pub struct UpdateHomebrewSpellInput {
    pub name: Option<String>,
    pub data: Option<String>,
    pub level: Option<Option<i32>>,
    pub school: Option<Option<String>>,
}

// ── Service ────────────────────────────────────────────────────────────

/// Service for homebrew content management.
pub struct HomebrewService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> HomebrewService<'a> {
    /// Create a new homebrew service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    // ── Items ──────────────────────────────────────────────────────

    /// List all homebrew items for a campaign.
    pub fn list_items(&mut self, campaign_id: &str) -> ServiceResult<Vec<CampaignHomebrewItem>> {
        dal::list_campaign_homebrew_items(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Get a homebrew item by ID.
    pub fn get_item(&mut self, id: &str) -> ServiceResult<CampaignHomebrewItem> {
        dal::get_campaign_homebrew_item(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a homebrew item by campaign ID and name.
    pub fn get_item_by_name(
        &mut self,
        campaign_id: &str,
        name: &str,
    ) -> ServiceResult<Option<CampaignHomebrewItem>> {
        dal::get_campaign_homebrew_item_by_name(self.conn, campaign_id, name)
            .map_err(ServiceError::from)
    }

    /// Create a homebrew item.
    ///
    /// When `cloned_from_name` + `cloned_from_source` are provided, looks up the
    /// catalog item and uses its data as the base, with user overrides merged on top.
    pub fn create_item(
        &mut self,
        input: CreateHomebrewItemInput,
    ) -> ServiceResult<CampaignHomebrewItem> {
        let data = resolve_clone_data(
            input.data.as_deref(),
            input.cloned_from_name.as_deref(),
            input.cloned_from_source.as_deref(),
            |name, source| {
                catalog_dal::get_item_by_name(self.conn, name, source)
                    .map_err(ServiceError::from)?
                    .map(|item| item.data)
                    .ok_or_else(|| ServiceError::not_found("CatalogItem", &format!("{name} ({source})")))
            },
        )?;

        let id = Uuid::new_v4().to_string();
        let mut new_item = NewCampaignHomebrewItem::new(&id, &input.campaign_id, &input.name, &data);

        if let Some(ref t) = input.item_type {
            new_item = new_item.with_item_type(t);
        }
        if let Some(ref r) = input.rarity {
            new_item = new_item.with_rarity(r);
        }
        if let (Some(ref n), Some(ref s)) = (&input.cloned_from_name, &input.cloned_from_source) {
            new_item = new_item.cloned_from(n, s);
        }

        dal::insert_campaign_homebrew_item(self.conn, &new_item)?;
        dal::get_campaign_homebrew_item(self.conn, &id).map_err(ServiceError::from)
    }

    /// Update a homebrew item.
    pub fn update_item(
        &mut self,
        id: &str,
        input: UpdateHomebrewItemInput,
    ) -> ServiceResult<CampaignHomebrewItem> {
        if let Some(ref data) = input.data {
            validate_json(data)?;
        }

        let now = now_rfc3339();
        let name_ref = input.name.as_deref();
        let data_ref = input.data.as_deref();
        let item_type_ref = input.item_type.as_ref().map(|v| v.as_deref());
        let rarity_ref = input.rarity.as_ref().map(|v| v.as_deref());

        let update = UpdateCampaignHomebrewItem {
            name: name_ref,
            data: data_ref,
            item_type: item_type_ref,
            rarity: rarity_ref,
            updated_at: Some(&now),
        };

        let rows = dal::update_campaign_homebrew_item(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewItem", id));
        }

        dal::get_campaign_homebrew_item(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a homebrew item.
    pub fn delete_item(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_campaign_homebrew_item(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewItem", id));
        }
        Ok(())
    }

    // ── Monsters ───────────────────────────────────────────────────

    /// List all homebrew monsters for a campaign.
    pub fn list_monsters(
        &mut self,
        campaign_id: &str,
    ) -> ServiceResult<Vec<CampaignHomebrewMonster>> {
        dal::list_campaign_homebrew_monsters(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Get a homebrew monster by ID.
    pub fn get_monster(&mut self, id: &str) -> ServiceResult<CampaignHomebrewMonster> {
        dal::get_campaign_homebrew_monster(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a homebrew monster by campaign ID and name.
    pub fn get_monster_by_name(
        &mut self,
        campaign_id: &str,
        name: &str,
    ) -> ServiceResult<Option<CampaignHomebrewMonster>> {
        dal::get_campaign_homebrew_monster_by_name(self.conn, campaign_id, name)
            .map_err(ServiceError::from)
    }

    /// Create a homebrew monster.
    ///
    /// When `cloned_from_name` + `cloned_from_source` are provided, looks up the
    /// catalog monster and uses its data as the base, with user overrides merged on top.
    pub fn create_monster(
        &mut self,
        input: CreateHomebrewMonsterInput,
    ) -> ServiceResult<CampaignHomebrewMonster> {
        let base_data = resolve_clone_data(
            input.data.as_deref(),
            input.cloned_from_name.as_deref(),
            input.cloned_from_source.as_deref(),
            |name, source| {
                catalog_dal::get_monster_by_name(self.conn, name, source)
                    .map_err(ServiceError::from)?
                    .map(|m| m.data)
                    .ok_or_else(|| ServiceError::not_found("CatalogMonster", &format!("{name} ({source})")))
            },
        )?;

        let data = enrich_monster_data(&base_data, input.cr.as_deref(), input.creature_type.as_deref(), input.size.as_deref())?;
        let id = Uuid::new_v4().to_string();
        let mut new_monster =
            NewCampaignHomebrewMonster::new(&id, &input.campaign_id, &input.name, &data);

        if let Some(ref cr) = input.cr {
            new_monster = new_monster.with_cr(cr);
        }
        if let Some(ref ct) = input.creature_type {
            new_monster = new_monster.with_creature_type(ct);
        }
        if let Some(ref sz) = input.size {
            new_monster = new_monster.with_size(sz);
        }
        if let (Some(ref n), Some(ref s)) = (&input.cloned_from_name, &input.cloned_from_source) {
            new_monster = new_monster.cloned_from(n, s);
        }

        dal::insert_campaign_homebrew_monster(self.conn, &new_monster)?;
        dal::get_campaign_homebrew_monster(self.conn, &id).map_err(ServiceError::from)
    }

    /// Update a homebrew monster.
    pub fn update_monster(
        &mut self,
        id: &str,
        input: UpdateHomebrewMonsterInput,
    ) -> ServiceResult<CampaignHomebrewMonster> {
        if let Some(ref data) = input.data {
            validate_json(data)?;
        }

        // Resolve the effective cr/creature_type/size for data enrichment
        let cr_val = input.cr.as_ref().and_then(|v| v.as_deref());
        let ct_val = input.creature_type.as_ref().and_then(|v| v.as_deref());
        let sz_val = input.size.as_ref().and_then(|v| v.as_deref());

        let enriched_data = match &input.data {
            Some(data) => Some(enrich_monster_data(data, cr_val, ct_val, sz_val)?),
            None => None,
        };

        let now = now_rfc3339();
        let name_ref = input.name.as_deref();
        let data_ref = enriched_data.as_deref().or(input.data.as_deref());
        let cr_ref = input.cr.as_ref().map(|v| v.as_deref());
        let creature_type_ref = input.creature_type.as_ref().map(|v| v.as_deref());
        let size_ref = input.size.as_ref().map(|v| v.as_deref());

        let update = UpdateCampaignHomebrewMonster {
            name: name_ref,
            data: data_ref,
            cr: cr_ref,
            creature_type: creature_type_ref,
            size: size_ref,
            updated_at: Some(&now),
        };

        let rows = dal::update_campaign_homebrew_monster(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewMonster", id));
        }

        dal::get_campaign_homebrew_monster(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a homebrew monster.
    pub fn delete_monster(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_campaign_homebrew_monster(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewMonster", id));
        }
        Ok(())
    }

    // ── Spells ─────────────────────────────────────────────────────

    /// List all homebrew spells for a campaign.
    pub fn list_spells(
        &mut self,
        campaign_id: &str,
    ) -> ServiceResult<Vec<CampaignHomebrewSpell>> {
        dal::list_campaign_homebrew_spells(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Get a homebrew spell by ID.
    pub fn get_spell(&mut self, id: &str) -> ServiceResult<CampaignHomebrewSpell> {
        dal::get_campaign_homebrew_spell(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a homebrew spell by campaign ID and name.
    pub fn get_spell_by_name(
        &mut self,
        campaign_id: &str,
        name: &str,
    ) -> ServiceResult<Option<CampaignHomebrewSpell>> {
        dal::get_campaign_homebrew_spell_by_name(self.conn, campaign_id, name)
            .map_err(ServiceError::from)
    }

    /// Create a homebrew spell.
    ///
    /// When `cloned_from_name` + `cloned_from_source` are provided, looks up the
    /// catalog spell and uses its data as the base, with user overrides merged on top.
    pub fn create_spell(
        &mut self,
        input: CreateHomebrewSpellInput,
    ) -> ServiceResult<CampaignHomebrewSpell> {
        let data = resolve_clone_data(
            input.data.as_deref(),
            input.cloned_from_name.as_deref(),
            input.cloned_from_source.as_deref(),
            |name, source| {
                catalog_dal::get_spell_by_name(self.conn, name, source)
                    .map_err(ServiceError::from)?
                    .map(|s| s.data)
                    .ok_or_else(|| ServiceError::not_found("CatalogSpell", &format!("{name} ({source})")))
            },
        )?;

        let id = Uuid::new_v4().to_string();
        let mut new_spell =
            NewCampaignHomebrewSpell::new(&id, &input.campaign_id, &input.name, &data);

        if let Some(level) = input.level {
            new_spell = new_spell.with_level(level);
        }
        if let Some(ref school) = input.school {
            new_spell = new_spell.with_school(school);
        }
        if let (Some(ref n), Some(ref s)) = (&input.cloned_from_name, &input.cloned_from_source) {
            new_spell = new_spell.cloned_from(n, s);
        }

        dal::insert_campaign_homebrew_spell(self.conn, &new_spell)?;
        dal::get_campaign_homebrew_spell(self.conn, &id).map_err(ServiceError::from)
    }

    /// Update a homebrew spell.
    pub fn update_spell(
        &mut self,
        id: &str,
        input: UpdateHomebrewSpellInput,
    ) -> ServiceResult<CampaignHomebrewSpell> {
        if let Some(ref data) = input.data {
            validate_json(data)?;
        }

        let now = now_rfc3339();
        let name_ref = input.name.as_deref();
        let data_ref = input.data.as_deref();
        let level_ref = input.level.as_ref().copied();
        let school_ref = input.school.as_ref().map(|v| v.as_deref());

        let update = UpdateCampaignHomebrewSpell {
            name: name_ref,
            data: data_ref,
            level: level_ref,
            school: school_ref,
            updated_at: Some(&now),
        };

        let rows = dal::update_campaign_homebrew_spell(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewSpell", id));
        }

        dal::get_campaign_homebrew_spell(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a homebrew spell.
    pub fn delete_spell(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_campaign_homebrew_spell(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("HomebrewSpell", id));
        }
        Ok(())
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Resolve the data blob for a homebrew entity, handling clone-from-catalog.
///
/// - If cloning (`cloned_from_name` + `cloned_from_source` provided): look up the
///   catalog entity's data, then deep-merge any user overrides on top.
/// - If not cloning: `user_data` is required and used as-is.
///
/// The `lookup_fn` fetches the catalog entity's data string given (name, source).
fn resolve_clone_data<F>(
    user_data: Option<&str>,
    cloned_from_name: Option<&str>,
    cloned_from_source: Option<&str>,
    lookup_fn: F,
) -> ServiceResult<String>
where
    F: FnOnce(&str, &str) -> ServiceResult<String>,
{
    match (cloned_from_name, cloned_from_source) {
        (Some(name), Some(source)) => {
            let catalog_data = lookup_fn(name, source)?;

            match user_data {
                Some(overrides) if overrides != "{}" => {
                    // Deep merge user overrides on top of catalog data
                    validate_json(&catalog_data)?;
                    validate_json(overrides)?;
                    let mut base: Value = serde_json::from_str(&catalog_data)
                        .map_err(|e| ServiceError::validation(format!("Invalid catalog JSON: {e}")))?;
                    let user_val: Value = serde_json::from_str(overrides)
                        .map_err(|e| ServiceError::validation(format!("Invalid override JSON: {e}")))?;
                    deep_merge(&mut base, &user_val);
                    serde_json::to_string(&base)
                        .map_err(|e| ServiceError::validation(format!("Failed to serialize merged data: {e}")))
                }
                _ => {
                    // No user overrides — use catalog data as-is
                    validate_json(&catalog_data)?;
                    Ok(catalog_data)
                }
            }
        }
        _ => {
            // Not cloning — data is required
            let data = user_data.ok_or_else(|| {
                ServiceError::validation("data is required when not cloning from catalog".to_string())
            })?;
            validate_json(data)?;
            Ok(data.to_string())
        }
    }
}

/// Deep merge `source` into `target`. For objects, recursively merge keys.
/// For all other types (arrays, strings, numbers), source overwrites target.
fn deep_merge(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(ref mut target_map), Value::Object(source_map)) => {
            for (key, source_val) in source_map {
                let target_val = target_map.entry(key.clone()).or_insert(Value::Null);
                deep_merge(target_val, source_val);
            }
        }
        (target, source) => {
            *target = source.clone();
        }
    }
}

/// Validate that a string is valid JSON.
fn validate_json(data: &str) -> ServiceResult<()> {
    serde_json::from_str::<Value>(data)
        .map(|_| ())
        .map_err(|e| ServiceError::validation(format!("Invalid JSON data: {e}")))
}

/// Ensure the monster data JSON blob includes cr, type, and size fields
/// that match the top-level metadata. The UI stat block renderer reads
/// these from the data blob, so they must be present for proper display.
fn enrich_monster_data(
    data: &str,
    cr: Option<&str>,
    creature_type: Option<&str>,
    size: Option<&str>,
) -> ServiceResult<String> {
    let mut obj: serde_json::Map<String, Value> = serde_json::from_str(data)
        .map_err(|e| ServiceError::validation(format!("Invalid JSON data: {e}")))?;

    // Set cr if missing (5etools format: string like "10" or "1/4")
    if !obj.contains_key("cr") {
        if let Some(cr) = cr {
            obj.insert("cr".to_string(), Value::String(cr.to_string()));
        }
    }

    // Set type if missing (5etools format: {"type": "elemental"} or just "elemental")
    if !obj.contains_key("type") {
        if let Some(ct) = creature_type {
            obj.insert(
                "type".to_string(),
                serde_json::json!({"type": ct}),
            );
        }
    }

    // Set size if missing (5etools format: array like ["H"])
    if !obj.contains_key("size") {
        if let Some(sz) = size {
            obj.insert(
                "size".to_string(),
                Value::Array(vec![Value::String(sz.to_string())]),
            );
        }
    }

    serde_json::to_string(&obj)
        .map_err(|e| ServiceError::validation(format!("Failed to serialize enriched data: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign::insert_campaign;
    use crate::models::campaign::NewCampaign;
    use crate::test_utils::setup_test_db;

    fn create_test_campaign(conn: &mut SqliteConnection) -> String {
        let campaign_id = Uuid::new_v4().to_string();
        let campaign = NewCampaign::new(&campaign_id, "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");
        campaign_id
    }

    // ── enrich_monster_data unit tests ─────────────────────────────

    #[test]
    fn test_enrich_adds_missing_cr() {
        let data = r#"{"name":"Test"}"#;
        let result = enrich_monster_data(data, Some("5"), None, None).unwrap();
        let obj: serde_json::Map<String, Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(obj["cr"], Value::String("5".to_string()));
    }

    #[test]
    fn test_enrich_preserves_existing_cr() {
        let data = r#"{"name":"Test","cr":"10"}"#;
        let result = enrich_monster_data(data, Some("5"), None, None).unwrap();
        let obj: serde_json::Map<String, Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(obj["cr"], Value::String("10".to_string()));
    }

    #[test]
    fn test_enrich_adds_missing_type() {
        let data = r#"{"name":"Test"}"#;
        let result = enrich_monster_data(data, None, Some("elemental"), None).unwrap();
        let obj: serde_json::Map<String, Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(obj["type"], serde_json::json!({"type": "elemental"}));
    }

    #[test]
    fn test_enrich_preserves_existing_type() {
        let data = r#"{"name":"Test","type":"dragon"}"#;
        let result = enrich_monster_data(data, None, Some("elemental"), None).unwrap();
        let obj: serde_json::Map<String, Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(obj["type"], Value::String("dragon".to_string()));
    }

    #[test]
    fn test_enrich_adds_missing_size() {
        let data = r#"{"name":"Test"}"#;
        let result = enrich_monster_data(data, None, None, Some("H")).unwrap();
        let obj: serde_json::Map<String, Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(obj["size"], serde_json::json!(["H"]));
    }

    #[test]
    fn test_enrich_preserves_existing_size() {
        let data = r#"{"name":"Test","size":["M"]}"#;
        let result = enrich_monster_data(data, None, None, Some("H")).unwrap();
        let obj: serde_json::Map<String, Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(obj["size"], serde_json::json!(["M"]));
    }

    #[test]
    fn test_enrich_adds_all_fields() {
        let data = r#"{"name":"Frost Colossus"}"#;
        let result = enrich_monster_data(data, Some("20"), Some("elemental"), Some("G")).unwrap();
        let obj: serde_json::Map<String, Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(obj["cr"], Value::String("20".to_string()));
        assert_eq!(obj["type"], serde_json::json!({"type": "elemental"}));
        assert_eq!(obj["size"], serde_json::json!(["G"]));
    }

    #[test]
    fn test_enrich_no_args_is_noop() {
        let data = r#"{"name":"Test"}"#;
        let result = enrich_monster_data(data, None, None, None).unwrap();
        let obj: serde_json::Map<String, Value> = serde_json::from_str(&result).unwrap();
        assert!(!obj.contains_key("cr"));
        assert!(!obj.contains_key("type"));
        assert!(!obj.contains_key("size"));
    }

    #[test]
    fn test_enrich_invalid_json() {
        let result = enrich_monster_data("not json", Some("5"), None, None);
        assert!(result.is_err());
    }

    // ── validate_json unit tests ──────────────────────────────────

    #[test]
    fn test_validate_json_valid() {
        assert!(validate_json(r#"{"name":"Test"}"#).is_ok());
    }

    #[test]
    fn test_validate_json_invalid() {
        assert!(validate_json("not json").is_err());
    }

    // ── Item CRUD tests ───────────────────────────────────────────

    #[test]
    fn test_create_item() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let item = service
            .create_item(CreateHomebrewItemInput {
                campaign_id: campaign_id.clone(),
                name: "Flame Sword".to_string(),
                data: Some(r#"{"name":"Flame Sword","rarity":"rare"}"#.to_string()),
                item_type: Some("weapon".to_string()),
                rarity: Some("rare".to_string()),
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .expect("Failed to create item");

        assert_eq!(item.name, "Flame Sword");
        assert_eq!(item.campaign_id, campaign_id);
        assert_eq!(item.item_type, Some("weapon".to_string()));
        assert_eq!(item.rarity, Some("rare".to_string()));
    }

    #[test]
    fn test_create_item_invalid_json() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let result = service.create_item(CreateHomebrewItemInput {
            campaign_id,
            name: "Bad Item".to_string(),
            data: Some("not json".to_string()),
            item_type: None,
            rarity: None,
            cloned_from_name: None,
            cloned_from_source: None,
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_list_items() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        service
            .create_item(CreateHomebrewItemInput {
                campaign_id: campaign_id.clone(),
                name: "Item 1".to_string(),
                data: Some(r#"{}"#.to_string()),
                item_type: None,
                rarity: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();
        service
            .create_item(CreateHomebrewItemInput {
                campaign_id: campaign_id.clone(),
                name: "Item 2".to_string(),
                data: Some(r#"{}"#.to_string()),
                item_type: None,
                rarity: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let items = service.list_items(&campaign_id).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_get_item() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let created = service
            .create_item(CreateHomebrewItemInput {
                campaign_id,
                name: "Test Item".to_string(),
                data: Some(r#"{"name":"Test Item"}"#.to_string()),
                item_type: None,
                rarity: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let retrieved = service.get_item(&created.id).unwrap();
        assert_eq!(retrieved.name, "Test Item");
    }

    #[test]
    fn test_update_item() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let created = service
            .create_item(CreateHomebrewItemInput {
                campaign_id,
                name: "Original".to_string(),
                data: Some(r#"{}"#.to_string()),
                item_type: None,
                rarity: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let updated = service
            .update_item(
                &created.id,
                UpdateHomebrewItemInput {
                    name: Some("Updated".to_string()),
                    data: Some(r#"{"updated":true}"#.to_string()),
                    item_type: Some(Some("armor".to_string())),
                    rarity: Some(Some("legendary".to_string())),
                },
            )
            .unwrap();

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.item_type, Some("armor".to_string()));
        assert_eq!(updated.rarity, Some("legendary".to_string()));
    }

    #[test]
    fn test_update_item_not_found() {
        let mut conn = setup_test_db();
        let mut service = HomebrewService::new(&mut conn);

        let result = service.update_item(
            "nonexistent",
            UpdateHomebrewItemInput {
                name: Some("test".to_string()),
                ..Default::default()
            },
        );
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_delete_item() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let created = service
            .create_item(CreateHomebrewItemInput {
                campaign_id: campaign_id.clone(),
                name: "To Delete".to_string(),
                data: Some(r#"{}"#.to_string()),
                item_type: None,
                rarity: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        service.delete_item(&created.id).unwrap();
        assert!(service.get_item(&created.id).is_err());
    }

    #[test]
    fn test_delete_item_not_found() {
        let mut conn = setup_test_db();
        let mut service = HomebrewService::new(&mut conn);

        let result = service.delete_item("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    // ── Monster CRUD tests ────────────────────────────────────────

    #[test]
    fn test_create_monster() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let monster = service
            .create_monster(CreateHomebrewMonsterInput {
                campaign_id: campaign_id.clone(),
                name: "Frost Colossus".to_string(),
                data: Some(r#"{"name":"Frost Colossus"}"#.to_string()),
                cr: Some("20".to_string()),
                creature_type: Some("elemental".to_string()),
                size: Some("G".to_string()),
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .expect("Failed to create monster");

        assert_eq!(monster.name, "Frost Colossus");
        assert_eq!(monster.cr, Some("20".to_string()));
        assert_eq!(monster.creature_type, Some("elemental".to_string()));
        assert_eq!(monster.size, Some("G".to_string()));

        // Verify data enrichment
        let data: serde_json::Map<String, Value> =
            serde_json::from_str(&monster.data).unwrap();
        assert_eq!(data["cr"], Value::String("20".to_string()));
        assert_eq!(data["type"], serde_json::json!({"type": "elemental"}));
        assert_eq!(data["size"], serde_json::json!(["G"]));
    }

    #[test]
    fn test_create_monster_enrichment_preserves_existing() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let monster = service
            .create_monster(CreateHomebrewMonsterInput {
                campaign_id,
                name: "Dragon".to_string(),
                data: Some(r#"{"name":"Dragon","cr":"15","type":"dragon","size":["H"]}"#.to_string()),
                cr: Some("20".to_string()),
                creature_type: Some("elemental".to_string()),
                size: Some("G".to_string()),
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let data: serde_json::Map<String, Value> =
            serde_json::from_str(&monster.data).unwrap();
        // Existing values should NOT be overwritten
        assert_eq!(data["cr"], Value::String("15".to_string()));
        assert_eq!(data["type"], Value::String("dragon".to_string()));
        assert_eq!(data["size"], serde_json::json!(["H"]));
    }

    #[test]
    fn test_create_monster_empty_data_gets_enriched() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let monster = service
            .create_monster(CreateHomebrewMonsterInput {
                campaign_id,
                name: "Empty Monster".to_string(),
                data: Some(r#"{}"#.to_string()),
                cr: Some("1/4".to_string()),
                creature_type: Some("beast".to_string()),
                size: Some("S".to_string()),
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let data: serde_json::Map<String, Value> =
            serde_json::from_str(&monster.data).unwrap();
        assert_eq!(data["cr"], Value::String("1/4".to_string()));
        assert_eq!(data["type"], serde_json::json!({"type": "beast"}));
        assert_eq!(data["size"], serde_json::json!(["S"]));
    }

    #[test]
    fn test_list_monsters() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        service
            .create_monster(CreateHomebrewMonsterInput {
                campaign_id: campaign_id.clone(),
                name: "Monster 1".to_string(),
                data: Some(r#"{}"#.to_string()),
                cr: None,
                creature_type: None,
                size: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let monsters = service.list_monsters(&campaign_id).unwrap();
        assert_eq!(monsters.len(), 1);
    }

    #[test]
    fn test_update_monster() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let created = service
            .create_monster(CreateHomebrewMonsterInput {
                campaign_id,
                name: "Original".to_string(),
                data: Some(r#"{"name":"Original"}"#.to_string()),
                cr: Some("5".to_string()),
                creature_type: None,
                size: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let updated = service
            .update_monster(
                &created.id,
                UpdateHomebrewMonsterInput {
                    name: Some("Updated".to_string()),
                    data: Some(r#"{"name":"Updated"}"#.to_string()),
                    cr: Some(Some("10".to_string())),
                    creature_type: Some(Some("fiend".to_string())),
                    size: Some(Some("L".to_string())),
                },
            )
            .unwrap();

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.cr, Some("10".to_string()));
    }

    #[test]
    fn test_update_monster_not_found() {
        let mut conn = setup_test_db();
        let mut service = HomebrewService::new(&mut conn);

        let result = service.update_monster(
            "nonexistent",
            UpdateHomebrewMonsterInput {
                name: Some("test".to_string()),
                ..Default::default()
            },
        );
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_delete_monster() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let created = service
            .create_monster(CreateHomebrewMonsterInput {
                campaign_id,
                name: "To Delete".to_string(),
                data: Some(r#"{}"#.to_string()),
                cr: None,
                creature_type: None,
                size: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        service.delete_monster(&created.id).unwrap();
        assert!(service.get_monster(&created.id).is_err());
    }

    #[test]
    fn test_delete_monster_not_found() {
        let mut conn = setup_test_db();
        let mut service = HomebrewService::new(&mut conn);

        let result = service.delete_monster("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    // ── Spell CRUD tests ──────────────────────────────────────────

    #[test]
    fn test_create_spell() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let spell = service
            .create_spell(CreateHomebrewSpellInput {
                campaign_id: campaign_id.clone(),
                name: "Arcane Blast".to_string(),
                data: Some(r#"{"name":"Arcane Blast","level":3}"#.to_string()),
                level: Some(3),
                school: Some("evocation".to_string()),
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .expect("Failed to create spell");

        assert_eq!(spell.name, "Arcane Blast");
        assert_eq!(spell.level, Some(3));
        assert_eq!(spell.school, Some("evocation".to_string()));
    }

    #[test]
    fn test_create_spell_invalid_json() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let result = service.create_spell(CreateHomebrewSpellInput {
            campaign_id,
            name: "Bad Spell".to_string(),
            data: Some("not json".to_string()),
            level: None,
            school: None,
            cloned_from_name: None,
            cloned_from_source: None,
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_list_spells() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        service
            .create_spell(CreateHomebrewSpellInput {
                campaign_id: campaign_id.clone(),
                name: "Spell 1".to_string(),
                data: Some(r#"{}"#.to_string()),
                level: Some(0),
                school: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();
        service
            .create_spell(CreateHomebrewSpellInput {
                campaign_id: campaign_id.clone(),
                name: "Spell 2".to_string(),
                data: Some(r#"{}"#.to_string()),
                level: Some(5),
                school: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let spells = service.list_spells(&campaign_id).unwrap();
        assert_eq!(spells.len(), 2);
    }

    #[test]
    fn test_update_spell() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let created = service
            .create_spell(CreateHomebrewSpellInput {
                campaign_id,
                name: "Original Spell".to_string(),
                data: Some(r#"{}"#.to_string()),
                level: Some(1),
                school: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let updated = service
            .update_spell(
                &created.id,
                UpdateHomebrewSpellInput {
                    name: Some("Updated Spell".to_string()),
                    data: Some(r#"{"updated":true}"#.to_string()),
                    level: Some(Some(5)),
                    school: Some(Some("necromancy".to_string())),
                },
            )
            .unwrap();

        assert_eq!(updated.name, "Updated Spell");
        assert_eq!(updated.level, Some(5));
        assert_eq!(updated.school, Some("necromancy".to_string()));
    }

    #[test]
    fn test_update_spell_not_found() {
        let mut conn = setup_test_db();
        let mut service = HomebrewService::new(&mut conn);

        let result = service.update_spell(
            "nonexistent",
            UpdateHomebrewSpellInput {
                name: Some("test".to_string()),
                ..Default::default()
            },
        );
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_delete_spell() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let created = service
            .create_spell(CreateHomebrewSpellInput {
                campaign_id,
                name: "To Delete".to_string(),
                data: Some(r#"{}"#.to_string()),
                level: None,
                school: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        service.delete_spell(&created.id).unwrap();
        assert!(service.get_spell(&created.id).is_err());
    }

    #[test]
    fn test_delete_spell_not_found() {
        let mut conn = setup_test_db();
        let mut service = HomebrewService::new(&mut conn);

        let result = service.delete_spell("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    // ── get_by_name tests ─────────────────────────────────────────

    #[test]
    fn test_get_item_by_name() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        service
            .create_item(CreateHomebrewItemInput {
                campaign_id: campaign_id.clone(),
                name: "Unique Item".to_string(),
                data: Some(r#"{}"#.to_string()),
                item_type: None,
                rarity: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let found = service.get_item_by_name(&campaign_id, "Unique Item").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Unique Item");

        let not_found = service.get_item_by_name(&campaign_id, "Nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_monster_by_name() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        service
            .create_monster(CreateHomebrewMonsterInput {
                campaign_id: campaign_id.clone(),
                name: "Custom Dragon".to_string(),
                data: Some(r#"{}"#.to_string()),
                cr: None,
                creature_type: None,
                size: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let found = service.get_monster_by_name(&campaign_id, "Custom Dragon").unwrap();
        assert!(found.is_some());

        let not_found = service.get_monster_by_name(&campaign_id, "Nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_spell_by_name() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        service
            .create_spell(CreateHomebrewSpellInput {
                campaign_id: campaign_id.clone(),
                name: "Custom Spell".to_string(),
                data: Some(r#"{}"#.to_string()),
                level: None,
                school: None,
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();

        let found = service.get_spell_by_name(&campaign_id, "Custom Spell").unwrap();
        assert!(found.is_some());

        let not_found = service.get_spell_by_name(&campaign_id, "Nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    // ── clone-from-catalog tests ──────────────────────────────────

    #[test]
    fn test_clone_item_catalog_not_found() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let result = service.create_item(CreateHomebrewItemInput {
            campaign_id,
            name: "Cloned Sword".to_string(),
            data: None,
            item_type: None,
            rarity: None,
            cloned_from_name: Some("Nonexistent".to_string()),
            cloned_from_source: Some("PHB".to_string()),
        });

        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_clone_monster_catalog_not_found() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let result = service.create_monster(CreateHomebrewMonsterInput {
            campaign_id,
            name: "Custom Goblin".to_string(),
            data: None,
            cr: None,
            creature_type: None,
            size: None,
            cloned_from_name: Some("Nonexistent".to_string()),
            cloned_from_source: Some("MM".to_string()),
        });

        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_clone_spell_catalog_not_found() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let result = service.create_spell(CreateHomebrewSpellInput {
            campaign_id,
            name: "Cloned Spell".to_string(),
            data: None,
            level: None,
            school: None,
            cloned_from_name: Some("Nonexistent".to_string()),
            cloned_from_source: Some("PHB".to_string()),
        });

        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_create_item_requires_data_when_not_cloning() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let result = service.create_item(CreateHomebrewItemInput {
            campaign_id,
            name: "No Data Item".to_string(),
            data: None,
            item_type: None,
            rarity: None,
            cloned_from_name: None,
            cloned_from_source: None,
        });

        assert!(matches!(result, Err(ServiceError::Validation { .. })));
    }

    #[test]
    fn test_create_monster_requires_data_when_not_cloning() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let result = service.create_monster(CreateHomebrewMonsterInput {
            campaign_id,
            name: "No Data Monster".to_string(),
            data: None,
            cr: None,
            creature_type: None,
            size: None,
            cloned_from_name: None,
            cloned_from_source: None,
        });

        assert!(matches!(result, Err(ServiceError::Validation { .. })));
    }

    #[test]
    fn test_create_spell_requires_data_when_not_cloning() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = HomebrewService::new(&mut conn);
        let result = service.create_spell(CreateHomebrewSpellInput {
            campaign_id,
            name: "No Data Spell".to_string(),
            data: None,
            level: None,
            school: None,
            cloned_from_name: None,
            cloned_from_source: None,
        });

        assert!(matches!(result, Err(ServiceError::Validation { .. })));
    }

    #[test]
    fn test_deep_merge_objects() {
        let mut base: Value = serde_json::from_str(r#"{"a":1,"b":{"c":2,"d":3}}"#).unwrap();
        let overrides: Value = serde_json::from_str(r#"{"b":{"c":99,"e":5},"f":6}"#).unwrap();
        deep_merge(&mut base, &overrides);

        assert_eq!(base["a"], serde_json::json!(1));
        assert_eq!(base["b"]["c"], serde_json::json!(99));
        assert_eq!(base["b"]["d"], serde_json::json!(3));
        assert_eq!(base["b"]["e"], serde_json::json!(5));
        assert_eq!(base["f"], serde_json::json!(6));
    }

    #[test]
    fn test_deep_merge_replaces_non_objects() {
        let mut base: Value = serde_json::from_str(r#"{"a":"old","b":[1,2]}"#).unwrap();
        let overrides: Value = serde_json::from_str(r#"{"a":"new","b":[3,4,5]}"#).unwrap();
        deep_merge(&mut base, &overrides);

        assert_eq!(base["a"], serde_json::json!("new"));
        assert_eq!(base["b"], serde_json::json!([3, 4, 5]));
    }
}

//! Token Service
//!
//! Business logic for token placements on maps.

use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{NewTokenPlacement, TokenPlacement, UpdateTokenPlacement};
use crate::services::{MapService, ServiceError, ServiceResult};

/// Input for creating a token placement.
#[derive(Debug, Clone)]
pub struct CreateTokenInput {
    /// Map to place the token on
    pub map_id: String,
    /// Module monster reference (for monster tokens)
    pub module_monster_id: Option<String>,
    /// Module NPC reference (for NPC tokens)
    pub module_npc_id: Option<String>,
    /// Grid X coordinate
    pub grid_x: i32,
    /// Grid Y coordinate
    pub grid_y: i32,
    /// Optional label (required for PC tokens)
    pub label: Option<String>,
    /// Faction color
    pub faction_color: Option<String>,
    /// Whether the token is hidden from players
    pub hidden: bool,
}

impl CreateTokenInput {
    /// Create input for a monster token.
    pub fn for_monster(
        map_id: impl Into<String>,
        module_monster_id: impl Into<String>,
        grid_x: i32,
        grid_y: i32,
    ) -> Self {
        Self {
            map_id: map_id.into(),
            module_monster_id: Some(module_monster_id.into()),
            module_npc_id: None,
            grid_x,
            grid_y,
            label: None,
            faction_color: None,
            hidden: false,
        }
    }

    /// Create input for an NPC token.
    pub fn for_npc(
        map_id: impl Into<String>,
        module_npc_id: impl Into<String>,
        grid_x: i32,
        grid_y: i32,
    ) -> Self {
        Self {
            map_id: map_id.into(),
            module_monster_id: None,
            module_npc_id: Some(module_npc_id.into()),
            grid_x,
            grid_y,
            label: None,
            faction_color: None,
            hidden: false,
        }
    }

    /// Create input for a PC token (requires label).
    pub fn for_pc(map_id: impl Into<String>, label: impl Into<String>, grid_x: i32, grid_y: i32) -> Self {
        Self {
            map_id: map_id.into(),
            module_monster_id: None,
            module_npc_id: None,
            grid_x,
            grid_y,
            label: Some(label.into()),
            faction_color: None,
            hidden: false,
        }
    }

    /// Set a label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set a faction color.
    pub fn with_faction_color(mut self, color: impl Into<String>) -> Self {
        self.faction_color = Some(color.into());
        self
    }

    /// Mark as hidden.
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}

/// Input for updating a token placement.
#[derive(Debug, Clone, Default)]
pub struct UpdateTokenInput {
    /// Update grid X coordinate
    pub grid_x: Option<i32>,
    /// Update grid Y coordinate
    pub grid_y: Option<i32>,
    /// Update label (Some(None) to clear)
    pub label: Option<Option<String>>,
    /// Update faction color (Some(None) to clear)
    pub faction_color: Option<Option<String>>,
    /// Update hidden state
    pub hidden: Option<bool>,
    /// Update vision bright range
    pub vision_bright_ft: Option<Option<i32>>,
    /// Update vision dim range
    pub vision_dim_ft: Option<Option<i32>>,
    /// Update vision dark range
    pub vision_dark_ft: Option<i32>,
    /// Update light radius
    pub light_radius_ft: Option<i32>,
}

impl UpdateTokenInput {
    /// Set position.
    pub fn set_position(grid_x: i32, grid_y: i32) -> Self {
        Self {
            grid_x: Some(grid_x),
            grid_y: Some(grid_y),
            ..Default::default()
        }
    }

    /// Set visibility.
    pub fn set_hidden(hidden: bool) -> Self {
        Self {
            hidden: Some(hidden),
            ..Default::default()
        }
    }

    /// Set vision settings.
    pub fn set_vision(
        vision_bright_ft: Option<i32>,
        vision_dim_ft: Option<i32>,
        vision_dark_ft: i32,
        light_radius_ft: i32,
    ) -> Self {
        Self {
            vision_bright_ft: Some(vision_bright_ft),
            vision_dim_ft: Some(vision_dim_ft),
            vision_dark_ft: Some(vision_dark_ft),
            light_radius_ft: Some(light_radius_ft),
            ..Default::default()
        }
    }
}

/// Token with resolved name and type information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub id: String,
    pub map_id: String,
    pub name: String,
    pub token_type: String,
    pub size: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub x: f64,
    pub y: f64,
    pub visible_to_players: bool,
    pub color: Option<String>,
    pub monster_id: Option<String>,
    pub character_id: Option<String>,
    pub vision_bright_ft: Option<i32>,
    pub vision_dim_ft: Option<i32>,
    pub vision_dark_ft: i32,
    pub light_radius_ft: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Service for token management.
pub struct TokenService<'a> {
    conn: &'a mut SqliteConnection,
    app_dir: &'a Path,
}

impl<'a> TokenService<'a> {
    /// Create a new token service.
    pub fn new(conn: &'a mut SqliteConnection, app_dir: &'a Path) -> Self {
        Self { conn, app_dir }
    }

    /// List all tokens for a map.
    pub fn list(&mut self, map_id: &str) -> ServiceResult<Vec<TokenResponse>> {
        let grid_size_px = self.get_grid_size(map_id);
        let placements = dal::list_token_placements(self.conn, map_id)?;

        let mut result = Vec::with_capacity(placements.len());
        for placement in placements {
            result.push(self.enrich(placement, grid_size_px)?);
        }
        Ok(result)
    }

    /// List visible tokens for a map (player view).
    pub fn list_visible(&mut self, map_id: &str) -> ServiceResult<Vec<TokenResponse>> {
        let grid_size_px = self.get_grid_size(map_id);
        let placements = dal::list_visible_token_placements(self.conn, map_id)?;

        let mut result = Vec::with_capacity(placements.len());
        for placement in placements {
            result.push(self.enrich(placement, grid_size_px)?);
        }
        Ok(result)
    }

    /// Get a token by ID.
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<TokenResponse>> {
        let placement = match dal::get_token_placement_optional(self.conn, id)? {
            Some(p) => p,
            None => return Ok(None),
        };
        let grid_size_px = self.get_grid_size(&placement.map_id);
        Ok(Some(self.enrich(placement, grid_size_px)?))
    }

    /// Create a new token.
    pub fn create(&mut self, input: CreateTokenInput) -> ServiceResult<TokenResponse> {
        // Validate: can't have both monster_id and npc_id
        if input.module_monster_id.is_some() && input.module_npc_id.is_some() {
            return Err(ServiceError::validation(
                "Only one of module_monster_id or module_npc_id can be provided",
            ));
        }

        // Validate: PC tokens require a label
        if input.module_monster_id.is_none()
            && input.module_npc_id.is_none()
            && input.label.is_none()
        {
            return Err(ServiceError::validation(
                "A label is required for PC tokens (when no monster_id or npc_id is provided)",
            ));
        }

        let id = Uuid::new_v4().to_string();
        let label_ref = input.label.as_deref();
        let color_ref = input.faction_color.as_deref();
        let monster_ref = input.module_monster_id.as_deref();
        let npc_ref = input.module_npc_id.as_deref();

        let placement = NewTokenPlacement {
            id: &id,
            map_id: &input.map_id,
            module_monster_id: monster_ref,
            module_npc_id: npc_ref,
            grid_x: input.grid_x,
            grid_y: input.grid_y,
            label: label_ref,
            faction_color: color_ref,
            hidden: if input.hidden { 1 } else { 0 },
            vision_bright_ft: None,
            vision_dim_ft: None,
            vision_dark_ft: 0,
            light_radius_ft: 0,
        };

        dal::insert_token_placement(self.conn, &placement)?;

        let token = dal::get_token_placement(self.conn, &id)?;
        let grid_size_px = self.get_grid_size(&input.map_id);
        self.enrich(token, grid_size_px)
    }

    /// Update a token.
    pub fn update(&mut self, id: &str, input: UpdateTokenInput) -> ServiceResult<TokenResponse> {
        let label: Option<Option<&str>> = input.label.as_ref().map(|o| o.as_deref());
        let faction_color: Option<Option<&str>> = input.faction_color.as_ref().map(|o| o.as_deref());

        let update = UpdateTokenPlacement {
            grid_x: input.grid_x,
            grid_y: input.grid_y,
            label,
            faction_color,
            hidden: input.hidden.map(|h| if h { 1 } else { 0 }),
            vision_bright_ft: input.vision_bright_ft,
            vision_dim_ft: input.vision_dim_ft,
            vision_dark_ft: input.vision_dark_ft,
            light_radius_ft: input.light_radius_ft,
        };

        let rows = dal::update_token_placement(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Token", id));
        }

        let token = dal::get_token_placement(self.conn, id)?;
        let grid_size_px = self.get_grid_size(&token.map_id);
        self.enrich(token, grid_size_px)
    }

    /// Update just the position (optimized for drag operations).
    pub fn update_position(&mut self, id: &str, grid_x: i32, grid_y: i32) -> ServiceResult<TokenResponse> {
        let update = UpdateTokenPlacement::set_position(grid_x, grid_y);

        let rows = dal::update_token_placement(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Token", id));
        }

        let token = dal::get_token_placement(self.conn, id)?;
        let grid_size_px = self.get_grid_size(&token.map_id);
        self.enrich(token, grid_size_px)
    }

    /// Update vision settings.
    pub fn update_vision(
        &mut self,
        id: &str,
        vision_bright_ft: Option<i32>,
        vision_dim_ft: Option<i32>,
        vision_dark_ft: i32,
        light_radius_ft: i32,
    ) -> ServiceResult<TokenResponse> {
        let update = UpdateTokenPlacement::set_vision(
            vision_bright_ft,
            vision_dim_ft,
            vision_dark_ft,
            light_radius_ft,
        );

        let rows = dal::update_token_placement(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Token", id));
        }

        let token = dal::get_token_placement(self.conn, id)?;
        let grid_size_px = self.get_grid_size(&token.map_id);
        self.enrich(token, grid_size_px)
    }

    /// Toggle visibility.
    pub fn toggle_visibility(&mut self, id: &str) -> ServiceResult<TokenResponse> {
        let token = dal::get_token_placement_optional(self.conn, id)?
            .ok_or_else(|| ServiceError::not_found("Token", id))?;

        let new_hidden = !token.is_hidden();
        let update = UpdateTokenPlacement::set_hidden(new_hidden);
        dal::update_token_placement(self.conn, id, &update)?;

        let updated = dal::get_token_placement(self.conn, id)?;
        let grid_size_px = self.get_grid_size(&updated.map_id);
        self.enrich(updated, grid_size_px)
    }

    /// Delete a token.
    pub fn delete(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_token_placement(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Token", id));
        }
        Ok(())
    }

    /// Count tokens for a map.
    pub fn count(&mut self, map_id: &str) -> ServiceResult<i64> {
        dal::count_token_placements(self.conn, map_id).map_err(ServiceError::from)
    }

    // ── Private helpers ────────────────────────────────────────────────────

    /// Get the grid size (pixels per grid) from a map's UVTT file.
    fn get_grid_size(&mut self, map_id: &str) -> i32 {
        let mut map_service = MapService::new(self.conn, self.app_dir);

        if let Ok(Some(map)) = map_service.get(map_id) {
            if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                if let Ok(uvtt_json) = serde_json::from_slice::<serde_json::Value>(&uvtt_bytes) {
                    return uvtt_json
                        .get("resolution")
                        .and_then(|r| r.get("pixels_per_grid"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(70) as i32;
                }
            }
        }
        70 // Default grid size
    }

    /// Enrich a TokenPlacement with resolved names and computed coordinates.
    fn enrich(&mut self, placement: TokenPlacement, grid_size_px: i32) -> ServiceResult<TokenResponse> {
        let (token_type, name, size) = self.resolve_names(&placement);

        // Convert grid coordinates to pixel coordinates (center of grid cell)
        let x = (placement.grid_x as f64 + 0.5) * grid_size_px as f64;
        let y = (placement.grid_y as f64 + 0.5) * grid_size_px as f64;

        Ok(TokenResponse {
            id: placement.id,
            map_id: placement.map_id,
            name: placement.label.or(name).unwrap_or_else(|| "Unknown".to_string()),
            token_type,
            size,
            grid_x: placement.grid_x,
            grid_y: placement.grid_y,
            x,
            y,
            visible_to_players: placement.hidden == 0,
            color: placement.faction_color,
            monster_id: placement.module_monster_id,
            character_id: placement.module_npc_id,
            vision_bright_ft: placement.vision_bright_ft,
            vision_dim_ft: placement.vision_dim_ft,
            vision_dark_ft: placement.vision_dark_ft,
            light_radius_ft: placement.light_radius_ft,
            created_at: placement.created_at.clone(),
            updated_at: placement.created_at, // Use created_at as updated_at since we don't track it
        })
    }

    /// Resolve token type and name from monster/NPC references.
    fn resolve_names(&mut self, token: &TokenPlacement) -> (String, Option<String>, String) {
        if let Some(ref monster_id) = token.module_monster_id {
            let monster = dal::get_module_monster_optional(self.conn, monster_id)
                .ok()
                .flatten();
            if let Some(m) = monster {
                // Look up the monster in the catalog to get its size
                let size = crate::dal::catalog::get_monster_by_name(
                    self.conn,
                    &m.monster_name,
                    &m.monster_source,
                )
                .ok()
                .flatten()
                .and_then(|catalog_monster| {
                    catalog_monster.size.as_ref().map(|s| normalize_size_code(s))
                })
                .unwrap_or_else(|| "medium".to_string());

                (
                    "monster".to_string(),
                    Some(m.display_name.unwrap_or(m.monster_name)),
                    size,
                )
            } else {
                ("monster".to_string(), None, "medium".to_string())
            }
        } else if let Some(ref npc_id) = token.module_npc_id {
            let npc = dal::get_module_npc_optional(self.conn, npc_id).ok().flatten();
            (
                "npc".to_string(),
                npc.map(|n| n.name),
                "medium".to_string(), // NPCs default to medium
            )
        } else {
            // PC token - use label as name
            ("pc".to_string(), token.label.clone(), "medium".to_string())
        }
    }
}

/// Normalize size codes (T, S, M, L, H, G) to full names.
fn normalize_size_code(size: &str) -> String {
    match size.to_uppercase().as_str() {
        "T" => "tiny".to_string(),
        "S" => "small".to_string(),
        "M" => "medium".to_string(),
        "L" => "large".to_string(),
        "H" => "huge".to_string(),
        "G" => "gargantuan".to_string(),
        other => other.to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign::{
        insert_campaign, insert_campaign_asset, insert_map, insert_module,
        insert_module_monster, insert_module_npc,
    };
    use crate::models::campaign::{
        NewCampaign, NewCampaignAsset, NewMap, NewModule, NewModuleMonster, NewModuleNpc,
    };
    use crate::test_utils::setup_test_db;
    use tempfile::TempDir;

    /// Set up a test environment with an in-memory DB and temp directory.
    fn setup_test_env() -> (SqliteConnection, TempDir) {
        let conn = setup_test_db();
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        (conn, temp_dir)
    }

    /// Create prerequisite entities: campaign -> module -> asset -> map.
    /// Returns (campaign_id, module_id, map_id).
    fn create_prerequisites(conn: &mut SqliteConnection) -> (String, String, String) {
        let campaign_id = "camp-test";
        let module_id = "mod-test";
        let asset_id = "asset-test";
        let map_id = "map-test";

        let campaign = NewCampaign::new(campaign_id, "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");

        let module = NewModule::new(module_id, campaign_id, "Test Module", 1);
        insert_module(conn, &module).expect("Failed to create module");

        let asset = NewCampaignAsset::for_campaign(
            asset_id,
            campaign_id,
            "test.uvtt",
            "application/octet-stream",
            "/blobs/test.uvtt",
        );
        insert_campaign_asset(conn, &asset).expect("Failed to create asset");

        let map = NewMap::for_campaign(map_id, campaign_id, "Test Map", asset_id);
        insert_map(conn, &map).expect("Failed to create map");

        (
            campaign_id.to_string(),
            module_id.to_string(),
            map_id.to_string(),
        )
    }

    /// Create a module monster in the DB and return its ID.
    fn create_module_monster(conn: &mut SqliteConnection, module_id: &str) -> String {
        let id = "mm-test";
        let monster = NewModuleMonster::new(id, module_id, "Goblin", "MM");
        insert_module_monster(conn, &monster).expect("Failed to create monster");
        id.to_string()
    }

    /// Create a module NPC in the DB and return its ID.
    fn create_module_npc(conn: &mut SqliteConnection, module_id: &str) -> String {
        let id = "npc-test";
        let npc = NewModuleNpc::new(id, module_id, "Sildar");
        insert_module_npc(conn, &npc).expect("Failed to create npc");
        id.to_string()
    }

    // ── CRUD: create ──────────────────────────────────────────────────────

    #[test]
    fn test_create_pc_token() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 3, 5);
        let token = service.create(input).expect("Failed to create PC token");

        assert_eq!(token.map_id, map_id);
        assert_eq!(token.name, "Fighter");
        assert_eq!(token.token_type, "pc");
        assert_eq!(token.grid_x, 3);
        assert_eq!(token.grid_y, 5);
        assert!(token.visible_to_players);
        assert!(token.monster_id.is_none());
        assert!(token.character_id.is_none());
    }

    #[test]
    fn test_create_monster_token() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, module_id, map_id) = create_prerequisites(&mut conn);
        let monster_id = create_module_monster(&mut conn, &module_id);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_monster(&map_id, &monster_id, 1, 2);
        let token = service.create(input).expect("Failed to create monster token");

        assert_eq!(token.token_type, "monster");
        assert_eq!(token.monster_id, Some(monster_id));
        assert_eq!(token.grid_x, 1);
        assert_eq!(token.grid_y, 2);
    }

    #[test]
    fn test_create_npc_token() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, module_id, map_id) = create_prerequisites(&mut conn);
        let npc_id = create_module_npc(&mut conn, &module_id);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_npc(&map_id, &npc_id, 7, 8);
        let token = service.create(input).expect("Failed to create NPC token");

        assert_eq!(token.token_type, "npc");
        assert_eq!(token.name, "Sildar");
        assert_eq!(token.character_id, Some(npc_id));
        assert_eq!(token.grid_x, 7);
        assert_eq!(token.grid_y, 8);
    }

    #[test]
    fn test_create_token_with_label_and_color() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, module_id, map_id) = create_prerequisites(&mut conn);
        let monster_id = create_module_monster(&mut conn, &module_id);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_monster(&map_id, &monster_id, 0, 0)
            .with_label("Goblin Boss")
            .with_faction_color("#FF0000");
        let token = service.create(input).expect("Failed to create token");

        // Label overrides resolved name
        assert_eq!(token.name, "Goblin Boss");
        assert_eq!(token.color, Some("#FF0000".to_string()));
    }

    #[test]
    fn test_create_hidden_token() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Rogue", 0, 0).hidden();
        let token = service.create(input).expect("Failed to create hidden token");

        assert!(!token.visible_to_players);
    }

    // ── CRUD: get ─────────────────────────────────────────────────────────

    #[test]
    fn test_get_token() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Wizard", 4, 6);
        let created = service.create(input).expect("Failed to create token");

        let retrieved = service
            .get(&created.id)
            .expect("Failed to get token")
            .expect("Token should exist");

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Wizard");
        assert_eq!(retrieved.grid_x, 4);
        assert_eq!(retrieved.grid_y, 6);
    }

    #[test]
    fn test_get_token_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let result = service.get("nonexistent").expect("Failed to query");
        assert!(result.is_none());
    }

    // ── CRUD: list ────────────────────────────────────────────────────────

    #[test]
    fn test_list_tokens() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        // Empty initially
        let tokens = service.list(&map_id).expect("Failed to list");
        assert!(tokens.is_empty());

        // Create two tokens
        let input1 = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let input2 = CreateTokenInput::for_pc(&map_id, "Wizard", 1, 1);
        service.create(input1).expect("Failed to create");
        service.create(input2).expect("Failed to create");

        let tokens = service.list(&map_id).expect("Failed to list");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_list_tokens_different_maps() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        // Create a second map
        let asset2 = NewCampaignAsset::for_campaign(
            "asset-2",
            "camp-test",
            "test2.uvtt",
            "application/octet-stream",
            "/blobs/test2.uvtt",
        );
        insert_campaign_asset(&mut conn, &asset2).expect("Failed to create asset");
        let map2 = NewMap::for_campaign("map-2", "camp-test", "Map 2", "asset-2");
        insert_map(&mut conn, &map2).expect("Failed to create map 2");

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        let input1 = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let input2 = CreateTokenInput::for_pc("map-2", "Wizard", 1, 1);
        service.create(input1).expect("Failed to create");
        service.create(input2).expect("Failed to create");

        let tokens_map1 = service.list(&map_id).expect("Failed to list");
        assert_eq!(tokens_map1.len(), 1);
        assert_eq!(tokens_map1[0].name, "Fighter");

        let tokens_map2 = service.list("map-2").expect("Failed to list");
        assert_eq!(tokens_map2.len(), 1);
        assert_eq!(tokens_map2[0].name, "Wizard");
    }

    // ── list_visible ──────────────────────────────────────────────────────

    #[test]
    fn test_list_visible_filters_hidden() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        let visible = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let hidden = CreateTokenInput::for_pc(&map_id, "Hidden Rogue", 1, 1).hidden();
        service.create(visible).expect("Failed to create");
        service.create(hidden).expect("Failed to create");

        // list() returns all tokens
        let all = service.list(&map_id).expect("Failed to list");
        assert_eq!(all.len(), 2);

        // list_visible() filters hidden tokens
        let visible_only = service.list_visible(&map_id).expect("Failed to list visible");
        assert_eq!(visible_only.len(), 1);
        assert_eq!(visible_only[0].name, "Fighter");
    }

    #[test]
    fn test_list_visible_empty_when_all_hidden() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        let input = CreateTokenInput::for_pc(&map_id, "Stealth", 0, 0).hidden();
        service.create(input).expect("Failed to create");

        let visible = service.list_visible(&map_id).expect("Failed to list");
        assert!(visible.is_empty());
    }

    // ── CRUD: update ──────────────────────────────────────────────────────

    #[test]
    fn test_update_token_position() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");

        let update = UpdateTokenInput::set_position(10, 15);
        let updated = service.update(&token.id, update).expect("Failed to update");

        assert_eq!(updated.grid_x, 10);
        assert_eq!(updated.grid_y, 15);
    }

    #[test]
    fn test_update_token_hidden() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");
        assert!(token.visible_to_players);

        let update = UpdateTokenInput::set_hidden(true);
        let updated = service.update(&token.id, update).expect("Failed to update");
        assert!(!updated.visible_to_players);
    }

    #[test]
    fn test_update_token_label() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");

        let update = UpdateTokenInput {
            label: Some(Some("Renamed Fighter".to_string())),
            ..Default::default()
        };
        let updated = service.update(&token.id, update).expect("Failed to update");
        assert_eq!(updated.name, "Renamed Fighter");
    }

    #[test]
    fn test_update_token_faction_color() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");

        let update = UpdateTokenInput {
            faction_color: Some(Some("#00FF00".to_string())),
            ..Default::default()
        };
        let updated = service.update(&token.id, update).expect("Failed to update");
        assert_eq!(updated.color, Some("#00FF00".to_string()));
    }

    #[test]
    fn test_update_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let update = UpdateTokenInput::set_position(1, 1);
        let result = service.update("nonexistent", update);
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    // ── update_position ───────────────────────────────────────────────────

    #[test]
    fn test_update_position() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");

        let updated = service
            .update_position(&token.id, 20, 25)
            .expect("Failed to update position");

        assert_eq!(updated.grid_x, 20);
        assert_eq!(updated.grid_y, 25);
        assert_eq!(updated.name, "Fighter");
    }

    #[test]
    fn test_update_position_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let result = service.update_position("nonexistent", 1, 1);
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    // ── update_vision ─────────────────────────────────────────────────────

    #[test]
    fn test_update_vision() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Elf", 0, 0);
        let token = service.create(input).expect("Failed to create");

        let updated = service
            .update_vision(&token.id, Some(120), Some(60), 60, 30)
            .expect("Failed to update vision");

        assert_eq!(updated.vision_bright_ft, Some(120));
        assert_eq!(updated.vision_dim_ft, Some(60));
        assert_eq!(updated.vision_dark_ft, 60);
        assert_eq!(updated.light_radius_ft, 30);
    }

    #[test]
    fn test_update_vision_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let result = service.update_vision("nonexistent", None, None, 0, 0);
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    // ── toggle_visibility ─────────────────────────────────────────────────

    #[test]
    fn test_toggle_visibility() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");
        assert!(token.visible_to_players);

        // Toggle to hidden
        let toggled = service
            .toggle_visibility(&token.id)
            .expect("Failed to toggle");
        assert!(!toggled.visible_to_players);

        // Toggle back to visible
        let toggled_back = service
            .toggle_visibility(&token.id)
            .expect("Failed to toggle");
        assert!(toggled_back.visible_to_players);
    }

    #[test]
    fn test_toggle_visibility_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let result = service.toggle_visibility("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    // ── CRUD: delete ──────────────────────────────────────────────────────

    #[test]
    fn test_delete_token() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");

        service.delete(&token.id).expect("Failed to delete");

        let result = service.get(&token.id).expect("Failed to query");
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_not_found() {
        let (mut conn, temp_dir) = setup_test_env();

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let result = service.delete("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    // ── count ─────────────────────────────────────────────────────────────

    #[test]
    fn test_count_tokens() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        assert_eq!(service.count(&map_id).expect("Failed to count"), 0);

        let input1 = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let input2 = CreateTokenInput::for_pc(&map_id, "Wizard", 1, 1);
        service.create(input1).expect("Failed to create");
        service.create(input2).expect("Failed to create");

        assert_eq!(service.count(&map_id).expect("Failed to count"), 2);
    }

    #[test]
    fn test_count_after_delete() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");
        assert_eq!(service.count(&map_id).expect("Failed to count"), 1);

        service.delete(&token.id).expect("Failed to delete");
        assert_eq!(service.count(&map_id).expect("Failed to count"), 0);
    }

    // ── Validation ────────────────────────────────────────────────────────

    #[test]
    fn test_create_pc_without_label_fails() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        let input = CreateTokenInput {
            map_id: map_id.clone(),
            module_monster_id: None,
            module_npc_id: None,
            grid_x: 0,
            grid_y: 0,
            label: None,
            faction_color: None,
            hidden: false,
        };
        let result = service.create(input);
        assert!(matches!(result, Err(ServiceError::Validation(_))));
    }

    #[test]
    fn test_create_both_monster_and_npc_fails() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, module_id, map_id) = create_prerequisites(&mut conn);
        let monster_id = create_module_monster(&mut conn, &module_id);
        let npc_id = create_module_npc(&mut conn, &module_id);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        let input = CreateTokenInput {
            map_id: map_id.clone(),
            module_monster_id: Some(monster_id),
            module_npc_id: Some(npc_id),
            grid_x: 0,
            grid_y: 0,
            label: None,
            faction_color: None,
            hidden: false,
        };
        let result = service.create(input);
        assert!(matches!(result, Err(ServiceError::Validation(_))));
    }

    // ── Pixel coordinate computation ──────────────────────────────────────

    #[test]
    fn test_pixel_coordinates_use_default_grid_size() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 3, 5);
        let token = service.create(input).expect("Failed to create");

        // Default grid size is 70px. Pixel coords = (grid + 0.5) * 70
        let expected_x = (3.0 + 0.5) * 70.0;
        let expected_y = (5.0 + 0.5) * 70.0;
        assert!((token.x - expected_x).abs() < f64::EPSILON);
        assert!((token.y - expected_y).abs() < f64::EPSILON);
    }

    // ── Token type defaults ───────────────────────────────────────────────

    #[test]
    fn test_pc_token_defaults_to_medium() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");

        assert_eq!(token.size, "medium");
    }

    #[test]
    fn test_npc_token_defaults_to_medium() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, module_id, map_id) = create_prerequisites(&mut conn);
        let npc_id = create_module_npc(&mut conn, &module_id);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_npc(&map_id, &npc_id, 0, 0);
        let token = service.create(input).expect("Failed to create");

        assert_eq!(token.size, "medium");
    }

    // ── Token with missing references ─────────────────────────────────────

    #[test]
    fn test_monster_token_with_missing_reference_errors() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        let input = CreateTokenInput::for_monster(&map_id, "nonexistent-monster", 0, 0);
        let result = service.create(input);

        assert!(result.is_err(), "Should fail with FK violation for nonexistent monster");
    }

    #[test]
    fn test_npc_token_with_missing_reference_errors() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        let input = CreateTokenInput::for_npc(&map_id, "nonexistent-npc", 0, 0);
        let result = service.create(input);

        assert!(result.is_err(), "Should fail with FK violation for nonexistent NPC");
    }

    // ── normalize_size_code ───────────────────────────────────────────────

    #[test]
    fn test_normalize_size_code_values() {
        assert_eq!(normalize_size_code("T"), "tiny");
        assert_eq!(normalize_size_code("S"), "small");
        assert_eq!(normalize_size_code("M"), "medium");
        assert_eq!(normalize_size_code("L"), "large");
        assert_eq!(normalize_size_code("H"), "huge");
        assert_eq!(normalize_size_code("G"), "gargantuan");
    }

    #[test]
    fn test_normalize_size_code_case_insensitive() {
        assert_eq!(normalize_size_code("t"), "tiny");
        assert_eq!(normalize_size_code("m"), "medium");
        assert_eq!(normalize_size_code("g"), "gargantuan");
    }

    #[test]
    fn test_normalize_size_code_unknown() {
        assert_eq!(normalize_size_code("XL"), "xl");
        assert_eq!(normalize_size_code("Unknown"), "unknown");
    }

    // ── Default vision values ─────────────────────────────────────────────

    #[test]
    fn test_default_vision_values() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");

        assert_eq!(token.vision_bright_ft, None);
        assert_eq!(token.vision_dim_ft, None);
        assert_eq!(token.vision_dark_ft, 0);
        assert_eq!(token.light_radius_ft, 0);
    }

    // ── Multiple operations sequence ──────────────────────────────────────

    #[test]
    fn test_create_update_delete_sequence() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());

        // Create
        let input = CreateTokenInput::for_pc(&map_id, "Fighter", 0, 0);
        let token = service.create(input).expect("Failed to create");
        assert_eq!(service.count(&map_id).expect("count"), 1);

        // Update position
        let updated = service
            .update_position(&token.id, 5, 5)
            .expect("Failed to update");
        assert_eq!(updated.grid_x, 5);

        // Toggle visibility
        let toggled = service
            .toggle_visibility(&token.id)
            .expect("Failed to toggle");
        assert!(!toggled.visible_to_players);

        // Update vision
        let visioned = service
            .update_vision(&token.id, Some(60), Some(30), 60, 20)
            .expect("Failed to update vision");
        assert_eq!(visioned.vision_dark_ft, 60);

        // Delete
        service.delete(&token.id).expect("Failed to delete");
        assert_eq!(service.count(&map_id).expect("count"), 0);
    }

    // ── Negative coordinates ──────────────────────────────────────────────

    #[test]
    fn test_negative_coordinates_allowed() {
        let (mut conn, temp_dir) = setup_test_env();
        let (_campaign_id, _module_id, map_id) = create_prerequisites(&mut conn);

        let mut service = TokenService::new(&mut conn, temp_dir.path());
        let input = CreateTokenInput::for_pc(&map_id, "Explorer", -5, -3);
        let token = service.create(input).expect("Failed to create");

        assert_eq!(token.grid_x, -5);
        assert_eq!(token.grid_y, -3);
    }

    // ── Input builder methods ─────────────────────────────────────────────

    #[test]
    fn test_create_token_input_builders() {
        let pc = CreateTokenInput::for_pc("map-1", "Fighter", 1, 2);
        assert_eq!(pc.map_id, "map-1");
        assert!(pc.module_monster_id.is_none());
        assert!(pc.module_npc_id.is_none());
        assert_eq!(pc.label, Some("Fighter".to_string()));

        let monster = CreateTokenInput::for_monster("map-1", "mm-1", 3, 4);
        assert_eq!(monster.module_monster_id, Some("mm-1".to_string()));
        assert!(monster.label.is_none());

        let npc = CreateTokenInput::for_npc("map-1", "npc-1", 5, 6);
        assert_eq!(npc.module_npc_id, Some("npc-1".to_string()));
        assert!(npc.label.is_none());
    }

    #[test]
    fn test_update_token_input_builders() {
        let pos = UpdateTokenInput::set_position(10, 20);
        assert_eq!(pos.grid_x, Some(10));
        assert_eq!(pos.grid_y, Some(20));
        assert!(pos.hidden.is_none());

        let hidden = UpdateTokenInput::set_hidden(true);
        assert_eq!(hidden.hidden, Some(true));
        assert!(hidden.grid_x.is_none());

        let vision = UpdateTokenInput::set_vision(Some(60), Some(30), 60, 20);
        assert_eq!(vision.vision_bright_ft, Some(Some(60)));
        assert_eq!(vision.vision_dim_ft, Some(Some(30)));
        assert_eq!(vision.vision_dark_ft, Some(60));
        assert_eq!(vision.light_radius_ft, Some(20));
    }
}

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

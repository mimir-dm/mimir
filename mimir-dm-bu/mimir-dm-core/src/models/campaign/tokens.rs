//! Token database models for Visual Display System
//!
//! Tokens represent entities placed on maps - monsters, PCs, NPCs, traps, and markers.

use crate::schema::tokens;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Token type - what kind of entity this token represents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Monster,
    PC,
    NPC,
    Trap,
    Marker,
}

impl TokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::Monster => "monster",
            TokenType::PC => "pc",
            TokenType::NPC => "npc",
            TokenType::Trap => "trap",
            TokenType::Marker => "marker",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "monster" => TokenType::Monster,
            "pc" => TokenType::PC,
            "npc" => TokenType::NPC,
            "trap" => TokenType::Trap,
            "marker" => TokenType::Marker,
            _ => TokenType::Monster,
        }
    }
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Monster
    }
}

/// Token size - D&D creature size categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenSize {
    Tiny,       // 0.5 grid squares (2.5 ft)
    Small,      // 1 grid square (5 ft)
    Medium,     // 1 grid square (5 ft)
    Large,      // 2x2 grid squares (10 ft)
    Huge,       // 3x3 grid squares (15 ft)
    Gargantuan, // 4x4 grid squares (20 ft)
}

impl TokenSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenSize::Tiny => "tiny",
            TokenSize::Small => "small",
            TokenSize::Medium => "medium",
            TokenSize::Large => "large",
            TokenSize::Huge => "huge",
            TokenSize::Gargantuan => "gargantuan",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "tiny" | "t" => TokenSize::Tiny,
            "small" | "s" => TokenSize::Small,
            "medium" | "m" => TokenSize::Medium,
            "large" | "l" => TokenSize::Large,
            "huge" | "h" => TokenSize::Huge,
            "gargantuan" | "g" => TokenSize::Gargantuan,
            _ => TokenSize::Medium,
        }
    }

    /// Get the number of grid squares this size occupies (width/height)
    pub fn grid_squares(&self) -> f32 {
        match self {
            TokenSize::Tiny => 0.5,
            TokenSize::Small => 1.0,
            TokenSize::Medium => 1.0,
            TokenSize::Large => 2.0,
            TokenSize::Huge => 3.0,
            TokenSize::Gargantuan => 4.0,
        }
    }
}

impl Default for TokenSize {
    fn default() -> Self {
        TokenSize::Medium
    }
}

/// Vision type - how a token perceives its environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisionType {
    Normal,
    Darkvision,
    Blindsight,
    Tremorsense,
    Truesight,
    DevilsSight,
}

impl VisionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VisionType::Normal => "normal",
            VisionType::Darkvision => "darkvision",
            VisionType::Blindsight => "blindsight",
            VisionType::Tremorsense => "tremorsense",
            VisionType::Truesight => "truesight",
            VisionType::DevilsSight => "devils_sight",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "darkvision" => VisionType::Darkvision,
            "blindsight" => VisionType::Blindsight,
            "tremorsense" => VisionType::Tremorsense,
            "truesight" => VisionType::Truesight,
            "devils_sight" | "devilssight" => VisionType::DevilsSight,
            _ => VisionType::Normal,
        }
    }

    /// Get a human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            VisionType::Normal => "Normal",
            VisionType::Darkvision => "Darkvision",
            VisionType::Blindsight => "Blindsight",
            VisionType::Tremorsense => "Tremorsense",
            VisionType::Truesight => "Truesight",
            VisionType::DevilsSight => "Devil's Sight",
        }
    }

    /// Whether this vision type can see in darkness (within range)
    pub fn sees_in_darkness(&self) -> bool {
        matches!(
            self,
            VisionType::Darkvision
                | VisionType::Blindsight
                | VisionType::Tremorsense
                | VisionType::Truesight
                | VisionType::DevilsSight
        )
    }

    /// Whether this vision type ignores all light conditions
    pub fn ignores_light(&self) -> bool {
        matches!(
            self,
            VisionType::Blindsight | VisionType::Tremorsense | VisionType::Truesight
        )
    }
}

impl Default for VisionType {
    fn default() -> Self {
        VisionType::Normal
    }
}

/// Database model for tokens
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Identifiable)]
#[diesel(table_name = tokens)]
pub struct Token {
    pub id: i32,
    pub map_id: i32,
    pub name: String,
    pub token_type: String,
    pub size: String,
    pub x: f32,
    pub y: f32,
    pub visible_to_players: bool,
    pub color: Option<String>,
    pub image_path: Option<String>,
    pub monster_id: Option<i32>,
    pub character_id: Option<i32>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub vision_type: String,
    pub vision_range_ft: Option<f32>,
}

impl Token {
    /// Get the token type enum
    pub fn token_type_enum(&self) -> TokenType {
        TokenType::from_str(&self.token_type)
    }

    /// Get the size enum
    pub fn size_enum(&self) -> TokenSize {
        TokenSize::from_str(&self.size)
    }

    /// Get the vision type enum
    pub fn vision_type_enum(&self) -> VisionType {
        VisionType::from_str(&self.vision_type)
    }

    /// Check if this token is linked to a catalog monster
    pub fn is_monster_linked(&self) -> bool {
        self.monster_id.is_some()
    }

    /// Check if this token is linked to a character
    pub fn is_character_linked(&self) -> bool {
        self.character_id.is_some()
    }

    /// Check if this token has special vision (not normal)
    pub fn has_special_vision(&self) -> bool {
        self.vision_type_enum() != VisionType::Normal
    }
}

/// New token for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = tokens)]
pub struct NewToken {
    pub map_id: i32,
    pub name: String,
    pub token_type: String,
    pub size: String,
    pub x: f32,
    pub y: f32,
    pub visible_to_players: bool,
    pub color: Option<String>,
    pub image_path: Option<String>,
    pub monster_id: Option<i32>,
    pub character_id: Option<i32>,
    pub notes: Option<String>,
    pub vision_type: String,
    pub vision_range_ft: Option<f32>,
}

impl NewToken {
    /// Create a new token at a position
    pub fn new(map_id: i32, name: String, x: f32, y: f32) -> Self {
        Self {
            map_id,
            name,
            token_type: TokenType::default().as_str().to_string(),
            size: TokenSize::default().as_str().to_string(),
            x,
            y,
            visible_to_players: true,
            color: None,
            image_path: None,
            monster_id: None,
            character_id: None,
            notes: None,
            vision_type: VisionType::default().as_str().to_string(),
            vision_range_ft: None,
        }
    }

    /// Create a monster token linked to a catalog monster
    pub fn monster(map_id: i32, name: String, monster_id: i32, size: TokenSize, x: f32, y: f32) -> Self {
        Self {
            map_id,
            name,
            token_type: TokenType::Monster.as_str().to_string(),
            size: size.as_str().to_string(),
            x,
            y,
            visible_to_players: true,
            color: None,
            image_path: None,
            monster_id: Some(monster_id),
            character_id: None,
            notes: None,
            vision_type: VisionType::default().as_str().to_string(),
            vision_range_ft: None,
        }
    }

    /// Create a character token (PC or NPC) linked to a character
    pub fn character(map_id: i32, name: String, character_id: i32, is_pc: bool, size: TokenSize, x: f32, y: f32) -> Self {
        Self {
            map_id,
            name,
            token_type: if is_pc { TokenType::PC } else { TokenType::NPC }.as_str().to_string(),
            size: size.as_str().to_string(),
            x,
            y,
            visible_to_players: true,
            color: None,
            image_path: None,
            monster_id: None,
            character_id: Some(character_id),
            notes: None,
            vision_type: VisionType::default().as_str().to_string(),
            vision_range_ft: None,
        }
    }

    /// Create a marker token (point of interest, note, etc.)
    pub fn marker(map_id: i32, name: String, x: f32, y: f32) -> Self {
        Self {
            map_id,
            name,
            token_type: TokenType::Marker.as_str().to_string(),
            size: TokenSize::Small.as_str().to_string(),
            x,
            y,
            visible_to_players: false, // Markers typically DM-only by default
            color: None,
            image_path: None,
            monster_id: None,
            character_id: None,
            notes: None,
            vision_type: VisionType::default().as_str().to_string(),
            vision_range_ft: None,
        }
    }

    /// Create a trap token
    pub fn trap(map_id: i32, name: String, x: f32, y: f32) -> Self {
        Self {
            map_id,
            name,
            token_type: TokenType::Trap.as_str().to_string(),
            size: TokenSize::Small.as_str().to_string(),
            x,
            y,
            visible_to_players: false, // Traps hidden by default
            color: None,
            image_path: None,
            monster_id: None,
            character_id: None,
            notes: None,
            vision_type: VisionType::default().as_str().to_string(),
            vision_range_ft: None,
        }
    }

    pub fn with_type(mut self, token_type: TokenType) -> Self {
        self.token_type = token_type.as_str().to_string();
        self
    }

    pub fn with_size(mut self, size: TokenSize) -> Self {
        self.size = size.as_str().to_string();
        self
    }

    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible_to_players = visible;
        self
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_image(mut self, image_path: String) -> Self {
        self.image_path = Some(image_path);
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn with_vision(mut self, vision_type: VisionType, range_ft: Option<f32>) -> Self {
        self.vision_type = vision_type.as_str().to_string();
        self.vision_range_ft = range_ft;
        self
    }
}

/// Token update structure.
///
/// Uses the `Option<Option<T>>` pattern for nullable fields. See [`crate::models`] for details.
///
/// - `Option<T>` fields (non-nullable columns): `None` = don't update, `Some(v)` = set to v
/// - `Option<Option<T>>` fields (nullable columns): `None` = don't update, `Some(None)` = set NULL, `Some(Some(v))` = set to v
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = tokens)]
pub struct UpdateToken {
    pub name: Option<String>,
    pub token_type: Option<String>,
    pub size: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub visible_to_players: Option<bool>,
    /// Nullable: `None` = keep, `Some(None)` = clear, `Some(Some(c))` = set color
    pub color: Option<Option<String>>,
    /// Nullable: `None` = keep, `Some(None)` = clear, `Some(Some(p))` = set path
    pub image_path: Option<Option<String>>,
    /// Nullable: `None` = keep, `Some(None)` = clear, `Some(Some(n))` = set notes
    pub notes: Option<Option<String>>,
    pub updated_at: Option<String>,
    pub vision_type: Option<String>,
    /// Nullable: `None` = keep, `Some(None)` = clear, `Some(Some(r))` = set range
    pub vision_range_ft: Option<Option<f32>>,
}

impl UpdateToken {
    /// Create an update for just the position
    pub fn position(x: f32, y: f32) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        }
    }

    /// Create an update for visibility
    pub fn visibility(visible: bool) -> Self {
        Self {
            visible_to_players: Some(visible),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        }
    }
}

/// Summary for listing tokens (includes resolved names for linked entities)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSummary {
    pub id: i32,
    pub map_id: i32,
    pub name: String,
    pub token_type: String,
    pub size: String,
    pub x: f32,
    pub y: f32,
    pub visible_to_players: bool,
    pub color: Option<String>,
    pub image_path: Option<String>,
    pub monster_id: Option<i32>,
    pub monster_name: Option<String>,
    pub character_id: Option<i32>,
    pub character_name: Option<String>,
    pub vision_type: String,
    pub vision_range_ft: Option<f32>,
}

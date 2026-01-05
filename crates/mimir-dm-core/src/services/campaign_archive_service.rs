//! Campaign archive service for import/export functionality
//!
//! This service handles exporting campaigns to portable `.mimir-campaign.tar.gz`
//! archives and importing them back into new campaign instances.

use crate::{
    connection::DbConnection,
    dal::campaign::module_monsters::ModuleMonsterRepository,
    dal::campaign::modules::ModuleRepository,
    dal::character::{CharacterRepository, CharacterVersionRepository},
    error::{DbError, Result},
    models::campaign::campaigns::Campaign,
    models::campaign::maps::{Map, NewMap},
    models::campaign::module_monsters::{ModuleMonster, NewModuleMonster},
    models::campaign::modules::{Module, NewModule},
    models::campaign::tokens::NewToken,
    models::character::{
        Character, CharacterVersion, NewCharacter, NewCharacterVersion, UpdateCharacter,
    },
    services::{CampaignService, MapService, TokenService},
};
use chrono::{DateTime, Utc};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tracing::{debug, info};

/// Archive format version
const ARCHIVE_VERSION: &str = "1.0";
/// Archive format identifier
const ARCHIVE_FORMAT: &str = "mimir-campaign";
/// File extension for campaign archives
pub const ARCHIVE_EXTENSION: &str = ".mimir-campaign.tar.gz";

/// Manifest file containing archive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveManifest {
    /// Archive format version
    pub version: String,
    /// Format identifier
    pub format: String,
    /// When the archive was created
    pub created_at: DateTime<Utc>,
    /// Campaign name from the source
    pub campaign_name: String,
    /// Campaign slug/directory name
    pub campaign_slug: String,
    /// Mimir version that created the archive
    pub mimir_version: String,
    /// Number of content files (markdown)
    pub file_count: usize,
    /// Number of asset files (images, maps, etc.)
    pub asset_count: usize,
    /// Catalog references found in content
    pub catalog_references: Vec<CatalogReference>,
}

/// A reference to a catalog item found in campaign content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CatalogReference {
    /// Type of catalog item (monster, spell, item, etc.)
    #[serde(rename = "type")]
    pub ref_type: String,
    /// Name of the referenced item
    pub name: String,
    /// Source book abbreviation
    pub source: String,
}

/// Preview information about an archive without importing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePreview {
    /// Campaign name from the archive
    pub campaign_name: String,
    /// Number of content files
    pub file_count: usize,
    /// Number of asset files
    pub asset_count: usize,
    /// Catalog references in the archive
    pub catalog_references: Vec<CatalogReference>,
    /// Mimir version that created the archive
    pub mimir_version: String,
    /// When the archive was created
    pub created_at: DateTime<Utc>,
}

/// Serializable campaign data for the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignData {
    /// Campaign name
    pub name: String,
    /// Current campaign status/stage
    pub status: String,
    /// When the campaign was created
    pub created_at: String,
    /// Last activity timestamp
    pub last_activity_at: String,
}

impl From<&Campaign> for CampaignData {
    fn from(campaign: &Campaign) -> Self {
        Self {
            name: campaign.name.clone(),
            status: campaign.status.clone(),
            created_at: campaign.created_at.clone(),
            last_activity_at: campaign.last_activity_at.clone(),
        }
    }
}

/// Serializable module data for the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleData {
    /// Original module ID (for reference mapping)
    pub original_id: i32,
    /// Module name
    pub name: String,
    /// Module number (ordering)
    pub module_number: i32,
    /// Current status
    pub status: String,
    /// Expected sessions
    pub expected_sessions: i32,
    /// Actual sessions held
    pub actual_sessions: i32,
}

impl From<&Module> for ModuleData {
    fn from(module: &Module) -> Self {
        Self {
            original_id: module.id,
            name: module.name.clone(),
            module_number: module.module_number,
            status: module.status.clone(),
            expected_sessions: module.expected_sessions,
            actual_sessions: module.actual_sessions,
        }
    }
}

/// Serializable map data for the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    /// Original map ID (for token reference mapping)
    pub original_id: i32,
    /// Map name
    pub name: String,
    /// Original module ID (for reference mapping)
    pub original_module_id: Option<i32>,
    /// Image/file path within archive
    pub image_path: String,
    /// Width in pixels
    pub width_px: i32,
    /// Height in pixels
    pub height_px: i32,
    /// Original width in pixels
    pub original_width_px: Option<i32>,
    /// Original height in pixels
    pub original_height_px: Option<i32>,
    /// Grid type (square, hex, none)
    pub grid_type: String,
    /// Grid size in pixels
    pub grid_size_px: Option<i32>,
    /// Grid offset X
    pub grid_offset_x: i32,
    /// Grid offset Y
    pub grid_offset_y: i32,
    /// Ambient light level
    pub ambient_light: String,
    /// Fog of war enabled
    pub fog_enabled: bool,
}

impl From<&Map> for MapData {
    fn from(map: &Map) -> Self {
        Self {
            original_id: map.id,
            name: map.name.clone(),
            original_module_id: map.module_id,
            image_path: map.image_path.clone(),
            width_px: map.width_px,
            height_px: map.height_px,
            original_width_px: map.original_width_px,
            original_height_px: map.original_height_px,
            grid_type: map.grid_type.clone(),
            grid_size_px: map.grid_size_px,
            grid_offset_x: map.grid_offset_x,
            grid_offset_y: map.grid_offset_y,
            ambient_light: map.ambient_light.clone(),
            fog_enabled: map.fog_enabled,
        }
    }
}

/// Serializable module monster data for the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMonsterData {
    /// Original module ID (for reference mapping)
    pub original_module_id: i32,
    /// Monster name (catalog reference)
    pub monster_name: String,
    /// Monster source (catalog reference)
    pub monster_source: String,
    /// Quantity
    pub quantity: i32,
    /// Encounter tag grouping
    pub encounter_tag: Option<String>,
}

impl From<&ModuleMonster> for ModuleMonsterData {
    fn from(mm: &ModuleMonster) -> Self {
        Self {
            original_module_id: mm.module_id,
            monster_name: mm.monster_name.clone(),
            monster_source: mm.monster_source.clone(),
            quantity: mm.quantity,
            encounter_tag: mm.encounter_tag.clone(),
        }
    }
}

/// Serializable token data for the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    /// Original map ID (for reference mapping)
    pub original_map_id: i32,
    /// Token name
    pub name: String,
    /// Token type (monster, pc, npc, trap, marker)
    pub token_type: String,
    /// Token size
    pub size: String,
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Visibility
    pub visible_to_players: bool,
    /// Token color
    pub color: Option<String>,
    /// Image path
    pub image_path: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// Vision type
    pub vision_type: String,
    /// Vision range in feet
    pub vision_range_ft: Option<f32>,
    /// Original character ID (for NPC tokens - will need lookup by name)
    pub original_character_id: Option<i32>,
    /// Character name (for matching on import)
    pub character_name: Option<String>,
    /// Monster name (for catalog lookup on import)
    pub monster_name: Option<String>,
    /// Monster source (for catalog lookup on import)
    pub monster_source: Option<String>,
}

/// Serializable character data for the archive (includes full version data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterArchiveData {
    /// Original character ID (for reference mapping)
    pub original_id: i32,
    /// Character name
    pub character_name: String,
    /// Whether this is an NPC
    pub is_npc: bool,
    /// Current level
    pub current_level: i32,
    /// Current version number
    pub current_version: i32,
    /// Directory path for character files
    pub directory_path: String,
    /// Primary class (for display)
    pub class: Option<String>,
    /// Race (for display)
    pub race: Option<String>,
    /// All character versions with their full data
    pub versions: Vec<CharacterVersionArchiveData>,
}

/// Serializable character version data for the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterVersionArchiveData {
    /// Version number
    pub version_number: i32,
    /// File path
    pub file_path: String,
    /// Full character data (YAML/JSON string)
    pub character_data: String,
    /// Reason for this snapshot
    pub snapshot_reason: Option<String>,
    /// Level at this version
    pub level: i32,
}

impl From<&CharacterVersion> for CharacterVersionArchiveData {
    fn from(v: &CharacterVersion) -> Self {
        Self {
            version_number: v.version_number,
            file_path: v.file_path.clone(),
            character_data: v.character_data.clone(),
            snapshot_reason: v.snapshot_reason.clone(),
            level: v.level,
        }
    }
}

/// Complete archive data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveData {
    /// Campaign data
    pub campaign: CampaignData,
    /// All modules
    pub modules: Vec<ModuleData>,
    /// All maps with metadata
    pub maps: Vec<MapData>,
    /// All characters/NPCs with their version data
    pub characters: Vec<CharacterArchiveData>,
    /// All module monsters
    pub module_monsters: Vec<ModuleMonsterData>,
    /// All tokens
    pub tokens: Vec<TokenData>,
}

/// Service for campaign archive operations
pub struct CampaignArchiveService;

impl CampaignArchiveService {
    /// Export a campaign to a tar.gz archive
    ///
    /// Creates a portable archive containing:
    /// - manifest.json: Archive metadata and catalog references
    /// - data.json: Complete structured data (campaign, modules, maps, characters, monsters, tokens)
    /// - content/: All markdown files preserving directory structure
    /// - assets/: All binary files (images, maps, handouts)
    /// - maps/: Map UVTT files from the database
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `campaign_id` - ID of the campaign to export
    /// * `output_path` - Path where the archive should be created
    /// * `data_dir` - Application data directory (where maps are stored)
    ///
    /// # Returns
    /// Path to the created archive file
    pub fn export_campaign(
        conn: &mut DbConnection,
        campaign_id: i32,
        output_path: &Path,
        data_dir: &Path,
    ) -> Result<PathBuf> {
        // Get campaign from database
        let mut service = CampaignService::new(conn);
        let campaign = service
            .get_campaign(campaign_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Campaign".to_string(),
                id: campaign_id.to_string(),
            })?;

        let campaign_dir = Path::new(&campaign.directory_path);
        if !campaign_dir.exists() {
            return Err(DbError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Campaign directory not found: {}", campaign.directory_path),
            )));
        }

        // Generate slug from directory name
        let slug = campaign_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("campaign")
            .to_string();

        // Collect files from campaign directory
        let (content_files, asset_files) = Self::collect_campaign_files(campaign_dir)?;

        // Collect maps from database
        let mut map_service = MapService::new(conn);
        let maps = map_service.list_all_campaign_maps(campaign_id)?;
        let map_files = Self::collect_map_files(&maps, data_dir, campaign_id)?;

        // Collect modules from database
        let mut module_repo = ModuleRepository::new(conn);
        let modules = module_repo.list_by_campaign(campaign_id)?;
        let module_data: Vec<ModuleData> = modules.iter().map(ModuleData::from).collect();

        // Collect module monsters for all modules
        let mut all_module_monsters = Vec::new();
        for module in &modules {
            let mut mm_repo = ModuleMonsterRepository::new(conn);
            let monsters = mm_repo.list_by_module(module.id)?;
            for monster in monsters {
                all_module_monsters.push(ModuleMonsterData::from(&monster));
            }
        }

        // Collect tokens for all maps (with character names for matching)
        let mut all_tokens = Vec::new();
        for map in &maps {
            let mut token_service = TokenService::new(conn);
            let tokens = token_service.list_tokens_for_map(map.id)?;
            for token in tokens {
                // Get character name if linked
                let character_name = if let Some(char_id) = token.character_id {
                    let mut char_repo = CharacterRepository::new(conn);
                    char_repo
                        .find_by_id(char_id)?
                        .map(|c| c.character_name)
                } else {
                    None
                };

                // Get monster info if linked (from catalog)
                let (monster_name, monster_source) = if token.monster_id.is_some() {
                    // Token name usually contains the monster name
                    // We'll store the token name as monster reference
                    (Some(token.name.clone()), None)
                } else {
                    (None, None)
                };

                all_tokens.push(TokenData {
                    original_map_id: map.id,
                    name: token.name,
                    token_type: token.token_type,
                    size: token.size,
                    x: token.x,
                    y: token.y,
                    visible_to_players: token.visible_to_players,
                    color: token.color,
                    image_path: token.image_path,
                    notes: token.notes,
                    vision_type: token.vision_type,
                    vision_range_ft: token.vision_range_ft,
                    original_character_id: token.character_id,
                    character_name,
                    monster_name,
                    monster_source,
                });
            }
        }

        // Collect characters with their version data
        let mut char_repo = CharacterRepository::new(conn);
        let characters = char_repo.list_for_campaign(campaign_id)?;

        let mut character_archive_data = Vec::new();
        for character in &characters {
            // Get all versions for this character
            let mut version_repo = CharacterVersionRepository::new(conn);
            let versions = version_repo.list_for_character(character.id)?;

            character_archive_data.push(CharacterArchiveData {
                original_id: character.id,
                character_name: character.character_name.clone(),
                is_npc: character.is_npc,
                current_level: character.current_level,
                current_version: character.current_version,
                directory_path: character.directory_path.clone(),
                class: character.class.clone(),
                race: character.race.clone(),
                versions: versions.iter().map(CharacterVersionArchiveData::from).collect(),
            });
        }

        info!(
            campaign_id = campaign_id,
            content_files = content_files.len(),
            asset_files = asset_files.len(),
            map_files = map_files.len(),
            modules = module_data.len(),
            characters = characters.len(),
            module_monsters = all_module_monsters.len(),
            tokens = all_tokens.len(),
            "Collected campaign files for export"
        );

        // Extract catalog references from content files
        let catalog_refs = Self::extract_catalog_references(&content_files);
        debug!(
            refs_count = catalog_refs.len(),
            "Extracted catalog references"
        );

        // Build archive data
        let archive_data = ArchiveData {
            campaign: CampaignData::from(&campaign),
            modules: module_data,
            maps: maps.iter().map(MapData::from).collect(),
            characters: character_archive_data,
            module_monsters: all_module_monsters,
            tokens: all_tokens,
        };

        // Build manifest
        let manifest = ArchiveManifest {
            version: ARCHIVE_VERSION.to_string(),
            format: ARCHIVE_FORMAT.to_string(),
            created_at: Utc::now(),
            campaign_name: campaign.name.clone(),
            campaign_slug: slug.clone(),
            mimir_version: env!("CARGO_PKG_VERSION").to_string(),
            file_count: content_files.len() + archive_data.characters.len(),
            asset_count: asset_files.len() + map_files.len(),
            catalog_references: catalog_refs,
        };

        // Determine output file path
        let archive_name = format!("{}{}", slug, ARCHIVE_EXTENSION);
        let archive_path = output_path.join(&archive_name);

        // Create the archive
        Self::create_archive(
            &archive_path,
            &manifest,
            &archive_data,
            &content_files,
            &asset_files,
            &map_files,
        )?;

        info!(
            path = %archive_path.display(),
            "Campaign archive created successfully"
        );

        Ok(archive_path)
    }

    /// Preview an archive without importing
    ///
    /// Extracts and validates the manifest to provide information about
    /// the archive contents.
    pub fn preview_archive(archive_path: &Path) -> Result<ArchivePreview> {
        let file = File::open(archive_path).map_err(|e| {
            DbError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to open archive: {}", e),
            ))
        })?;

        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);

        // Find and read manifest.json
        for entry in archive.entries().map_err(|e| {
            DbError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to read archive entries: {}", e),
            ))
        })? {
            let mut entry = entry.map_err(|e| {
                DbError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to read archive entry: {}", e),
                ))
            })?;

            let path = entry.path().map_err(|e| {
                DbError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid path in archive: {}", e),
                ))
            })?;

            // Look for manifest.json (may be at root or in subdirectory)
            if path.file_name() == Some(std::ffi::OsStr::new("manifest.json")) {
                let mut content = String::new();
                entry.read_to_string(&mut content).map_err(|e| {
                    DbError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to read manifest: {}", e),
                    ))
                })?;

                let manifest: ArchiveManifest =
                    serde_json::from_str(&content).map_err(|e| {
                        DbError::InvalidData(format!("Invalid manifest format: {}", e))
                    })?;

                // Validate format
                if manifest.format != ARCHIVE_FORMAT {
                    return Err(DbError::InvalidData(format!(
                        "Invalid archive format: expected '{}', got '{}'",
                        ARCHIVE_FORMAT, manifest.format
                    )));
                }

                return Ok(ArchivePreview {
                    campaign_name: manifest.campaign_name,
                    file_count: manifest.file_count,
                    asset_count: manifest.asset_count,
                    catalog_references: manifest.catalog_references,
                    mimir_version: manifest.mimir_version,
                    created_at: manifest.created_at,
                });
            }
        }

        Err(DbError::InvalidData(
            "Archive does not contain a manifest.json file".to_string(),
        ))
    }

    /// Import a campaign from an archive
    ///
    /// Creates a new campaign with the specified name and populates it
    /// with the content from the archive, including:
    /// - Content and asset files
    /// - Modules
    /// - Maps (with files)
    /// - Characters/NPCs
    /// - Module monsters
    /// - Tokens
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `archive_path` - Path to the archive file
    /// * `target_name` - Name for the new campaign
    /// * `campaigns_dir` - Base directory where campaigns are stored
    /// * `data_dir` - Application data directory (where maps are stored)
    ///
    /// # Returns
    /// The newly created campaign
    pub fn import_campaign(
        conn: &mut DbConnection,
        archive_path: &Path,
        target_name: String,
        campaigns_dir: &Path,
        data_dir: &Path,
    ) -> Result<Campaign> {
        // Preview first to validate
        let preview = Self::preview_archive(archive_path)?;
        info!(
            source_name = %preview.campaign_name,
            target_name = %target_name,
            "Importing campaign from archive"
        );

        // Create temp directory for extraction
        let temp_dir = tempfile::tempdir().map_err(|e| {
            DbError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create temp directory: {}", e),
            ))
        })?;

        // Extract archive to temp directory
        Self::extract_archive(archive_path, temp_dir.path())?;

        // Create the new campaign in database
        let mut service = CampaignService::new(conn);
        let campaign = service.create_campaign(
            &target_name,
            None,
            campaigns_dir.to_str().ok_or_else(|| {
                DbError::InvalidData("Invalid campaigns directory path".to_string())
            })?,
        )?;

        // Find the extracted archive root directory
        let mut archive_root = None;
        let mut content_source = None;
        let mut assets_source = None;
        let mut maps_source = None;

        for entry in fs::read_dir(temp_dir.path()).map_err(DbError::Io)? {
            let entry = entry.map_err(DbError::Io)?;
            let path = entry.path();
            if path.is_dir() {
                archive_root = Some(path.clone());
                let content_path = path.join("content");
                let assets_path = path.join("assets");
                let maps_path = path.join("maps");
                if content_path.exists() {
                    content_source = Some(content_path);
                }
                if assets_path.exists() {
                    assets_source = Some(assets_path);
                }
                if maps_path.exists() {
                    maps_source = Some(maps_path);
                }
                break;
            }
        }

        // Copy content files to new campaign directory
        let campaign_path = Path::new(&campaign.directory_path);
        if let Some(source) = content_source {
            Self::copy_directory_contents(&source, campaign_path)?;
        }

        // Copy asset files
        if let Some(source) = assets_source {
            let assets_dest = campaign_path.join("resources");
            Self::copy_directory_contents(&source, &assets_dest)?;
        }

        // Read and parse data.json if it exists
        let archive_data = if let Some(ref root) = archive_root {
            let data_path = root.join("data.json");
            if data_path.exists() {
                let data_content = fs::read_to_string(&data_path).map_err(DbError::Io)?;
                Some(serde_json::from_str::<ArchiveData>(&data_content).map_err(|e| {
                    DbError::InvalidData(format!("Failed to parse data.json: {}", e))
                })?)
            } else {
                None
            }
        } else {
            None
        };

        // If we have structured data, import it
        if let Some(data) = archive_data {
            // Import modules - build ID mapping
            let mut module_id_map: HashMap<i32, i32> = HashMap::new();
            for module_data in &data.modules {
                let mut module_repo = ModuleRepository::new(conn);
                let new_module = NewModule {
                    campaign_id: campaign.id,
                    name: module_data.name.clone(),
                    module_number: module_data.module_number,
                    status: module_data.status.clone(),
                    expected_sessions: module_data.expected_sessions,
                };
                let created_module = module_repo.create(new_module)?;
                module_id_map.insert(module_data.original_id, created_module.id);
                debug!(
                    old_id = module_data.original_id,
                    new_id = created_module.id,
                    name = %module_data.name,
                    "Imported module"
                );
            }

            // Import characters with their version data - build ID mapping
            let mut character_id_map: HashMap<i32, i32> = HashMap::new();
            for char_data in &data.characters {
                let mut char_repo = CharacterRepository::new(conn);
                let new_character = NewCharacter {
                    campaign_id: Some(campaign.id),
                    player_id: None, // Don't preserve player associations
                    character_name: char_data.character_name.clone(),
                    is_npc: Some(char_data.is_npc),
                    directory_path: char_data.directory_path.clone(),
                    class: char_data.class.clone(),
                    race: char_data.race.clone(),
                };
                let created_char = char_repo.create(new_character)?;
                character_id_map.insert(char_data.original_id, created_char.id);

                // Update character with correct level and version
                char_repo.update(
                    created_char.id,
                    UpdateCharacter {
                        character_name: None,
                        is_npc: None,
                        current_level: Some(char_data.current_level),
                        current_version: Some(char_data.current_version),
                        updated_at: None,
                        campaign_id: None,
                        directory_path: None,
                    },
                )?;

                // Import all character versions
                for version_data in &char_data.versions {
                    let mut version_repo = CharacterVersionRepository::new(conn);
                    let new_version = NewCharacterVersion {
                        character_id: created_char.id,
                        version_number: version_data.version_number,
                        file_path: version_data.file_path.clone(),
                        character_data: version_data.character_data.clone(),
                        snapshot_reason: version_data.snapshot_reason.clone(),
                        level: version_data.level,
                    };
                    version_repo.create(new_version)?;
                }

                debug!(
                    old_id = char_data.original_id,
                    new_id = created_char.id,
                    name = %char_data.character_name,
                    versions = char_data.versions.len(),
                    "Imported character with versions"
                );
            }

            // Import maps - build ID mapping and copy files
            let mut map_id_map: HashMap<i32, i32> = HashMap::new();
            for map_data in &data.maps {
                // Remap module_id
                let new_module_id = map_data
                    .original_module_id
                    .and_then(|id| module_id_map.get(&id).copied());

                let mut map_service = MapService::new(conn);
                let new_map = NewMap {
                    campaign_id: campaign.id,
                    module_id: new_module_id,
                    name: map_data.name.clone(),
                    image_path: map_data.image_path.clone(),
                    width_px: map_data.width_px,
                    height_px: map_data.height_px,
                    original_width_px: map_data.original_width_px,
                    original_height_px: map_data.original_height_px,
                    grid_type: map_data.grid_type.clone(),
                    grid_size_px: map_data.grid_size_px,
                    grid_offset_x: map_data.grid_offset_x,
                    grid_offset_y: map_data.grid_offset_y,
                    ambient_light: map_data.ambient_light.clone(),
                    fog_enabled: map_data.fog_enabled,
                };
                let created_map = map_service.create_map(new_map)?;
                map_id_map.insert(map_data.original_id, created_map.id);

                // Copy map file to data directory
                // Path structure:
                // - Module maps: data_dir/modules/{module_id}/maps/{filename}
                // - Campaign maps: data_dir/campaigns/{campaign_id}/maps/{filename}
                if let Some(ref maps_dir) = maps_source {
                    let map_filename = format!(
                        "{}_{}",
                        map_data.original_id,
                        map_data.image_path.replace('/', "_")
                    );
                    let src_map_file = maps_dir.join(&map_filename);

                    if src_map_file.exists() {
                        // Determine destination path based on actual storage structure
                        let dest_maps_dir = if let Some(mod_id) = new_module_id {
                            data_dir.join("modules").join(mod_id.to_string()).join("maps")
                        } else {
                            data_dir
                                .join("campaigns")
                                .join(campaign.id.to_string())
                                .join("maps")
                        };
                        fs::create_dir_all(&dest_maps_dir).map_err(DbError::Io)?;
                        let dest_map_file = dest_maps_dir.join(&map_data.image_path);
                        fs::copy(&src_map_file, &dest_map_file).map_err(DbError::Io)?;
                        debug!(
                            src = %src_map_file.display(),
                            dest = %dest_map_file.display(),
                            "Copied map file"
                        );
                    }
                }

                debug!(
                    old_id = map_data.original_id,
                    new_id = created_map.id,
                    name = %map_data.name,
                    "Imported map"
                );
            }

            // Import module monsters with remapped module IDs
            for mm_data in &data.module_monsters {
                if let Some(&new_module_id) = module_id_map.get(&mm_data.original_module_id) {
                    let mut mm_repo = ModuleMonsterRepository::new(conn);
                    let new_mm = NewModuleMonster {
                        module_id: new_module_id,
                        monster_name: mm_data.monster_name.clone(),
                        monster_source: mm_data.monster_source.clone(),
                        quantity: mm_data.quantity,
                        encounter_tag: mm_data.encounter_tag.clone(),
                    };
                    mm_repo.create(new_mm)?;
                    debug!(
                        module_id = new_module_id,
                        monster = %mm_data.monster_name,
                        "Imported module monster"
                    );
                }
            }

            // Import tokens with remapped map and character IDs
            for token_data in &data.tokens {
                if let Some(&new_map_id) = map_id_map.get(&token_data.original_map_id) {
                    // Remap character_id if present
                    let new_character_id = token_data
                        .original_character_id
                        .and_then(|id| character_id_map.get(&id).copied());

                    let mut token_service = TokenService::new(conn);
                    let new_token = NewToken {
                        map_id: new_map_id,
                        name: token_data.name.clone(),
                        token_type: token_data.token_type.clone(),
                        size: token_data.size.clone(),
                        x: token_data.x,
                        y: token_data.y,
                        visible_to_players: token_data.visible_to_players,
                        color: token_data.color.clone(),
                        image_path: token_data.image_path.clone(),
                        monster_id: None, // Don't preserve catalog monster links
                        character_id: new_character_id,
                        notes: token_data.notes.clone(),
                        vision_type: token_data.vision_type.clone(),
                        vision_range_ft: token_data.vision_range_ft,
                    };
                    token_service.create_token(new_token)?;
                    debug!(
                        map_id = new_map_id,
                        name = %token_data.name,
                        "Imported token"
                    );
                }
            }

            info!(
                campaign_id = campaign.id,
                modules = module_id_map.len(),
                characters = character_id_map.len(),
                maps = map_id_map.len(),
                module_monsters = data.module_monsters.len(),
                tokens = data.tokens.len(),
                "Campaign data imported successfully"
            );
        }

        info!(
            campaign_id = campaign.id,
            path = %campaign.directory_path,
            "Campaign imported successfully"
        );

        Ok(campaign)
    }

    /// Collect map files from database and data directory
    fn collect_map_files(
        maps: &[Map],
        data_dir: &Path,
        campaign_id: i32,
    ) -> Result<HashMap<String, Vec<u8>>> {
        let mut map_files = HashMap::new();

        for map in maps {
            // Determine the map file path based on actual storage structure:
            // - Module maps: data_dir/modules/{module_id}/maps/{filename}
            // - Campaign maps: data_dir/campaigns/{campaign_id}/maps/{filename}
            let maps_dir = if let Some(module_id) = map.module_id {
                data_dir.join("modules").join(module_id.to_string()).join("maps")
            } else {
                data_dir
                    .join("campaigns")
                    .join(campaign_id.to_string())
                    .join("maps")
            };
            let map_path = maps_dir.join(&map.image_path);

            if map_path.exists() {
                match fs::read(&map_path) {
                    Ok(data) => {
                        // Store map metadata as JSON alongside the file
                        let map_meta = serde_json::to_vec_pretty(&map).unwrap_or_default();
                        let meta_key = format!("{}.json", map.id);
                        map_files.insert(meta_key, map_meta);

                        // Store the actual map file
                        let file_key = format!("{}_{}", map.id, map.image_path.replace('/', "_"));
                        map_files.insert(file_key, data);
                    }
                    Err(e) => {
                        debug!(
                            map_id = map.id,
                            path = %map_path.display(),
                            error = %e,
                            "Failed to read map file, skipping"
                        );
                    }
                }
            } else {
                debug!(
                    map_id = map.id,
                    path = %map_path.display(),
                    "Map file not found, skipping"
                );
            }
        }

        Ok(map_files)
    }

    /// Collect files from campaign directory, separating content and assets
    fn collect_campaign_files(
        campaign_dir: &Path,
    ) -> Result<(HashMap<String, Vec<u8>>, HashMap<String, Vec<u8>>)> {
        let mut content_files = HashMap::new();
        let mut asset_files = HashMap::new();

        Self::walk_directory(campaign_dir, campaign_dir, &mut |relative_path, data| {
            let path_str = relative_path.to_string_lossy().to_string();
            let extension = relative_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            // Classify files
            match extension.to_lowercase().as_str() {
                "md" | "markdown" | "txt" | "json" | "yaml" | "yml" => {
                    content_files.insert(path_str, data);
                }
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "pdf" | "uvtt" => {
                    asset_files.insert(path_str, data);
                }
                _ => {
                    // Include other files as content by default
                    content_files.insert(path_str, data);
                }
            }
        })?;

        Ok((content_files, asset_files))
    }

    /// Walk directory recursively and call handler for each file
    fn walk_directory<F>(base_dir: &Path, current_dir: &Path, handler: &mut F) -> Result<()>
    where
        F: FnMut(&Path, Vec<u8>),
    {
        if !current_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(current_dir).map_err(DbError::Io)? {
            let entry = entry.map_err(DbError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                Self::walk_directory(base_dir, &path, handler)?;
            } else if path.is_file() {
                let relative_path = path.strip_prefix(base_dir).map_err(|e| {
                    DbError::InvalidData(format!("Failed to get relative path: {}", e))
                })?;

                let data = fs::read(&path).map_err(DbError::Io)?;
                handler(relative_path, data);
            }
        }

        Ok(())
    }

    /// Extract catalog references from markdown content
    fn extract_catalog_references(content_files: &HashMap<String, Vec<u8>>) -> Vec<CatalogReference> {
        let mut refs = std::collections::HashSet::new();

        // Pattern for 5etools-style references: {@type name|source}
        // Examples: {@monster Adult White Dragon|MM}, {@spell Fireball|PHB}
        let re = Regex::new(r"\{@(\w+)\s+([^|]+)\|([^}]+)\}").unwrap();

        for data in content_files.values() {
            if let Ok(content) = std::str::from_utf8(data) {
                for cap in re.captures_iter(content) {
                    let ref_type = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                    let source = cap.get(3).map(|m| m.as_str()).unwrap_or("");

                    // Filter to known catalog types
                    if matches!(
                        ref_type,
                        "monster" | "spell" | "item" | "creature" | "condition" | "feat" | "race" | "class" | "background"
                    ) {
                        refs.insert(CatalogReference {
                            ref_type: ref_type.to_string(),
                            name: name.to_string(),
                            source: source.to_string(),
                        });
                    }
                }
            }
        }

        refs.into_iter().collect()
    }

    /// Create the tar.gz archive
    fn create_archive(
        archive_path: &Path,
        manifest: &ArchiveManifest,
        archive_data: &ArchiveData,
        content_files: &HashMap<String, Vec<u8>>,
        asset_files: &HashMap<String, Vec<u8>>,
        map_files: &HashMap<String, Vec<u8>>,
    ) -> Result<()> {
        let file = File::create(archive_path).map_err(DbError::Io)?;
        let gz = GzEncoder::new(file, Compression::default());
        let mut archive = Builder::new(gz);

        let base_dir = &manifest.campaign_slug;

        // Add manifest.json
        let manifest_json = serde_json::to_string_pretty(manifest)
            .map_err(|e| DbError::InvalidData(format!("Failed to serialize manifest: {}", e)))?;
        Self::add_file_to_archive(
            &mut archive,
            &format!("{}/manifest.json", base_dir),
            manifest_json.as_bytes(),
        )?;

        // Add data.json (complete structured data)
        let data_json = serde_json::to_string_pretty(archive_data)
            .map_err(|e| DbError::InvalidData(format!("Failed to serialize data: {}", e)))?;
        Self::add_file_to_archive(
            &mut archive,
            &format!("{}/data.json", base_dir),
            data_json.as_bytes(),
        )?;

        // Add content files
        for (path, data) in content_files {
            Self::add_file_to_archive(
                &mut archive,
                &format!("{}/content/{}", base_dir, path),
                data,
            )?;
        }

        // Add asset files
        for (path, data) in asset_files {
            Self::add_file_to_archive(
                &mut archive,
                &format!("{}/assets/{}", base_dir, path),
                data,
            )?;
        }

        // Add map files
        for (path, data) in map_files {
            Self::add_file_to_archive(
                &mut archive,
                &format!("{}/maps/{}", base_dir, path),
                data,
            )?;
        }

        // Finish the archive
        archive
            .into_inner()
            .map_err(|e| {
                DbError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to finalize archive: {}", e),
                ))
            })?
            .finish()
            .map_err(DbError::Io)?;

        Ok(())
    }

    /// Add a single file to the archive
    fn add_file_to_archive<W: Write>(
        archive: &mut Builder<W>,
        path: &str,
        data: &[u8],
    ) -> Result<()> {
        let mut header = tar::Header::new_gnu();
        header.set_path(path).map_err(|e| {
            DbError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid archive path '{}': {}", path, e),
            ))
        })?;
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        archive.append(&header, data).map_err(|e| {
            DbError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to add '{}' to archive: {}", path, e),
            ))
        })?;

        Ok(())
    }

    /// Extract archive to a directory
    fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<()> {
        let file = File::open(archive_path).map_err(DbError::Io)?;
        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);

        archive.unpack(dest_dir).map_err(|e| {
            DbError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to extract archive: {}", e),
            ))
        })?;

        Ok(())
    }

    /// Copy directory contents recursively
    fn copy_directory_contents(src: &Path, dest: &Path) -> Result<()> {
        if !src.exists() {
            return Ok(());
        }

        fs::create_dir_all(dest).map_err(DbError::Io)?;

        for entry in fs::read_dir(src).map_err(DbError::Io)? {
            let entry = entry.map_err(DbError::Io)?;
            let src_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_directory_contents(&src_path, &dest_path)?;
            } else {
                // Ensure parent directory exists
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).map_err(DbError::Io)?;
                }
                fs::copy(&src_path, &dest_path).map_err(DbError::Io)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_reference_extraction() {
        let mut files = HashMap::new();
        files.insert(
            "test.md".to_string(),
            b"# Test\nFight a {@monster Adult White Dragon|MM} and cast {@spell Fireball|PHB}".to_vec(),
        );

        let refs = CampaignArchiveService::extract_catalog_references(&files);

        assert_eq!(refs.len(), 2);
        assert!(refs.iter().any(|r| r.ref_type == "monster" && r.name == "Adult White Dragon"));
        assert!(refs.iter().any(|r| r.ref_type == "spell" && r.name == "Fireball"));
    }

    #[test]
    fn test_archive_manifest_serialization() {
        let manifest = ArchiveManifest {
            version: "1.0".to_string(),
            format: "mimir-campaign".to_string(),
            created_at: Utc::now(),
            campaign_name: "Test Campaign".to_string(),
            campaign_slug: "test-campaign".to_string(),
            mimir_version: "0.1.0".to_string(),
            file_count: 10,
            asset_count: 5,
            catalog_references: vec![CatalogReference {
                ref_type: "monster".to_string(),
                name: "Goblin".to_string(),
                source: "MM".to_string(),
            }],
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let parsed: ArchiveManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.campaign_name, "Test Campaign");
        assert_eq!(parsed.catalog_references.len(), 1);
    }
}

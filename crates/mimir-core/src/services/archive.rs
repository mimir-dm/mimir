//! Campaign Archive Service
//!
//! Handles exporting campaigns to portable `.mimir-campaign.tar.gz` archives
//! and importing them back into new campaign instances.

use crate::dal::campaign as dal;
use crate::models::campaign::{
    Campaign, CampaignAsset, Character, CharacterClass, CharacterFeat, CharacterFeature,
    CharacterInventory, CharacterProficiency, CharacterSpell, Document, FogRevealedArea,
    LightSource, Map, MapPoi, MapTrap, Module, ModuleMonster, ModuleNpc, NewCampaign,
    NewCampaignAsset, NewCharacter, NewCharacterClass, NewCharacterFeat, NewCharacterFeature,
    NewCharacterInventory, NewCharacterProficiency, NewCharacterSpell, NewDocument,
    NewFogRevealedArea, NewLightSource, NewMap, NewMapPoi, NewMapTrap, NewModule,
    NewModuleMonster, NewModuleNpc, NewTokenPlacement, TokenPlacement,
};
use crate::services::{ServiceError, ServiceResult};
use chrono::{DateTime, Utc};
use diesel::SqliteConnection;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tracing::{info, warn};

/// Archive format version
const ARCHIVE_VERSION: &str = "2.0";
/// Archive format identifier
const ARCHIVE_FORMAT: &str = "mimir-campaign";
/// File extension for campaign archives
pub const ARCHIVE_EXTENSION: &str = ".mimir-campaign.tar.gz";

// =============================================================================
// Archive Data Structures
// =============================================================================

/// Manifest file containing archive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub version: String,
    pub format: String,
    pub created_at: DateTime<Utc>,
    pub campaign_name: String,
    pub mimir_version: String,
    pub counts: ArchiveCounts,
    pub catalog_references: Vec<CatalogReference>,
}

/// Counts of entities in the archive
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArchiveCounts {
    pub modules: usize,
    pub documents: usize,
    pub characters: usize,
    pub maps: usize,
    pub tokens: usize,
    pub module_monsters: usize,
    pub module_npcs: usize,
    pub assets: usize,
}

/// A reference to a catalog item found in campaign content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CatalogReference {
    #[serde(rename = "type")]
    pub ref_type: String,
    pub name: String,
    pub source: String,
}

/// Preview information about an archive without importing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePreview {
    pub campaign_name: String,
    pub counts: ArchiveCounts,
    pub catalog_references: Vec<CatalogReference>,
    pub mimir_version: String,
    pub created_at: DateTime<Utc>,
    pub archive_version: String,
}

/// Result of importing an archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub campaign_id: String,
    pub campaign_name: String,
    pub counts: ArchiveCounts,
}

/// ID mapping tables for import (old ID -> new ID)
#[derive(Debug, Default)]
struct IdMaps {
    campaign: HashMap<String, String>,
    modules: HashMap<String, String>,
    documents: HashMap<String, String>,
    characters: HashMap<String, String>,
    maps: HashMap<String, String>,
    assets: HashMap<String, String>,
    module_monsters: HashMap<String, String>,
    module_npcs: HashMap<String, String>,
}

/// Complete archive data - uses existing models directly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveData {
    pub campaign: Campaign,
    pub sources: Vec<String>,
    pub modules: Vec<Module>,
    pub documents: Vec<Document>,
    pub characters: Vec<CharacterWithRelated>,
    pub maps: Vec<MapWithRelated>,
    pub tokens: Vec<TokenPlacement>,
    pub module_monsters: Vec<ModuleMonster>,
    pub module_npcs: Vec<ModuleNpc>,
    pub assets: Vec<CampaignAsset>,
}

/// Character with all related data aggregated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterWithRelated {
    #[serde(flatten)]
    pub character: Character,
    pub classes: Vec<CharacterClass>,
    pub inventory: Vec<CharacterInventory>,
    pub spells: Vec<CharacterSpell>,
    pub proficiencies: Vec<CharacterProficiency>,
    pub features: Vec<CharacterFeature>,
    pub feats: Vec<CharacterFeat>,
}

/// Map with all related data aggregated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapWithRelated {
    #[serde(flatten)]
    pub map: Map,
    pub pois: Vec<MapPoi>,
    pub traps: Vec<MapTrap>,
    pub light_sources: Vec<LightSource>,
    pub fog_areas: Vec<FogRevealedArea>,
}

// =============================================================================
// Archive Service
// =============================================================================

/// Service for campaign archive operations
pub struct ArchiveService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> ArchiveService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Export a campaign to a tar.gz archive.
    pub fn export_campaign(
        &mut self,
        campaign_id: &str,
        output_dir: &Path,
        assets_dir: &Path,
    ) -> ServiceResult<PathBuf> {
        let campaign = dal::get_campaign(self.conn, campaign_id).map_err(|e| match e {
            diesel::result::Error::NotFound => ServiceError::not_found("Campaign", campaign_id),
            other => ServiceError::Database(other),
        })?;

        info!(campaign_id = %campaign_id, name = %campaign.name, "Exporting campaign");

        let archive_data = self.collect_campaign_data(campaign_id, campaign)?;
        let asset_files = self.collect_asset_files(&archive_data, assets_dir)?;
        let catalog_refs = self.extract_catalog_references(&archive_data.documents);

        let manifest = ArchiveManifest {
            version: ARCHIVE_VERSION.to_string(),
            format: ARCHIVE_FORMAT.to_string(),
            created_at: Utc::now(),
            campaign_name: archive_data.campaign.name.clone(),
            mimir_version: env!("CARGO_PKG_VERSION").to_string(),
            counts: ArchiveCounts {
                modules: archive_data.modules.len(),
                documents: archive_data.documents.len(),
                characters: archive_data.characters.len(),
                maps: archive_data.maps.len(),
                tokens: archive_data.tokens.len(),
                module_monsters: archive_data.module_monsters.len(),
                module_npcs: archive_data.module_npcs.len(),
                assets: asset_files.len(),
            },
            catalog_references: catalog_refs,
        };

        let slug = Self::slugify(&archive_data.campaign.name);
        let archive_name = format!("{}{}", slug, ARCHIVE_EXTENSION);
        let archive_path = output_dir.join(&archive_name);

        self.create_archive(&archive_path, &manifest, &archive_data, &asset_files)?;

        info!(path = %archive_path.display(), "Campaign archive created");
        Ok(archive_path)
    }

    /// Import a campaign from an archive.
    ///
    /// Creates a new campaign with all data from the archive, generating new UUIDs
    /// for all entities and remapping all references.
    pub fn import_campaign(
        &mut self,
        archive_path: &Path,
        assets_dir: &Path,
        campaign_name_override: Option<&str>,
    ) -> ServiceResult<ImportResult> {
        info!(path = %archive_path.display(), "Importing campaign archive");

        // Extract archive to temp directory
        let temp_dir = tempfile::tempdir().map_err(ServiceError::Io)?;
        Self::extract_archive(archive_path, temp_dir.path())?;

        // Read data.json
        let data_path = temp_dir.path().join("data.json");
        let data_content = fs::read_to_string(&data_path).map_err(ServiceError::Io)?;
        let data: ArchiveData = serde_json::from_str(&data_content)
            .map_err(|e| ServiceError::validation(format!("Invalid data.json: {}", e)))?;

        // Determine campaign name
        let campaign_name = campaign_name_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.generate_unique_campaign_name(&data.campaign.name));

        // Build ID mappings and insert data
        let mut id_maps = IdMaps::default();

        // 1. Create campaign
        let new_campaign_id = uuid::Uuid::new_v4().to_string();
        id_maps.campaign.insert(data.campaign.id.clone(), new_campaign_id.clone());

        let new_campaign = NewCampaign::new(&new_campaign_id, &campaign_name);
        dal::insert_campaign(self.conn, &new_campaign)?;

        // Add campaign sources
        for source_code in &data.sources {
            let source_id = uuid::Uuid::new_v4().to_string();
            let new_source = crate::models::campaign::NewCampaignSource::new(
                &source_id,
                &new_campaign_id,
                source_code,
            );
            dal::insert_campaign_source(self.conn, &new_source)?;
        }

        // 2. Import assets first (needed for map references)
        self.import_assets(&data, &mut id_maps, &new_campaign_id, assets_dir, temp_dir.path())?;

        // 3. Import modules
        self.import_modules(&data, &mut id_maps, &new_campaign_id)?;

        // 4. Import characters
        self.import_characters(&data, &mut id_maps, &new_campaign_id)?;

        // 5. Import documents
        self.import_documents(&data, &mut id_maps, &new_campaign_id)?;

        // 6. Import maps
        self.import_maps(&data, &mut id_maps, &new_campaign_id)?;

        // 7. Import module monsters and NPCs
        self.import_module_entities(&data, &mut id_maps)?;

        // 8. Import tokens (need map and module_monster/module_npc IDs)
        self.import_tokens(&data, &id_maps)?;

        let counts = ArchiveCounts {
            modules: data.modules.len(),
            documents: data.documents.len(),
            characters: data.characters.len(),
            maps: data.maps.len(),
            tokens: data.tokens.len(),
            module_monsters: data.module_monsters.len(),
            module_npcs: data.module_npcs.len(),
            assets: data.assets.len(),
        };

        info!(
            campaign_id = %new_campaign_id,
            name = %campaign_name,
            "Campaign imported successfully"
        );

        Ok(ImportResult {
            campaign_id: new_campaign_id,
            campaign_name,
            counts,
        })
    }

    /// Preview an archive without importing.
    pub fn preview_archive(archive_path: &Path) -> ServiceResult<ArchivePreview> {
        let file = File::open(archive_path).map_err(ServiceError::Io)?;
        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);

        for entry in archive.entries().map_err(ServiceError::Io)? {
            let mut entry = entry.map_err(ServiceError::Io)?;
            let path = entry.path().map_err(ServiceError::Io)?;

            if path.file_name() == Some(std::ffi::OsStr::new("manifest.json")) {
                let mut content = String::new();
                entry.read_to_string(&mut content).map_err(ServiceError::Io)?;

                let manifest: ArchiveManifest = serde_json::from_str(&content)
                    .map_err(|e| ServiceError::validation(format!("Invalid manifest: {}", e)))?;

                if manifest.format != ARCHIVE_FORMAT {
                    return Err(ServiceError::validation(format!(
                        "Invalid archive format: expected '{}', got '{}'",
                        ARCHIVE_FORMAT, manifest.format
                    )));
                }

                return Ok(ArchivePreview {
                    campaign_name: manifest.campaign_name,
                    counts: manifest.counts,
                    catalog_references: manifest.catalog_references,
                    mimir_version: manifest.mimir_version,
                    created_at: manifest.created_at,
                    archive_version: manifest.version,
                });
            }
        }

        Err(ServiceError::validation("Archive missing manifest.json"))
    }

    fn collect_campaign_data(
        &mut self,
        campaign_id: &str,
        campaign: Campaign,
    ) -> ServiceResult<ArchiveData> {
        let sources = dal::list_campaign_source_codes(self.conn, campaign_id)?;
        let modules = dal::list_modules(self.conn, campaign_id)?;
        let documents = dal::list_campaign_documents(self.conn, campaign_id)?;

        // Characters with related data
        let characters_raw = dal::list_campaign_characters(self.conn, campaign_id)?;
        let mut characters = Vec::new();
        for c in characters_raw {
            characters.push(CharacterWithRelated {
                classes: dal::list_character_classes(self.conn, &c.id)?,
                inventory: dal::list_character_inventory(self.conn, &c.id)?,
                spells: dal::list_character_spells(self.conn, &c.id)?,
                proficiencies: dal::list_character_proficiencies(self.conn, &c.id)?,
                features: dal::list_character_features(self.conn, &c.id)?,
                feats: dal::list_character_feats(self.conn, &c.id)?,
                character: c,
            });
        }

        // Maps with related data
        let maps_raw = dal::list_campaign_maps(self.conn, campaign_id)?;
        let mut maps = Vec::new();
        let mut tokens = Vec::new();
        for m in maps_raw {
            let map_tokens = dal::list_token_placements(self.conn, &m.id)?;
            tokens.extend(map_tokens);
            maps.push(MapWithRelated {
                pois: dal::list_map_pois(self.conn, &m.id)?,
                traps: dal::list_map_traps(self.conn, &m.id)?,
                light_sources: dal::list_light_sources(self.conn, &m.id)?,
                fog_areas: dal::list_fog_revealed_areas(self.conn, &m.id)?,
                map: m,
            });
        }

        // Module entities
        let mut module_monsters = Vec::new();
        let mut module_npcs = Vec::new();
        for module in &modules {
            module_monsters.extend(dal::list_module_monsters(self.conn, &module.id)?);
            module_npcs.extend(dal::list_module_npcs(self.conn, &module.id)?);
        }

        // All campaign assets (maps, tokens, images, etc.)
        let assets = dal::list_campaign_assets(self.conn, campaign_id)?;

        Ok(ArchiveData {
            campaign,
            sources,
            modules,
            documents,
            characters,
            maps,
            tokens,
            module_monsters,
            module_npcs,
            assets,
        })
    }

    fn collect_asset_files(
        &self,
        data: &ArchiveData,
        assets_dir: &Path,
    ) -> ServiceResult<HashMap<String, Vec<u8>>> {
        let mut files = HashMap::new();

        // Include all campaign assets - these are the actual binary files (maps, tokens, images)
        for asset in &data.assets {
            let file_path = assets_dir.join(&asset.blob_path);
            if file_path.exists() {
                match fs::read(&file_path) {
                    Ok(bytes) => {
                        // Store with asset ID as key to preserve references
                        let archive_path = format!("{}/{}", asset.id, asset.filename);
                        files.insert(archive_path, bytes);
                    }
                    Err(e) => {
                        warn!(
                            asset_id = %asset.id,
                            path = %file_path.display(),
                            error = %e,
                            "Failed to read asset file"
                        );
                    }
                }
            } else {
                warn!(
                    asset_id = %asset.id,
                    path = %file_path.display(),
                    "Asset file not found"
                );
            }
        }

        Ok(files)
    }

    fn extract_catalog_references(&self, documents: &[Document]) -> Vec<CatalogReference> {
        let mut refs = HashSet::new();
        let re = Regex::new(r"\{@(\w+)\s+([^|]+)\|([^}]+)\}").unwrap();

        for doc in documents {
            for cap in re.captures_iter(&doc.content) {
                let ref_type = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                let source = cap.get(3).map(|m| m.as_str()).unwrap_or("");

                if matches!(ref_type, "monster" | "spell" | "item" | "creature" | "condition" | "feat" | "race" | "class" | "background") {
                    refs.insert(CatalogReference {
                        ref_type: ref_type.to_string(),
                        name: name.to_string(),
                        source: source.to_string(),
                    });
                }
            }
        }

        refs.into_iter().collect()
    }

    fn create_archive(
        &self,
        archive_path: &Path,
        manifest: &ArchiveManifest,
        data: &ArchiveData,
        asset_files: &HashMap<String, Vec<u8>>,
    ) -> ServiceResult<()> {
        let file = File::create(archive_path).map_err(ServiceError::Io)?;
        let gz = GzEncoder::new(file, Compression::default());
        let mut archive = Builder::new(gz);

        let manifest_json = serde_json::to_string_pretty(manifest)
            .map_err(|e| ServiceError::validation(format!("Failed to serialize manifest: {}", e)))?;
        Self::add_file_to_archive(&mut archive, "manifest.json", manifest_json.as_bytes())?;

        let data_json = serde_json::to_string_pretty(data)
            .map_err(|e| ServiceError::validation(format!("Failed to serialize data: {}", e)))?;
        Self::add_file_to_archive(&mut archive, "data.json", data_json.as_bytes())?;

        for (path, bytes) in asset_files {
            Self::add_file_to_archive(&mut archive, &format!("assets/{}", path), bytes)?;
        }

        archive
            .into_inner()
            .map_err(|e| ServiceError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            .finish()
            .map_err(ServiceError::Io)?;

        Ok(())
    }

    fn add_file_to_archive<W: std::io::Write>(
        archive: &mut Builder<W>,
        path: &str,
        data: &[u8],
    ) -> ServiceResult<()> {
        let mut header = tar::Header::new_gnu();
        header.set_path(path).map_err(|e| {
            ServiceError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid archive path '{}': {}", path, e),
            ))
        })?;
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append(&header, data).map_err(|e| {
            ServiceError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to add '{}' to archive: {}", path, e),
            ))
        })?;
        Ok(())
    }

    // =========================================================================
    // Import Helpers
    // =========================================================================

    fn extract_archive(archive_path: &Path, dest: &Path) -> ServiceResult<()> {
        let file = File::open(archive_path).map_err(ServiceError::Io)?;
        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);
        archive.unpack(dest).map_err(ServiceError::Io)?;
        Ok(())
    }

    fn generate_unique_campaign_name(&mut self, base_name: &str) -> String {
        let campaigns = dal::list_campaigns(self.conn, false).unwrap_or_default();
        let existing_names: HashSet<_> = campaigns.iter().map(|c| c.name.as_str()).collect();

        if !existing_names.contains(base_name) {
            return base_name.to_string();
        }

        for i in 2..=100 {
            let candidate = format!("{} ({})", base_name, i);
            if !existing_names.contains(candidate.as_str()) {
                return candidate;
            }
        }

        format!("{} ({})", base_name, uuid::Uuid::new_v4())
    }

    fn import_assets(
        &mut self,
        data: &ArchiveData,
        id_maps: &mut IdMaps,
        campaign_id: &str,
        assets_dir: &Path,
        temp_dir: &Path,
    ) -> ServiceResult<()> {
        for asset in &data.assets {
            let new_id = uuid::Uuid::new_v4().to_string();
            id_maps.assets.insert(asset.id.clone(), new_id.clone());

            // Copy asset file
            let src_path = temp_dir.join("assets").join(&asset.id).join(&asset.filename);
            let blob_path = format!("campaigns/{}/{}", campaign_id, asset.filename);
            let dest_path = assets_dir.join(&blob_path);

            if src_path.exists() {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).map_err(ServiceError::Io)?;
                }
                fs::copy(&src_path, &dest_path).map_err(ServiceError::Io)?;
            }

            let new_asset = NewCampaignAsset::for_campaign(
                &new_id,
                campaign_id,
                &asset.filename,
                &asset.mime_type,
                &blob_path,
            );
            dal::insert_campaign_asset(self.conn, &new_asset)?;
        }
        Ok(())
    }

    fn import_modules(
        &mut self,
        data: &ArchiveData,
        id_maps: &mut IdMaps,
        campaign_id: &str,
    ) -> ServiceResult<()> {
        for module in &data.modules {
            let new_id = uuid::Uuid::new_v4().to_string();
            id_maps.modules.insert(module.id.clone(), new_id.clone());

            let mut new_module = NewModule::new(&new_id, campaign_id, &module.name, module.module_number);
            if let Some(ref desc) = module.description {
                new_module = new_module.with_description(desc);
            }
            dal::insert_module(self.conn, &new_module)?;
        }
        Ok(())
    }

    fn import_characters(
        &mut self,
        data: &ArchiveData,
        id_maps: &mut IdMaps,
        campaign_id: &str,
    ) -> ServiceResult<()> {
        for char_data in &data.characters {
            let c = &char_data.character;
            let new_id = uuid::Uuid::new_v4().to_string();
            id_maps.characters.insert(c.id.clone(), new_id.clone());

            // Create character based on type
            let mut new_char = if c.is_npc != 0 {
                NewCharacter::new_npc(&new_id, Some(campaign_id), &c.name)
            } else {
                let player_name = c.player_name.as_deref().unwrap_or("Unknown Player");
                NewCharacter::new_pc(&new_id, Some(campaign_id), &c.name, player_name)
            };

            // Override with actual values from archive
            new_char.race_name = c.race_name.as_deref();
            new_char.race_source = c.race_source.as_deref();
            new_char.background_name = c.background_name.as_deref();
            new_char.background_source = c.background_source.as_deref();
            new_char.strength = c.strength;
            new_char.dexterity = c.dexterity;
            new_char.constitution = c.constitution;
            new_char.intelligence = c.intelligence;
            new_char.wisdom = c.wisdom;
            new_char.charisma = c.charisma;
            new_char.cp = c.cp;
            new_char.sp = c.sp;
            new_char.ep = c.ep;
            new_char.gp = c.gp;
            new_char.pp = c.pp;
            new_char.traits = c.traits.as_deref();
            new_char.ideals = c.ideals.as_deref();
            new_char.bonds = c.bonds.as_deref();
            new_char.flaws = c.flaws.as_deref();
            new_char.role = c.role.as_deref();
            new_char.location = c.location.as_deref();
            new_char.faction = c.faction.as_deref();

            dal::insert_character(self.conn, &new_char)?;

            // Import character classes - respect original starting_class flag
            for class in &char_data.classes {
                let class_id = uuid::Uuid::new_v4().to_string();
                let mut new_class = if class.starting_class != 0 {
                    NewCharacterClass::starting(
                        &class_id,
                        &new_id,
                        &class.class_name,
                        &class.class_source,
                    )
                } else {
                    NewCharacterClass::multiclass(
                        &class_id,
                        &new_id,
                        &class.class_name,
                        &class.class_source,
                    )
                };
                // Set level and subclass if present
                new_class = new_class.with_level(class.level);
                if let (Some(subclass_name), Some(subclass_source)) =
                    (&class.subclass_name, &class.subclass_source)
                {
                    new_class = new_class.with_subclass(subclass_name, subclass_source);
                }
                dal::insert_character_class(self.conn, &new_class)?;
            }

            // Import inventory
            for item in &char_data.inventory {
                let item_id = uuid::Uuid::new_v4().to_string();
                let mut new_item = NewCharacterInventory::new(
                    &item_id,
                    &new_id,
                    &item.item_name,
                    &item.item_source,
                );
                new_item.quantity = item.quantity;
                new_item.equipped = item.equipped;
                new_item.attuned = item.attuned;
                new_item.notes = item.notes.as_deref();
                dal::insert_character_inventory(self.conn, &new_item)?;
            }

            // Import spells
            for spell in &char_data.spells {
                let spell_id = uuid::Uuid::new_v4().to_string();
                let mut new_spell = NewCharacterSpell::new(
                    &spell_id,
                    &new_id,
                    &spell.spell_name,
                    &spell.spell_source,
                    &spell.source_class,
                );
                if spell.prepared != 0 {
                    new_spell = new_spell.prepared();
                }
                dal::insert_character_spell(self.conn, &new_spell)?;
            }

            // Import proficiencies
            for prof in &char_data.proficiencies {
                let prof_id = uuid::Uuid::new_v4().to_string();
                let new_prof = NewCharacterProficiency {
                    id: &prof_id,
                    character_id: &new_id,
                    proficiency_type: &prof.proficiency_type,
                    name: &prof.name,
                    expertise: prof.expertise,
                };
                dal::insert_character_proficiency(self.conn, &new_prof)?;
            }

            // Import features
            for feat in &char_data.features {
                let feat_id = uuid::Uuid::new_v4().to_string();
                let new_feat = NewCharacterFeature::new(
                    &feat_id,
                    &new_id,
                    crate::models::campaign::FeatureType::from_str(&feat.feature_type)
                        .unwrap_or(crate::models::campaign::FeatureType::FightingStyle),
                    &feat.feature_name,
                    &feat.feature_source,
                    &feat.source_class,
                );
                dal::insert_character_feature(self.conn, &new_feat)?;
            }

            // Import feats
            for feat in &char_data.feats {
                let feat_id = uuid::Uuid::new_v4().to_string();
                let new_feat = NewCharacterFeat {
                    id: &feat_id,
                    character_id: &new_id,
                    feat_name: &feat.feat_name,
                    feat_source: &feat.feat_source,
                    source_type: &feat.source_type,
                };
                dal::insert_character_feat(self.conn, &new_feat)?;
            }
        }
        Ok(())
    }

    fn import_documents(
        &mut self,
        data: &ArchiveData,
        id_maps: &mut IdMaps,
        campaign_id: &str,
    ) -> ServiceResult<()> {
        for doc in &data.documents {
            let new_id = uuid::Uuid::new_v4().to_string();
            id_maps.documents.insert(doc.id.clone(), new_id.clone());

            let module_id = doc.module_id.as_ref().and_then(|old_id| id_maps.modules.get(old_id));

            let mut new_doc = if let Some(mod_id) = module_id {
                NewDocument::for_module(&new_id, campaign_id, mod_id, &doc.title, &doc.doc_type)
            } else {
                NewDocument::for_campaign(&new_id, campaign_id, &doc.title, &doc.doc_type)
            };
            new_doc.content = &doc.content;
            dal::insert_document(self.conn, &new_doc)?;
        }
        Ok(())
    }

    fn import_maps(
        &mut self,
        data: &ArchiveData,
        id_maps: &mut IdMaps,
        campaign_id: &str,
    ) -> ServiceResult<()> {
        for map_data in &data.maps {
            let m = &map_data.map;
            let new_id = uuid::Uuid::new_v4().to_string();
            id_maps.maps.insert(m.id.clone(), new_id.clone());

            // Map the uvtt_asset_id to new ID
            let new_uvtt_asset_id = id_maps.assets.get(&m.uvtt_asset_id)
                .cloned()
                .unwrap_or_else(|| m.uvtt_asset_id.clone());

            let module_id = m.module_id.as_ref().and_then(|old_id| id_maps.modules.get(old_id));

            let new_map = if let Some(mod_id) = module_id {
                NewMap::for_module(&new_id, campaign_id, mod_id, &m.name, &new_uvtt_asset_id)
            } else {
                NewMap::for_campaign(&new_id, campaign_id, &m.name, &new_uvtt_asset_id)
            };
            dal::insert_map(self.conn, &new_map)?;

            // Import POIs
            for poi in &map_data.pois {
                let poi_id = uuid::Uuid::new_v4().to_string();
                let mut new_poi = NewMapPoi::new(&poi_id, &new_id, &poi.name, poi.grid_x, poi.grid_y);
                if let Some(ref desc) = poi.description {
                    new_poi = new_poi.with_description(desc);
                }
                new_poi = new_poi.with_icon(&poi.icon);
                if let Some(ref color) = poi.color {
                    new_poi = new_poi.with_color(color);
                }
                if poi.visible != 0 {
                    new_poi = new_poi.visible();
                }
                dal::insert_map_poi(self.conn, &new_poi)?;
            }

            // Import traps
            for trap in &map_data.traps {
                let trap_id = uuid::Uuid::new_v4().to_string();
                let mut new_trap = NewMapTrap::new(
                    &trap_id,
                    &new_id,
                    &trap.name,
                    trap.grid_x,
                    trap.grid_y,
                );
                if let Some(ref desc) = trap.description {
                    new_trap = new_trap.with_description(desc);
                }
                if let Some(ref trigger) = trap.trigger_description {
                    new_trap = new_trap.with_trigger(trigger);
                }
                if let Some(ref effect) = trap.effect_description {
                    new_trap = new_trap.with_effect(effect);
                }
                if let Some(dc) = trap.dc {
                    new_trap = new_trap.with_dc(dc);
                }
                if trap.visible != 0 {
                    new_trap = new_trap.visible();
                }
                // Note: triggered state is preserved in new_trap.triggered field
                dal::insert_map_trap(self.conn, &new_trap)?;
            }

            // Import light sources
            for light in &map_data.light_sources {
                let light_id = uuid::Uuid::new_v4().to_string();
                let mut new_light = NewLightSource::new(
                    &light_id,
                    &new_id,
                    light.grid_x,
                    light.grid_y,
                    light.bright_radius,
                    light.dim_radius,
                );
                if let Some(ref name) = light.name {
                    new_light = new_light.with_name(name);
                }
                if let Some(ref color) = light.color {
                    new_light = new_light.with_color(color);
                }
                if light.active == 0 {
                    new_light = new_light.inactive();
                }
                dal::insert_light_source(self.conn, &new_light)?;
            }

            // Import fog areas
            for fog in &map_data.fog_areas {
                let fog_id = uuid::Uuid::new_v4().to_string();
                let new_fog = NewFogRevealedArea::new(
                    &fog_id,
                    &new_id,
                    fog.x,
                    fog.y,
                    fog.width,
                    fog.height,
                );
                dal::insert_fog_revealed_area(self.conn, &new_fog)?;
            }
        }
        Ok(())
    }

    fn import_module_entities(
        &mut self,
        data: &ArchiveData,
        id_maps: &mut IdMaps,
    ) -> ServiceResult<()> {
        // Import module monsters
        for monster in &data.module_monsters {
            let new_id = uuid::Uuid::new_v4().to_string();
            id_maps.module_monsters.insert(monster.id.clone(), new_id.clone());

            let module_id = id_maps.modules.get(&monster.module_id)
                .ok_or_else(|| ServiceError::validation("Module not found for monster"))?;

            let mut new_monster = NewModuleMonster::new(
                &new_id,
                module_id,
                &monster.monster_name,
                &monster.monster_source,
            );
            new_monster.quantity = monster.quantity;
            new_monster.display_name = monster.display_name.as_deref();
            new_monster.notes = monster.notes.as_deref();
            dal::insert_module_monster(self.conn, &new_monster)?;
        }

        // Import module NPCs
        for npc in &data.module_npcs {
            let new_id = uuid::Uuid::new_v4().to_string();
            id_maps.module_npcs.insert(npc.id.clone(), new_id.clone());

            let module_id = id_maps.modules.get(&npc.module_id)
                .ok_or_else(|| ServiceError::validation("Module not found for NPC"))?;

            let mut new_npc = NewModuleNpc::new(&new_id, module_id, &npc.name);
            new_npc.role = npc.role.as_deref();
            new_npc.description = npc.description.as_deref();
            new_npc.appearance = npc.appearance.as_deref();
            new_npc.personality = npc.personality.as_deref();
            new_npc.motivation = npc.motivation.as_deref();
            new_npc.secrets = npc.secrets.as_deref();
            new_npc.stat_block = npc.stat_block.as_deref();
            // Map token_asset_id if present
            if let Some(ref old_asset_id) = npc.token_asset_id {
                new_npc.token_asset_id = id_maps.assets.get(old_asset_id).map(|s| s.as_str());
            }
            dal::insert_module_npc(self.conn, &new_npc)?;
        }

        Ok(())
    }

    fn import_tokens(&mut self, data: &ArchiveData, id_maps: &IdMaps) -> ServiceResult<()> {
        for token in &data.tokens {
            let new_id = uuid::Uuid::new_v4().to_string();

            let map_id = id_maps.maps.get(&token.map_id)
                .ok_or_else(|| ServiceError::validation("Map not found for token"))?;

            let module_monster_id = token.module_monster_id.as_ref()
                .and_then(|old_id| id_maps.module_monsters.get(old_id))
                .map(|s| s.as_str());

            let module_npc_id = token.module_npc_id.as_ref()
                .and_then(|old_id| id_maps.module_npcs.get(old_id))
                .map(|s| s.as_str());

            // Skip if neither monster nor NPC reference could be resolved
            if module_monster_id.is_none() && module_npc_id.is_none() {
                warn!(token_id = %token.id, "Skipping token with unresolved references");
                continue;
            }

            let new_token = if let Some(monster_id) = module_monster_id {
                NewTokenPlacement::for_monster(&new_id, map_id, monster_id, token.grid_x, token.grid_y)
            } else if let Some(npc_id) = module_npc_id {
                NewTokenPlacement::for_npc(&new_id, map_id, npc_id, token.grid_x, token.grid_y)
            } else {
                continue;
            };

            dal::insert_token_placement(self.conn, &new_token)?;
        }
        Ok(())
    }

    fn slugify(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(ArchiveService::slugify("Lost Mine of Phandelver"), "lost-mine-of-phandelver");
        assert_eq!(ArchiveService::slugify("Test  Campaign!"), "test-campaign");
        assert_eq!(ArchiveService::slugify("D&D 5e"), "d-d-5e");
    }

    #[test]
    fn test_catalog_reference_hash() {
        let ref1 = CatalogReference {
            ref_type: "monster".to_string(),
            name: "Goblin".to_string(),
            source: "MM".to_string(),
        };
        let ref2 = CatalogReference {
            ref_type: "monster".to_string(),
            name: "Goblin".to_string(),
            source: "MM".to_string(),
        };
        assert_eq!(ref1, ref2);

        let mut set = HashSet::new();
        set.insert(ref1);
        set.insert(ref2);
        assert_eq!(set.len(), 1);
    }
}

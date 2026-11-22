//! Module Service
//!
//! Business logic for module management including type-based document creation.

use diesel::SqliteConnection;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    Module, ModuleMonster, ModuleNpc, NewDocument, NewModule, NewModuleMonster, NewModuleNpc,
    UpdateModule as DalUpdateModule, UpdateModuleMonster, UpdateModuleNpc,
};
use crate::services::{ServiceError, ServiceResult};
use crate::templates;
use crate::utils::now_rfc3339;

/// Module types that determine which template is used for the overview document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleType {
    /// General-purpose module (default)
    #[default]
    General,
    /// Investigation and clue-based adventures
    Mystery,
    /// Exploration and combat-focused adventures
    Dungeon,
    /// Planning and execution of complex operations
    Heist,
    /// Tension, dread, and survival
    Horror,
    /// Factions, diplomacy, and scheming
    Political,
}

impl ModuleType {
    /// The key of each type (its `template_key`), in menu order.
    pub const KEYS: [&'static str; 6] = [
        "general",
        "mystery",
        "dungeon",
        "heist",
        "horror",
        "political",
    ];

    /// The type of a key; `None` for an unknown key.
    pub fn parse(key: &str) -> Option<Self> {
        Some(match key {
            "general" => ModuleType::General,
            "mystery" => ModuleType::Mystery,
            "dungeon" => ModuleType::Dungeon,
            "heist" => ModuleType::Heist,
            "horror" => ModuleType::Horror,
            "political" => ModuleType::Political,
            _ => return None,
        })
    }

    /// Get the template key for this module type.
    pub fn template_key(&self) -> &'static str {
        match self {
            ModuleType::General => "general",
            ModuleType::Mystery => "mystery",
            ModuleType::Dungeon => "dungeon",
            ModuleType::Heist => "heist",
            ModuleType::Horror => "horror",
            ModuleType::Political => "political",
        }
    }

    /// Get the display name for this module type.
    pub fn display_name(&self) -> &'static str {
        match self {
            ModuleType::General => "General",
            ModuleType::Mystery => "Mystery",
            ModuleType::Dungeon => "Dungeon Crawl",
            ModuleType::Heist => "Heist",
            ModuleType::Horror => "Horror",
            ModuleType::Political => "Political Intrigue",
        }
    }
}

impl std::fmt::Display for ModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<Option<&str>> for ModuleType {
    fn from(s: Option<&str>) -> Self {
        s.and_then(ModuleType::parse).unwrap_or_default()
    }
}

/// Input for creating a new module.
#[derive(Debug, Clone)]
pub struct CreateModuleInput {
    /// Campaign this module belongs to
    pub campaign_id: String,
    /// Module name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Module type (determines which template is used)
    pub module_type: ModuleType,
}

impl CreateModuleInput {
    /// Create a new module input.
    pub fn new(campaign_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            name: name.into(),
            description: None,
            module_type: ModuleType::default(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the module type.
    pub fn with_type(mut self, module_type: ModuleType) -> Self {
        self.module_type = module_type;
        self
    }
}

/// Input for updating a module.
#[derive(Debug, Clone, Default)]
pub struct UpdateModuleInput {
    /// New name (if changing)
    pub name: Option<String>,
    /// New description (if changing). Use Some(None) to clear.
    pub description: Option<Option<String>>,
}

impl UpdateModuleInput {
    /// Create an update to change the name.
    pub fn set_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            description: None,
        }
    }

    /// Create an update to change the description.
    pub fn set_description(description: Option<String>) -> Self {
        Self {
            name: None,
            description: Some(description),
        }
    }
}

/// Reference to a monster being added to a module: either a catalog entry
/// (exact name + source book) or a campaign homebrew monster (by id).
#[derive(Debug, Clone)]
pub enum MonsterRef {
    /// Catalog monster referenced by exact name and source book code (e.g. "MM").
    Catalog {
        /// Exact catalog monster name.
        name: String,
        /// Source book code.
        source: String,
    },
    /// Campaign homebrew monster referenced by id.
    Homebrew {
        /// Homebrew monster id.
        id: String,
    },
}

/// Input for adding a monster to a module.
#[derive(Debug, Clone)]
pub struct AddMonsterInput {
    /// Module to add the monster to.
    pub module_id: String,
    /// Which monster to add.
    pub monster: MonsterRef,
    /// Number of this monster.
    pub quantity: i32,
    /// Optional display name override.
    pub display_name: Option<String>,
    /// Optional DM notes.
    pub notes: Option<String>,
}

impl AddMonsterInput {
    /// Create an input with quantity 1 and no overrides.
    pub fn new(module_id: impl Into<String>, monster: MonsterRef) -> Self {
        Self {
            module_id: module_id.into(),
            monster,
            quantity: 1,
            display_name: None,
            notes: None,
        }
    }

    /// Set the quantity.
    pub fn with_quantity(mut self, quantity: i32) -> Self {
        self.quantity = quantity;
        self
    }

    /// Set a display name override.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set DM notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Service for module management.
///
/// Handles module CRUD operations and automatic creation of type-specific documents.
pub struct ModuleService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> ModuleService<'a> {
    /// Create a new module service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Create a new module with type-specific overview and play notes documents.
    ///
    /// This creates the module and populates it with:
    /// 1. A type-specific overview document (mystery, dungeon, etc.)
    /// 2. A blank play notes document for session tracking
    pub fn create(&mut self, input: CreateModuleInput) -> ServiceResult<Module> {
        use diesel::Connection;

        let module_id = Uuid::new_v4().to_string();

        self.conn.transaction(|conn| {
            // Get next module number for this campaign
            let module_number = dal::next_module_number(conn, &input.campaign_id)?;

            // Create the module
            let mut new_module =
                NewModule::new(&module_id, &input.campaign_id, &input.name, module_number);
            if let Some(ref desc) = input.description {
                new_module = new_module.with_description(desc);
            }
            dal::insert_module(conn, &new_module)?;

            // Get the type-specific template content
            let overview_content = templates::get_module_template(input.module_type.template_key())
                .unwrap_or_else(|| {
                    templates::get_module_template("general").expect("General template must exist")
                });

            // Create overview document from type-specific template
            let overview_id = Uuid::new_v4().to_string();
            let overview_doc = NewDocument::for_module(
                &overview_id,
                &input.campaign_id,
                &module_id,
                "Module Overview",
                "module_overview",
            )
            .with_content(overview_content);
            dal::insert_document(conn, &overview_doc)?;

            // Create play notes document
            let play_notes_id = Uuid::new_v4().to_string();
            let play_notes_content = templates::get_play_notes_template();
            let play_notes_doc = NewDocument::for_module(
                &play_notes_id,
                &input.campaign_id,
                &module_id,
                "Play Notes",
                "play_notes",
            )
            .with_content(play_notes_content);
            dal::insert_document(conn, &play_notes_doc)?;

            // Fetch and return the created module
            dal::get_module(conn, &module_id).map_err(ServiceError::from)
        })
    }

    /// List all modules for a campaign, ordered by module number.
    pub fn list_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<Vec<Module>> {
        dal::list_modules(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Get a module by ID.
    ///
    /// Returns `None` if the module doesn't exist.
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<Module>> {
        dal::get_module_optional(self.conn, id).map_err(ServiceError::from)
    }

    /// Get a module by campaign ID and module number.
    pub fn get_by_number(
        &mut self,
        campaign_id: &str,
        module_number: i32,
    ) -> ServiceResult<Option<Module>> {
        dal::get_module_by_number(self.conn, campaign_id, module_number).map_err(ServiceError::from)
    }

    /// Update a module.
    ///
    /// Returns the updated module, or an error if not found.
    pub fn update(&mut self, id: &str, input: UpdateModuleInput) -> ServiceResult<Module> {
        let now = now_rfc3339();

        // Build the update changeset
        let name_ref = input.name.as_deref();
        let desc_ref = input.description.as_ref().map(|d| d.as_deref());

        let update = DalUpdateModule {
            name: name_ref,
            description: desc_ref,
            module_number: None,
            updated_at: Some(&now),
        };

        let rows = dal::update_module(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Module", id));
        }

        dal::get_module(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a module permanently.
    ///
    /// This will cascade delete all related data (documents, maps, etc.).
    pub fn delete(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_module(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Module", id));
        }

        Ok(())
    }

    /// Count modules for a campaign.
    pub fn count_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_modules(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Reorder a module by moving it to a new position (1-indexed).
    ///
    /// Returns the updated list of modules in their new order.
    pub fn reorder(&mut self, module_id: &str, new_position: i32) -> ServiceResult<Vec<Module>> {
        // Look up the module to get campaign_id
        let module = dal::get_module_optional(self.conn, module_id)?
            .ok_or_else(|| ServiceError::not_found("Module", module_id))?;

        dal::reorder_module(self.conn, &module.campaign_id, module_id, new_position).map_err(
            |e| match e {
                diesel::result::Error::RollbackTransaction => ServiceError::Validation(format!(
                    "Invalid position {}. Must be between 1 and the number of modules.",
                    new_position
                )),
                other => ServiceError::from(other),
            },
        )?;

        dal::list_modules(self.conn, &module.campaign_id).map_err(ServiceError::from)
    }

    /// Add a monster to a module.
    ///
    /// If the module already contains an entry for the same monster reference,
    /// the existing entry's quantity is incremented instead of inserting a
    /// duplicate row. This is the single implementation shared by the desktop
    /// UI and the MCP server, so client retries never create duplicates.
    pub fn add_monster(&mut self, input: AddMonsterInput) -> ServiceResult<ModuleMonster> {
        if !self.exists(&input.module_id)? {
            return Err(ServiceError::NotFound {
                entity_type: "Module".to_string(),
                id: input.module_id.clone(),
            });
        }

        if let MonsterRef::Homebrew { id } = &input.monster {
            dal::get_campaign_homebrew_monster(self.conn, id).map_err(|_| {
                ServiceError::NotFound {
                    entity_type: "HomebrewMonster".to_string(),
                    id: id.clone(),
                }
            })?;
        }

        let existing = dal::list_module_monsters(self.conn, &input.module_id)?
            .into_iter()
            .find(|m| match &input.monster {
                MonsterRef::Homebrew { id } => {
                    m.homebrew_monster_id.as_deref() == Some(id.as_str())
                }
                MonsterRef::Catalog { name, source } => {
                    m.monster_name.as_deref() == Some(name.as_str())
                        && m.monster_source.as_deref() == Some(source.as_str())
                }
            });

        if let Some(existing) = existing {
            let now = now_rfc3339();
            let update = UpdateModuleMonster {
                display_name: None,
                notes: None,
                quantity: Some(existing.quantity + input.quantity),
                updated_at: Some(&now),
            };
            dal::update_module_monster(self.conn, &existing.id, &update)?;
            Ok(dal::get_module_monster(self.conn, &existing.id)?)
        } else {
            let id = Uuid::new_v4().to_string();
            let mut new_monster = match &input.monster {
                MonsterRef::Homebrew { id: hb_id } => {
                    NewModuleMonster::from_homebrew(&id, &input.module_id, hb_id)
                }
                MonsterRef::Catalog { name, source } => {
                    NewModuleMonster::new(&id, &input.module_id, name, source)
                }
            };
            new_monster.quantity = input.quantity;
            new_monster.display_name = input.display_name.as_deref();
            new_monster.notes = input.notes.as_deref();

            dal::insert_module_monster(self.conn, &new_monster)?;
            Ok(dal::get_module_monster(self.conn, &id)?)
        }
    }

    /// Update a module monster's display name, notes, or quantity.
    ///
    /// `None` fields are left unchanged.
    pub fn update_monster(
        &mut self,
        monster_id: &str,
        display_name: Option<&str>,
        notes: Option<&str>,
        quantity: Option<i32>,
    ) -> ServiceResult<ModuleMonster> {
        let now = now_rfc3339();
        let update = UpdateModuleMonster {
            display_name: display_name.map(Some),
            notes: notes.map(Some),
            quantity,
            updated_at: Some(&now),
        };
        let rows = dal::update_module_monster(self.conn, monster_id, &update)?;
        if rows == 0 {
            return Err(ServiceError::NotFound {
                entity_type: "ModuleMonster".to_string(),
                id: monster_id.to_string(),
            });
        }
        Ok(dal::get_module_monster(self.conn, monster_id)?)
    }

    /// Remove a monster entry from a module.
    pub fn remove_monster(&mut self, monster_id: &str) -> ServiceResult<()> {
        let rows = dal::delete_module_monster(self.conn, monster_id)?;
        if rows == 0 {
            return Err(ServiceError::NotFound {
                entity_type: "ModuleMonster".to_string(),
                id: monster_id.to_string(),
            });
        }
        Ok(())
    }

    /// List all monster entries for a module.
    pub fn list_monsters(&mut self, module_id: &str) -> ServiceResult<Vec<ModuleMonster>> {
        Ok(dal::list_module_monsters(self.conn, module_id)?)
    }

    /// List the NPCs of a module.
    pub fn list_npcs(&mut self, module_id: &str) -> ServiceResult<Vec<ModuleNpc>> {
        Ok(dal::list_module_npcs(self.conn, module_id)?)
    }

    /// Add an NPC to a module (MIMIR-T-0750). The name is required; empty
    /// text in the other fields is no value.
    pub fn add_npc(&mut self, module_id: &str, fields: &NpcFields) -> ServiceResult<ModuleNpc> {
        if !self.exists(module_id)? {
            return Err(ServiceError::NotFound {
                entity_type: "Module".to_string(),
                id: module_id.to_string(),
            });
        }
        let name = fields
            .name
            .as_deref()
            .and_then(clean)
            .ok_or_else(|| ServiceError::validation("an NPC needs a name"))?;
        let id = Uuid::new_v4().to_string();
        let mut npc = NewModuleNpc::new(&id, module_id, name);
        npc.role = fields.role.as_deref().and_then(clean);
        npc.description = fields.description.as_deref().and_then(clean);
        npc.appearance = fields.appearance.as_deref().and_then(clean);
        npc.personality = fields.personality.as_deref().and_then(clean);
        npc.motivation = fields.motivation.as_deref().and_then(clean);
        npc.secrets = fields.secrets.as_deref().and_then(clean);
        dal::insert_module_npc(self.conn, &npc)?;
        Ok(dal::get_module_npc(self.conn, &id)?)
    }

    /// Add a campaign NPC to a module: the module NPC takes the
    /// character's name, role and personality (traits) where `fields` gives
    /// none. The character must be an NPC of the module's campaign.
    pub fn add_npc_from_character(
        &mut self,
        module_id: &str,
        character_id: &str,
        mut fields: NpcFields,
    ) -> ServiceResult<ModuleNpc> {
        let module = self.get(module_id)?.ok_or_else(|| ServiceError::NotFound {
            entity_type: "Module".to_string(),
            id: module_id.to_string(),
        })?;
        let character = dal::get_character_optional(self.conn, character_id)?
            .filter(|c| c.is_npc() && c.campaign_id.as_deref() == Some(module.campaign_id.as_str()))
            .ok_or_else(|| ServiceError::NotFound {
                entity_type: "NPC of this campaign".to_string(),
                id: character_id.to_string(),
            })?;
        let given = |f: &Option<String>| f.as_deref().and_then(clean).is_some();
        if !given(&fields.name) {
            fields.name = Some(character.name);
        }
        if !given(&fields.role) {
            fields.role = character.role;
        }
        if !given(&fields.personality) {
            fields.personality = character.traits;
        }
        self.add_npc(module_id, &fields)
    }

    /// Change a module NPC: each field given is set; empty text clears it.
    /// The name cannot be cleared.
    pub fn update_npc(&mut self, npc_id: &str, fields: &NpcFields) -> ServiceResult<ModuleNpc> {
        let name = match fields.name.as_deref() {
            Some(n) => {
                Some(clean(n).ok_or_else(|| ServiceError::validation("an NPC needs a name"))?)
            }
            None => None,
        };
        let now = now_rfc3339();
        let update = UpdateModuleNpc {
            name,
            role: set(&fields.role),
            description: set(&fields.description),
            appearance: set(&fields.appearance),
            personality: set(&fields.personality),
            motivation: set(&fields.motivation),
            secrets: set(&fields.secrets),
            updated_at: Some(&now),
            ..Default::default()
        };
        let rows = dal::update_module_npc(self.conn, npc_id, &update)?;
        if rows == 0 {
            return Err(ServiceError::NotFound {
                entity_type: "ModuleNpc".to_string(),
                id: npc_id.to_string(),
            });
        }
        Ok(dal::get_module_npc(self.conn, npc_id)?)
    }

    /// Remove an NPC from a module.
    pub fn remove_npc(&mut self, npc_id: &str) -> ServiceResult<()> {
        let rows = dal::delete_module_npc(self.conn, npc_id)?;
        if rows == 0 {
            return Err(ServiceError::NotFound {
                entity_type: "ModuleNpc".to_string(),
                id: npc_id.to_string(),
            });
        }
        Ok(())
    }

    /// Check if a module exists.
    pub fn exists(&mut self, id: &str) -> ServiceResult<bool> {
        dal::module_exists(self.conn, id).map_err(ServiceError::from)
    }
}

/// The fields of a module NPC, to add or to change one (MIMIR-T-0750).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NpcFields {
    pub name: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub appearance: Option<String>,
    pub personality: Option<String>,
    pub motivation: Option<String>,
    pub secrets: Option<String>,
}

/// A field to change: given (`Some`), its trimmed text, or none to clear it.
fn set(field: &Option<String>) -> Option<Option<&str>> {
    field.as_deref().map(clean)
}

/// Trimmed text, or none when empty.
fn clean(text: &str) -> Option<&str> {
    let t = text.trim();
    (!t.is_empty()).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign::{count_module_documents, insert_campaign, list_module_documents};
    use crate::models::campaign::NewCampaign;
    use crate::test_utils::setup_test_db;

    fn create_test_campaign(conn: &mut SqliteConnection) -> String {
        let campaign_id = Uuid::new_v4().to_string();
        let campaign = NewCampaign::new(&campaign_id, "Test Campaign");
        insert_campaign(conn, &campaign).expect("Failed to create campaign");
        campaign_id
    }

    #[test]
    fn module_npcs_are_added_changed_and_removed() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let mut service = ModuleService::new(&mut conn);
        let module = service
            .create(CreateModuleInput::new(&campaign_id, "Scene 3"))
            .unwrap();

        let fields = NpcFields {
            name: Some("  Alax Jadescales ".into()),
            role: Some("Prisoner".into()),
            secrets: Some("   ".into()),
            ..Default::default()
        };
        let npc = service.add_npc(&module.id, &fields).unwrap();
        assert_eq!(npc.name, "Alax Jadescales");
        assert_eq!(npc.role.as_deref(), Some("Prisoner"));
        assert_eq!(npc.secrets, None, "empty text is no value");
        assert_eq!(service.list_npcs(&module.id).unwrap().len(), 1);

        // Each field given is set; empty text clears; the others stay.
        let changed = service
            .update_npc(
                &npc.id,
                &NpcFields {
                    role: Some(String::new()),
                    motivation: Some("Get the frying pan back".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(changed.name, "Alax Jadescales");
        assert_eq!(changed.role, None);
        assert_eq!(
            changed.motivation.as_deref(),
            Some("Get the frying pan back")
        );

        service.remove_npc(&npc.id).unwrap();
        assert!(service.list_npcs(&module.id).unwrap().is_empty());
    }

    #[test]
    fn a_campaign_npc_fills_a_module_npc() {
        use crate::dal::campaign::insert_character;
        use crate::models::campaign::NewCharacter;
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let other_campaign = create_test_campaign(&mut conn);
        let alax = NewCharacter::new_npc("alax", Some(&campaign_id), "Alax Jadescales")
            .with_npc_info(Some("Innkeeper"), None, None);
        insert_character(&mut conn, &alax).unwrap();
        let stranger = NewCharacter::new_npc("stranger", Some(&other_campaign), "Stranger");
        insert_character(&mut conn, &stranger).unwrap();
        let mut service = ModuleService::new(&mut conn);
        let module = service
            .create(CreateModuleInput::new(&campaign_id, "Scene 3"))
            .unwrap();

        let npc = service
            .add_npc_from_character(
                &module.id,
                "alax",
                NpcFields {
                    description: Some("In the cell".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(npc.name, "Alax Jadescales");
        assert_eq!(npc.role.as_deref(), Some("Innkeeper"));
        assert_eq!(npc.description.as_deref(), Some("In the cell"));

        // A role given wins over the character's.
        let npc = service
            .add_npc_from_character(
                &module.id,
                "alax",
                NpcFields {
                    role: Some("Prisoner".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(npc.role.as_deref(), Some("Prisoner"));

        // An NPC of another campaign, or none, is refused.
        for id in ["stranger", "nobody"] {
            assert!(matches!(
                service.add_npc_from_character(&module.id, id, NpcFields::default()),
                Err(ServiceError::NotFound { .. })
            ));
        }
    }

    #[test]
    fn a_module_npc_needs_a_name_and_a_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let mut service = ModuleService::new(&mut conn);
        let module = service
            .create(CreateModuleInput::new(&campaign_id, "Scene 1"))
            .unwrap();
        let no_name = NpcFields {
            role: Some("Innkeeper".into()),
            ..Default::default()
        };
        assert!(matches!(
            service.add_npc(&module.id, &no_name),
            Err(ServiceError::Validation(_))
        ));
        let named = NpcFields {
            name: Some("Merry Rumwell".into()),
            ..Default::default()
        };
        assert!(matches!(
            service.add_npc("no-such-module", &named),
            Err(ServiceError::NotFound { .. })
        ));
        let npc = service.add_npc(&module.id, &named).unwrap();
        let clear_name = NpcFields {
            name: Some(" ".into()),
            ..Default::default()
        };
        assert!(matches!(
            service.update_npc(&npc.id, &clear_name),
            Err(ServiceError::Validation(_))
        ));
        assert!(matches!(
            service.update_npc("nope", &named),
            Err(ServiceError::NotFound { .. })
        ));
        assert!(matches!(
            service.remove_npc("nope"),
            Err(ServiceError::NotFound { .. })
        ));
    }

    #[test]
    fn test_create_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let module = service.create(input).expect("Failed to create module");

        assert_eq!(module.name, "Chapter 1");
        assert_eq!(module.campaign_id, campaign_id);
        assert_eq!(module.module_number, 1);
        assert!(module.description.is_none());
    }

    #[test]
    fn test_create_module_with_description() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1")
            .with_description("The beginning of our story");
        let module = service.create(input).expect("Failed to create module");

        assert_eq!(
            module.description,
            Some("The beginning of our story".to_string())
        );
    }

    #[test]
    fn test_create_module_creates_2_documents() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let module = service.create(input).expect("Failed to create module");

        // Check that 2 documents were created
        let doc_count =
            count_module_documents(&mut conn, &module.id).expect("Failed to count documents");
        assert_eq!(doc_count, 2);

        // Verify document types
        let docs = list_module_documents(&mut conn, &module.id).expect("Failed to list documents");
        assert_eq!(docs.len(), 2);

        let doc_types: Vec<&str> = docs.iter().map(|d| d.doc_type.as_str()).collect();
        assert!(doc_types.contains(&"module_overview"));
        assert!(doc_types.contains(&"play_notes"));
    }

    #[test]
    fn test_create_module_with_type() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input =
            CreateModuleInput::new(&campaign_id, "The Haunted Manor").with_type(ModuleType::Horror);
        let module = service.create(input).expect("Failed to create module");

        // Verify the overview document has horror template content
        let docs = list_module_documents(&mut conn, &module.id).expect("Failed to list documents");
        let overview = docs
            .iter()
            .find(|d| d.doc_type == "module_overview")
            .expect("Overview not found");

        // Horror template should have horror-specific content
        assert!(overview.content.contains("Horror"));
    }

    #[test]
    fn test_create_modules_auto_increment_number() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input1 = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let module1 = service.create(input1).expect("Failed to create module");
        assert_eq!(module1.module_number, 1);

        let input2 = CreateModuleInput::new(&campaign_id, "Chapter 2");
        let module2 = service.create(input2).expect("Failed to create module");
        assert_eq!(module2.module_number, 2);

        let input3 = CreateModuleInput::new(&campaign_id, "Chapter 3");
        let module3 = service.create(input3).expect("Failed to create module");
        assert_eq!(module3.module_number, 3);
    }

    #[test]
    fn test_list_modules_for_campaign() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input1 = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let input2 = CreateModuleInput::new(&campaign_id, "Chapter 2");
        service.create(input1).expect("Failed to create module");
        service.create(input2).expect("Failed to create module");

        let modules = service
            .list_for_campaign(&campaign_id)
            .expect("Failed to list modules");
        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0].module_number, 1);
        assert_eq!(modules[1].module_number, 2);
    }

    #[test]
    fn test_get_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let created = service.create(input).expect("Failed to create module");

        let retrieved = service
            .get(&created.id)
            .expect("Failed to get module")
            .expect("Module not found");

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Chapter 1");
    }

    #[test]
    fn test_get_module_not_found() {
        let mut conn = setup_test_db();

        let mut service = ModuleService::new(&mut conn);

        let result = service.get("nonexistent").expect("Failed to query module");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_module_by_number() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let created = service.create(input).expect("Failed to create module");

        let retrieved = service
            .get_by_number(&campaign_id, 1)
            .expect("Failed to get module")
            .expect("Module not found");

        assert_eq!(retrieved.id, created.id);
    }

    #[test]
    fn test_update_module_name() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Original Name");
        let created = service.create(input).expect("Failed to create module");

        let update = UpdateModuleInput::set_name("New Name");
        let updated = service
            .update(&created.id, update)
            .expect("Failed to update module");

        assert_eq!(updated.name, "New Name");
    }

    #[test]
    fn test_update_module_not_found() {
        let mut conn = setup_test_db();

        let mut service = ModuleService::new(&mut conn);

        let update = UpdateModuleInput::set_name("New Name");
        let result = service.update("nonexistent", update);

        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_delete_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let created = service.create(input).expect("Failed to create module");

        assert!(service.exists(&created.id).expect("Failed to check exists"));

        service
            .delete(&created.id)
            .expect("Failed to delete module");

        assert!(!service.exists(&created.id).expect("Failed to check exists"));
    }

    #[test]
    fn test_delete_module_not_found() {
        let mut conn = setup_test_db();

        let mut service = ModuleService::new(&mut conn);

        let result = service.delete("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_count_modules() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        assert_eq!(
            service
                .count_for_campaign(&campaign_id)
                .expect("Failed to count"),
            0
        );

        let input1 = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let input2 = CreateModuleInput::new(&campaign_id, "Chapter 2");
        service.create(input1).expect("Failed to create module");
        service.create(input2).expect("Failed to create module");

        assert_eq!(
            service
                .count_for_campaign(&campaign_id)
                .expect("Failed to count"),
            2
        );
    }

    #[test]
    fn test_reorder_module() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = ModuleService::new(&mut conn);

        let input1 = CreateModuleInput::new(&campaign_id, "Chapter 1");
        let input2 = CreateModuleInput::new(&campaign_id, "Chapter 2");
        let input3 = CreateModuleInput::new(&campaign_id, "Chapter 3");
        let m1 = service.create(input1).expect("Failed to create");
        service.create(input2).expect("Failed to create");
        service.create(input3).expect("Failed to create");

        // Move chapter 1 to position 3
        let modules = service.reorder(&m1.id, 3).expect("Failed to reorder");
        assert_eq!(modules.len(), 3);
        assert_eq!(modules[0].name, "Chapter 2");
        assert_eq!(modules[1].name, "Chapter 3");
        assert_eq!(modules[2].name, "Chapter 1");
    }

    #[test]
    fn test_reorder_module_not_found() {
        let mut conn = setup_test_db();

        let mut service = ModuleService::new(&mut conn);
        let result = service.reorder("nonexistent", 1);
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_module_type_display() {
        assert_eq!(ModuleType::General.display_name(), "General");
        assert_eq!(ModuleType::Mystery.display_name(), "Mystery");
        assert_eq!(ModuleType::Dungeon.display_name(), "Dungeon Crawl");
        assert_eq!(ModuleType::Heist.display_name(), "Heist");
        assert_eq!(ModuleType::Horror.display_name(), "Horror");
        assert_eq!(ModuleType::Political.display_name(), "Political Intrigue");
    }

    #[test]
    fn test_module_type_from_str() {
        assert_eq!(ModuleType::from(Some("mystery")), ModuleType::Mystery);
        assert_eq!(ModuleType::from(Some("dungeon")), ModuleType::Dungeon);
        assert_eq!(ModuleType::from(Some("heist")), ModuleType::Heist);
        assert_eq!(ModuleType::from(Some("horror")), ModuleType::Horror);
        assert_eq!(ModuleType::from(Some("political")), ModuleType::Political);
        assert_eq!(ModuleType::from(Some("general")), ModuleType::General);
        assert_eq!(ModuleType::from(Some("unknown")), ModuleType::General);
        assert_eq!(ModuleType::from(None), ModuleType::General);
        for key in ModuleType::KEYS {
            assert_eq!(ModuleType::parse(key).map(|t| t.template_key()), Some(key));
        }
        assert_eq!(ModuleType::parse("adventure"), None);
    }

    #[test]
    fn test_all_module_types_have_templates() {
        let types = [
            ModuleType::General,
            ModuleType::Mystery,
            ModuleType::Dungeon,
            ModuleType::Heist,
            ModuleType::Horror,
            ModuleType::Political,
        ];

        for module_type in types {
            let template = templates::get_module_template(module_type.template_key());
            assert!(template.is_some(), "Template missing for {:?}", module_type);
            assert!(
                !template.unwrap().is_empty(),
                "Template empty for {:?}",
                module_type
            );
        }
    }
}

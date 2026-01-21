//! Character Service
//!
//! Business logic for character management (PCs and NPCs).

use chrono::Utc;
use diesel::SqliteConnection;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{
    Character, CharacterInventory, NewCharacter, NewCharacterInventory, UpdateCharacter,
    UpdateCharacterInventory,
};
use crate::services::{ServiceError, ServiceResult};

/// Input for creating a new character.
#[derive(Debug, Clone)]
pub struct CreateCharacterInput {
    /// Campaign this character belongs to
    pub campaign_id: String,
    /// Character name
    pub name: String,
    /// Whether this is an NPC
    pub is_npc: bool,
    /// Player name (for PCs)
    pub player_name: Option<String>,
    /// Race name (e.g., "Elf")
    pub race_name: Option<String>,
    /// Race source (e.g., "PHB")
    pub race_source: Option<String>,
    /// Background name (e.g., "Acolyte")
    pub background_name: Option<String>,
    /// Background source (e.g., "PHB")
    pub background_source: Option<String>,
    /// Ability scores [STR, DEX, CON, INT, WIS, CHA]
    pub ability_scores: Option<[i32; 6]>,
}

impl CreateCharacterInput {
    /// Create input for a new player character.
    pub fn new_pc(
        campaign_id: impl Into<String>,
        name: impl Into<String>,
        player_name: impl Into<String>,
    ) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            name: name.into(),
            is_npc: false,
            player_name: Some(player_name.into()),
            race_name: None,
            race_source: None,
            background_name: None,
            background_source: None,
            ability_scores: None,
        }
    }

    /// Create input for a new NPC.
    pub fn new_npc(campaign_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            campaign_id: campaign_id.into(),
            name: name.into(),
            is_npc: true,
            player_name: None,
            race_name: None,
            race_source: None,
            background_name: None,
            background_source: None,
            ability_scores: None,
        }
    }

    /// Set the race.
    pub fn with_race(mut self, name: impl Into<String>, source: impl Into<String>) -> Self {
        self.race_name = Some(name.into());
        self.race_source = Some(source.into());
        self
    }

    /// Set the background.
    pub fn with_background(mut self, name: impl Into<String>, source: impl Into<String>) -> Self {
        self.background_name = Some(name.into());
        self.background_source = Some(source.into());
        self
    }

    /// Set ability scores [STR, DEX, CON, INT, WIS, CHA].
    pub fn with_ability_scores(mut self, scores: [i32; 6]) -> Self {
        self.ability_scores = Some(scores);
        self
    }
}

/// Input for updating a character.
#[derive(Debug, Clone, Default)]
pub struct UpdateCharacterInput {
    pub name: Option<String>,
    pub player_name: Option<Option<String>>,
    pub race_name: Option<Option<String>>,
    pub race_source: Option<Option<String>>,
    pub background_name: Option<Option<String>>,
    pub background_source: Option<Option<String>>,
    pub ability_scores: Option<[i32; 6]>,
    pub currency: Option<[i32; 5]>,
    pub traits: Option<Option<String>>,
    pub ideals: Option<Option<String>>,
    pub bonds: Option<Option<String>>,
    pub flaws: Option<Option<String>>,
    pub role: Option<Option<String>>,
    pub location: Option<Option<String>>,
    pub faction: Option<Option<String>>,
}

impl UpdateCharacterInput {
    /// Update character name.
    pub fn set_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Update race.
    pub fn set_race(name: Option<String>, source: Option<String>) -> Self {
        Self {
            race_name: Some(name),
            race_source: Some(source),
            ..Default::default()
        }
    }

    /// Update background.
    pub fn set_background(name: Option<String>, source: Option<String>) -> Self {
        Self {
            background_name: Some(name),
            background_source: Some(source),
            ..Default::default()
        }
    }

    /// Update ability scores [STR, DEX, CON, INT, WIS, CHA].
    pub fn set_ability_scores(scores: [i32; 6]) -> Self {
        Self {
            ability_scores: Some(scores),
            ..Default::default()
        }
    }

    /// Update currency [CP, SP, EP, GP, PP].
    pub fn set_currency(currency: [i32; 5]) -> Self {
        Self {
            currency: Some(currency),
            ..Default::default()
        }
    }

    /// Update roleplay elements.
    pub fn set_roleplay(
        traits: Option<String>,
        ideals: Option<String>,
        bonds: Option<String>,
        flaws: Option<String>,
    ) -> Self {
        Self {
            traits: Some(traits),
            ideals: Some(ideals),
            bonds: Some(bonds),
            flaws: Some(flaws),
            ..Default::default()
        }
    }

    /// Update NPC info.
    pub fn set_npc_info(
        role: Option<String>,
        location: Option<String>,
        faction: Option<String>,
    ) -> Self {
        Self {
            role: Some(role),
            location: Some(location),
            faction: Some(faction),
            ..Default::default()
        }
    }
}

/// Input for adding an item to inventory.
#[derive(Debug, Clone)]
pub struct AddInventoryInput {
    /// Item name from catalog
    pub item_name: String,
    /// Item source (e.g., "PHB")
    pub item_source: String,
    /// Quantity (default 1)
    pub quantity: Option<i32>,
    /// Whether equipped
    pub equipped: bool,
    /// Whether attuned
    pub attuned: bool,
    /// Notes about the item
    pub notes: Option<String>,
}

impl AddInventoryInput {
    /// Create input for adding an item.
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            item_name: name.into(),
            item_source: source.into(),
            quantity: None,
            equipped: false,
            attuned: false,
            notes: None,
        }
    }

    /// Set quantity.
    pub fn with_quantity(mut self, quantity: i32) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Mark as equipped.
    pub fn equipped(mut self) -> Self {
        self.equipped = true;
        self
    }

    /// Mark as attuned.
    pub fn attuned(mut self) -> Self {
        self.attuned = true;
        self
    }

    /// Add notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Service for character management.
///
/// Handles character CRUD operations and inventory management.
pub struct CharacterService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> CharacterService<'a> {
    /// Create a new character service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Create a new character.
    pub fn create(&mut self, input: CreateCharacterInput) -> ServiceResult<Character> {
        let char_id = Uuid::new_v4().to_string();

        let race_name = input.race_name.as_deref();
        let race_source = input.race_source.as_deref();
        let background_name = input.background_name.as_deref();
        let background_source = input.background_source.as_deref();
        let player_name = input.player_name.as_deref();

        let mut new_char = if input.is_npc {
            NewCharacter::new_npc(&char_id, &input.campaign_id, &input.name)
        } else {
            NewCharacter::new_pc(
                &char_id,
                &input.campaign_id,
                &input.name,
                player_name.unwrap_or(""),
            )
        };

        // Set race if provided
        if let (Some(name), Some(source)) = (race_name, race_source) {
            new_char = new_char.with_race(name, source);
        }

        // Set background if provided
        if let (Some(name), Some(source)) = (background_name, background_source) {
            new_char = new_char.with_background(name, source);
        }

        // Set ability scores if provided
        if let Some([str, dex, con, int, wis, cha]) = input.ability_scores {
            new_char = new_char.with_ability_scores(str, dex, con, int, wis, cha);
        }

        dal::insert_character(self.conn, &new_char)?;
        dal::get_character(self.conn, &char_id).map_err(ServiceError::from)
    }

    /// List all characters for a campaign.
    pub fn list_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<Vec<Character>> {
        dal::list_campaign_characters(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List only player characters for a campaign.
    pub fn list_pcs(&mut self, campaign_id: &str) -> ServiceResult<Vec<Character>> {
        dal::list_pcs(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List only NPCs for a campaign.
    pub fn list_npcs(&mut self, campaign_id: &str) -> ServiceResult<Vec<Character>> {
        dal::list_npcs(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// List NPCs by location.
    pub fn list_npcs_by_location(
        &mut self,
        campaign_id: &str,
        location: &str,
    ) -> ServiceResult<Vec<Character>> {
        dal::list_npcs_by_location(self.conn, campaign_id, location)
            .map_err(ServiceError::from)
    }

    /// List NPCs by faction.
    pub fn list_npcs_by_faction(
        &mut self,
        campaign_id: &str,
        faction: &str,
    ) -> ServiceResult<Vec<Character>> {
        dal::list_npcs_by_faction(self.conn, campaign_id, faction).map_err(ServiceError::from)
    }

    /// Get a character by ID.
    pub fn get(&mut self, id: &str) -> ServiceResult<Option<Character>> {
        dal::get_character_optional(self.conn, id).map_err(ServiceError::from)
    }

    /// Update a character.
    pub fn update(&mut self, id: &str, input: UpdateCharacterInput) -> ServiceResult<Character> {
        let now = Utc::now().to_rfc3339();

        // Build the update changeset
        let name_ref = input.name.as_deref();
        let player_name_ref = input.player_name.as_ref().map(|o| o.as_deref());
        let race_name_ref = input.race_name.as_ref().map(|o| o.as_deref());
        let race_source_ref = input.race_source.as_ref().map(|o| o.as_deref());
        let background_name_ref = input.background_name.as_ref().map(|o| o.as_deref());
        let background_source_ref = input.background_source.as_ref().map(|o| o.as_deref());
        let traits_ref = input.traits.as_ref().map(|o| o.as_deref());
        let ideals_ref = input.ideals.as_ref().map(|o| o.as_deref());
        let bonds_ref = input.bonds.as_ref().map(|o| o.as_deref());
        let flaws_ref = input.flaws.as_ref().map(|o| o.as_deref());
        let role_ref = input.role.as_ref().map(|o| o.as_deref());
        let location_ref = input.location.as_ref().map(|o| o.as_deref());
        let faction_ref = input.faction.as_ref().map(|o| o.as_deref());

        let update = UpdateCharacter {
            name: name_ref,
            player_name: player_name_ref,
            race_name: race_name_ref,
            race_source: race_source_ref,
            background_name: background_name_ref,
            background_source: background_source_ref,
            strength: input.ability_scores.map(|s| s[0]),
            dexterity: input.ability_scores.map(|s| s[1]),
            constitution: input.ability_scores.map(|s| s[2]),
            intelligence: input.ability_scores.map(|s| s[3]),
            wisdom: input.ability_scores.map(|s| s[4]),
            charisma: input.ability_scores.map(|s| s[5]),
            cp: input.currency.map(|c| c[0]),
            sp: input.currency.map(|c| c[1]),
            ep: input.currency.map(|c| c[2]),
            gp: input.currency.map(|c| c[3]),
            pp: input.currency.map(|c| c[4]),
            traits: traits_ref,
            ideals: ideals_ref,
            bonds: bonds_ref,
            flaws: flaws_ref,
            role: role_ref,
            location: location_ref,
            faction: faction_ref,
            updated_at: Some(&now),
        };

        let rows = dal::update_character(self.conn, id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Character", id));
        }

        dal::get_character(self.conn, id).map_err(ServiceError::from)
    }

    /// Delete a character and all related data.
    pub fn delete(&mut self, id: &str) -> ServiceResult<()> {
        let rows = dal::delete_character(self.conn, id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("Character", id));
        }
        Ok(())
    }

    /// Check if a character exists.
    pub fn exists(&mut self, id: &str) -> ServiceResult<bool> {
        dal::character_exists(self.conn, id).map_err(ServiceError::from)
    }

    /// Count all characters for a campaign.
    pub fn count_for_campaign(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_campaign_characters(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Count PCs for a campaign.
    pub fn count_pcs(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_pcs(self.conn, campaign_id).map_err(ServiceError::from)
    }

    /// Count NPCs for a campaign.
    pub fn count_npcs(&mut self, campaign_id: &str) -> ServiceResult<i64> {
        dal::count_npcs(self.conn, campaign_id).map_err(ServiceError::from)
    }

    // --- Inventory Management ---

    /// Add an item to a character's inventory.
    pub fn add_to_inventory(
        &mut self,
        character_id: &str,
        input: AddInventoryInput,
    ) -> ServiceResult<CharacterInventory> {
        // Verify character exists
        if !dal::character_exists(self.conn, character_id)? {
            return Err(ServiceError::not_found("Character", character_id));
        }

        let inv_id = Uuid::new_v4().to_string();
        let notes_ref = input.notes.as_deref();

        let mut new_item =
            NewCharacterInventory::new(&inv_id, character_id, &input.item_name, &input.item_source);

        if let Some(qty) = input.quantity {
            new_item = new_item.with_quantity(qty);
        }
        if input.equipped {
            new_item = new_item.equipped();
        }
        if input.attuned {
            new_item = new_item.attuned();
        }
        if let Some(notes) = notes_ref {
            new_item = new_item.with_notes(notes);
        }

        dal::insert_character_inventory(self.conn, &new_item)?;
        dal::get_character_inventory(self.conn, &inv_id).map_err(ServiceError::from)
    }

    /// Remove an item from a character's inventory.
    pub fn remove_from_inventory(&mut self, inventory_id: &str) -> ServiceResult<()> {
        let rows = dal::delete_character_inventory(self.conn, inventory_id)?;
        if rows == 0 {
            return Err(ServiceError::not_found("InventoryItem", inventory_id));
        }
        Ok(())
    }

    /// Get a character's inventory.
    pub fn get_inventory(&mut self, character_id: &str) -> ServiceResult<Vec<CharacterInventory>> {
        dal::list_character_inventory(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Get equipped items for a character.
    pub fn get_equipped_items(
        &mut self,
        character_id: &str,
    ) -> ServiceResult<Vec<CharacterInventory>> {
        dal::list_equipped_items(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Get attuned items for a character.
    pub fn get_attuned_items(
        &mut self,
        character_id: &str,
    ) -> ServiceResult<Vec<CharacterInventory>> {
        dal::list_attuned_items(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Update an inventory item (quantity, equipped, attuned, notes).
    pub fn update_inventory_item(
        &mut self,
        inventory_id: &str,
        quantity: Option<i32>,
        equipped: Option<bool>,
        attuned: Option<bool>,
    ) -> ServiceResult<CharacterInventory> {
        let update = UpdateCharacterInventory {
            quantity,
            equipped: equipped.map(|e| if e { 1 } else { 0 }),
            attuned: attuned.map(|a| if a { 1 } else { 0 }),
            notes: None,
        };

        let rows = dal::update_character_inventory(self.conn, inventory_id, &update)?;
        if rows == 0 {
            return Err(ServiceError::not_found("InventoryItem", inventory_id));
        }

        dal::get_character_inventory(self.conn, inventory_id).map_err(ServiceError::from)
    }

    /// Count attuned items for a character (D&D 5e max is 3).
    pub fn count_attuned_items(&mut self, character_id: &str) -> ServiceResult<i64> {
        dal::count_attuned_items(self.conn, character_id).map_err(ServiceError::from)
    }
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

    #[test]
    fn test_create_pc() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Gandalf", "John");
        let character = service.create(input).expect("Failed to create character");

        assert_eq!(character.name, "Gandalf");
        assert_eq!(character.is_npc, 0);
        assert_eq!(character.player_name, Some("John".to_string()));
    }

    #[test]
    fn test_create_npc() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_npc(&campaign_id, "Shopkeeper");
        let character = service.create(input).expect("Failed to create character");

        assert_eq!(character.name, "Shopkeeper");
        assert_eq!(character.is_npc, 1);
        assert!(character.player_name.is_none());
    }

    #[test]
    fn test_create_with_race_and_background() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Legolas", "Jane")
            .with_race("Elf", "PHB")
            .with_background("Outlander", "PHB");
        let character = service.create(input).expect("Failed to create character");

        assert_eq!(character.race_name, Some("Elf".to_string()));
        assert_eq!(character.race_source, Some("PHB".to_string()));
        assert_eq!(character.background_name, Some("Outlander".to_string()));
        assert_eq!(character.background_source, Some("PHB".to_string()));
    }

    #[test]
    fn test_create_with_ability_scores() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Fighter", "John")
            .with_ability_scores([16, 14, 15, 8, 10, 12]);
        let character = service.create(input).expect("Failed to create character");

        assert_eq!(character.strength, 16);
        assert_eq!(character.dexterity, 14);
        assert_eq!(character.constitution, 15);
        assert_eq!(character.intelligence, 8);
        assert_eq!(character.wisdom, 10);
        assert_eq!(character.charisma, 12);
    }

    #[test]
    fn test_list_for_campaign() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input1 = CreateCharacterInput::new_pc(&campaign_id, "PC1", "John");
        let input2 = CreateCharacterInput::new_pc(&campaign_id, "PC2", "Jane");
        let input3 = CreateCharacterInput::new_npc(&campaign_id, "NPC1");
        service.create(input1).expect("Failed to create character");
        service.create(input2).expect("Failed to create character");
        service.create(input3).expect("Failed to create character");

        let all = service
            .list_for_campaign(&campaign_id)
            .expect("Failed to list");
        assert_eq!(all.len(), 3);

        let pcs = service.list_pcs(&campaign_id).expect("Failed to list");
        assert_eq!(pcs.len(), 2);

        let npcs = service.list_npcs(&campaign_id).expect("Failed to list");
        assert_eq!(npcs.len(), 1);
    }

    #[test]
    fn test_get_character() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let created = service.create(input).expect("Failed to create character");

        let retrieved = service
            .get(&created.id)
            .expect("Failed to get character")
            .expect("Character not found");

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Hero");
    }

    #[test]
    fn test_get_character_not_found() {
        let mut conn = setup_test_db();

        let mut service = CharacterService::new(&mut conn);

        let result = service.get("nonexistent").expect("Failed to query");
        assert!(result.is_none());
    }

    #[test]
    fn test_update_character_name() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Original", "John");
        let created = service.create(input).expect("Failed to create character");

        let update = UpdateCharacterInput::set_name("Updated");
        let updated = service
            .update(&created.id, update)
            .expect("Failed to update");

        assert_eq!(updated.name, "Updated");
    }

    #[test]
    fn test_update_character_race() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let created = service.create(input).expect("Failed to create character");
        assert!(created.race_name.is_none());

        let update = UpdateCharacterInput::set_race(Some("Dwarf".to_string()), Some("PHB".to_string()));
        let updated = service
            .update(&created.id, update)
            .expect("Failed to update");

        assert_eq!(updated.race_name, Some("Dwarf".to_string()));
        assert_eq!(updated.race_source, Some("PHB".to_string()));
    }

    #[test]
    fn test_update_character_not_found() {
        let mut conn = setup_test_db();

        let mut service = CharacterService::new(&mut conn);

        let update = UpdateCharacterInput::set_name("Updated");
        let result = service.update("nonexistent", update);

        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_delete_character() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Doomed", "John");
        let created = service.create(input).expect("Failed to create character");

        assert!(service.exists(&created.id).expect("Failed to check"));

        service.delete(&created.id).expect("Failed to delete");

        assert!(!service.exists(&created.id).expect("Failed to check"));
    }

    #[test]
    fn test_delete_character_not_found() {
        let mut conn = setup_test_db();

        let mut service = CharacterService::new(&mut conn);

        let result = service.delete("nonexistent");
        assert!(matches!(result, Err(ServiceError::NotFound { .. })));
    }

    #[test]
    fn test_count_characters() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        assert_eq!(
            service
                .count_for_campaign(&campaign_id)
                .expect("Failed to count"),
            0
        );

        let input1 = CreateCharacterInput::new_pc(&campaign_id, "PC", "John");
        let input2 = CreateCharacterInput::new_npc(&campaign_id, "NPC1");
        let input3 = CreateCharacterInput::new_npc(&campaign_id, "NPC2");
        service.create(input1).expect("Failed to create");
        service.create(input2).expect("Failed to create");
        service.create(input3).expect("Failed to create");

        assert_eq!(
            service
                .count_for_campaign(&campaign_id)
                .expect("Failed to count"),
            3
        );
        assert_eq!(
            service.count_pcs(&campaign_id).expect("Failed to count"),
            1
        );
        assert_eq!(
            service.count_npcs(&campaign_id).expect("Failed to count"),
            2
        );
    }

    #[test]
    fn test_add_to_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item_input = AddInventoryInput::new("Longsword", "PHB");
        let item = service
            .add_to_inventory(&character.id, item_input)
            .expect("Failed to add item");

        assert_eq!(item.item_name, "Longsword");
        assert_eq!(item.item_source, "PHB");
        assert_eq!(item.quantity, 1);
        assert!(!item.is_equipped());
        assert!(!item.is_attuned());
    }

    #[test]
    fn test_add_to_inventory_with_options() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item_input = AddInventoryInput::new("Cloak of Protection", "DMG")
            .equipped()
            .attuned()
            .with_notes("Found in dungeon");
        let item = service
            .add_to_inventory(&character.id, item_input)
            .expect("Failed to add item");

        assert!(item.is_equipped());
        assert!(item.is_attuned());
        assert_eq!(item.notes, Some("Found in dungeon".to_string()));
    }

    #[test]
    fn test_get_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        service
            .add_to_inventory(&character.id, AddInventoryInput::new("Sword", "PHB"))
            .expect("Failed to add item");
        service
            .add_to_inventory(&character.id, AddInventoryInput::new("Shield", "PHB"))
            .expect("Failed to add item");

        let inventory = service
            .get_inventory(&character.id)
            .expect("Failed to get inventory");
        assert_eq!(inventory.len(), 2);
    }

    #[test]
    fn test_remove_from_inventory() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item = service
            .add_to_inventory(&character.id, AddInventoryInput::new("Sword", "PHB"))
            .expect("Failed to add item");

        service
            .remove_from_inventory(&item.id)
            .expect("Failed to remove item");

        let inventory = service
            .get_inventory(&character.id)
            .expect("Failed to get inventory");
        assert_eq!(inventory.len(), 0);
    }

    #[test]
    fn test_update_inventory_item() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);

        let mut service = CharacterService::new(&mut conn);

        let input = CreateCharacterInput::new_pc(&campaign_id, "Hero", "John");
        let character = service.create(input).expect("Failed to create character");

        let item = service
            .add_to_inventory(
                &character.id,
                AddInventoryInput::new("Arrow", "PHB").with_quantity(20),
            )
            .expect("Failed to add item");

        let updated = service
            .update_inventory_item(&item.id, Some(15), Some(true), None)
            .expect("Failed to update item");

        assert_eq!(updated.quantity, 15);
        assert!(updated.is_equipped());
        assert!(!updated.is_attuned());
    }
}

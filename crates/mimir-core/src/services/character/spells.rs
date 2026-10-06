//! Character spells (known spells and prepared state).

use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{CharacterSpell, NewCharacterSpell, UpdateCharacterSpell};
use crate::services::{ServiceError, ServiceResult};

use super::CharacterService;

impl<'a> CharacterService<'a> {
    /// List all spells a character knows.
    pub fn list_spells(&mut self, character_id: &str) -> ServiceResult<Vec<CharacterSpell>> {
        dal::list_character_spells(self.conn, character_id).map_err(ServiceError::from)
    }

    /// Add a spell to a character's known spells.
    ///
    /// Rejects the add if the character already knows the spell from the same
    /// class. Single implementation shared by the desktop UI and MCP server.
    pub fn add_spell(
        &mut self,
        character_id: &str,
        spell_name: &str,
        spell_source: &str,
        source_class: &str,
        prepared: bool,
    ) -> ServiceResult<CharacterSpell> {
        if dal::find_character_spell_by_name(self.conn, character_id, spell_name, source_class)?
            .is_some()
        {
            return Err(ServiceError::Validation(format!(
                "Character already knows {} from {}",
                spell_name, source_class
            )));
        }

        let id = Uuid::new_v4().to_string();
        let mut spell =
            NewCharacterSpell::new(&id, character_id, spell_name, spell_source, source_class);
        if prepared {
            spell = spell.prepared();
        }
        dal::insert_character_spell(self.conn, &spell)?;

        Ok(CharacterSpell {
            id,
            character_id: character_id.to_string(),
            spell_name: spell_name.to_string(),
            spell_source: spell_source.to_string(),
            source_class: source_class.to_string(),
            prepared: if prepared { 1 } else { 0 },
        })
    }

    /// Remove a spell from a character's known spells.
    ///
    /// With `source_class`, removes that class's instance only. Without it,
    /// removes every instance matching the name case-insensitively.
    pub fn remove_spell(
        &mut self,
        character_id: &str,
        spell_name: &str,
        source_class: Option<&str>,
    ) -> ServiceResult<()> {
        if let Some(class) = source_class {
            let spell =
                dal::find_character_spell_by_name(self.conn, character_id, spell_name, class)?
                    .ok_or_else(|| ServiceError::Validation(format!(
                        "Character doesn't know {} from {}",
                        spell_name, class
                    )))?;
            dal::delete_character_spell(self.conn, &spell.id)?;
        } else {
            let spells = dal::list_character_spells(self.conn, character_id)?;
            let matching: Vec<_> = spells
                .iter()
                .filter(|s| s.spell_name.to_lowercase() == spell_name.to_lowercase())
                .collect();
            if matching.is_empty() {
                return Err(ServiceError::Validation(format!(
                    "Character doesn't know {}",
                    spell_name
                )));
            }
            for spell in matching {
                dal::delete_character_spell(self.conn, &spell.id)?;
            }
        }
        Ok(())
    }

    /// Toggle a spell's prepared status.
    pub fn toggle_spell_prepared(&mut self, spell_id: &str) -> ServiceResult<CharacterSpell> {
        let spell = dal::get_character_spell_optional(self.conn, spell_id)?
            .ok_or_else(|| ServiceError::not_found("CharacterSpell", spell_id))?;

        let new_prepared = !spell.is_prepared();
        let update = UpdateCharacterSpell::set_prepared(new_prepared);
        dal::update_character_spell(self.conn, spell_id, &update)?;

        Ok(CharacterSpell {
            prepared: if new_prepared { 1 } else { 0 },
            ..spell
        })
    }
}

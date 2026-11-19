//! Player notes and party sharing (MIMIR-T-0718, ADR MIMIR-A-0010).
//!
//! - Notes: per player character, markdown, optionally about a module of
//!   the character's campaign. The server routes decide who writes (the
//!   character's player) and who reads (that player and the DM).
//! - Sharing: a player may share their sheet with the party (default off);
//!   the other players of the campaign then read it.

use diesel::SqliteConnection;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{Character, NewPlayerNote, PlayerNote, UpdatePlayerNote};
use crate::services::{ServiceError, ServiceResult};
use crate::utils::now_rfc3339;

/// A member of a campaign's party.
#[derive(Debug, Clone)]
pub struct PartyMember {
    pub character: Character,
    pub shared: bool,
}

pub struct PartyService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> PartyService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    fn player_character(&mut self, id: &str) -> ServiceResult<Character> {
        let c = dal::get_character_optional(self.conn, id)?
            .ok_or_else(|| ServiceError::not_found("Character", id))?;
        if c.is_npc != 0 {
            return Err(ServiceError::validation(
                "notes and sharing are for player characters",
            ));
        }
        Ok(c)
    }

    /// The module must be in the character's campaign.
    fn check_module(&mut self, c: &Character, module_id: Option<&str>) -> ServiceResult<()> {
        let Some(m) = module_id else { return Ok(()) };
        let module = dal::get_module_optional(self.conn, m)?
            .ok_or_else(|| ServiceError::not_found("Module", m))?;
        if Some(&module.campaign_id) != c.campaign_id.as_ref() {
            return Err(ServiceError::validation(
                "the module is not in the character's campaign",
            ));
        }
        Ok(())
    }

    // ---- Notes ----

    pub fn notes(&mut self, character_id: &str) -> ServiceResult<Vec<PlayerNote>> {
        self.player_character(character_id)?;
        Ok(dal::list_player_notes(self.conn, character_id)?)
    }

    pub fn note(&mut self, id: &str) -> ServiceResult<PlayerNote> {
        dal::get_player_note(self.conn, id)?
            .ok_or_else(|| ServiceError::not_found("PlayerNote", id))
    }

    /// The notes about a module, with each note's character name.
    pub fn module_notes(&mut self, module_id: &str) -> ServiceResult<Vec<(PlayerNote, String)>> {
        Ok(dal::list_module_player_notes(self.conn, module_id)?)
    }

    pub fn add_note(
        &mut self,
        character_id: &str,
        module_id: Option<&str>,
        title: &str,
        content: &str,
    ) -> ServiceResult<PlayerNote> {
        let c = self.player_character(character_id)?;
        let title = title.trim();
        if title.is_empty() {
            return Err(ServiceError::validation("the note needs a title"));
        }
        self.check_module(&c, module_id)?;
        let id = Uuid::new_v4().to_string();
        dal::insert_player_note(
            self.conn,
            &NewPlayerNote {
                id: &id,
                character_id,
                module_id,
                title,
                content,
            },
        )?;
        self.note(&id)
    }

    /// Change a note; `module_id`: `Some(None)` clears it.
    pub fn update_note(
        &mut self,
        id: &str,
        title: Option<&str>,
        content: Option<&str>,
        module_id: Option<Option<&str>>,
    ) -> ServiceResult<PlayerNote> {
        let note = self.note(id)?;
        if title.is_some_and(|t| t.trim().is_empty()) {
            return Err(ServiceError::validation("the note needs a title"));
        }
        if let Some(Some(m)) = module_id {
            let c = self.player_character(&note.character_id)?;
            self.check_module(&c, Some(m))?;
        }
        dal::update_player_note(
            self.conn,
            id,
            &UpdatePlayerNote {
                module_id: module_id.map(|m| m.map(String::from)),
                title: title.map(|t| t.trim().to_string()),
                content: content.map(String::from),
                updated_at: Some(now_rfc3339()),
            },
        )?;
        self.note(id)
    }

    pub fn delete_note(&mut self, id: &str) -> ServiceResult<()> {
        if dal::delete_player_note(self.conn, id)? == 0 {
            return Err(ServiceError::not_found("PlayerNote", id));
        }
        Ok(())
    }

    // ---- Sharing ----

    pub fn is_shared(&mut self, character_id: &str) -> ServiceResult<bool> {
        Ok(dal::is_sheet_shared(self.conn, character_id)?)
    }

    pub fn set_shared(&mut self, character_id: &str, shared: bool) -> ServiceResult<bool> {
        self.player_character(character_id)?;
        if shared {
            dal::share_sheet(self.conn, character_id)?;
        } else {
            dal::unshare_sheet(self.conn, character_id)?;
        }
        Ok(shared)
    }

    /// The player characters of a campaign and whether each is shared.
    pub fn party(&mut self, campaign_id: &str) -> ServiceResult<Vec<PartyMember>> {
        let pcs = dal::list_pcs(self.conn, campaign_id)?;
        let ids: Vec<String> = pcs.iter().map(|c| c.id.clone()).collect();
        let shared = dal::shared_among(self.conn, &ids)?;
        Ok(pcs
            .into_iter()
            .map(|c| PartyMember {
                shared: shared.contains(&c.id),
                character: c,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{
        CampaignService, CharacterService, CreateCampaignInput, CreateCharacterInput,
        CreateModuleInput, ModuleService,
    };
    use crate::test_utils::setup_test_db;

    struct Setup {
        conn: diesel::SqliteConnection,
        campaign: String,
        pc: String,
        npc: String,
        module: String,
    }

    fn setup() -> Setup {
        let mut conn = setup_test_db();
        let campaign = CampaignService::new(&mut conn)
            .create(CreateCampaignInput::new("Test"))
            .unwrap()
            .id;
        let module = ModuleService::new(&mut conn)
            .create(CreateModuleInput::new(campaign.clone(), "Caves"))
            .unwrap()
            .id;
        let mut chars = CharacterService::new(&mut conn);
        let pc = chars
            .create(CreateCharacterInput::new_pc(
                Some(campaign.clone()),
                "Robin",
                "Sam",
            ))
            .unwrap()
            .id;
        let npc = chars
            .create(CreateCharacterInput::new_npc(
                Some(campaign.clone()),
                "Gundren",
            ))
            .unwrap()
            .id;
        Setup {
            conn,
            campaign,
            pc,
            npc,
            module,
        }
    }

    #[test]
    fn notes_are_written_listed_changed_and_deleted() {
        let mut s = setup();
        let mut party = PartyService::new(&mut s.conn);
        let n = party
            .add_note(&s.pc, Some(&s.module), " Goblins ", "We met *goblins*.")
            .unwrap();
        assert_eq!(
            (n.title.as_str(), n.module_id.as_deref()),
            ("Goblins", Some(s.module.as_str()))
        );
        party.add_note(&s.pc, None, "Town", "").unwrap();
        assert_eq!(party.notes(&s.pc).unwrap().len(), 2);
        let about = party.module_notes(&s.module).unwrap();
        assert_eq!(about.len(), 1);
        assert_eq!(about[0].1, "Robin");
        let changed = party
            .update_note(&n.id, Some("Goblin cave"), Some("Two goblins."), Some(None))
            .unwrap();
        assert_eq!(changed.title, "Goblin cave");
        assert_eq!(changed.module_id, None, "None clears the module");
        assert!(party.module_notes(&s.module).unwrap().is_empty());
        party.delete_note(&n.id).unwrap();
        assert!(matches!(
            party.delete_note(&n.id),
            Err(ServiceError::NotFound { .. })
        ));
    }

    #[test]
    fn notes_are_checked() {
        let mut s = setup();
        let mut party = PartyService::new(&mut s.conn);
        assert!(matches!(
            party.add_note(&s.pc, None, "  ", "x"),
            Err(ServiceError::Validation(_))
        ));
        assert!(matches!(
            party.add_note(&s.npc, None, "A", "x"),
            Err(ServiceError::Validation(_))
        ));
        assert!(matches!(
            party.add_note(&s.pc, Some("nope"), "A", "x"),
            Err(ServiceError::NotFound { .. })
        ));
        // A module of another campaign is refused.
        let other = CampaignService::new(&mut s.conn)
            .create(CreateCampaignInput::new("Other"))
            .unwrap()
            .id;
        let elsewhere = ModuleService::new(&mut s.conn)
            .create(CreateModuleInput::new(other, "Far"))
            .unwrap()
            .id;
        let mut party = PartyService::new(&mut s.conn);
        assert!(matches!(
            party.add_note(&s.pc, Some(&elsewhere), "A", "x"),
            Err(ServiceError::Validation(_))
        ));
    }

    #[test]
    fn sharing_defaults_off_and_lists_the_party() {
        let mut s = setup();
        let mut party = PartyService::new(&mut s.conn);
        assert!(!party.is_shared(&s.pc).unwrap());
        let members = party.party(&s.campaign).unwrap();
        assert_eq!(members.len(), 1, "player characters only");
        assert!(!members[0].shared);
        party.set_shared(&s.pc, true).unwrap();
        party.set_shared(&s.pc, true).unwrap();
        assert!(party.is_shared(&s.pc).unwrap());
        assert!(party.party(&s.campaign).unwrap()[0].shared);
        party.set_shared(&s.pc, false).unwrap();
        assert!(!party.is_shared(&s.pc).unwrap());
        assert!(matches!(
            party.set_shared(&s.npc, true),
            Err(ServiceError::Validation(_))
        ));
    }
}

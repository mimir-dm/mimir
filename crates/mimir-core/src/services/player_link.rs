//! Player links (MIMIR-T-0716, ADR MIMIR-A-0010): one link per player
//! character. The token is random (256 bits) and shown once; only its
//! SHA-256 hash is stored. A new link replaces the old one, so the old
//! token stops working.

use diesel::SqliteConnection;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::dal::campaign as dal;
use crate::models::campaign::{Character, NewPlayerLink};
use crate::services::{ServiceError, ServiceResult};

/// Who a link token belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkHolder {
    pub character_id: String,
    pub campaign_id: String,
    pub name: String,
}

pub struct PlayerLinkService<'a> {
    conn: &'a mut SqliteConnection,
}

/// The stored form of a token.
pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn new_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    // URL-safe: hex.
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl<'a> PlayerLinkService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    fn player_character(&mut self, character_id: &str) -> ServiceResult<Character> {
        let c = dal::get_character_optional(self.conn, character_id)?
            .ok_or_else(|| ServiceError::not_found("Character", character_id))?;
        if c.is_npc != 0 {
            return Err(ServiceError::validation(
                "only a player character has a player link",
            ));
        }
        if c.campaign_id.is_none() {
            return Err(ServiceError::validation("the character is in no campaign"));
        }
        Ok(c)
    }

    /// Make a new link for a player character (replacing any old one) and
    /// return its token. The token is not stored: show it now.
    pub fn issue(&mut self, character_id: &str) -> ServiceResult<String> {
        self.player_character(character_id)?;
        let token = new_token();
        let hash = hash_token(&token);
        dal::replace_player_link(
            self.conn,
            &NewPlayerLink {
                character_id,
                token_hash: &hash,
            },
        )?;
        Ok(token)
    }

    /// Remove a character's link. True if there was one.
    pub fn revoke(&mut self, character_id: &str) -> ServiceResult<bool> {
        Ok(dal::delete_player_link(self.conn, character_id)? > 0)
    }

    /// When the character's link was made, if it has one.
    pub fn created_at(&mut self, character_id: &str) -> ServiceResult<Option<String>> {
        Ok(dal::get_player_link(self.conn, character_id)?.map(|l| l.created_at))
    }

    /// The character a token belongs to, if the token is a current link.
    pub fn holder(&mut self, token: &str) -> ServiceResult<Option<LinkHolder>> {
        let Some(link) = dal::find_player_link_by_hash(self.conn, &hash_token(token))? else {
            return Ok(None);
        };
        let Some(c) = dal::get_character_optional(self.conn, &link.character_id)? else {
            return Ok(None);
        };
        Ok(c.campaign_id.map(|campaign_id| LinkHolder {
            character_id: c.id,
            campaign_id,
            name: c.name,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{
        CampaignService, CharacterService, CreateCampaignInput, CreateCharacterInput,
    };
    use crate::test_utils::setup_test_db;

    fn setup() -> (diesel::SqliteConnection, String, String) {
        let mut conn = setup_test_db();
        let campaign = CampaignService::new(&mut conn)
            .create(CreateCampaignInput::new("Test"))
            .unwrap();
        let mut chars = CharacterService::new(&mut conn);
        let pc = chars
            .create(CreateCharacterInput::new_pc(
                Some(campaign.id.clone()),
                "Robin",
                "Sam",
            ))
            .unwrap();
        let npc = chars
            .create(CreateCharacterInput::new_npc(
                Some(campaign.id.clone()),
                "Gundren",
            ))
            .unwrap();
        (conn, pc.id, npc.id)
    }

    #[test]
    fn a_link_names_its_character_and_is_stored_hashed() {
        let (mut conn, pc, _) = setup();
        let mut links = PlayerLinkService::new(&mut conn);
        let token = links.issue(&pc).unwrap();
        assert_eq!(token.len(), 64);
        let holder = links.holder(&token).unwrap().unwrap();
        assert_eq!(
            (holder.character_id.as_str(), holder.name.as_str()),
            (pc.as_str(), "Robin")
        );
        assert!(links.created_at(&pc).unwrap().is_some());
        let stored = dal::get_player_link(&mut conn, &pc).unwrap().unwrap();
        assert_ne!(stored.token_hash, token, "the token is not stored");
        assert_eq!(stored.token_hash, hash_token(&token));
    }

    #[test]
    fn reissue_and_revoke_end_the_old_token() {
        let (mut conn, pc, _) = setup();
        let mut links = PlayerLinkService::new(&mut conn);
        let first = links.issue(&pc).unwrap();
        let second = links.issue(&pc).unwrap();
        assert_ne!(first, second);
        assert!(
            links.holder(&first).unwrap().is_none(),
            "reissue ends the old link"
        );
        assert!(links.holder(&second).unwrap().is_some());
        assert!(links.revoke(&pc).unwrap());
        assert!(links.holder(&second).unwrap().is_none());
        assert!(!links.revoke(&pc).unwrap());
        assert!(links.holder("not-a-token").unwrap().is_none());
    }

    #[test]
    fn npcs_and_unknown_characters_get_no_link() {
        let (mut conn, _, npc) = setup();
        let mut links = PlayerLinkService::new(&mut conn);
        assert!(matches!(
            links.issue(&npc),
            Err(ServiceError::Validation(_))
        ));
        assert!(matches!(
            links.issue("nope"),
            Err(ServiceError::NotFound { .. })
        ));
    }
}

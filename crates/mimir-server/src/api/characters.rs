//! Characters (MIMIR-T-0717): the sheet, its changes, inventory, spells and
//! level-up over CharacterService. Player routes: a player reaches only
//! their own character (`scope::character`). Each change tells the sockets
//! (`Change::Character`): the DM sees a player's edit live.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use diesel::SqliteConnection;
use mimir_core::models::campaign::{CharacterInventory, CharacterSpell};
use mimir_core::services::{
    AddInventoryInput, BackgroundService, CatalogEntityService, CharacterService, ClassService,
    CreateCharacterInput, LevelUpRequest, PartyService, RaceService, ServiceError,
    UpdateCharacterInput,
};
use mimir_wire as wire;
use serde_json::Value;

use super::scope;
use crate::auth::Caller;
use crate::error::{ApiError, JsonBody};
use crate::live::{Change, Hub};
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

fn item(i: CharacterInventory) -> wire::InventoryItem {
    wire::InventoryItem {
        id: i.id,
        item_name: i.item_name,
        item_source: i.item_source,
        quantity: i.quantity,
        equipped: i.equipped != 0,
        attuned: i.attuned != 0,
        notes: i.notes,
    }
}

fn spell(s: CharacterSpell) -> wire::KnownSpell {
    wire::KnownSpell {
        id: s.id,
        spell_name: s.spell_name,
        spell_source: s.spell_source,
        source_class: s.source_class,
        prepared: s.prepared != 0,
        level: None,
    }
}

/// A known spell with its level.
fn spell_with_level(
    conn: &mut SqliteConnection,
    campaign: Option<&str>,
    s: CharacterSpell,
) -> wire::KnownSpell {
    let level = super::sheet::spell_level(conn, campaign, &s.spell_name, &s.spell_source);
    wire::KnownSpell { level, ..spell(s) }
}

/// Walking speed from the race's catalog data (30 when unknown).
fn race_speed(conn: &mut SqliteConnection, name: Option<&str>, source: Option<&str>) -> i32 {
    let (Some(name), Some(source)) = (name, source) else {
        return 30;
    };
    let Ok(Some(race)) = RaceService::new(conn).get_by_name_and_source(name, source) else {
        return 30;
    };
    let data: Value = serde_json::from_str(&race.data).unwrap_or(Value::Null);
    match data.get("speed") {
        Some(Value::Number(n)) => n.as_i64().map(|n| n as i32).unwrap_or(30),
        Some(Value::Object(o)) => o
            .get("walk")
            .and_then(Value::as_i64)
            .map(|n| n as i32)
            .unwrap_or(30),
        _ => 30,
    }
}

/// The whole sheet of a character.
pub fn sheet(conn: &mut SqliteConnection, id: &str) -> Result<wire::CharacterSheet, ApiError> {
    let mut chars = CharacterService::new(conn);
    let c = chars
        .get_enriched(id)?
        .ok_or_else(|| ServiceError::not_found("Character", id))?;
    let inventory = chars.get_inventory(id)?.into_iter().map(item).collect();
    let known = chars.list_spells(id)?;
    let feats = chars
        .list_feats(id)?
        .into_iter()
        .map(|f| wire::CharacterFeat {
            name: f.feat_name,
            source: f.feat_source,
            source_type: f.source_type,
        })
        .collect();
    let features = chars
        .list_features(id)?
        .into_iter()
        .map(|f| wire::ChosenFeature {
            kind: f.feature_type,
            name: f.feature_name,
            source: f.feature_source,
            source_class: f.source_class,
        })
        .collect();
    let spells = known
        .into_iter()
        .map(|s| spell_with_level(conn, c.campaign_id.as_deref(), s))
        .collect();
    let speed = race_speed(conn, c.race_name.as_deref(), c.race_source.as_deref());
    let shared = PartyService::new(conn).is_shared(id)?;
    Ok(wire::CharacterSheet {
        id: c.id,
        campaign_id: c.campaign_id,
        name: c.name,
        is_npc: c.is_npc != 0,
        player_name: c.player_name,
        race_name: c.race_name,
        race_source: c.race_source,
        background_name: c.background_name,
        background_source: c.background_source,
        abilities: wire::Abilities {
            strength: c.strength,
            dexterity: c.dexterity,
            constitution: c.constitution,
            intelligence: c.intelligence,
            wisdom: c.wisdom,
            charisma: c.charisma,
        },
        currency: wire::Currency {
            cp: c.cp,
            sp: c.sp,
            ep: c.ep,
            gp: c.gp,
            pp: c.pp,
        },
        traits: c.traits,
        ideals: c.ideals,
        bonds: c.bonds,
        flaws: c.flaws,
        role: c.role,
        location: c.location,
        faction: c.faction,
        classes: c
            .classes
            .into_iter()
            .map(|k| wire::SheetClass {
                class_name: k.class_name,
                class_source: k.class_source,
                level: k.level,
                subclass_name: k.subclass_name,
                subclass_source: k.subclass_source,
                starting: k.starting_class != 0,
            })
            .collect(),
        proficiencies: c
            .proficiencies
            .into_iter()
            .map(|p| wire::Proficiency {
                kind: p.proficiency_type,
                name: p.name,
                expertise: p.expertise != 0,
            })
            .collect(),
        inventory,
        spells,
        feats,
        features,
        speed,
        shared,
        updated_at: c.updated_at,
    })
}

fn notify(live: &Hub, conn: &mut SqliteConnection, character_id: &str) {
    if let Ok(Some(c)) = CharacterService::new(conn).get(character_id) {
        live.publish(Change::Character {
            campaign_id: c.campaign_id,
            character_id: character_id.to_string(),
        });
    }
}

/// The character of an inventory row.
fn owner_of_item(conn: &mut SqliteConnection, id: &str) -> Result<String, ApiError> {
    CharacterService::new(conn)
        .inventory_owner(id)?
        .ok_or_else(|| ServiceError::not_found("InventoryItem", id).into())
}

/// `GET /characters/{id}`
pub async fn get(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> ApiResult<wire::CharacterSheet> {
    state
        .with_db(move |conn| {
            scope::read_character(&caller, conn, &id)?;
            sheet(conn, &id)
        })
        .await
        .map(Json)
}

/// `PATCH /characters/{id}`
pub async fn patch(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
    JsonBody(p): JsonBody<wire::CharacterPatch>,
) -> ApiResult<wire::CharacterSheet> {
    scope::character(&caller, &id)?;
    if let Some(a) = p.abilities {
        let scores = [
            a.strength,
            a.dexterity,
            a.constitution,
            a.intelligence,
            a.wisdom,
            a.charisma,
        ];
        if scores.iter().any(|s| !(1..=30).contains(s)) {
            return Err(ServiceError::validation("ability scores go from 1 to 30").into());
        }
    }
    if let Some(c) = p.currency {
        if [c.cp, c.sp, c.ep, c.gp, c.pp].iter().any(|n| *n < 0) {
            return Err(ServiceError::validation("coins cannot be negative").into());
        }
    }
    if p.name.as_deref().is_some_and(|n| n.trim().is_empty()) {
        return Err(ServiceError::validation("the name is empty").into());
    }
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            CharacterService::new(conn).update(
                &id,
                UpdateCharacterInput {
                    name: p.name,
                    player_name: p.player_name,
                    ability_scores: p.abilities.map(|a| {
                        [
                            a.strength,
                            a.dexterity,
                            a.constitution,
                            a.intelligence,
                            a.wisdom,
                            a.charisma,
                        ]
                    }),
                    currency: p.currency.map(|c| [c.cp, c.sp, c.ep, c.gp, c.pp]),
                    traits: p.traits,
                    ideals: p.ideals,
                    bonds: p.bonds,
                    flaws: p.flaws,
                    role: p.role,
                    location: p.location,
                    faction: p.faction,
                    ..UpdateCharacterInput::default()
                },
            )?;
            notify(&live, conn, &id);
            sheet(conn, &id)
        })
        .await
        .map(Json)
}

/// `POST /characters/{id}/inventory`
pub async fn add_item(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewInventoryItem>,
) -> Result<(StatusCode, Json<wire::InventoryItem>), ApiError> {
    scope::character(&caller, &id)?;
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let mut input = AddInventoryInput::new(body.item_name, body.item_source);
            input.quantity = body.quantity;
            input.equipped = body.equipped;
            input.attuned = body.attuned;
            input.notes = body.notes;
            let added = CharacterService::new(conn).add_to_inventory(&id, input)?;
            notify(&live, conn, &id);
            Ok((StatusCode::CREATED, Json(item(added))))
        })
        .await
}

/// `PATCH /inventory/{id}`
pub async fn patch_item(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
    JsonBody(p): JsonBody<wire::InventoryPatch>,
) -> ApiResult<wire::InventoryItem> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let owner = owner_of_item(conn, &id)?;
            scope::character(&caller, &owner)?;
            let updated = CharacterService::new(conn)
                .update_inventory_item(&id, p.quantity, p.equipped, p.attuned)?;
            notify(&live, conn, &owner);
            Ok(item(updated))
        })
        .await
        .map(Json)
}

/// `DELETE /inventory/{id}`
pub async fn remove_item(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let owner = owner_of_item(conn, &id)?;
            scope::character(&caller, &owner)?;
            CharacterService::new(conn).remove_from_inventory(&id)?;
            notify(&live, conn, &owner);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// `POST /characters/{id}/spells`
pub async fn add_spell(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::NewKnownSpell>,
) -> Result<(StatusCode, Json<wire::KnownSpell>), ApiError> {
    scope::character(&caller, &id)?;
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let s = CharacterService::new(conn).add_spell(
                &id,
                &body.spell_name,
                &body.spell_source,
                &body.source_class,
                body.prepared,
            )?;
            notify(&live, conn, &id);
            let campaign = CharacterService::new(conn)
                .get(&id)?
                .and_then(|c| c.campaign_id);
            Ok((
                StatusCode::CREATED,
                Json(spell_with_level(conn, campaign.as_deref(), s)),
            ))
        })
        .await
}

fn find_spell(
    conn: &mut SqliteConnection,
    character: &str,
    spell_id: &str,
) -> Result<CharacterSpell, ApiError> {
    CharacterService::new(conn)
        .list_spells(character)?
        .into_iter()
        .find(|s| s.id == spell_id)
        .ok_or_else(|| ServiceError::not_found("CharacterSpell", spell_id).into())
}

/// `PATCH /characters/{id}/spells/{spell_id}`: prepared or not.
pub async fn patch_spell(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path((id, spell_id)): Path<(String, String)>,
    JsonBody(p): JsonBody<wire::SpellPatch>,
) -> ApiResult<wire::KnownSpell> {
    scope::character(&caller, &id)?;
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let s = find_spell(conn, &id, &spell_id)?;
            let s = if (s.prepared != 0) == p.prepared {
                s
            } else {
                CharacterService::new(conn).toggle_spell_prepared(&spell_id)?
            };
            notify(&live, conn, &id);
            let campaign = CharacterService::new(conn)
                .get(&id)?
                .and_then(|c| c.campaign_id);
            Ok(spell_with_level(conn, campaign.as_deref(), s))
        })
        .await
        .map(Json)
}

/// `DELETE /characters/{id}/spells/{spell_id}`
pub async fn remove_spell(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path((id, spell_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    scope::character(&caller, &id)?;
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let s = find_spell(conn, &id, &spell_id)?;
            CharacterService::new(conn).remove_spell(&id, &s.spell_name, Some(&s.source_class))?;
            notify(&live, conn, &id);
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// `POST /characters/{id}/level-up`: one level, all steps at once (the
/// server checks them; any error changes nothing).
pub async fn level_up(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<LevelUpRequest>,
) -> ApiResult<wire::LevelUpResult> {
    scope::character(&caller, &id)?;
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let r = CharacterService::new(conn).level_up(&id, request)?;
            notify(&live, conn, &id);
            Ok(wire::LevelUpResult {
                sheet: sheet(conn, &id)?,
                hp_gained: r.hp_gained,
                new_total_level: r.new_total_level,
                is_multiclass: r.is_multiclass,
            })
        })
        .await
        .map(Json)
}

// ------------------------------------------------------------ the DM's management (MIMIR-T-0721)

/// An empty text is no text.
fn text(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// A catalog pick: both name and source, or neither.
fn pick(
    what: &str,
    name: Option<String>,
    source: Option<String>,
) -> Result<Option<(String, String)>, ApiError> {
    match (text(name), text(source)) {
        (Some(n), Some(s)) => Ok(Some((n, s))),
        (None, None) => Ok(None),
        _ => Err(ServiceError::validation(format!("give the {what}'s name and source")).into()),
    }
}

/// `POST /characters` (DM): a PC from the wizard, or an NPC. Race,
/// background and class must be in the catalog; the class starts at
/// level 1.
pub async fn create(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    JsonBody(body): JsonBody<wire::NewCharacter>,
) -> Result<(StatusCode, Json<wire::CharacterSheet>), ApiError> {
    scope::dm(&caller)?;
    let name = text(Some(body.name.clone()))
        .ok_or_else(|| ServiceError::validation("the name is empty"))?;
    if let Some(scores) = body.ability_scores {
        if scores.iter().any(|s| !(1..=30).contains(s)) {
            return Err(ServiceError::validation("ability scores go from 1 to 30").into());
        }
    }
    let race = pick("race", body.race_name, body.race_source)?;
    let background = pick("background", body.background_name, body.background_source)?;
    let class = pick("class", body.class_name, body.class_source)?;
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            if let Some((n, s)) = &race {
                RaceService::new(conn)
                    .get_by_name_and_source(n, s)?
                    .ok_or_else(|| ServiceError::not_found("Race", format!("{n} ({s})")))?;
            }
            if let Some((n, s)) = &background {
                BackgroundService::new(conn)
                    .get_by_name_and_source(n, s)?
                    .ok_or_else(|| ServiceError::not_found("Background", format!("{n} ({s})")))?;
            }
            if let Some((n, s)) = &class {
                ClassService::new(conn)
                    .get_by_name_and_source(n, s)?
                    .ok_or_else(|| ServiceError::not_found("Class", format!("{n} ({s})")))?;
            }
            let input = CreateCharacterInput {
                campaign_id: text(body.campaign_id),
                name,
                is_npc: body.is_npc,
                player_name: if body.is_npc {
                    None
                } else {
                    text(body.player_name)
                },
                race_name: race.as_ref().map(|r| r.0.clone()),
                race_source: race.map(|r| r.1),
                background_name: background.as_ref().map(|b| b.0.clone()),
                background_source: background.map(|b| b.1),
                ability_scores: body.ability_scores,
                class_name: class.as_ref().map(|c| c.0.clone()),
                class_source: class.map(|c| c.1),
                selected_skills: Some(body.selected_skills).filter(|s| !s.is_empty()),
            };
            let c = CharacterService::new(conn).create(input)?;
            let (role, location, faction) =
                (text(body.role), text(body.location), text(body.faction));
            if role.is_some() || location.is_some() || faction.is_some() {
                CharacterService::new(conn).update(
                    &c.id,
                    UpdateCharacterInput {
                        role: Some(role),
                        location: Some(location),
                        faction: Some(faction),
                        ..UpdateCharacterInput::default()
                    },
                )?;
            }
            notify(&live, conn, &c.id);
            Ok((StatusCode::CREATED, Json(sheet(conn, &c.id)?)))
        })
        .await
}

/// `DELETE /characters/{id}` (DM): with its sheet, notes and link. A
/// device on its link is signed out.
pub async fn delete(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    scope::dm(&caller)?;
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let c = CharacterService::new(conn)
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Character", &id))?;
            CharacterService::new(conn).delete(&id)?;
            live.publish(Change::SignedOut {
                character_id: id.clone(),
            });
            live.publish(Change::Character {
                campaign_id: c.campaign_id,
                character_id: id,
            });
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// `PUT /characters/{id}/campaign` (DM): move it to a campaign, or out of
/// any.
pub async fn set_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    JsonBody(body): JsonBody<wire::CampaignAssignment>,
) -> ApiResult<wire::CharacterSheet> {
    let live = state.live.clone();
    state
        .with_db(move |conn| {
            let before = CharacterService::new(conn)
                .get(&id)?
                .ok_or_else(|| ServiceError::not_found("Character", &id))?;
            let campaign = text(body.campaign_id);
            CharacterService::new(conn).set_campaign(&id, campaign.as_deref())?;
            live.publish(Change::Character {
                campaign_id: before.campaign_id,
                character_id: id.clone(),
            });
            notify(&live, conn, &id);
            sheet(conn, &id)
        })
        .await
        .map(Json)
}

/// `GET /characters/unassigned` (DM): the characters in no campaign.
pub async fn unassigned(State(state): State<AppState>) -> ApiResult<Vec<wire::CharacterSummary>> {
    state
        .with_db(move |conn| {
            let mut s = CharacterService::new(conn);
            let list = s.list_unassigned()?;
            let list = s.enrich_many(list)?;
            Ok(list.into_iter().map(super::convert::character).collect())
        })
        .await
        .map(Json)
}

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
    AddInventoryInput, CatalogEntityService, CharacterService, LevelUpRequest, RaceService,
    ServiceError, UpdateCharacterInput,
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
    }
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
    let spells = chars.list_spells(id)?.into_iter().map(spell).collect();
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
    let speed = race_speed(conn, c.race_name.as_deref(), c.race_source.as_deref());
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
    scope::character(&caller, &id)?;
    state.with_db(move |conn| sheet(conn, &id)).await.map(Json)
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
            Ok((StatusCode::CREATED, Json(spell(s))))
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
            Ok(spell(s))
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

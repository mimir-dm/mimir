//! A campaign's homebrew items, monsters and spells (MIMIR-T-0722), DM
//! only. The data is 5etools-shaped JSON; the indexed fields (an item's
//! type and rarity, a monster's CR, type and size, a spell's level and
//! school) come from it, so the list and the catalog searches agree with
//! what the editor wrote.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use diesel::SqliteConnection;
use mimir_core::services::{
    CharacterService, CreateHomebrewItemInput, CreateHomebrewMonsterInput,
    CreateHomebrewSpellInput, HomebrewService, ModuleService, ServiceError,
    UpdateHomebrewItemInput, UpdateHomebrewMonsterInput, UpdateHomebrewSpellInput,
};
use mimir_render::{detail, summary};
use mimir_wire as wire;
use serde_json::Value;

use crate::error::{ApiError, JsonBody};
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

/// One stored row, whatever its kind.
struct Row {
    id: String,
    campaign_id: String,
    name: String,
    data: String,
    cloned_from_name: Option<String>,
    cloned_from_source: Option<String>,
    updated_at: String,
}

macro_rules! row {
    ($r:expr) => {{
        let r = $r;
        Row {
            id: r.id,
            campaign_id: r.campaign_id,
            name: r.name,
            data: r.data,
            cloned_from_name: r.cloned_from_name,
            cloned_from_source: r.cloned_from_source,
            updated_at: r.updated_at,
        }
    }};
}

fn check_kind(kind: &str) -> Result<(), ApiError> {
    match kind {
        "items" | "monsters" | "spells" => Ok(()),
        other => Err(ServiceError::not_found("Homebrew kind", other).into()),
    }
}

fn singular(kind: &str) -> &str {
    kind.trim_end_matches('s')
}

/// The data as shown: with `name` and `source: "HB"`.
fn data_of(row: &Row) -> Value {
    let mut v: Value =
        serde_json::from_str(&row.data).unwrap_or_else(|_| Value::Object(Default::default()));
    if let Value::Object(m) = &mut v {
        m.insert("name".into(), Value::String(row.name.clone()));
        m.insert("source".into(), Value::String("HB".into()));
    }
    v
}

/// The line under a name: the tooltip without the name.
pub fn detail_line(kind: &str, data: &Value) -> String {
    let t = detail::tooltip(kind, data);
    t.split_once(" — ")
        .map(|(_, r)| r.to_string())
        .unwrap_or_default()
}

fn summary_of(kind: &str, row: Row) -> wire::HomebrewSummary {
    let data = data_of(&row);
    wire::HomebrewSummary {
        detail: detail_line(kind, &data),
        id: row.id,
        name: row.name,
        cloned_from_name: row.cloned_from_name,
        cloned_from_source: row.cloned_from_source,
        updated_at: row.updated_at,
    }
}

fn entry_of(row: Row) -> wire::HomebrewEntry {
    wire::HomebrewEntry {
        data: data_of(&row),
        id: row.id,
        name: row.name,
        cloned_from_name: row.cloned_from_name,
        cloned_from_source: row.cloned_from_source,
        updated_at: row.updated_at,
    }
}

fn list_rows(
    conn: &mut SqliteConnection,
    kind: &str,
    campaign: &str,
) -> Result<Vec<Row>, ApiError> {
    let mut s = HomebrewService::new(conn);
    Ok(match kind {
        "items" => s
            .list_items(campaign)?
            .into_iter()
            .map(|r| row!(r))
            .collect(),
        "monsters" => s
            .list_monsters(campaign)?
            .into_iter()
            .map(|r| row!(r))
            .collect(),
        _ => s
            .list_spells(campaign)?
            .into_iter()
            .map(|r| row!(r))
            .collect(),
    })
}

fn get_row(conn: &mut SqliteConnection, kind: &str, id: &str) -> Result<Row, ApiError> {
    let mut s = HomebrewService::new(conn);
    let found = match kind {
        "items" => s.get_item(id).map(|r| row!(r)),
        "monsters" => s.get_monster(id).map(|r| row!(r)),
        _ => s.get_spell(id).map(|r| row!(r)),
    };
    found.map_err(|_| ServiceError::not_found(format!("Homebrew {}", singular(kind)), id).into())
}

/// The indexed fields of an item: its type code and rarity.
pub fn item_fields(d: &Value) -> (Option<String>, Option<String>) {
    let t = d["type"]
        .as_str()
        .map(|t| t.split('|').next().unwrap_or(t).to_string());
    let t = t.or_else(|| (d["wondrous"].as_bool() == Some(true)).then(|| "W".to_string()));
    (t, d["rarity"].as_str().map(String::from))
}

/// The indexed fields of a monster: CR, creature type, size.
pub fn monster_fields(d: &Value) -> (Option<String>, Option<String>, Option<String>) {
    let cr = Some(summary::cr(&d["cr"])).filter(|c| !c.is_empty());
    let ty = match &d["type"] {
        Value::String(t) => Some(t.clone()),
        Value::Object(o) => o.get("type").and_then(Value::as_str).map(String::from),
        _ => None,
    };
    let size = match &d["size"] {
        Value::Array(l) => l.first().and_then(Value::as_str).map(String::from),
        Value::String(s) => Some(s.clone()),
        _ => None,
    };
    (cr, ty, size)
}

/// The indexed fields of a spell: level and school.
pub fn spell_fields(d: &Value) -> (Option<i32>, Option<String>) {
    (
        d["level"].as_i64().map(|l| l as i32),
        d["school"].as_str().map(String::from),
    )
}

/// The stored JSON: the data without the fields the row holds itself.
fn stored(mut d: Value) -> Result<String, ApiError> {
    let Value::Object(m) = &mut d else {
        return Err(ServiceError::validation("the data must be a JSON object").into());
    };
    m.remove("name");
    m.remove("source");
    Ok(d.to_string())
}

/// A name in use in the campaign, in any case ("Rope" and "rope" would
/// confuse the lookups by name).
fn name_taken(
    conn: &mut SqliteConnection,
    kind: &str,
    campaign: &str,
    name: &str,
) -> Result<bool, ApiError> {
    let lower = name.to_lowercase();
    Ok(list_rows(conn, kind, campaign)?
        .iter()
        .any(|r| r.name.to_lowercase() == lower))
}

fn not_empty(name: &str) -> Result<String, ApiError> {
    let n = name.trim();
    if n.is_empty() {
        return Err(ServiceError::validation("the name is empty").into());
    }
    Ok(n.to_string())
}

/// `GET /campaigns/{id}/homebrew/{kind}`
pub async fn list(
    State(state): State<AppState>,
    Path((campaign, kind)): Path<(String, String)>,
) -> ApiResult<Vec<wire::HomebrewSummary>> {
    check_kind(&kind)?;
    state
        .with_db(move |conn| {
            let mut rows: Vec<_> = list_rows(conn, &kind, &campaign)?
                .into_iter()
                .map(|r| summary_of(&kind, r))
                .collect();
            rows.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            Ok(rows)
        })
        .await
        .map(Json)
}

/// `GET /homebrew/{kind}/{id}`
pub async fn get(
    State(state): State<AppState>,
    Path((kind, id)): Path<(String, String)>,
) -> ApiResult<wire::HomebrewEntry> {
    check_kind(&kind)?;
    state
        .with_db(move |conn| Ok(entry_of(get_row(conn, &kind, &id)?)))
        .await
        .map(Json)
}

/// `POST /campaigns/{id}/homebrew/{kind}`: new data, or a copy of a
/// catalog entry with the data merged over it.
pub async fn create(
    State(state): State<AppState>,
    Path((campaign, kind)): Path<(String, String)>,
    JsonBody(body): JsonBody<wire::NewHomebrew>,
) -> Result<(StatusCode, Json<wire::HomebrewEntry>), ApiError> {
    check_kind(&kind)?;
    let name = not_empty(&body.name)?;
    let clone = match (
        body.cloned_from_name.clone(),
        body.cloned_from_source.clone(),
    ) {
        (Some(n), Some(s)) if !n.trim().is_empty() && !s.trim().is_empty() => Some((n, s)),
        (None, None) => None,
        _ => {
            return Err(ServiceError::validation(
                "give both cloned_from_name and cloned_from_source",
            )
            .into())
        }
    };
    let data = match body.data {
        Some(d) => Some(stored(d)?),
        None if clone.is_some() => None,
        None => Some("{}".to_string()),
    };
    state
        .with_db(move |conn| {
            if name_taken(conn, &kind, &campaign, &name)? {
                return Err(ServiceError::validation(format!(
                    "a homebrew {} named \"{name}\" is already in this campaign",
                    singular(&kind)
                ))
                .into());
            }
            let parsed: Value = data
                .as_deref()
                .and_then(|d| serde_json::from_str(d).ok())
                .unwrap_or(Value::Null);
            let (cfn, cfs) = (
                clone.as_ref().map(|c| c.0.clone()),
                clone.as_ref().map(|c| c.1.clone()),
            );
            let mut s = HomebrewService::new(conn);
            let id = match kind.as_str() {
                "items" => {
                    let (item_type, rarity) = item_fields(&parsed);
                    s.create_item(CreateHomebrewItemInput {
                        campaign_id: campaign.clone(),
                        name,
                        item_type,
                        rarity,
                        data,
                        cloned_from_name: cfn,
                        cloned_from_source: cfs,
                    })?
                    .id
                }
                "monsters" => {
                    let (cr, creature_type, size) = monster_fields(&parsed);
                    s.create_monster(CreateHomebrewMonsterInput {
                        campaign_id: campaign.clone(),
                        name,
                        cr,
                        creature_type,
                        size,
                        data,
                        cloned_from_name: cfn,
                        cloned_from_source: cfs,
                    })?
                    .id
                }
                _ => {
                    let (level, school) = spell_fields(&parsed);
                    s.create_spell(CreateHomebrewSpellInput {
                        campaign_id: campaign.clone(),
                        name,
                        level,
                        school,
                        data,
                        cloned_from_name: cfn,
                        cloned_from_source: cfs,
                    })?
                    .id
                }
            };
            // A copy: the indexed fields from the merged data.
            let row = get_row(conn, &kind, &id)?;
            if clone.is_some() {
                let merged: Value = serde_json::from_str(&row.data).unwrap_or(Value::Null);
                reindex(conn, &kind, &id, &merged)?;
            }
            Ok((
                StatusCode::CREATED,
                Json(entry_of(get_row(conn, &kind, &id)?)),
            ))
        })
        .await
}

/// Set the indexed fields from the data.
fn reindex(conn: &mut SqliteConnection, kind: &str, id: &str, d: &Value) -> Result<(), ApiError> {
    let mut s = HomebrewService::new(conn);
    match kind {
        "items" => {
            let (t, r) = item_fields(d);
            s.update_item(
                id,
                UpdateHomebrewItemInput {
                    item_type: Some(t),
                    rarity: Some(r),
                    ..UpdateHomebrewItemInput::default()
                },
            )?;
        }
        "monsters" => {
            let (cr, ty, size) = monster_fields(d);
            s.update_monster(
                id,
                UpdateHomebrewMonsterInput {
                    cr: Some(cr),
                    creature_type: Some(ty),
                    size: Some(size),
                    ..UpdateHomebrewMonsterInput::default()
                },
            )?;
        }
        _ => {
            let (level, school) = spell_fields(d);
            s.update_spell(
                id,
                UpdateHomebrewSpellInput {
                    level: Some(level),
                    school: Some(school),
                    ..UpdateHomebrewSpellInput::default()
                },
            )?;
        }
    }
    Ok(())
}

/// `PATCH /homebrew/{kind}/{id}`
pub async fn patch(
    State(state): State<AppState>,
    Path((kind, id)): Path<(String, String)>,
    JsonBody(p): JsonBody<wire::HomebrewPatch>,
) -> ApiResult<wire::HomebrewEntry> {
    check_kind(&kind)?;
    let name = p.name.as_deref().map(not_empty).transpose()?;
    let data = p.data.map(stored).transpose()?;
    state
        .with_db(move |conn| {
            let old = get_row(conn, &kind, &id)?;
            if let Some(n) = &name {
                if !n.eq_ignore_ascii_case(&old.name)
                    && name_taken(conn, &kind, &old.campaign_id, n)?
                {
                    return Err(ServiceError::validation(format!(
                        "a homebrew {} named \"{n}\" is already in this campaign",
                        singular(&kind)
                    ))
                    .into());
                }
            }
            let mut s = HomebrewService::new(conn);
            match kind.as_str() {
                "items" => {
                    s.update_item(
                        &id,
                        UpdateHomebrewItemInput {
                            name: name.clone(),
                            data: data.clone(),
                            ..UpdateHomebrewItemInput::default()
                        },
                    )?;
                }
                "monsters" => {
                    s.update_monster(
                        &id,
                        UpdateHomebrewMonsterInput {
                            name: name.clone(),
                            data: data.clone(),
                            ..UpdateHomebrewMonsterInput::default()
                        },
                    )?;
                }
                _ => {
                    s.update_spell(
                        &id,
                        UpdateHomebrewSpellInput {
                            name: name.clone(),
                            data: data.clone(),
                            ..UpdateHomebrewSpellInput::default()
                        },
                    )?;
                }
            }
            if let Some(d) = &data {
                let v: Value = serde_json::from_str(d).unwrap_or(Value::Null);
                reindex(conn, &kind, &id, &v)?;
            }
            Ok(entry_of(get_row(conn, &kind, &id)?))
        })
        .await
        .map(Json)
}

/// What uses an entry: characters holding an item or knowing a spell,
/// modules with a monster (which go with it).
fn usage(conn: &mut SqliteConnection, kind: &str, row: &Row) -> Result<Vec<String>, ApiError> {
    let mut uses = Vec::new();
    match kind {
        "monsters" => {
            for m in ModuleService::new(conn).list_for_campaign(&row.campaign_id)? {
                for mm in ModuleService::new(conn).list_monsters(&m.id)? {
                    if mm.homebrew_monster_id.as_deref() == Some(row.id.as_str()) {
                        uses.push(format!(
                            "Module \"{}\": {} × {}",
                            m.name, mm.quantity, row.name
                        ));
                    }
                }
            }
        }
        _ => {
            for c in CharacterService::new(conn).list_for_campaign(&row.campaign_id)? {
                let held = if kind == "items" {
                    CharacterService::new(conn)
                        .get_inventory(&c.id)?
                        .iter()
                        .any(|i| {
                            i.item_source == "HB" && i.item_name.eq_ignore_ascii_case(&row.name)
                        })
                } else {
                    CharacterService::new(conn)
                        .list_spells(&c.id)?
                        .iter()
                        .any(|sp| {
                            sp.spell_source == "HB" && sp.spell_name.eq_ignore_ascii_case(&row.name)
                        })
                };
                if held {
                    let what = if kind == "items" {
                        "carries it"
                    } else {
                        "knows it"
                    };
                    uses.push(format!("{} {what}", c.name));
                }
            }
        }
    }
    Ok(uses)
}

/// `GET /homebrew/{kind}/{id}/usage`
pub async fn usage_of(
    State(state): State<AppState>,
    Path((kind, id)): Path<(String, String)>,
) -> ApiResult<wire::HomebrewUsage> {
    check_kind(&kind)?;
    state
        .with_db(move |conn| {
            let row = get_row(conn, &kind, &id)?;
            Ok(wire::HomebrewUsage {
                uses: usage(conn, &kind, &row)?,
            })
        })
        .await
        .map(Json)
}

/// `DELETE /homebrew/{kind}/{id}`: a monster leaves the modules that use
/// it; an item or a spell stays on the sheets (by name).
pub async fn delete(
    State(state): State<AppState>,
    Path((kind, id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    check_kind(&kind)?;
    state
        .with_db(move |conn| {
            let mut s = HomebrewService::new(conn);
            match kind.as_str() {
                "items" => s.delete_item(&id)?,
                "monsters" => s.delete_monster(&id)?,
                _ => s.delete_spell(&id)?,
            }
            Ok(StatusCode::NO_CONTENT)
        })
        .await
}

/// The campaign's homebrew of a kind as catalog choices ("HB"), for the
/// item and monster searches.
pub fn choices(
    conn: &mut SqliteConnection,
    kind: &str,
    campaign: &str,
    needle: &str,
) -> Result<Vec<wire::Choice>, ApiError> {
    let lower = needle.to_lowercase();
    Ok(list_rows(conn, kind, campaign)?
        .into_iter()
        .filter(|r| r.name.to_lowercase().contains(&lower))
        .map(|r| {
            let data = data_of(&r);
            let line = detail_line(kind, &data);
            wire::Choice {
                name: r.name,
                source: "HB".into(),
                detail: Some(if line.is_empty() {
                    "Homebrew".into()
                } else {
                    format!("Homebrew · {line}")
                }),
                level: None,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn indexed_fields_from_the_data() {
        assert_eq!(
            item_fields(&json!({"type": "M|XPHB", "rarity": "rare"})),
            (Some("M".into()), Some("rare".into()))
        );
        assert_eq!(item_fields(&json!({"wondrous": true})).0, Some("W".into()));
        assert_eq!(
            monster_fields(&json!({"cr": {"cr": "1/2"}, "type": {"type": "fiend"}, "size": ["L"]})),
            (Some("1/2".into()), Some("fiend".into()), Some("L".into()))
        );
        assert_eq!(
            spell_fields(&json!({"level": 3, "school": "V"})),
            (Some(3), Some("V".into()))
        );
        assert!(stored(json!([1])).is_err());
        assert_eq!(
            stored(json!({"name": "x", "source": "HB", "a": 1})).unwrap(),
            r#"{"a":1}"#
        );
    }
}

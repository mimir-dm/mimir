//! The catalog choices the character screens offer (MIMIR-T-0717): classes
//! and their level-up facts, subclasses, feats, optional features, spells
//! of a class, items. Reference data: player routes, read-only. The full
//! catalog reader is MIMIR-T-0723.

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use mimir_core::models::catalog::{ItemFilter, MonsterFilter};
use mimir_core::services::catalog::find_class_level_info;
use mimir_core::services::{
    creation_skills, BackgroundService, CatalogEntityService, ClassService, FeatService,
    ItemService, MonsterService, OptionalFeatureService, RaceService, ServiceError, SourceService,
    SpellService, SubclassService,
};
use mimir_wire as wire;
use serde::Deserialize;
use serde_json::Value;

use crate::error::ApiError;
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

fn choice(name: String, source: String, detail: Option<String>) -> wire::Choice {
    wire::Choice {
        name,
        source,
        detail,
        level: None,
    }
}

/// `GET /catalog/classes`
pub async fn classes(State(state): State<AppState>) -> ApiResult<Vec<wire::Choice>> {
    state
        .with_db(|conn| {
            let mut list: Vec<wire::Choice> = ClassService::new(conn)
                .list_all()?
                .into_iter()
                .map(|c| choice(c.name, c.source, None))
                .collect();
            list.sort_by(|a, b| a.name.cmp(&b.name).then(a.source.cmp(&b.source)));
            Ok(list)
        })
        .await
        .map(Json)
}

/// `GET /catalog/classes/{name}/{source}/level-info`
pub async fn class_level_info(
    State(state): State<AppState>,
    Path((name, source)): Path<(String, String)>,
) -> ApiResult<wire::ClassLevelInfo> {
    state
        .with_db(move |conn| {
            let i = find_class_level_info(conn, &name, &source)?
                .ok_or_else(|| ServiceError::not_found("Class", format!("{name} ({source})")))?;
            Ok(wire::ClassLevelInfo {
                name: i.name,
                source: i.source,
                hit_die: i.hit_die,
                subclass_level: i.subclass_level,
                asi_levels: i.asi_levels,
                multiclass_requirements: i.multiclass_requirements,
                caster: i.caster,
                spellcasting_ability: i.spellcasting_ability,
                cantrips_known: i.cantrips_known,
                spells_known: i.spells_known,
                spells_added: i.spells_added,
                optional_features: i.optional_features,
            })
        })
        .await
        .map(Json)
}

/// `GET /catalog/classes/{name}/subclasses`
pub async fn subclasses(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Vec<wire::Choice>> {
    state
        .with_db(move |conn| {
            Ok(SubclassService::new(conn)
                .list_by_class(&name)?
                .into_iter()
                .map(|s| choice(s.name, s.source, None))
                .collect())
        })
        .await
        .map(Json)
}

/// A short line for a feat's prerequisites ("STR 13; spellcasting").
pub fn prerequisite_line(data: &Value) -> Option<String> {
    let list = data.get("prerequisite")?.as_array()?;
    let mut parts: Vec<String> = Vec::new();
    for p in list {
        let Some(o) = p.as_object() else { continue };
        for (k, v) in o {
            match k.as_str() {
                "ability" => {
                    for a in v.as_array().into_iter().flatten() {
                        for (ab, n) in a.as_object().into_iter().flatten() {
                            parts.push(format!("{} {}", ab.to_uppercase(), n));
                        }
                    }
                }
                "level" => parts.push(format!(
                    "level {}",
                    v.as_i64()
                        .or_else(|| v.get("level").and_then(Value::as_i64))
                        .unwrap_or(0)
                )),
                "race" => {
                    let names: Vec<String> = v
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|r| r.get("name").and_then(Value::as_str).map(String::from))
                        .collect();
                    parts.push(names.join(" or "));
                }
                other => parts.push(other.replace('_', " ")),
            }
        }
    }
    (!parts.is_empty()).then(|| parts.join("; "))
}

/// `GET /catalog/feats`
pub async fn feats(State(state): State<AppState>) -> ApiResult<Vec<wire::Choice>> {
    state
        .with_db(|conn| {
            let mut list: Vec<wire::Choice> = FeatService::new(conn)
                .list_all()?
                .into_iter()
                .map(|f| {
                    let data: Value = serde_json::from_str(&f.data).unwrap_or(Value::Null);
                    choice(f.name, f.source, prerequisite_line(&data))
                })
                .collect();
            list.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(list)
        })
        .await
        .map(Json)
}

#[derive(Debug, Deserialize)]
pub struct FeatureQuery {
    /// The feature type: "FS" (fighting styles), "MM" (metamagic), "MV"
    /// (maneuvers), "EI" (invocations). Matches the type or its subtypes
    /// ("FS:F").
    #[serde(rename = "type")]
    pub kind: String,
}

/// `GET /catalog/optional-features?type=EI`
pub async fn optional_features(
    State(state): State<AppState>,
    Query(q): Query<FeatureQuery>,
) -> ApiResult<Vec<wire::Choice>> {
    state
        .with_db(move |conn| {
            let mut list: Vec<wire::Choice> = OptionalFeatureService::new(conn)
                .list_all()?
                .into_iter()
                .filter(|f| {
                    f.feature_type.as_deref().is_some_and(|t| {
                        t.split(',').any(|t| {
                            t.trim() == q.kind || t.trim().starts_with(&format!("{}:", q.kind))
                        })
                    })
                })
                .map(|f| choice(f.name, f.source, f.feature_type))
                .collect();
            list.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(list)
        })
        .await
        .map(Json)
}

#[derive(Debug, Deserialize)]
pub struct SpellQuery {
    pub class: String,
    #[serde(default)]
    pub max_level: Option<i32>,
}

/// `GET /catalog/spells?class=Wizard&max_level=3`
pub async fn spells(
    State(state): State<AppState>,
    Query(q): Query<SpellQuery>,
) -> ApiResult<Vec<wire::Choice>> {
    state
        .with_db(move |conn| {
            let mut list: Vec<wire::Choice> = SpellService::new(conn)
                .list_by_class(&q.class)?
                .into_iter()
                .filter(|s| q.max_level.is_none_or(|m| s.level <= m))
                .map(|s| wire::Choice {
                    detail: Some(match (s.level, s.school.as_deref()) {
                        (0, Some(sc)) => format!("cantrip · {sc}"),
                        (0, None) => "cantrip".into(),
                        (l, Some(sc)) => format!("level {l} · {sc}"),
                        (l, None) => format!("level {l}"),
                    }),
                    level: Some(s.level),
                    name: s.name,
                    source: s.source,
                })
                .collect();
            list.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            list.dedup_by(|a, b| a.name == b.name && a.source == b.source);
            Ok(list)
        })
        .await
        .map(Json)
}

#[derive(Debug, Deserialize)]
pub struct ItemQuery {
    pub search: String,
    /// The campaign whose homebrew joins the results (source "HB",
    /// MIMIR-T-0722).
    #[serde(default)]
    pub campaign: Option<String>,
}

/// `GET /catalog/items?search=rope`: up to 50 by name.
pub async fn items(
    State(state): State<AppState>,
    Extension(caller): Extension<crate::auth::Caller>,
    Query(q): Query<ItemQuery>,
) -> ApiResult<Vec<wire::Choice>> {
    // A player sees only their own campaign's homebrew.
    if let Some(c) = q.campaign.as_deref().filter(|c| !c.is_empty()) {
        super::scope::campaign(&caller, c)?;
    }
    let needle = q.search.trim().to_string();
    if needle.chars().count() < 2 {
        return Ok(Json(Vec::new()));
    }
    state
        .with_db(move |conn| {
            let filter = ItemFilter {
                name_contains: Some(needle.clone()),
                ..ItemFilter::default()
            };
            let mut list: Vec<wire::Choice> = ItemService::new(conn)
                .search_paginated(&filter, 50, 0)?
                .into_iter()
                .map(|i| {
                    let detail = [
                        i.item_type.clone(),
                        i.rarity.clone().filter(|r| r != "none"),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" · ");
                    choice(i.name, i.source, (!detail.is_empty()).then_some(detail))
                })
                .collect();
            if let Some(c) = q.campaign.as_deref().filter(|c| !c.is_empty()) {
                list.extend(super::homebrew::choices(conn, "items", c, &needle)?);
            }
            let lower = needle.to_lowercase();
            list.sort_by_key(|c| {
                let n = c.name.to_lowercase();
                (n != lower, !n.starts_with(&lower), n)
            });
            Ok(list)
        })
        .await
        .map(Json)
}

/// `GET /catalog/races` (DM, MIMIR-T-0721): every race, by name.
pub async fn races(State(state): State<AppState>) -> ApiResult<Vec<wire::Choice>> {
    state
        .with_db(|conn| {
            let mut list: Vec<wire::Choice> = RaceService::new(conn)
                .list_all()?
                .into_iter()
                .map(|r| choice(r.name, r.source, None))
                .collect();
            list.sort_by(|a, b| a.name.cmp(&b.name).then(a.source.cmp(&b.source)));
            Ok(list)
        })
        .await
        .map(Json)
}

/// `GET /catalog/backgrounds` (DM, MIMIR-T-0721): every background.
pub async fn backgrounds(State(state): State<AppState>) -> ApiResult<Vec<wire::Choice>> {
    state
        .with_db(|conn| {
            let mut list: Vec<wire::Choice> = BackgroundService::new(conn)
                .list_all()?
                .into_iter()
                .map(|b| choice(b.name, b.source, None))
                .collect();
            list.sort_by(|a, b| a.name.cmp(&b.name).then(a.source.cmp(&b.source)));
            Ok(list)
        })
        .await
        .map(Json)
}

#[derive(Debug, Default, Deserialize)]
pub struct CreationQuery {
    pub class: Option<String>,
    pub class_source: Option<String>,
    pub background: Option<String>,
    pub background_source: Option<String>,
}

/// `GET /catalog/creation-skills?class=&class_source=&background=&background_source=`
/// (DM, MIMIR-T-0721): the background's skills and the class's pick.
pub async fn creation_skills_of(
    State(state): State<AppState>,
    Query(q): Query<CreationQuery>,
) -> ApiResult<wire::CreationSkills> {
    state
        .with_db(move |conn| {
            let class = q.class.as_deref().zip(q.class_source.as_deref());
            let bg = q.background.as_deref().zip(q.background_source.as_deref());
            let s = creation_skills(conn, class, bg)?;
            Ok(wire::CreationSkills {
                background: s.background,
                choose: s.choose,
                from: s.from,
            })
        })
        .await
        .map(Json)
}

/// `GET /catalog/monsters?search=gob` (DM, MIMIR-T-0720): up to 50 by
/// name, the closest first. The detail is the challenge and the type.
/// With `campaign`, the campaign's homebrew joins, and the catalog keeps to
/// the campaign's source books when it has any (MIMIR-T-0732).
pub async fn monsters(
    State(state): State<AppState>,
    Query(q): Query<ItemQuery>,
) -> ApiResult<Vec<wire::Choice>> {
    let needle = q.search.trim().to_string();
    if needle.chars().count() < 2 {
        return Ok(Json(Vec::new()));
    }
    state
        .with_db(move |conn| {
            let books = match q.campaign.as_deref().filter(|c| !c.is_empty()) {
                Some(c) => SourceService::new(conn).list_campaign_sources(c)?,
                None => Vec::new(),
            };
            let filter = MonsterFilter {
                name_contains: Some(needle.clone()),
                sources: (!books.is_empty()).then_some(books),
                ..MonsterFilter::default()
            };
            let mut list: Vec<wire::Choice> = MonsterService::new(conn)
                .search_paginated(&filter, 50, 0)?
                .into_iter()
                .map(|m| {
                    let detail = [
                        m.cr.clone().map(|c| format!("CR {c}")),
                        m.creature_type.clone(),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" · ");
                    choice(m.name, m.source, (!detail.is_empty()).then_some(detail))
                })
                .collect();
            if let Some(c) = q.campaign.as_deref().filter(|c| !c.is_empty()) {
                list.extend(super::homebrew::choices(conn, "monsters", c, &needle)?);
            }
            let lower = needle.to_lowercase();
            list.sort_by_key(|c| {
                let n = c.name.to_lowercase();
                (n != lower, !n.starts_with(&lower), n)
            });
            Ok(list)
        })
        .await
        .map(Json)
}

#[cfg(test)]
mod tests {
    use super::prerequisite_line;
    use serde_json::json;

    #[test]
    fn prerequisite_lines() {
        assert_eq!(prerequisite_line(&json!({})), None);
        assert_eq!(
            prerequisite_line(&json!({"prerequisite": [{"ability": [{"str": 13}]}]})).as_deref(),
            Some("STR 13")
        );
        assert_eq!(
            prerequisite_line(&json!({"prerequisite": [{"spellcasting": true}]})).as_deref(),
            Some("spellcasting")
        );
        assert_eq!(
            prerequisite_line(
                &json!({"prerequisite": [{"race": [{"name": "elf"}, {"name": "half-elf"}]}]})
            )
            .as_deref(),
            Some("elf or half-elf")
        );
    }
}

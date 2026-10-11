//! The catalog choices the character screens offer (MIMIR-T-0717): classes
//! and their level-up facts, subclasses, feats, optional features, spells
//! of a class, items. Reference data: player routes, read-only. The full
//! catalog reader is MIMIR-T-0723.

use axum::extract::{Path, Query, State};
use axum::Json;
use mimir_core::models::catalog::ItemFilter;
use mimir_core::services::catalog::find_class_level_info;
use mimir_core::services::{
    CatalogEntityService, ClassService, FeatService, ItemService, OptionalFeatureService,
    ServiceError, SpellService, SubclassService,
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
}

/// `GET /catalog/items?search=rope`: up to 50 by name.
pub async fn items(
    State(state): State<AppState>,
    Query(q): Query<ItemQuery>,
) -> ApiResult<Vec<wire::Choice>> {
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

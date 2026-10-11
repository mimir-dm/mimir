//! The text a character sheet names (MIMIR-T-0733): its class and subclass
//! features with their entries, and the catalog entries of its items,
//! background, spells and feats. Whoever may read the sheet may read these
//! (a player: their own, or a shared one), so a player sees the details of
//! their own equipment without the DM's catalog.

use axum::extract::{Path, State};
use axum::{Extension, Json};
use diesel::SqliteConnection;
use mimir_core::dal;
use mimir_core::services::{CharacterService, HomebrewService, ServiceError};
use mimir_wire as wire;
use serde_json::Value;

use super::reference::entry_json;
use super::scope;
use crate::auth::Caller;
use crate::error::ApiError;
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

/// The catalog categories a sheet names.
pub const SHEET_CATEGORIES: &[&str] = &[
    "items",
    "backgrounds",
    "spells",
    "feats",
    "races",
    "classes",
    "optional-features",
    "conditions",
];

fn entries_of(data: &str) -> Value {
    serde_json::from_str::<Value>(data)
        .ok()
        .and_then(|v| v.get("entries").cloned())
        .unwrap_or(Value::Array(Vec::new()))
}

/// The features of each class of the character up to its level, then its
/// subclass's, each in level order.
pub fn features_of(
    conn: &mut SqliteConnection,
    character_id: &str,
) -> Result<Vec<wire::FeatureText>, ApiError> {
    let classes = dal::list_character_classes(conn, character_id).map_err(ServiceError::from)?;
    let mut out = Vec::new();
    for k in classes {
        for f in dal::list_class_features_by_class(conn, &k.class_name, &k.class_source)
            .map_err(ServiceError::from)?
            .into_iter()
            .filter(|f| f.level <= k.level)
        {
            out.push(wire::FeatureText {
                entries: entries_of(&f.data),
                name: f.name,
                class_name: k.class_name.clone(),
                subclass_name: None,
                level: f.level,
            });
        }
        if let (Some(sub), Some(sub_source)) = (&k.subclass_name, &k.subclass_source) {
            for f in dal::list_subclass_features_by_subclass(conn, sub, sub_source)
                .map_err(ServiceError::from)?
                .into_iter()
                .filter(|f| f.level <= k.level)
            {
                out.push(wire::FeatureText {
                    entries: entries_of(&f.data),
                    name: f.name,
                    class_name: k.class_name.clone(),
                    subclass_name: Some(sub.clone()),
                    level: f.level,
                });
            }
        }
    }
    Ok(out)
}

/// `GET /characters/{id}/features`
pub async fn features(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::FeatureText>> {
    state
        .with_db(move |conn| {
            scope::read_character(&caller, conn, &id)?;
            features_of(conn, &id)
        })
        .await
        .map(Json)
}

/// The level of a spell: the catalog's, or the campaign's homebrew one.
pub fn spell_level(
    conn: &mut SqliteConnection,
    campaign: Option<&str>,
    name: &str,
    source: &str,
) -> Option<i32> {
    if source == "HB" {
        let c = campaign?;
        return HomebrewService::new(conn)
            .get_spell_by_name(c, name)
            .ok()
            .flatten()
            .and_then(|s| s.level);
    }
    dal::get_spell_by_name(conn, name, source)
        .ok()
        .flatten()
        .map(|s| s.level)
}

/// `GET /characters/{id}/entry/{category}/{source}/{name}`: a catalog
/// entry the sheet names, as `/catalog/entry` gives it; a homebrew item
/// or spell ("HB") from the character's campaign.
pub async fn entry(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path((id, category, source, name)): Path<(String, String, String, String)>,
) -> ApiResult<Value> {
    if !SHEET_CATEGORIES.contains(&category.as_str()) {
        return Err(ServiceError::not_found("Catalog category", category).into());
    }
    state
        .with_db(move |conn| {
            scope::read_character(&caller, conn, &id)?;
            if source == "HB" {
                let campaign = CharacterService::new(conn)
                    .get(&id)?
                    .and_then(|c| c.campaign_id)
                    .ok_or_else(|| ServiceError::not_found(category.as_str(), &name))?;
                let data = match category.as_str() {
                    "items" => HomebrewService::new(conn)
                        .get_item_by_name(&campaign, &name)?
                        .map(|i| i.data),
                    "spells" => HomebrewService::new(conn)
                        .get_spell_by_name(&campaign, &name)?
                        .map(|s| s.data),
                    _ => None,
                }
                .ok_or_else(|| ServiceError::not_found(category.as_str(), &name))?;
                let mut v: Value = serde_json::from_str(&data).unwrap_or(Value::Null);
                if let Value::Object(m) = &mut v {
                    m.insert("name".into(), Value::String(name.clone()));
                    m.insert("source".into(), Value::String("HB".into()));
                }
                return Ok(v);
            }
            entry_json(conn, &category, &name, &source)?.ok_or_else(|| {
                ServiceError::not_found(category.as_str(), format!("{name} ({source})")).into()
            })
        })
        .await
        .map(Json)
}

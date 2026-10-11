//! The catalog and the book reader (MIMIR-T-0723), DM only (books hold
//! adventures). Search a category (its table columns, a page at a time,
//! the campaign's sources unless others are asked for), one entry as its
//! 5etools JSON (the web app renders it), the sources and their books,
//! and the books' images.

use std::path::{Component, Path as FsPath};

use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use diesel::SqliteConnection;
use mimir_core::dal::catalog as catalog_dal;
use mimir_core::models::catalog::*;
use mimir_core::services::*;
use mimir_render::summary;
use mimir_wire as wire;
use serde::Deserialize;
use serde_json::Value;

use crate::error::ApiError;
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

/// The parts of a catalog entity: name, source, JSON, fluff JSON.
trait Parts {
    fn parts(self) -> (String, String, String, Option<String>);
}

macro_rules! parts {
    ($($t:ty),*; fluff $($f:ty),*) => {
        $(impl Parts for $t {
            fn parts(self) -> (String, String, String, Option<String>) {
                (self.name, self.source, self.data, None)
            }
        })*
        $(impl Parts for $f {
            fn parts(self) -> (String, String, String, Option<String>) {
                (self.name, self.source, self.data, self.fluff)
            }
        })*
    };
}

parts!(Action, Cult, Deity, OptionalFeature, Psionic, Reward, CatalogTable, VariantRule;
    fluff Background, Class, Condition, Feat, Hazard, Item, Language, Monster, Object, Race, Spell, Trap, Vehicle);

/// `GET /catalog/search/{category}` query. `sources`: codes with commas
/// (none: the campaign's, else all). The other fields filter the
/// categories they belong to.
#[derive(Debug, Default, Deserialize)]
pub struct SearchQuery {
    pub search: Option<String>,
    pub sources: Option<String>,
    pub campaign: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub level: Option<i32>,
    pub school: Option<String>,
    pub ritual: Option<bool>,
    pub concentration: Option<bool>,
    pub cr: Option<String>,
    pub creature_type: Option<String>,
    pub size: Option<String>,
    pub item_type: Option<String>,
    pub rarity: Option<String>,
    pub pantheon: Option<String>,
    pub language_type: Option<String>,
    pub object_type: Option<String>,
    pub feature_type: Option<String>,
    pub psionic_type: Option<String>,
    pub reward_type: Option<String>,
    pub rule_type: Option<String>,
    pub vehicle_type: Option<String>,
}

fn opt(v: &Option<String>) -> Option<String> {
    v.as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Search a category: (name, source, JSON).
macro_rules! find {
    ($conn:expr, $svc:ident, $filter:ident, $q:expr, $sources:expr, $limit:expr, $offset:expr $(, $field:ident = $val:expr)*) => {{
        let mut f = $filter::default();
        f.name_contains = opt(&$q.search);
        f.sources = $sources.clone();
        $( f.$field = $val; )*
        $svc::new($conn)
            .search_paginated(&f, $limit, $offset)?
            .into_iter()
            .map(|e| { let (n, s, d, _) = e.parts(); (n, s, d) })
            .collect::<Vec<_>>()
    }};
}

fn search_rows(
    conn: &mut SqliteConnection,
    category: &str,
    q: &SearchQuery,
    sources: Option<Vec<String>>,
    limit: i64,
    offset: i64,
) -> Result<Vec<(String, String, String)>, ApiError> {
    let (l, o) = (limit, offset);
    Ok(match category {
        "spells" => find!(
            conn,
            SpellService,
            SpellFilter,
            q,
            sources,
            l,
            o,
            level = q.level,
            school = opt(&q.school),
            ritual = q.ritual,
            concentration = q.concentration
        ),
        "items" => find!(
            conn,
            ItemService,
            ItemFilter,
            q,
            sources,
            l,
            o,
            item_type = opt(&q.item_type),
            rarity = opt(&q.rarity)
        ),
        "monsters" => find!(
            conn,
            MonsterService,
            MonsterFilter,
            q,
            sources,
            l,
            o,
            cr = opt(&q.cr),
            creature_type = opt(&q.creature_type),
            size = opt(&q.size)
        ),
        "classes" => find!(conn, ClassService, ClassFilter, q, sources, l, o),
        "races" => find!(conn, RaceService, RaceFilter, q, sources, l, o),
        "backgrounds" => find!(conn, BackgroundService, BackgroundFilter, q, sources, l, o),
        "feats" => find!(conn, FeatService, FeatFilter, q, sources, l, o),
        "optional-features" => find!(
            conn,
            OptionalFeatureService,
            OptionalFeatureFilter,
            q,
            sources,
            l,
            o,
            feature_type = opt(&q.feature_type)
        ),
        "actions" => find!(conn, ActionService, ActionFilter, q, sources, l, o),
        "conditions" => find!(conn, ConditionService, ConditionFilter, q, sources, l, o),
        "deities" => find!(
            conn,
            DeityService,
            DeityFilter,
            q,
            sources,
            l,
            o,
            pantheon = opt(&q.pantheon)
        ),
        "objects" => find!(
            conn,
            ObjectService,
            ObjectFilter,
            q,
            sources,
            l,
            o,
            object_type = opt(&q.object_type)
        ),
        "traps" => find!(conn, TrapService, TrapFilter, q, sources, l, o),
        "hazards" => find!(conn, HazardService, HazardFilter, q, sources, l, o),
        "languages" => find!(
            conn,
            LanguageService,
            LanguageFilter,
            q,
            sources,
            l,
            o,
            language_type = opt(&q.language_type)
        ),
        "rewards" => find!(
            conn,
            RewardService,
            RewardFilter,
            q,
            sources,
            l,
            o,
            reward_type = opt(&q.reward_type)
        ),
        "tables" => find!(
            conn,
            CatalogTableService,
            CatalogTableFilter,
            q,
            sources,
            l,
            o
        ),
        "variant-rules" => find!(
            conn,
            VariantRuleService,
            VariantRuleFilter,
            q,
            sources,
            l,
            o,
            rule_type = opt(&q.rule_type)
        ),
        "vehicles" => find!(
            conn,
            VehicleService,
            VehicleFilter,
            q,
            sources,
            l,
            o,
            vehicle_type = opt(&q.vehicle_type)
        ),
        "cults" => find!(conn, CultService, CultFilter, q, sources, l, o),
        "psionics" => find!(
            conn,
            PsionicService,
            PsionicFilter,
            q,
            sources,
            l,
            o,
            psionic_type = opt(&q.psionic_type)
        ),
        other => return Err(ServiceError::not_found("Catalog category", other).into()),
    })
}

/// `GET /catalog/search/{category}`: up to `limit` (default 100, at most
/// 500) rows from `offset`, closest names first within the page order.
pub async fn search(
    State(state): State<AppState>,
    Path(category): Path<String>,
    Query(q): Query<SearchQuery>,
) -> ApiResult<wire::CatalogPage> {
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let offset = q.offset.unwrap_or(0).max(0);
    state
        .with_db(move |conn| {
            let explicit = opt(&q.sources).map(|s| {
                s.split(',')
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty())
                    .collect::<Vec<_>>()
            });
            let sources = CatalogSearch::new(conn)
                .effective_sources(explicit, opt(&q.campaign).as_deref())?;
            let mut rows = search_rows(conn, &category, &q, sources, limit + 1, offset)?;
            let more = rows.len() as i64 > limit;
            rows.truncate(limit as usize);
            Ok(wire::CatalogPage {
                headers: summary::headers(&category)
                    .iter()
                    .map(|h| h.to_string())
                    .collect(),
                rows: rows
                    .into_iter()
                    .map(|(name, source, data)| {
                        let v: Value = serde_json::from_str(&data).unwrap_or(Value::Null);
                        wire::CatalogRow {
                            cells: summary::cells(&category, &v),
                            name,
                            source,
                        }
                    })
                    .collect(),
                more,
            })
        })
        .await
        .map(Json)
}

/// One entry: its JSON, with `name`, `source` and `fluff`.
macro_rules! get_one {
    ($conn:expr, $svc:ident, $name:expr, $source:expr) => {
        $svc::new($conn)
            .get_by_name_and_source($name, $source)?
            .map(Parts::parts)
    };
}

pub fn entry_json(
    conn: &mut SqliteConnection,
    category: &str,
    name: &str,
    source: &str,
) -> Result<Option<Value>, ApiError> {
    let found = match category {
        "spells" => get_one!(conn, SpellService, name, source),
        "items" => get_one!(conn, ItemService, name, source),
        "monsters" => get_one!(conn, MonsterService, name, source),
        "classes" => get_one!(conn, ClassService, name, source),
        "races" => get_one!(conn, RaceService, name, source),
        "backgrounds" => get_one!(conn, BackgroundService, name, source),
        "feats" => get_one!(conn, FeatService, name, source),
        "optional-features" => get_one!(conn, OptionalFeatureService, name, source),
        "actions" => get_one!(conn, ActionService, name, source),
        "conditions" => get_one!(conn, ConditionService, name, source),
        "deities" => get_one!(conn, DeityService, name, source),
        "objects" => get_one!(conn, ObjectService, name, source),
        "traps" => get_one!(conn, TrapService, name, source),
        "hazards" => get_one!(conn, HazardService, name, source),
        "languages" => get_one!(conn, LanguageService, name, source),
        "rewards" => get_one!(conn, RewardService, name, source),
        "tables" => get_one!(conn, CatalogTableService, name, source),
        "variant-rules" => get_one!(conn, VariantRuleService, name, source),
        "vehicles" => get_one!(conn, VehicleService, name, source),
        "cults" => get_one!(conn, CultService, name, source),
        "psionics" => get_one!(conn, PsionicService, name, source),
        other => return Err(ServiceError::not_found("Catalog category", other).into()),
    };
    Ok(found.map(|(name, source, data, fluff)| {
        let mut v: Value =
            serde_json::from_str(&data).unwrap_or_else(|_| Value::Object(Default::default()));
        if let Value::Object(m) = &mut v {
            m.insert("name".into(), Value::String(name));
            m.insert("source".into(), Value::String(source));
            if let Some(f) = fluff.and_then(|f| serde_json::from_str::<Value>(&f).ok()) {
                m.insert("fluff".into(), f);
            }
        }
        v
    }))
}

/// `GET /catalog/entry/{category}/{source}/{name}`: the entry's JSON. A
/// name with no entry in that source falls back to the first with the
/// name (a cross-reference's default source can be wrong).
pub async fn entry(
    State(state): State<AppState>,
    Path((category, source, name)): Path<(String, String, String)>,
) -> ApiResult<Value> {
    state
        .with_db(move |conn| {
            entry_named(conn, &category, &name, &source)?.ok_or_else(|| {
                ServiceError::not_found(category.as_str(), format!("{name} ({source})")).into()
            })
        })
        .await
        .map(Json)
}

/// The entry of that name and source; else the first entry with the name,
/// in any source (a tag's default source can be wrong); else none.
pub fn entry_named(
    conn: &mut SqliteConnection,
    category: &str,
    name: &str,
    source: &str,
) -> Result<Option<Value>, ApiError> {
    if let Some(v) = entry_json(conn, category, name, source)? {
        return Ok(Some(v));
    }
    let q = SearchQuery {
        search: Some(name.to_string()),
        ..SearchQuery::default()
    };
    let rows = search_rows(conn, category, &q, None, 50, 0)?;
    match rows
        .into_iter()
        .find(|(n, _, _)| n.eq_ignore_ascii_case(name))
    {
        Some((n, s, _)) => entry_json(conn, category, &n, &s),
        None => Ok(None),
    }
}

/// `GET /catalog/sources`: the source books, and which have a book to
/// read.
pub async fn sources(State(state): State<AppState>) -> ApiResult<Vec<wire::SourceInfo>> {
    state
        .with_db(|conn| {
            let books: Vec<String> = catalog_dal::list_books(conn)
                .map_err(ServiceError::from)?
                .into_iter()
                .map(|b| b.source)
                .collect();
            let mut list: Vec<wire::SourceInfo> = catalog_dal::list_sources(conn)
                .map_err(ServiceError::from)?
                .into_iter()
                .map(|s| wire::SourceInfo {
                    book: books.contains(&s.code),
                    code: s.code,
                    name: s.name,
                })
                .collect();
            list.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(list)
        })
        .await
        .map(Json)
}

/// `GET /books/{source}`
pub async fn book(
    State(state): State<AppState>,
    Path(source): Path<String>,
) -> ApiResult<wire::BookData> {
    state
        .with_db(move |conn| {
            let b = catalog_dal::get_book_by_source(conn, &source)
                .map_err(ServiceError::from)?
                .ok_or_else(|| ServiceError::not_found("Book", &source))?;
            Ok(wire::BookData {
                data: serde_json::from_str(&b.data).unwrap_or(Value::Array(Vec::new())),
                source: b.source,
                name: b.name,
            })
        })
        .await
        .map(Json)
}

/// `GET /books`: the books and adventures the reader can open, books
/// first, each by name (MIMIR-T-0736).
pub async fn books(State(state): State<AppState>) -> ApiResult<Vec<wire::BookInfo>> {
    state
        .with_db(|conn| {
            let mut list: Vec<wire::BookInfo> = catalog_dal::list_books(conn)
                .map_err(ServiceError::from)?
                .into_iter()
                .map(|b| {
                    let meta: Value = b
                        .meta
                        .as_deref()
                        .and_then(|m| serde_json::from_str(m).ok())
                        .unwrap_or(Value::Null);
                    wire::BookInfo {
                        level_start: meta.pointer("/level/start").and_then(Value::as_i64),
                        level_end: meta.pointer("/level/end").and_then(Value::as_i64),
                        storyline: meta["storyline"].as_str().map(str::to_string),
                        source: b.source,
                        name: b.name,
                        kind: b.kind,
                    }
                })
                .collect();
            list.sort_by(|a, b| {
                (a.kind != "book")
                    .cmp(&(b.kind != "book"))
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
            Ok(list)
        })
        .await
        .map(Json)
}

/// `GET /books/{source}/plan`: the adventure as a campaign, before it is
/// made (MIMIR-T-0739).
pub async fn adventure_plan(
    State(state): State<AppState>,
    Path(source): Path<String>,
) -> ApiResult<wire::AdventurePlan> {
    state
        .with_db(move |conn| {
            let a = crate::adventure::adventure_text(conn, &source)?;
            Ok(crate::adventure::plan_view(&a))
        })
        .await
        .map(Json)
}

/// `POST /books/{source}/campaign`: make the campaign (one transaction).
pub async fn adventure_campaign(
    State(state): State<AppState>,
    Path(source): Path<String>,
    Json(req): Json<wire::AdventureCampaignRequest>,
) -> Result<(axum::http::StatusCode, Json<wire::AdventureCampaign>), ApiError> {
    let data_dir = state.config.data_dir.clone();
    let made = state
        .with_db(move |conn| {
            let a = crate::adventure::adventure_text(conn, &source)?;
            crate::adventure::create_campaign(conn, &data_dir, &a, &req)
        })
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(made)))
}

/// A book image path under the catalog assets, or none when it would
/// leave them (`..`, a root, a drive).
pub fn safe_image_path(path: &str) -> Option<&FsPath> {
    let p = FsPath::new(path);
    let ok = !path.is_empty() && p.components().all(|c| matches!(c, Component::Normal(_)));
    ok.then_some(p)
}

/// `GET /book-images/{*path}`: an image of a book (or an entry's fluff).
pub async fn book_image(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response, ApiError> {
    let rel = path.trim_start_matches("img/").to_string();
    let Some(rel_path) = safe_image_path(&rel) else {
        return Err(ServiceError::validation("bad image path").into());
    };
    let full = state.config.assets_dir().join("catalog").join(rel_path);
    let bytes = tokio::fs::read(&full)
        .await
        .map_err(|_| ApiError::from(ServiceError::not_found("Image", &path)))?;
    let mime = mime_guess::from_path(&full).first_or_octet_stream();
    Ok((
        [
            (header::CONTENT_TYPE, mime.to_string()),
            (header::CACHE_CONTROL, "private, max-age=86400".to_string()),
        ],
        bytes,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::safe_image_path;

    #[test]
    fn image_paths_stay_under_the_catalog() {
        assert!(safe_image_path("book/PHB/cover.webp").is_some());
        assert!(safe_image_path("../secret.txt").is_none());
        assert!(safe_image_path("book/../../x").is_none());
        assert!(safe_image_path("/etc/passwd").is_none());
        assert!(safe_image_path("").is_none());
    }
}

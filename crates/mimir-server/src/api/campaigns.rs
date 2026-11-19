//! Campaign content, read side (MIMIR-T-0708). A list under a campaign or
//! a module answers 404 when that campaign or module does not exist.

use axum::extract::{Path, Query, State};
use axum::Json;
use diesel::SqliteConnection;
use mimir_core::models::campaign::{Campaign, Module};
use mimir_core::services::{
    CampaignService, CharacterService, DocumentService, HomebrewService, MapService, ModuleService,
    ServiceError,
};
use mimir_wire as wire;
use serde::Deserialize;

use super::convert;
use crate::error::ApiError;
use crate::state::AppState;

type ApiResult<T> = Result<Json<T>, ApiError>;

fn require_campaign(conn: &mut SqliteConnection, id: &str) -> Result<Campaign, ApiError> {
    CampaignService::new(conn)
        .get(id)?
        .ok_or_else(|| ServiceError::not_found("Campaign", id).into())
}

fn require_module(conn: &mut SqliteConnection, id: &str) -> Result<Module, ApiError> {
    ModuleService::new(conn)
        .get(id)?
        .ok_or_else(|| ServiceError::not_found("Module", id).into())
}

#[derive(Debug, Default, Deserialize)]
pub struct CampaignListQuery {
    /// Include archived campaigns.
    #[serde(default)]
    pub archived: bool,
}

/// `GET /campaigns[?archived=true]`
pub async fn list_campaigns(
    State(state): State<AppState>,
    Query(q): Query<CampaignListQuery>,
) -> ApiResult<Vec<wire::CampaignSummary>> {
    state
        .with_db(move |conn| {
            let list = CampaignService::new(conn).list(q.archived)?;
            Ok(list.into_iter().map(convert::campaign).collect())
        })
        .await
        .map(Json)
}

/// `GET /campaigns/{id}`
pub async fn get_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::CampaignSummary> {
    state
        .with_db(move |conn| require_campaign(conn, &id).map(convert::campaign))
        .await
        .map(Json)
}

/// `GET /campaigns/{id}/documents`: the campaign-level documents.
pub async fn campaign_documents(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::DocumentSummary>> {
    state
        .with_db(move |conn| {
            require_campaign(conn, &id)?;
            let docs = DocumentService::new(conn).list_for_campaign(&id)?;
            Ok(docs.into_iter().map(convert::document_summary).collect())
        })
        .await
        .map(Json)
}

/// `GET /campaigns/{id}/modules`
pub async fn campaign_modules(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::ModuleSummary>> {
    state
        .with_db(move |conn| {
            require_campaign(conn, &id)?;
            let modules = ModuleService::new(conn).list_for_campaign(&id)?;
            Ok(modules.into_iter().map(convert::module).collect())
        })
        .await
        .map(Json)
}

#[derive(Clone, Copy)]
enum Kind {
    Pc,
    Npc,
}

fn characters(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    kind: Kind,
) -> Result<Vec<wire::CharacterSummary>, ApiError> {
    require_campaign(conn, campaign_id)?;
    let mut service = CharacterService::new(conn);
    let list = match kind {
        Kind::Pc => service.list_pcs(campaign_id)?,
        Kind::Npc => service.list_npcs(campaign_id)?,
    };
    let mut out = Vec::with_capacity(list.len());
    for c in list {
        if let Some(full) = service.get_enriched(&c.id)? {
            out.push(convert::character(full));
        }
    }
    Ok(out)
}

/// `GET /campaigns/{id}/pcs`
pub async fn campaign_pcs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::CharacterSummary>> {
    state
        .with_db(move |conn| characters(conn, &id, Kind::Pc))
        .await
        .map(Json)
}

/// `GET /campaigns/{id}/npcs`: the campaign's NPC characters.
pub async fn campaign_npcs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::CharacterSummary>> {
    state
        .with_db(move |conn| characters(conn, &id, Kind::Npc))
        .await
        .map(Json)
}

/// `GET /campaigns/{id}/maps`: the campaign-level maps.
pub async fn campaign_maps(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::MapSummary>> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_campaign(conn, &id)?;
            let maps = MapService::new(conn, app_dir).list_campaign_level(&id)?;
            Ok(maps.into_iter().map(convert::map).collect())
        })
        .await
        .map(Json)
}

/// `GET /modules/{id}`
pub async fn get_module(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::ModuleSummary> {
    state
        .with_db(move |conn| require_module(conn, &id).map(convert::module))
        .await
        .map(Json)
}

/// `GET /modules/{id}/documents`
pub async fn module_documents(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::DocumentSummary>> {
    state
        .with_db(move |conn| {
            require_module(conn, &id)?;
            let docs = DocumentService::new(conn).list_for_module(&id)?;
            Ok(docs.into_iter().map(convert::document_summary).collect())
        })
        .await
        .map(Json)
}

/// `GET /modules/{id}/monsters`
pub async fn module_monsters(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::ModuleMonsterSummary>> {
    state
        .with_db(move |conn| {
            require_module(conn, &id)?;
            let monsters = ModuleService::new(conn).list_monsters(&id)?;
            let mut out = Vec::with_capacity(monsters.len());
            for m in monsters {
                let homebrew_name = match &m.homebrew_monster_id {
                    Some(hb) => Some(HomebrewService::new(conn).get_monster(hb)?.name),
                    None => None,
                };
                out.push(convert::module_monster(m, homebrew_name));
            }
            Ok(out)
        })
        .await
        .map(Json)
}

/// `GET /modules/{id}/npcs`: the module's own NPCs.
pub async fn module_npcs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::ModuleNpcSummary>> {
    state
        .with_db(move |conn| {
            require_module(conn, &id)?;
            let npcs = ModuleService::new(conn).list_npcs(&id)?;
            Ok(npcs.into_iter().map(convert::module_npc).collect())
        })
        .await
        .map(Json)
}

/// `GET /modules/{id}/maps`
pub async fn module_maps(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Vec<wire::MapSummary>> {
    let app_dir = state.config.data_dir.clone();
    state
        .with_db(move |conn| {
            require_module(conn, &id)?;
            let maps = MapService::new(conn, app_dir).list_for_module(&id)?;
            Ok(maps.into_iter().map(convert::map).collect())
        })
        .await
        .map(Json)
}

/// `GET /documents/{id}`: a whole document, with its markdown.
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<wire::Document> {
    state
        .with_db(move |conn| {
            DocumentService::new(conn)
                .get(&id)?
                .map(convert::document)
                .ok_or_else(|| ServiceError::not_found("Document", &id).into())
        })
        .await
        .map(Json)
}

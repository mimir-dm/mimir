//! The read API (MIMIR-T-0708) against a fixture-seeded database, through
//! the real router.

use std::path::PathBuf;
use std::sync::OnceLock;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_core::seed::FIXTURE_CAMPAIGN_NAME;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use mimir_wire as wire;
use serde::de::DeserializeOwned;
use tower::ServiceExt;

/// One seeded data directory for the whole file: the tests only read.
fn seeded_dir() -> PathBuf {
    static DIR: OnceLock<tempfile::TempDir> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        let config = config(dir.path().to_path_buf(), None, false);
        mimir_server::prepare(&config).unwrap();
        // Seed through mimir-core (its `fixtures` feature is on for tests).
        let url = config.database_path().to_string_lossy().into_owned();
        let mut conn = mimir_core::db::create_connection(&url).unwrap();
        mimir_core::seed::seed_ui_fixture(&mut conn, &config.data_dir).unwrap();
        dir
    })
    .path()
    .to_path_buf()
}

fn config(data_dir: PathBuf, token: Option<&str>, seed_fixture: bool) -> Config {
    Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir,
        api_token: token.map(String::from),
        web_dist: None,
        seed_fixture,
    }
}

fn router(token: Option<&str>) -> axum::Router {
    mimir_server::router(AppState::new(config(seeded_dir(), token, false)))
}

async fn call(router: &axum::Router, path: &str, bearer: Option<&str>) -> (StatusCode, String) {
    let mut req = Request::get(path);
    if let Some(t) = bearer {
        req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let res = router
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

async fn ok<T: DeserializeOwned>(router: &axum::Router, path: &str) -> T {
    let (status, body) = call(router, path, None).await;
    assert_eq!(status, StatusCode::OK, "{path}: {body}");
    serde_json::from_str(&body).unwrap_or_else(|e| panic!("{path}: {e}: {body}"))
}

async fn not_found(router: &axum::Router, path: &str, entity: &str) {
    let (status, body) = call(router, path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{path}: {body}");
    let env: wire::ErrorBody = serde_json::from_str(&body).unwrap();
    assert_eq!(env.error.code, wire::ErrorCode::NotFound);
    assert_eq!(env.error.entity.as_deref(), Some(entity), "{path}");
}

async fn fixture_campaign(r: &axum::Router) -> wire::CampaignSummary {
    let list: Vec<wire::CampaignSummary> = ok(r, "/api/v1/campaigns").await;
    list.into_iter()
        .find(|c| c.name == FIXTURE_CAMPAIGN_NAME)
        .expect("fixture campaign in the list")
}

#[tokio::test]
async fn campaigns_list_and_get() {
    let r = router(None);
    let c = fixture_campaign(&r).await;
    let one: wire::CampaignSummary = ok(&r, &format!("/api/v1/campaigns/{}", c.id)).await;
    assert_eq!(one, c);
    let with_archived: Vec<wire::CampaignSummary> = ok(&r, "/api/v1/campaigns?archived=true").await;
    assert!(with_archived.iter().any(|x| x.id == c.id));
    not_found(&r, "/api/v1/campaigns/no-such-id", "Campaign").await;
}

#[tokio::test]
async fn a_campaigns_contents() {
    let r = router(None);
    let c = fixture_campaign(&r).await;
    let base = format!("/api/v1/campaigns/{}", c.id);

    let modules: Vec<wire::ModuleSummary> = ok(&r, &format!("{base}/modules")).await;
    assert!(!modules.is_empty());
    assert!(modules.iter().all(|m| m.campaign_id == c.id));

    let pcs: Vec<wire::CharacterSummary> = ok(&r, &format!("{base}/pcs")).await;
    assert!(!pcs.is_empty());
    assert!(pcs.iter().all(|p| !p.is_npc));
    assert!(
        pcs.iter().any(|p| !p.classes.is_empty() && p.level > 0),
        "PCs carry their classes"
    );
    for p in &pcs {
        assert_eq!(p.level, p.classes.iter().map(|k| k.level).sum::<i32>());
    }

    let npcs: Vec<wire::CharacterSummary> = ok(&r, &format!("{base}/npcs")).await;
    assert!(npcs.iter().all(|n| n.is_npc));

    let docs: Vec<wire::DocumentSummary> = ok(&r, &format!("{base}/documents")).await;
    assert!(docs.iter().all(|d| d.module_id.is_none()));

    let maps: Vec<wire::MapSummary> = ok(&r, &format!("{base}/maps")).await;
    assert!(maps.iter().all(|m| m.module_id.is_none()));

    for sub in ["modules", "pcs", "npcs", "documents", "maps"] {
        not_found(&r, &format!("/api/v1/campaigns/nope/{sub}"), "Campaign").await;
    }
}

#[tokio::test]
async fn a_modules_contents_and_a_document() {
    let r = router(None);
    let c = fixture_campaign(&r).await;
    let modules: Vec<wire::ModuleSummary> =
        ok(&r, &format!("/api/v1/campaigns/{}/modules", c.id)).await;
    let m = &modules[0];
    let base = format!("/api/v1/modules/{}", m.id);

    let one: wire::ModuleSummary = ok(&r, &base).await;
    assert_eq!(&one, m);

    let monsters: Vec<wire::ModuleMonsterSummary> = ok(&r, &format!("{base}/monsters")).await;
    assert!(!monsters.is_empty());
    assert!(monsters
        .iter()
        .all(|x| !x.name.is_empty() && x.quantity > 0));
    assert!(
        monsters.iter().any(|x| x.homebrew_monster_id.is_some()),
        "the fixture has a homebrew monster"
    );

    let npcs: Vec<wire::ModuleNpcSummary> = ok(&r, &format!("{base}/npcs")).await;
    assert!(!npcs.is_empty());
    assert!(npcs.iter().all(|n| n.module_id == m.id));

    let maps: Vec<wire::MapSummary> = ok(&r, &format!("{base}/maps")).await;
    assert!(maps
        .iter()
        .all(|x| x.module_id.as_deref() == Some(m.id.as_str())));

    let docs: Vec<wire::DocumentSummary> = ok(&r, &format!("{base}/documents")).await;
    assert!(!docs.is_empty(), "a new module has its template documents");
    let doc: wire::Document = ok(&r, &format!("/api/v1/documents/{}", docs[0].id)).await;
    assert_eq!(doc.id, docs[0].id);
    assert_eq!(doc.title, docs[0].title);

    for sub in ["", "/monsters", "/npcs", "/maps", "/documents"] {
        not_found(&r, &format!("/api/v1/modules/nope{sub}"), "Module").await;
    }
    not_found(&r, "/api/v1/documents/nope", "Document").await;
}

#[tokio::test]
async fn every_route_needs_the_dm_token() {
    let r = router(Some("dm-secret"));
    for route in mimir_server::api::routes() {
        assert_eq!(route.access, mimir_server::api::Access::Dm);
        let path = format!("/api/v1{}", route.path.replace("{id}", "x"));
        let (status, _) = call(&r, &path, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{path} without a token");
        let (status, _) = call(&r, &path, Some("wrong")).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "{path} with a wrong token"
        );
    }
    let (status, _) = call(&r, "/api/v1/campaigns", Some("dm-secret")).await;
    assert_eq!(status, StatusCode::OK);
}

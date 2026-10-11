//! PDF export (MIMIR-T-0724): each route answers a PDF for the fixture
//! (typst with the system fonts), a missing id 404.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use serde_json::Value;
use tower::ServiceExt;

const DM: &str = "dm-token-for-print-tests";

async fn get(r: &axum::Router, path: &str) -> (StatusCode, Option<String>, Vec<u8>) {
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1{path}"))
        .header(header::AUTHORIZATION, format!("Bearer {DM}"))
        .body(Body::empty())
        .unwrap();
    let res = r.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let ct = res
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string());
    (
        status,
        ct,
        res.into_body().collect().await.unwrap().to_bytes().to_vec(),
    )
}

async fn json(r: &axum::Router, path: &str) -> Value {
    serde_json::from_slice(&get(r, path).await.2).unwrap()
}

#[tokio::test]
async fn every_pdf_route_answers_a_pdf() {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().to_path_buf(),
        api_token: Some(DM.into()),
        web_dist: None,
        seed_fixture: false,
    };
    mimir_server::prepare(&config).unwrap();
    let url = config.database_path().to_string_lossy().into_owned();
    let mut conn = mimir_core::db::create_connection(&url).unwrap();
    mimir_core::seed::seed_ui_fixture(&mut conn, &config.data_dir).unwrap();
    let r = mimir_server::router(AppState::new(config));

    let campaign = json(&r, "/campaigns").await[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let pc = json(&r, &format!("/campaigns/{campaign}/pcs")).await[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let module = json(&r, &format!("/campaigns/{campaign}/modules")).await[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let map = json(&r, &format!("/modules/{module}/maps")).await[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let doc = json(&r, &format!("/campaigns/{campaign}/documents")).await[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    // A document with the adventure forms (MIMIR-T-0737): boxes, a stat
    // block line from the catalog, one it does not have, an escaped line.
    let forms_doc = {
        let mut conn = mimir_core::db::create_connection(&url).unwrap();
        let campaign_id = campaign.clone();
        let d = mimir_core::services::DocumentService::new(&mut conn)
            .create(
                mimir_core::services::CreateDocumentInput::for_campaign(&campaign_id, "Forms")
                    .with_content("> [!read-aloud]\n>\n> Fog.\n\n> [!sidebar] Tactics\n>\n> Flee.\n\n{@creature Goblin|MM}\n\n{@creatureFluff Goblin|MM}\n\n{@creature Nobody|XX}\n\n\\{@creature Rat|MM}\n\nMeet {@creature Lich|MM|the keeper}.\n"),
            )
            .unwrap();
        d.id
    };

    for path in [
        format!("/characters/{pc}/pdf?include_compact_sheet=true&include_battle_card=true"),
        format!("/documents/{doc}/pdf"),
        format!("/documents/{forms_doc}/pdf"),
        format!("/modules/{module}/pdf?include_documents=true&include_monsters=true"),
        format!("/modules/{module}/monster-cards?show_cut_lines=true"),
        format!("/maps/{map}/pdf?include_preview=true&preview_grid=true"),
        format!(
            "/campaigns/{campaign}/pdf?include_campaign_docs=true&include_module_content=false"
        ),
    ] {
        let (status, ct, bytes) = get(&r, &path).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "{path}: {}",
            String::from_utf8_lossy(&bytes[..bytes.len().min(300)])
        );
        assert_eq!(ct.as_deref(), Some("application/pdf"), "{path}");
        assert_eq!(&bytes[..4], b"%PDF", "{path}");
    }
    let (status, _, _) = get(&r, "/characters/nope/pdf").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// The campaign as one book (MIMIR-T-0742): a campaign made from the
/// fixture adventure, its text read back out of the PDF.
#[tokio::test]
async fn a_campaign_prints_as_one_book() {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().to_path_buf(),
        api_token: Some(DM.into()),
        web_dist: None,
        seed_fixture: false,
    };
    mimir_server::prepare(&config).unwrap();
    let url = config.database_path().to_string_lossy().into_owned();
    let mut conn = mimir_core::db::create_connection(&url).unwrap();
    mimir_core::seed::seed_ui_fixture(&mut conn, &config.data_dir).unwrap();
    let r = mimir_server::router(AppState::new(config));

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/books/MFA/campaign")
        .header(header::AUTHORIZATION, format!("Bearer {DM}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name": "Low Fen", "parts": [{"index": 3, "role": "skip"}]}"#,
        ))
        .unwrap();
    let res = r.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let made: Value =
        serde_json::from_slice(&res.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let cid = made["campaign_id"].as_str().unwrap();

    let (status, ct, pdf) = get(
        &r,
        &format!("/campaigns/{cid}/book?authors=Robin,%20Sam&level_start=1&level_end=3"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ct.as_deref(), Some("application/pdf"));
    let text = pdf_extract::extract_text_from_mem(&pdf).expect("the PDF's text");
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        "Low Fen",
        "By Robin, Sam",
        "For characters of levels 1 to 3",
        "Table of Contents",
        "Introduction",
        "The Old Causeway",
        "Fog lies thick on the causeway",
        "The Lantern Tower",
        "1. Gatehouse",
        "Appendix: Stat Blocks and Items",
        "Scimitar",
        "Lantern Wisp",
    ] {
        assert!(flat.contains(want), "{want:?} not in the book:\n{flat}");
    }
    // The tower's map prints in its chapter (a figure with its title).
    assert!(flat.matches("The Lantern Tower").count() >= 3, "{flat}");

    // The fixture campaign (UVTT maps, no adventure text) prints too.
    let campaigns: Value = {
        let (_, _, b) = get(&r, "/campaigns").await;
        serde_json::from_slice(&b).unwrap()
    };
    let fixture = campaigns
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] != "Low Fen")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let (status, _, pdf) = get(&r, &format!("/campaigns/{fixture}/book")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(pdf.starts_with(b"%PDF"));
}

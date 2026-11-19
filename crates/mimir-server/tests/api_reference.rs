//! The catalog and the book reader (MIMIR-T-0723): every category
//! searches, table cells, paging, one entry as JSON (with a source
//! fallback), sources, books and their images.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use serde_json::Value;
use tower::ServiceExt;

const DM: &str = "dm-token-for-reference-tests";

async fn fixture() -> (axum::Router, tempfile::TempDir) {
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
    (mimir_server::router(AppState::new(config)), dir)
}

async fn get(router: &axum::Router, path: &str) -> (StatusCode, Value, Vec<u8>) {
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/v1{path}"))
        .header(header::AUTHORIZATION, format!("Bearer {DM}"))
        .body(Body::empty())
        .unwrap();
    let res = router.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes().to_vec();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        bytes,
    )
}

#[tokio::test]
async fn every_category_searches() {
    let (r, _dir) = fixture().await;
    for (key, _, _) in mimir_render::summary::CATEGORIES {
        let (status, page, _) = get(&r, &format!("/catalog/search/{key}?limit=5")).await;
        assert_eq!(status, StatusCode::OK, "{key}: {page}");
        let headers = page["headers"].as_array().unwrap().len();
        for row in page["rows"].as_array().unwrap() {
            assert_eq!(row["cells"].as_array().unwrap().len(), headers, "{key}");
        }
    }
    let (status, _, _) = get(&r, "/catalog/search/dragons").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn search_cells_and_paging() {
    let (r, _dir) = fixture().await;
    let (_, page, _) = get(&r, "/catalog/search/spells?search=fireball").await;
    let fireball = page["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["name"] == "Fireball")
        .expect("Fireball in the SRD");
    assert_eq!(fireball["cells"][0], "3rd");
    assert_eq!(fireball["cells"][1], "Evocation");
    assert_eq!(page["headers"][0], "Level");

    let (_, first, _) = get(&r, "/catalog/search/spells?limit=2").await;
    assert_eq!(first["rows"].as_array().unwrap().len(), 2);
    assert_eq!(first["more"], true);
    let (_, second, _) = get(&r, "/catalog/search/spells?limit=2&offset=2").await;
    assert_ne!(first["rows"][0]["name"], second["rows"][0]["name"]);

    // A filter of the category: cantrips only.
    let (_, cantrips, _) = get(&r, "/catalog/search/spells?level=0&limit=50").await;
    assert!(cantrips["rows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|x| x["cells"][0] == "Cantrip"));
    let (_, goblins, _) = get(&r, "/catalog/search/monsters?search=goblin").await;
    assert!(!goblins["rows"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn entries_sources_books_images() {
    let (r, _dir) = fixture().await;
    let (_, page, _) = get(&r, "/catalog/search/spells?search=fireball").await;
    let source = page["rows"][0]["source"].as_str().unwrap().to_string();
    let (status, spell, _) = get(&r, &format!("/catalog/entry/spells/{source}/Fireball")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(spell["name"], "Fireball");
    assert_eq!(spell["level"], 3);
    assert!(spell["entries"].is_array());
    // A wrong source falls back to the name.
    let (status, again, _) = get(&r, "/catalog/entry/spells/NOPE/fireball").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again["name"], "Fireball");
    let (status, _, _) = get(&r, "/catalog/entry/spells/PHB/No%20Such%20Spell").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, sources, _) = get(&r, "/catalog/sources").await;
    assert_eq!(status, StatusCode::OK);
    assert!(sources
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["code"] == source.as_str()));
    let (status, _, _) = get(&r, "/books/NOPE").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Images: under the catalog assets only.
    let (status, _, _) = get(&r, "/book-images/book/X/none.webp").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = get(&r, "/book-images/..%2F..%2Fdata%2Fmimir.db").await;
    assert_ne!(status, StatusCode::OK, "no escape from the catalog assets");
}

/// Every SRD entry renders: no panic, its name's kind of content, and no
/// raw `{@` tag left.
#[tokio::test]
async fn every_srd_entry_renders() {
    let (r, _dir) = fixture().await;
    let mut total = 0;
    for (key, _, _) in mimir_render::summary::CATEGORIES {
        let mut offset = 0;
        loop {
            let (_, page, _) = get(
                &r,
                &format!("/catalog/search/{key}?limit=500&offset={offset}"),
            )
            .await;
            let rows = page["rows"].as_array().unwrap().clone();
            for row in &rows {
                let (name, source) = (
                    row["name"].as_str().unwrap(),
                    row["source"].as_str().unwrap(),
                );
                let path = format!(
                    "/catalog/entry/{key}/{}/{}",
                    urlencode(source),
                    urlencode(name)
                );
                let (status, v, _) = get(&r, &path).await;
                assert_eq!(status, StatusCode::OK, "{path}");
                let html = mimir_render::detail::detail(key, &v);
                assert!(html.contains("detail--"), "{key} {name}");
                assert!(!html.contains("{@"), "{key} {name}: a tag left in {html}");
                total += 1;
            }
            if page["more"] != true {
                break;
            }
            offset += 500;
        }
    }
    assert!(
        total > 100,
        "the fixture catalog has over a hundred entries ({total})"
    );
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[tokio::test]
async fn the_fixture_book_reads() {
    let (r, _dir) = fixture().await;
    let (_, sources, _) = get(&r, "/catalog/sources").await;
    let dmg = sources
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["code"] == "DMG")
        .expect("DMG");
    assert_eq!(dmg["book"], true);
    let (status, book, _) = get(&r, "/books/DMG").await;
    assert_eq!(status, StatusCode::OK);
    let c = mimir_render::book::contents(&book["data"]);
    assert_eq!(c.len(), 2);
    assert!(c[0].headings.iter().any(|h| h.id == "fixture-using"));
    let html = mimir_render::book::section(&book["data"], 0);
    assert!(html.contains(r#"data-image-path="book/DMG/fixture-hideout.png""#));
    let (status, _, bytes) = get(&r, "/book-images/book/DMG/fixture-hideout.png").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(&bytes[1..4], b"PNG");
}

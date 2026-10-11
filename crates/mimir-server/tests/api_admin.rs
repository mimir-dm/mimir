//! Settings and admin (MIMIR-T-0724): a campaign archive round trip,
//! campaign sources, a catalog import and an image import as jobs,
//! removing a source.

use std::io::Write;
use std::time::Duration;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

const DM: &str = "dm-token-for-admin-tests";

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

async fn call(
    r: &axum::Router,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Vec<u8>) {
    let mut req = Request::builder()
        .method(method)
        .uri(format!("/api/v1{path}"))
        .header(header::AUTHORIZATION, format!("Bearer {DM}"));
    let body = match body {
        Some(v) => {
            req = req.header(header::CONTENT_TYPE, "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let res = r.clone().oneshot(req.body(body).unwrap()).await.unwrap();
    let status = res.status();
    (
        status,
        res.into_body().collect().await.unwrap().to_bytes().to_vec(),
    )
}

async fn json(r: &axum::Router, method: Method, path: &str, body: Option<Value>) -> Value {
    let (status, bytes) = call(r, method.clone(), path, body).await;
    assert!(
        status.is_success(),
        "{method} {path}: {status} {}",
        String::from_utf8_lossy(&bytes)
    );
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn upload(
    r: &axum::Router,
    path: &str,
    fields: &[(&str, &str)],
    file: (&str, &[u8]),
) -> (StatusCode, Value) {
    let b = "admin-test-boundary";
    let mut body = Vec::new();
    for (k, v) in fields {
        body.extend(
            format!("--{b}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n").bytes(),
        );
    }
    body.extend(
        format!(
            "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n\r\n",
            file.0
        )
        .bytes(),
    );
    body.extend_from_slice(file.1);
    body.extend(format!("\r\n--{b}--\r\n").bytes());
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/v1{path}"))
        .header(header::AUTHORIZATION, format!("Bearer {DM}"))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={b}"),
        )
        .body(Body::from(body))
        .unwrap();
    let res = r.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// A gzip tar of (path, content); directories first.
fn tarball(dirs: &[&str], files: &[(&str, &[u8])]) -> Vec<u8> {
    let gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut b = tar::Builder::new(gz);
    for d in dirs {
        let mut h = tar::Header::new_gnu();
        h.set_entry_type(tar::EntryType::Directory);
        h.set_size(0);
        h.set_mode(0o755);
        h.set_cksum();
        b.append_data(&mut h, d, std::io::empty()).unwrap();
    }
    for (p, data) in files {
        let mut h = tar::Header::new_gnu();
        h.set_size(data.len() as u64);
        h.set_mode(0o644);
        h.set_cksum();
        b.append_data(&mut h, p, *data).unwrap();
    }
    let mut gz = b.into_inner().unwrap();
    gz.flush().unwrap();
    gz.finish().unwrap()
}

async fn wait(r: &axum::Router, job: &Value) -> Value {
    let id = job["id"].as_str().unwrap();
    for _ in 0..200 {
        let j = json(r, Method::GET, &format!("/jobs/{id}"), None).await;
        if j["state"] != "running" {
            return j;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("the job did not end");
}

#[tokio::test]
async fn a_campaign_archive_round_trip() {
    let (r, _dir) = fixture().await;
    let campaigns = json(&r, Method::GET, "/campaigns", None).await;
    let id = campaigns[0]["id"].as_str().unwrap().to_string();
    let name = campaigns[0]["name"].as_str().unwrap().to_string();
    let (status, bytes) = call(&r, Method::GET, &format!("/campaigns/{id}/export"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(&bytes[..2], &[0x1f, 0x8b], "gzip");

    let (status, preview) = upload(
        &r,
        "/archives/preview",
        &[],
        ("c.mimir-campaign.tar.gz", &bytes),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["campaign_name"], name.as_str());
    let modules = preview["counts"]["modules"].as_u64().unwrap();
    assert!(modules > 0 && preview["counts"]["maps"].as_u64().unwrap() > 0);

    let (status, done) = upload(
        &r,
        "/archives/import",
        &[("name", "Phandelver (copy)")],
        ("c.tar.gz", &bytes),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{done}");
    assert_eq!(done["campaign_name"], "Phandelver (copy)");
    let new_id = done["campaign_id"].as_str().unwrap().to_string();
    assert_ne!(new_id, id);
    let new_modules = json(
        &r,
        Method::GET,
        &format!("/campaigns/{new_id}/modules"),
        None,
    )
    .await;
    assert_eq!(new_modules.as_array().unwrap().len() as u64, modules);
    // Its maps' images came along.
    let maps = json(&r, Method::GET, &format!("/campaigns/{new_id}/maps"), None).await;
    let map = maps[0]["id"].as_str().unwrap();
    let (status, _) = call(&r, Method::GET, &format!("/maps/{map}/image"), None).await;
    assert_eq!(status, StatusCode::OK);

    // Not an archive.
    let (status, _) = upload(&r, "/archives/preview", &[], ("x.tar.gz", b"nothing")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = upload(&r, "/archives/import", &[], ("x.tar.gz", b"nothing")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let after = json(&r, Method::GET, "/campaigns", None).await;
    assert_eq!(after.as_array().unwrap().len(), 2, "no half campaign");
}

#[tokio::test]
async fn campaign_sources() {
    let (r, _dir) = fixture().await;
    let campaigns = json(&r, Method::GET, "/campaigns", None).await;
    let id = campaigns[0]["id"].as_str().unwrap().to_string();
    let path = format!("/campaigns/{id}/sources");
    let s = json(&r, Method::PUT, &path, Some(json!({"codes": ["PHB"]}))).await;
    assert_eq!(s["codes"], json!(["PHB"]));
    assert_eq!(
        json(&r, Method::GET, &path, None).await["codes"],
        json!(["PHB"])
    );
    // The catalog search in the campaign keeps to them.
    let page = json(
        &r,
        Method::GET,
        &format!("/catalog/search/monsters?campaign={id}"),
        None,
    )
    .await;
    assert!(page["rows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|x| x["source"] == "PHB"));
    let (status, _) = call(&r, Method::PUT, &path, Some(json!({"codes": ["NOPE"]}))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let s = json(&r, Method::PUT, &path, Some(json!({"codes": []}))).await;
    assert_eq!(s["codes"], json!([]));
}

#[tokio::test]
async fn catalog_and_image_imports_then_a_source_goes() {
    let (r, _dir) = fixture().await;
    let books = json!({"book": [{"id": "TST", "name": "Test Rules", "source": "TST", "group": "supplement",
        "contents": [{"name": "One"}]}]});
    let book = json!({"data": [{"type": "section", "name": "One", "entries": ["Hello from the test book."]}]});
    let spells = json!({"spell": [{"name": "Test Bolt", "source": "TST", "level": 1, "school": "V",
        "time": [{"number": 1, "unit": "action"}], "range": {"type": "point", "distance": {"type": "feet", "amount": 60}},
        "components": {"v": true}, "duration": [{"type": "instant"}], "entries": ["A test bolt."]}]});
    let data = tarball(
        &["5etools-test/", "5etools-test/data/"],
        &[
            ("5etools-test/data/books.json", books.to_string().as_bytes()),
            (
                "5etools-test/data/book/book-tst.json",
                book.to_string().as_bytes(),
            ),
            (
                "5etools-test/data/spells/spells-tst.json",
                spells.to_string().as_bytes(),
            ),
        ],
    );
    let (status, job) = upload(&r, "/catalog/import", &[], ("5etools.tar.gz", &data)).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{job}");
    let done = wait(&r, &job).await;
    assert_eq!(done["state"], "done", "{done}");
    assert!(done["result"]["sources_imported"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s == "TST"));
    let sources = json(&r, Method::GET, "/catalog/sources", None).await;
    assert!(sources
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["code"] == "TST" && s["book"] == true));
    let found = json(
        &r,
        Method::GET,
        "/catalog/search/spells?search=test%20bolt",
        None,
    )
    .await;
    assert_eq!(found["rows"][0]["source"], "TST");

    // Images: under the catalog assets, readable.
    let imgs = tarball(
        &["5etools-img-1/"],
        &[("5etools-img-1/book/TST/pic.webp", b"RIFF....WEBP")],
    );
    let (status, job) = upload(&r, "/catalog/import-images", &[], ("img.tar.gz", &imgs)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let done = wait(&r, &job).await;
    assert_eq!(done["result"]["images"], 1, "{done}");
    let (status, _) = call(&r, Method::GET, "/book-images/book/TST/pic.webp", None).await;
    assert_eq!(status, StatusCode::OK);

    // A bad archive: the job fails with a reason.
    let (_, job) = upload(&r, "/catalog/import", &[], ("x.tar.gz", b"nothing")).await;
    let failed = wait(&r, &job).await;
    assert_eq!(failed["state"], "failed");
    assert!(!failed["message"].as_str().unwrap().is_empty());

    // Remove the source: its entries go.
    let (status, _) = call(&r, Method::DELETE, "/catalog/sources/TST", None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let found = json(
        &r,
        Method::GET,
        "/catalog/search/spells?search=test%20bolt",
        None,
    )
    .await;
    assert!(found["rows"].as_array().unwrap().is_empty());
    let (status, _) = call(&r, Method::DELETE, "/catalog/sources/TST", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn adventures_in_the_reader_and_homebrew_files() {
    let (r, _dir) = fixture().await;
    // The fixture adventure (a homebrew file seeded with the fixture) is
    // listed after the books, with its level range.
    let books = json(&r, Method::GET, "/books", None).await;
    let list = books.as_array().unwrap();
    let kinds: Vec<&str> = list.iter().map(|b| b["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds.first(), Some(&"book"));
    let mfa = list.iter().find(|b| b["source"] == "MFA").expect("MFA");
    assert_eq!(mfa["kind"], "adventure");
    assert_eq!(mfa["name"], "The Lantern of Low Fen");
    assert_eq!(
        (mfa["level_start"].as_i64(), mfa["level_end"].as_i64()),
        (Some(1), Some(3))
    );
    let text = json(&r, Method::GET, "/books/MFA", None).await;
    assert_eq!(text["data"].as_array().unwrap().len(), 4);
    // Its own monster is in the catalog.
    let wisp = json(
        &r,
        Method::GET,
        "/catalog/entry/monsters/MFA/Lantern%20Wisp",
        None,
    )
    .await;
    assert_eq!(wisp["cr"], "1");

    // The same file again: its source is in the catalog.
    let file = include_bytes!("../../mimir-core/src/seed/assets/fixture-adventure.json");
    let (status, err) = upload(&r, "/catalog/import-homebrew", &[], ("mfa.json", file)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{err}");
    assert!(err.to_string().contains("already in the catalog"), "{err}");

    // Another homebrew file: a monster and an adventure of a new source.
    let other = serde_json::json!({
        "_meta": {"sources": [{"json": "HBX", "abbreviation": "HBX", "full": "Homebrew Extra"}]},
        "monster": [{"name": "Bog Hag", "source": "HBX", "size": ["M"], "type": "fey", "ac": [14],
            "hp": {"average": 30, "formula": "5d8+5"}, "speed": {"walk": 30},
            "str": 12, "dex": 12, "con": 12, "int": 12, "wis": 12, "cha": 12, "cr": "2"}],
        "adventure": [{"name": "Hag Hollow", "id": "HBX", "source": "HBX", "level": {"start": 2, "end": 2},
            "contents": [{"name": "The Hollow"}]}],
        "adventureData": [{"id": "HBX", "source": "HBX",
            "data": [{"type": "section", "name": "The Hollow", "entries": ["A {@creature Bog Hag|HBX} waits."]}]}]
    });
    let bytes = serde_json::to_vec(&other).unwrap();
    let (status, done) = upload(&r, "/catalog/import-homebrew", &[], ("hbx.json", &bytes)).await;
    assert_eq!(status, StatusCode::CREATED, "{done}");
    assert_eq!(done["sources"], serde_json::json!(["HBX"]));
    assert_eq!(done["adventures"], 1);
    assert!(done["entries"].as_u64().unwrap() >= 1);
    let books = json(&r, Method::GET, "/books", None).await;
    assert!(books
        .as_array()
        .unwrap()
        .iter()
        .any(|b| b["source"] == "HBX" && b["kind"] == "adventure"));

    // Not JSON, and JSON that is not homebrew.
    let (status, _) = upload(&r, "/catalog/import-homebrew", &[], ("x.json", b"nope")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = upload(
        &r,
        "/catalog/import-homebrew",
        &[],
        ("x.json", b"{\"monster\": []}"),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

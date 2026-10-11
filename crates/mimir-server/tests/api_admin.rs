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
    // Its cover from the index, and the homebrew mark (MIMIR-T-0745).
    assert_eq!(mfa["cover"], "covers/MFA.png");
    assert_eq!(mfa["homebrew"], true);
    let dmg = list.iter().find(|b| b["source"] == "DMG").expect("DMG");
    assert_eq!(dmg["homebrew"], false);
    let (status, _) = call(&r, Method::GET, "/book-images/covers/MFA.png", None).await;
    assert_eq!(status, StatusCode::OK);
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

#[tokio::test]
async fn an_adventure_becomes_a_campaign() {
    let (r, _dir) = fixture().await;
    // The plan of the fixture adventure.
    let plan = json(&r, Method::GET, "/books/MFA/plan", None).await;
    let parts = plan["parts"].as_array().unwrap();
    let roles: Vec<&str> = parts.iter().map(|p| p["role"].as_str().unwrap()).collect();
    assert_eq!(roles, vec!["document", "module", "module", "document"]);
    assert_eq!(
        parts[1]["documents"],
        serde_json::json!(["Arrival", "Ambush at the Reeds", "Causeway Encounters"])
    );
    assert_eq!(parts[2]["maps"], serde_json::json!(["The Lantern Tower"]));
    // A book is not an adventure.
    let (status, _) = call(&r, Method::GET, "/books/DMG/plan", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Make it, without the creatures appendix.
    let (status, bytes) = call(
        &r,
        Method::POST,
        "/books/MFA/campaign",
        Some(serde_json::json!({"name": "Low Fen", "parts": [{"index": 3, "role": "skip"}]})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let made: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(made["modules"], 2);
    assert_eq!(made["documents"], 5, "{made}");
    assert_eq!(made["maps"], 1, "{made}");
    assert_eq!(made["monsters"], 5, "{made}");
    assert_eq!(made["missing"], serde_json::json!([]), "{made}");
    let cid = made["campaign_id"].as_str().unwrap().to_string();

    let campaigns = json(&r, Method::GET, "/campaigns", None).await;
    assert!(campaigns
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["name"] == "Low Fen"));
    let modules = json(&r, Method::GET, &format!("/campaigns/{cid}/modules"), None).await;
    let names: Vec<&str> = modules
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["The Old Causeway", "The Lantern Tower"]);
    let tower = modules[1]["id"].as_str().unwrap().to_string();
    // Only the adventure's documents: no templates.
    let docs = json(
        &r,
        Method::GET,
        &format!("/campaigns/{cid}/documents"),
        None,
    )
    .await;
    let titles: Vec<&str> = docs
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, vec!["Introduction"]);
    // The introduction's area link goes to the tower's document.
    let intro = json(
        &r,
        Method::GET,
        &format!("/documents/{}", docs[0]["id"].as_str().unwrap()),
        None,
    )
    .await;
    let content = intro["content"].as_str().unwrap();
    let tower_docs = json(
        &r,
        Method::GET,
        &format!("/modules/{tower}/documents"),
        None,
    )
    .await;
    let tower_doc = tower_docs[0]["id"].as_str().unwrap();
    assert!(
        content.contains(&format!(
            "[the gatehouse](/modules/{tower}?doc={tower_doc}#mfa-g1)"
        )),
        "{content}"
    );
    assert!(
        content.contains("> [!sidebar] Running This Adventure"),
        "{content}"
    );
    // The tower's map, and its monsters from the catalog.
    let maps = json(&r, Method::GET, &format!("/modules/{tower}/maps"), None).await;
    assert_eq!(maps[0]["name"], "The Lantern Tower");
    let monsters = json(&r, Method::GET, &format!("/modules/{tower}/monsters"), None).await;
    let mnames: Vec<&str> = monsters
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap_or(""))
        .collect();
    assert!(
        mnames.contains(&"Lantern Wisp") && mnames.contains(&"Skeleton"),
        "{monsters}"
    );
    // The campaign's books: the adventure and what it names.
    let books = json(&r, Method::GET, &format!("/campaigns/{cid}/sources"), None).await;
    let codes = books["codes"].as_array().unwrap();
    assert!(
        codes.contains(&serde_json::json!("MFA")) && codes.contains(&serde_json::json!("MM")),
        "{books}"
    );
}

/// The vendored 5etools schemas (mimir-core/schema/5etools), by URI.
struct SchemaFiles;

const SCHEMA_BASE: &str = "https://schema.mimir.invalid/5etools/";

impl jsonschema::Retrieve for SchemaFiles {
    fn retrieve(
        &self,
        uri: &jsonschema::Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../mimir-core/schema/5etools");
        // A schema outside the vendored set (Foundry's, for its own
        // fields): no constraint, with each definition the vendored files
        // name in it.
        let Some(rel) = uri.as_str().strip_prefix(SCHEMA_BASE) else {
            let mut defs = serde_json::Map::new();
            let needle = format!("{}#/$defs/", uri.as_str());
            let mut stack = vec![root.clone()];
            while let Some(dir) = stack.pop() {
                for e in std::fs::read_dir(&dir)?.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        stack.push(p);
                    } else if let Ok(text) = std::fs::read_to_string(&p) {
                        for part in text.split(needle.as_str()).skip(1) {
                            let name: String = part
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            defs.insert(name, serde_json::json!({}));
                        }
                    }
                }
            }
            return Ok(serde_json::json!({"$defs": defs}));
        };
        let path = root.join(rel.split('#').next().unwrap_or(rel));
        Ok(serde_json::from_slice(
            &std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        )?)
    }
}

#[tokio::test]
async fn a_campaign_exports_as_a_homebrew_adventure() {
    let (r, _dir) = fixture().await;
    // A campaign made from the fixture adventure, its creatures left out.
    let (_, bytes) = call(
        &r,
        Method::POST,
        "/books/MFA/campaign",
        Some(serde_json::json!({"name": "Low Fen", "parts": [{"index": 3, "role": "skip"}]})),
    )
    .await;
    let cid = serde_json::from_slice::<Value>(&bytes).unwrap()["campaign_id"]
        .as_str()
        .unwrap()
        .to_string();

    // A bad source code, a missing name.
    let (status, _) = call(
        &r,
        Method::GET,
        &format!("/campaigns/{cid}/adventure-export?source=a%7Cb&name=X"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = call(
        &r,
        Method::GET,
        &format!("/campaigns/{cid}/adventure-export?source=LowFen&name=%20"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, bytes) = call(
        &r,
        Method::GET,
        &format!("/campaigns/{cid}/adventure-export?source=LowFen&name=Low%20Fen&authors=Robin,%20Sam&version=1.2.0&level_start=1&level_end=3"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let file: Value = serde_json::from_slice(&bytes).unwrap();

    // Valid against the vendored homebrew schema.
    let schema: Value = serde_json::from_slice(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../mimir-core/schema/5etools/homebrew.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let validator = jsonschema::options()
        .with_base_uri(format!("{SCHEMA_BASE}homebrew.json"))
        .with_retriever(SchemaFiles)
        .build(&schema)
        .expect("the vendored schema compiles");
    let errors: Vec<String> = validator
        .iter_errors(&file)
        .take(20)
        .map(|e| {
            let m = e.to_string();
            format!(
                "{} at {}",
                &m[m.len().saturating_sub(160)..],
                e.instance_path()
            )
        })
        .collect();
    assert!(errors.is_empty(), "schema errors:\n{}", errors.join("\n"));

    // What it holds: chapters, its own source on the copies and the tags.
    let adv = &file["adventure"][0];
    assert_eq!(adv["id"], "LowFen");
    assert_eq!(adv["level"], serde_json::json!({"start": 1, "end": 3}));
    let names: Vec<&str> = adv["contents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["Introduction", "The Old Causeway", "The Lantern Tower"]
    );
    let text = serde_json::to_string(&file["adventureData"]).unwrap();
    assert!(text.contains("{@area the gatehouse|mfa-g1|x}"), "{text}");
    assert!(
        text.contains("{@creature Goblin|LowFen}")
            || text.contains(r#""name":"Goblin","source":"LowFen""#),
        "{text}"
    );
    assert!(
        !text.contains("|MM}"),
        "every catalog tag points at the copies: {text}"
    );
    let monsters: Vec<&str> = file["monster"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    for m in ["Goblin", "Lantern Wisp", "Skeleton", "Lich"] {
        assert!(monsters.contains(&m), "{m} in {monsters:?}");
    }
    assert!(file["monster"]
        .as_array()
        .unwrap()
        .iter()
        .all(|m| m["source"] == "LowFen"));

    // Mimir reads it again: a new server takes it as a homebrew file.
    let (r2, _dir2) = fixture().await;
    let (status, done) = upload(
        &r2,
        "/catalog/import-homebrew",
        &[],
        ("lowfen.json", &bytes),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{done}");
    assert_eq!(done["adventures"], 1);
    let book = json(&r2, Method::GET, "/books/LowFen", None).await;
    assert_eq!(book["data"].as_array().unwrap().len(), 3);
    let wisp = json(
        &r2,
        Method::GET,
        "/catalog/entry/monsters/LowFen/Lantern%20Wisp",
        None,
    )
    .await;
    assert_eq!(wisp["source"], "LowFen");
}

/// A server with no catalog and no campaign.
fn bare() -> (axum::Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().to_path_buf(),
        api_token: Some(DM.into()),
        web_dist: None,
        seed_fixture: false,
    };
    mimir_server::prepare(&config).unwrap();
    (mimir_server::router(AppState::new(config)), dir)
}

#[tokio::test]
async fn a_campaign_travels_as_a_mimir_adventure_pack() {
    let (r, _dir) = fixture().await;
    let campaigns = json(&r, Method::GET, "/campaigns", None).await;
    let cid = campaigns[0]["id"].as_str().unwrap().to_string();
    let npcs_before = json(&r, Method::GET, &format!("/campaigns/{cid}/npcs"), None).await;
    let pcs_before = json(&r, Method::GET, &format!("/campaigns/{cid}/pcs"), None).await;
    assert!(!pcs_before.as_array().unwrap().is_empty());

    let (status, pack) = call(
        &r,
        Method::GET,
        &format!("/campaigns/{cid}/adventure-pack?source=PhanPack&name=Phandelver%20Pack&level_start=1&level_end=5"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(&pack[..2], b"PK", "a zip");

    // What it holds: no player characters.
    let (status, preview) = upload(
        &r,
        "/adventure-packs/preview",
        &[],
        ("p.mimir-adventure.zip", &pack),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["adventure_source"], "PhanPack");
    assert_eq!(preview["version"], "1.0");
    assert_eq!(
        preview["counts"]["characters"].as_u64().unwrap() as usize,
        npcs_before.as_array().unwrap().len()
    );
    let maps = preview["counts"]["maps"].as_u64().unwrap();
    let monsters = preview["counts"]["module_monsters"].as_u64().unwrap();
    assert!(maps > 0 && monsters > 0, "{preview}");

    // Into a Mimir with no catalog: the pack's copies stand in.
    let (r2, _dir2) = bare();
    let (status, done) = upload(
        &r2,
        "/adventure-packs/import",
        &[("name", "Phandelver at Sam's")],
        ("p.zip", &pack),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{done}");
    assert_eq!(done["campaign_name"], "Phandelver at Sam's");
    let new_id = done["campaign_id"].as_str().unwrap().to_string();
    assert_eq!(
        json(&r2, Method::GET, &format!("/campaigns/{new_id}/pcs"), None).await,
        serde_json::json!([])
    );
    assert_eq!(
        json(&r2, Method::GET, &format!("/campaigns/{new_id}/npcs"), None)
            .await
            .as_array()
            .unwrap()
            .len(),
        npcs_before.as_array().unwrap().len()
    );
    let new_maps = json(&r2, Method::GET, &format!("/campaigns/{new_id}/maps"), None).await;
    let old_maps = json(&r, Method::GET, &format!("/campaigns/{cid}/maps"), None).await;
    assert_eq!(
        new_maps.as_array().unwrap().len(),
        old_maps.as_array().unwrap().len()
    );
    // A map's image and walls came along.
    let map = new_maps[0]["id"].as_str().unwrap();
    let (status, _) = call(&r2, Method::GET, &format!("/maps/{map}/image"), None).await;
    assert_eq!(status, StatusCode::OK);
    // The adventure is in the reader, and its creatures in the catalog.
    let book = json(&r2, Method::GET, "/books/PhanPack", None).await;
    assert!(!book["data"].as_array().unwrap().is_empty());
    let modules = json(
        &r2,
        Method::GET,
        &format!("/campaigns/{new_id}/modules"),
        None,
    )
    .await;
    let mid = modules[0]["id"].as_str().unwrap();
    let mms = json(&r2, Method::GET, &format!("/modules/{mid}/monsters"), None).await;
    let goblin = mms
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "Goblin")
        .expect("the goblins");
    assert_eq!(goblin["monster_source"], "PhanPack", "{goblin}");
    let (status, _) = call(
        &r2,
        Method::GET,
        "/catalog/entry/monsters/PhanPack/Goblin",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Not a pack; a pack of another major version.
    let (status, _) = upload(&r2, "/adventure-packs/preview", &[], ("x.zip", b"nothing")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let mut zin = zip::ZipArchive::new(std::io::Cursor::new(pack.clone())).unwrap();
    let mut zout = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for i in 0..zin.len() {
        use std::io::{Read, Write};
        let mut f = zin.by_index(i).unwrap();
        let name = f.name().unwrap().to_string();
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).unwrap();
        if name == "mimir.json" {
            let mut v: Value = serde_json::from_slice(&bytes).unwrap();
            v["version"] = "2.0".into();
            bytes = serde_json::to_vec(&v).unwrap();
        }
        zout.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zout.write_all(&bytes).unwrap();
    }
    let v2 = zout.finish().unwrap().into_inner();
    let (status, err) = upload(&r2, "/adventure-packs/import", &[], ("v2.zip", &v2)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(err.to_string().contains("version 2.0"), "{err}");
}

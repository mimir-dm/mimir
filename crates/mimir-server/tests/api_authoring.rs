//! The DM's writes on campaign content (MIMIR-T-0720): campaigns,
//! documents, modules, module monsters, map uploads.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

const DM: &str = "dm-token-for-authoring-tests";

struct F {
    router: axum::Router,
    dir: tempfile::TempDir,
}

async fn fixture() -> F {
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
    F {
        router: mimir_server::router(AppState::new(config)),
        dir,
    }
}

impl F {
    async fn raw(&self, req: Request<Body>) -> (StatusCode, Value) {
        let res = self.router.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn send(&self, method: Method, path: &str, body: Option<Value>) -> (StatusCode, Value) {
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
        self.raw(req.body(body).unwrap()).await
    }

    async fn ok(&self, method: Method, path: &str, body: Option<Value>) -> Value {
        let (status, v) = self.send(method.clone(), path, body).await;
        assert!(status.is_success(), "{method} {path}: {status} {v}");
        v
    }

    /// A multipart upload: text fields, and the file.
    async fn upload(
        &self,
        path: &str,
        fields: &[(&str, &str)],
        file: (&str, &[u8]),
    ) -> (StatusCode, Value) {
        let b = "mimir-test-boundary";
        let mut body = Vec::new();
        for (k, v) in fields {
            body.extend(
                format!("--{b}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n")
                    .bytes(),
            );
        }
        body.extend(
            format!(
                "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
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
        self.raw(req).await
    }
}

fn ids(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect()
}

fn png(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    image::RgbImage::new(w, h)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

#[tokio::test]
async fn campaigns_create_edit_archive_delete() {
    let f = fixture().await;
    let (status, c) = f
        .send(
            Method::POST,
            "/campaigns",
            Some(json!({"name": "  Storm King  ", "description": "Giants"})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(c["name"], "Storm King");
    let id = c["id"].as_str().unwrap().to_string();
    // The starting documents.
    let docs = f
        .ok(Method::GET, &format!("/campaigns/{id}/documents"), None)
        .await;
    assert!(docs.as_array().unwrap().len() >= 10);

    let (status, _) = f
        .send(Method::POST, "/campaigns", Some(json!({"name": " "})))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let c = f
        .ok(
            Method::PATCH,
            &format!("/campaigns/{id}"),
            Some(json!({"name": "Storm King's Thunder", "description": null})),
        )
        .await;
    assert_eq!(c["name"], "Storm King's Thunder");
    assert!(c["description"].is_null());

    // Not deleted until archived.
    let (status, _) = f
        .send(Method::DELETE, &format!("/campaigns/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let c = f
        .ok(Method::PUT, &format!("/campaigns/{id}/archive"), None)
        .await;
    assert!(!c["archived_at"].is_null());
    let active = f.ok(Method::GET, "/campaigns", None).await;
    assert!(!ids(&active).contains(&id));
    let c = f
        .ok(Method::DELETE, &format!("/campaigns/{id}/archive"), None)
        .await;
    assert!(c["archived_at"].is_null());
    f.ok(Method::PUT, &format!("/campaigns/{id}/archive"), None)
        .await;
    let (status, _) = f
        .send(Method::DELETE, &format!("/campaigns/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = f.send(Method::GET, &format!("/campaigns/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn documents_and_modules() {
    let f = fixture().await;
    let campaigns = f.ok(Method::GET, "/campaigns", None).await;
    let campaign = campaigns[0]["id"].as_str().unwrap().to_string();

    // Documents: create, edit, move, delete.
    let path = format!("/campaigns/{campaign}/documents");
    let a = f
        .ok(Method::POST, &path, Some(json!({"title": "Rumours"})))
        .await;
    assert_eq!(a["doc_type"], "user_document");
    let b = f
        .ok(
            Method::POST,
            &path,
            Some(json!({"title": "Calendar", "content": "# Year"})),
        )
        .await;
    let (a, b) = (
        a["id"].as_str().unwrap().to_string(),
        b["id"].as_str().unwrap().to_string(),
    );
    let d = f
        .ok(
            Method::PATCH,
            &format!("/documents/{a}"),
            Some(json!({"content": "1. The mine\n2. The tower"})),
        )
        .await;
    assert_eq!(d["content"], "1. The mine\n2. The tower");
    assert_eq!(d["title"], "Rumours");
    let (status, _) = f
        .send(
            Method::PATCH,
            &format!("/documents/{a}"),
            Some(json!({"title": ""})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let before = ids(&f.ok(Method::GET, &path, None).await);
    let pos = |l: &[String], id: &str| l.iter().position(|x| x == id).unwrap();
    assert_eq!(pos(&before, &b), pos(&before, &a) + 1);
    let after = ids(&f
        .ok(
            Method::POST,
            &format!("/documents/{b}/move"),
            Some(json!({"direction": "up"})),
        )
        .await);
    assert_eq!(pos(&after, &b) + 1, pos(&after, &a));
    // At the end: no change.
    let last = after.last().unwrap().clone();
    let same = ids(&f
        .ok(
            Method::POST,
            &format!("/documents/{last}/move"),
            Some(json!({"direction": "down"})),
        )
        .await);
    assert_eq!(same, after);
    let (status, _) = f
        .send(Method::DELETE, &format!("/documents/{b}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Modules: a dungeon module with its documents; edit, move, delete.
    let mpath = format!("/campaigns/{campaign}/modules");
    let m = f
        .ok(
            Method::POST,
            &mpath,
            Some(json!({"name": "Wave Echo Cave", "module_type": "dungeon"})),
        )
        .await;
    let module = m["id"].as_str().unwrap().to_string();
    let docs = f
        .ok(Method::GET, &format!("/modules/{module}/documents"), None)
        .await;
    assert!(!docs.as_array().unwrap().is_empty(), "starting documents");
    let (status, _) = f
        .send(
            Method::POST,
            &mpath,
            Some(json!({"name": "X", "module_type": "space opera"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    f.ok(
        Method::POST,
        &format!("/modules/{module}/documents"),
        Some(json!({"title": "Encounters"})),
    )
    .await;
    let m = f
        .ok(
            Method::PATCH,
            &format!("/modules/{module}"),
            Some(json!({"name": "Wave Echo", "description": "The mine"})),
        )
        .await;
    assert_eq!(m["description"], "The mine");
    let list = ids(&f
        .ok(
            Method::POST,
            &format!("/modules/{module}/move"),
            Some(json!({"direction": "up"})),
        )
        .await);
    let n = list.len();
    assert_eq!(list[n - 2], module, "one place up from the end");
    let (status, _) = f
        .send(Method::DELETE, &format!("/modules/{module}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = f
        .send(Method::GET, &format!("/modules/{module}"), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn module_monsters_from_the_catalog() {
    let f = fixture().await;
    let campaigns = f.ok(Method::GET, "/campaigns", None).await;
    let campaign = campaigns[0]["id"].as_str().unwrap().to_string();
    let modules = f
        .ok(Method::GET, &format!("/campaigns/{campaign}/modules"), None)
        .await;
    let module = modules[0]["id"].as_str().unwrap().to_string();

    let found = f
        .ok(Method::GET, "/catalog/monsters?search=goblin", None)
        .await;
    let goblin = &found[0];
    assert!(goblin["name"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("goblin"));
    // With the campaign, the search keeps to its source books (MIMIR-T-0732).
    let source = goblin["source"].as_str().unwrap().to_string();
    let books = format!("/campaigns/{campaign}/sources");
    let sources = f.ok(Method::GET, "/catalog/sources", None).await;
    let other = sources
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["code"].as_str().unwrap().to_string())
        .find(|c| c != &source)
        .expect("the fixture catalog has another book");
    f.ok(Method::PUT, &books, Some(json!({ "codes": [other] })))
        .await;
    let none = f
        .ok(
            Method::GET,
            &format!("/catalog/monsters?search=goblin&campaign={campaign}"),
            None,
        )
        .await;
    assert!(
        none.as_array().unwrap().iter().all(|c| c["source"] == "HB"),
        "only homebrew outside the books: {none}"
    );
    f.ok(Method::PUT, &books, Some(json!({ "codes": [source] })))
        .await;
    let kept = f
        .ok(
            Method::GET,
            &format!("/catalog/monsters?search=goblin&campaign={campaign}"),
            None,
        )
        .await;
    assert!(kept
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["source"] == source.as_str()));
    f.ok(Method::PUT, &books, Some(json!({ "codes": [] })))
        .await;

    let path = format!("/modules/{module}/monsters");
    let body =
        json!({"monster_name": goblin["name"], "monster_source": goblin["source"], "quantity": 2});
    let (status, m) = f.send(Method::POST, &path, Some(body.clone())).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = m["id"].as_str().unwrap().to_string();
    // The same monster again: more of it, not a second row.
    let again = f.ok(Method::POST, &path, Some(body)).await;
    assert_eq!(again["id"], id.as_str());
    assert!(again["quantity"].as_i64().unwrap() >= 4);

    let m = f
        .ok(
            Method::PATCH,
            &format!("/module-monsters/{id}"),
            Some(json!({"quantity": 3, "display_name": "Cragmaw scouts", "notes": "On the ridge"})),
        )
        .await;
    assert_eq!(m["quantity"], 3);
    assert_eq!(m["name"], "Cragmaw scouts");
    for bad in [
        json!({"quantity": 0}),
        json!({}),
        json!({"monster_name": "Goblin"}),
    ] {
        let (status, _) = if bad.get("quantity").is_some() {
            f.send(Method::PATCH, &format!("/module-monsters/{id}"), Some(bad))
                .await
        } else {
            f.send(Method::POST, &path, Some(bad)).await
        };
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
    let (status, _) = f
        .send(Method::DELETE, &format!("/module-monsters/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let list = f.ok(Method::GET, &path, None).await;
    assert!(!ids(&list).contains(&id));
}

#[tokio::test]
async fn map_uploads() {
    let f = fixture().await;
    let campaigns = f.ok(Method::GET, "/campaigns", None).await;
    let campaign = campaigns[0]["id"].as_str().unwrap().to_string();
    let modules = f
        .ok(Method::GET, &format!("/campaigns/{campaign}/modules"), None)
        .await;
    let module = modules[0]["id"].as_str().unwrap().to_string();
    let path = format!("/campaigns/{campaign}/maps");

    // An image with a grid of 50 px a square: 4 × 2 squares.
    let (status, map) = f
        .upload(
            &path,
            &[("grid_px", "50"), ("lighting_mode", "dim")],
            ("old-owl-well.png", &png(200, 100)),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{map}");
    assert_eq!(map["name"], "old-owl-well");
    assert_eq!(map["lighting_mode"], "dim");
    assert!(map["module_id"].is_null());
    let id = map["id"].as_str().unwrap().to_string();
    let g = f
        .ok(Method::GET, &format!("/maps/{id}/geometry"), None)
        .await;
    assert_eq!(g["columns"], 4.0);
    assert_eq!(g["rows"], 2.0);
    let (status, _) = f
        .send(Method::GET, &format!("/maps/{id}/image"), None)
        .await;
    assert_eq!(status, StatusCode::OK);

    // A UVTT file into a module, named.
    let uvtt = br#"{"format":1.0,"resolution":{"map_origin":{"x":0,"y":0},"map_size":{"x":2,"y":2},"pixels_per_grid":70},"line_of_sight":[],"portals":[],"lights":[],"image":"aGk="}"#;
    let (status, m2) = f
        .upload(
            &path,
            &[("name", "Ruins"), ("module_id", &module)],
            ("ruins.dd2vtt", uvtt),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{m2}");
    assert_eq!(m2["module_id"], module.as_str());

    // Refused: not a map, a broken UVTT, a module of another campaign.
    for (name, bytes) in [
        ("notes.txt", b"hello".as_slice()),
        ("bad.dd2vtt", b"{not json".as_slice()),
        ("bad.png", b"not a png".as_slice()),
    ] {
        let (status, _) = f.upload(&path, &[], (name, bytes)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{name}");
    }
    let other = f
        .ok(Method::POST, "/campaigns", Some(json!({"name": "Other"})))
        .await;
    let (status, _) = f
        .upload(
            &format!("/campaigns/{}/maps", other["id"].as_str().unwrap()),
            &[("module_id", &module)],
            ("x.png", &png(70, 70)),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Edit: rename, move into the module, then back.
    let m = f
        .ok(
            Method::PATCH,
            &format!("/maps/{id}"),
            Some(json!({"name": "Old Owl Well", "lighting_mode": "dark", "module_id": module})),
        )
        .await;
    assert_eq!(m["name"], "Old Owl Well");
    assert_eq!(m["module_id"], module.as_str());
    let m = f
        .ok(
            Method::PATCH,
            &format!("/maps/{id}"),
            Some(json!({"module_id": null})),
        )
        .await;
    assert!(m["module_id"].is_null());

    // Delete: the record and the file.
    let assets = f.dir.path().join("assets");
    let count = || std::fs::read_dir(&assets).map(|d| d.count()).unwrap_or(0);
    let before = count();
    let (status, _) = f.send(Method::DELETE, &format!("/maps/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(count() < before, "the map's files are gone");
    let (status, _) = f.send(Method::GET, &format!("/maps/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// Images of a campaign and a module (MIMIR-T-0731): upload, list, the
/// file (an SVG under a no-script policy), refusals, delete.
#[tokio::test]
async fn images_upload_list_serve_delete() {
    let f = fixture().await;
    let campaigns = f.ok(Method::GET, "/campaigns", None).await;
    let campaign = campaigns[0]["id"].as_str().unwrap().to_string();
    let modules = f
        .ok(Method::GET, &format!("/campaigns/{campaign}/modules"), None)
        .await;
    let module = modules[0]["id"].as_str().unwrap().to_string();
    let path = format!("/campaigns/{campaign}/images");
    let before = f.ok(Method::GET, &path, None).await;

    let (status, img) = f
        .upload(
            &path,
            &[("title", "Wanted poster")],
            ("poster.png", &png(4, 4)),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{img}");
    assert_eq!(img["title"], "Wanted poster");
    assert_eq!(img["mime_type"], "image/png");
    let id = img["id"].as_str().unwrap().to_string();
    let after = f.ok(Method::GET, &path, None).await;
    assert_eq!(ids(&after).len(), ids(&before).len() + 1);

    // A module's image lists under the module, not the campaign.
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><script>alert(1)</script></svg>"#;
    let (status, seal) = f
        .upload(&path, &[("module_id", &module)], ("seal.svg", svg))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{seal}");
    let seal_id = seal["id"].as_str().unwrap().to_string();
    let listed = f
        .ok(Method::GET, &format!("/modules/{module}/images"), None)
        .await;
    assert!(ids(&listed).contains(&seal_id));
    assert!(!ids(&f.ok(Method::GET, &path, None).await).contains(&seal_id));
    // The module's own route.
    let (status, map_pic) = f
        .upload(
            &format!("/modules/{module}/images"),
            &[("title", "Cave mouth")],
            ("cave.png", &png(2, 2)),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{map_pic}");
    assert_eq!(map_pic["module_id"], module.as_str());

    // The file: an SVG never runs its script.
    let res = f
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/assets/{seal_id}/file"))
                .header(header::AUTHORIZATION, format!("Bearer {DM}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()[header::CONTENT_TYPE], "image/svg+xml");
    let csp = res.headers()[header::CONTENT_SECURITY_POLICY]
        .to_str()
        .unwrap();
    assert!(csp.contains("default-src 'none'") && csp.contains("sandbox"));
    assert_eq!(res.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");

    // Refused: not an image type, or not the bytes of one.
    let (status, _) = f.upload(&path, &[], ("notes.md", b"# hi")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = f.upload(&path, &[], ("fake.png", b"GIF89a....")).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Delete: the file goes too.
    let (status, _) = f.send(Method::DELETE, &format!("/assets/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = f
        .send(Method::GET, &format!("/assets/{id}/file"), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

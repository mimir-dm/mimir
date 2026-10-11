//! Homebrew (MIMIR-T-0722): items, monsters, spells; new or copied from
//! the catalog; edits; what uses them; the catalog searches and module
//! monsters with "HB"; delete.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

const DM: &str = "dm-token-for-homebrew-tests";

struct F {
    router: axum::Router,
    _dir: tempfile::TempDir,
    campaign: String,
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
    let mut f = F {
        router: mimir_server::router(AppState::new(config)),
        _dir: dir,
        campaign: String::new(),
    };
    let c = f.ok(Method::GET, "/campaigns", None).await;
    f.campaign = c[0]["id"].as_str().unwrap().into();
    f
}

impl F {
    async fn send_as(
        &self,
        token: &str,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut req = Request::builder()
            .method(method)
            .uri(format!("/api/v1{path}"))
            .header(header::AUTHORIZATION, format!("Bearer {token}"));
        let body = match body {
            Some(v) => {
                req = req.header(header::CONTENT_TYPE, "application/json");
                Body::from(v.to_string())
            }
            None => Body::empty(),
        };
        let res = self
            .router
            .clone()
            .oneshot(req.body(body).unwrap())
            .await
            .unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn send(&self, method: Method, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        self.send_as(DM, method, path, body).await
    }

    async fn ok(&self, method: Method, path: &str, body: Option<Value>) -> Value {
        let (status, v) = self.send(method.clone(), path, body).await;
        assert!(status.is_success(), "{method} {path}: {status} {v}");
        v
    }
}

fn named<'a>(list: &'a Value, name: &str) -> &'a Value {
    list.as_array()
        .unwrap()
        .iter()
        .find(|x| x["name"] == name)
        .unwrap_or_else(|| panic!("{name} in {list}"))
}

#[tokio::test]
async fn new_entries_edits_and_names() {
    let f = fixture().await;
    let path = format!("/campaigns/{}/homebrew/items", f.campaign);
    let (status, ring) = f
        .send(
            Method::POST,
            &path,
            Some(json!({"name": "Ring of Embers", "data": {"type": "RG", "rarity": "rare", "reqAttune": true, "entries": ["Warm to the touch."]}})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{ring}");
    assert_eq!(ring["data"]["source"], "HB");
    assert_eq!(ring["data"]["name"], "Ring of Embers");
    let id = ring["id"].as_str().unwrap().to_string();
    let list = f.ok(Method::GET, &path, None).await;
    assert_eq!(named(&list, "Ring of Embers")["detail"], "Ring");

    // Edit: the new data replaces the old; the name.
    let e = f
        .ok(
            Method::PATCH,
            &format!("/homebrew/items/{id}"),
            Some(json!({"name": "Ring of Cinders", "data": {"type": "RG", "rarity": "legendary", "entries": ["Hot."]}})),
        )
        .await;
    assert_eq!(e["data"]["rarity"], "legendary");
    assert_eq!(e["name"], "Ring of Cinders");

    // Refused: a name already used, an empty name, data not an object.
    f.ok(
        Method::POST,
        &path,
        Some(json!({"name": "Rope of Holding"})),
    )
    .await;
    for (body, want) in [
        (
            json!({"name": "rope of holding"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (json!({"name": " "}), StatusCode::UNPROCESSABLE_ENTITY),
        (
            json!({"name": "X", "data": [1, 2]}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({"name": "X", "cloned_from_name": "Dagger"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ] {
        let (status, _) = f.send(Method::POST, &path, Some(body.clone())).await;
        assert_eq!(status, want, "{body}");
    }
    let (status, _) = f
        .send(
            Method::PATCH,
            &format!("/homebrew/items/{id}"),
            Some(json!({"name": "Rope of Holding"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // A spell: its line from the data.
    let spells = format!("/campaigns/{}/homebrew/spells", f.campaign);
    f.ok(
        Method::POST,
        &spells,
        Some(json!({"name": "Ember Lance", "data": {"level": 3, "school": "V", "entries": ["Fire."]}})),
    )
    .await;
    let list = f.ok(Method::GET, &spells, None).await;
    assert_eq!(named(&list, "Ember Lance")["detail"], "3rd-level evocation");
    let (status, _) = f
        .send(Method::GET, "/campaigns/x/homebrew/vehicles", None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn copies_usage_and_the_catalog() {
    let f = fixture().await;
    let monsters = format!("/campaigns/{}/homebrew/monsters", f.campaign);
    let found = f
        .ok(Method::GET, "/catalog/search/monsters?search=goblin", None)
        .await;
    let goblin = found["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "Goblin")
        .unwrap()
        .clone();
    // A copy of the goblin, tougher: the data is merged over the catalog's.
    let (status, boss) = f
        .send(
            Method::POST,
            &monsters,
            Some(json!({"name": "Goblin Boss", "cloned_from_name": "Goblin", "cloned_from_source": goblin["source"],
                "data": {"hp": {"average": 21, "formula": "6d6"}}})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{boss}");
    assert_eq!(boss["data"]["hp"]["average"], 21);
    assert!(boss["data"]["str"].is_number(), "the catalog's data stays");
    assert_eq!(boss["cloned_from_name"], "Goblin");
    let id = boss["id"].as_str().unwrap().to_string();
    let list = f.ok(Method::GET, &monsters, None).await;
    assert_eq!(named(&list, "Goblin Boss")["detail"], "CR 1/4");
    let (status, _) = f
        .send(
            Method::POST,
            &monsters,
            Some(json!({"name": "Y", "cloned_from_name": "Nobody", "cloned_from_source": "MM"})),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // The monster search has it, as HB; a module takes it by that name.
    let choices = f
        .ok(
            Method::GET,
            &format!("/catalog/monsters?search=boss&campaign={}", f.campaign),
            None,
        )
        .await;
    assert!(choices
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["name"] == "Goblin Boss" && c["source"] == "HB"));
    let modules = f
        .ok(
            Method::GET,
            &format!("/campaigns/{}/modules", f.campaign),
            None,
        )
        .await;
    let module = modules[0]["id"].as_str().unwrap().to_string();
    let added = f
        .ok(
            Method::POST,
            &format!("/modules/{module}/monsters"),
            Some(json!({"monster_name": "Goblin Boss", "monster_source": "HB"})),
        )
        .await;
    assert_eq!(added["homebrew_monster_id"], id.as_str());
    let usage = f
        .ok(Method::GET, &format!("/homebrew/monsters/{id}/usage"), None)
        .await;
    assert!(usage["uses"][0].as_str().unwrap().contains("Goblin Boss"));

    // An item: in the item search as HB, on a sheet, in its usage.
    let items = format!("/campaigns/{}/homebrew/items", f.campaign);
    let charm = f
        .ok(
            Method::POST,
            &items,
            Some(json!({"name": "Lucky Charm", "data": {"type": "G", "rarity": "common"}})),
        )
        .await;
    let charm_id = charm["id"].as_str().unwrap().to_string();
    let choices = f
        .ok(
            Method::GET,
            &format!("/catalog/items?search=lucky&campaign={}", f.campaign),
            None,
        )
        .await;
    assert_eq!(named(&choices, "Lucky Charm")["source"], "HB");
    let pcs = f
        .ok(Method::GET, &format!("/campaigns/{}/pcs", f.campaign), None)
        .await;
    let pc = pcs[0]["id"].as_str().unwrap().to_string();
    f.ok(
        Method::POST,
        &format!("/characters/{pc}/inventory"),
        Some(json!({"item_name": "Lucky Charm", "item_source": "HB"})),
    )
    .await;
    let usage = f
        .ok(
            Method::GET,
            &format!("/homebrew/items/{charm_id}/usage"),
            None,
        )
        .await;
    assert!(usage["uses"][0].as_str().unwrap().ends_with("carries it"));
    // A player of the campaign sees its homebrew; another campaign's id is refused.
    let link = f
        .ok(Method::POST, &format!("/characters/{pc}/link"), None)
        .await;
    let token = link["token"].as_str().unwrap().to_string();
    let (status, _) = f
        .send_as(
            &token,
            Method::GET,
            &format!("/catalog/items?search=lucky&campaign={}", f.campaign),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = f
        .send_as(
            &token,
            Method::GET,
            "/catalog/items?search=lucky&campaign=other",
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Delete the monster: it leaves the module.
    let (status, _) = f
        .send(Method::DELETE, &format!("/homebrew/monsters/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let left = f
        .ok(Method::GET, &format!("/modules/{module}/monsters"), None)
        .await;
    assert!(!left
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["homebrew_monster_id"] == id.as_str()));
    let (status, _) = f
        .send(Method::GET, &format!("/homebrew/monsters/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

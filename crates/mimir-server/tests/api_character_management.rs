//! The DM's character management (MIMIR-T-0721): the wizard's catalog
//! lists, create (PC with a class at level 1, NPC), move between
//! campaigns, the unassigned list, delete.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

const DM: &str = "dm-token-for-character-tests";

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

fn find<'a>(list: &'a Value, name: &str) -> &'a Value {
    list.as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap_or_else(|| panic!("{name} not in the catalog"))
}

#[tokio::test]
async fn a_pc_from_the_wizard() {
    let f = fixture().await;
    let races = f.ok(Method::GET, "/catalog/races", None).await;
    let classes = f.ok(Method::GET, "/catalog/classes", None).await;
    let backgrounds = f.ok(Method::GET, "/catalog/backgrounds", None).await;
    let (race, class, bg) = (
        &races[0],
        find(&classes, "Fighter"),
        find(&backgrounds, "Acolyte"),
    );
    let skills = f
        .ok(
            Method::GET,
            &format!(
                "/catalog/creation-skills?class=Fighter&class_source={}&background=Acolyte&background_source={}",
                class["source"].as_str().unwrap(),
                bg["source"].as_str().unwrap()
            ),
            None,
        )
        .await;
    assert_eq!(skills["choose"], 2);
    let background: Vec<String> = serde_json::from_value(skills["background"].clone()).unwrap();
    assert!(!background.is_empty());
    let from: Vec<String> = serde_json::from_value(skills["from"].clone()).unwrap();
    assert!(
        from.iter().all(|s| !background.contains(s)),
        "no double picks"
    );
    let picks: Vec<String> = from.iter().take(2).cloned().collect();
    let mut selected = background.clone();
    selected.extend(picks.clone());

    let (status, sheet) = f
        .send(
            Method::POST,
            "/characters",
            Some(json!({
                "campaign_id": f.campaign,
                "name": "Robin Brightwater",
                "player_name": "Sam",
                "race_name": race["name"], "race_source": race["source"],
                "class_name": "Fighter", "class_source": class["source"],
                "background_name": "Acolyte", "background_source": bg["source"],
                "ability_scores": [15, 14, 13, 12, 10, 8],
                "selected_skills": selected,
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{sheet}");
    assert_eq!(sheet["classes"][0]["class_name"], "Fighter");
    assert_eq!(
        sheet["classes"][0]["level"], 1,
        "the class starts at level 1"
    );
    assert_eq!(sheet["abilities"]["strength"], 15);
    let profs: Vec<String> = sheet["proficiencies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["kind"] == "skill")
        .map(|p| p["name"].as_str().unwrap().to_lowercase())
        .collect();
    for s in picks.iter().chain(background.iter()) {
        assert!(profs.contains(&s.to_lowercase()), "{s} in {profs:?}");
    }
    let pcs = f
        .ok(Method::GET, &format!("/campaigns/{}/pcs", f.campaign), None)
        .await;
    assert!(pcs
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["name"] == "Robin Brightwater" && c["level"] == 1));

    // Refused: an unknown race, half a pick, no name, bad scores.
    for (body, want) in [
        (
            json!({"name": "X", "race_name": "Moon Elf", "race_source": "NOPE"}),
            StatusCode::NOT_FOUND,
        ),
        (
            json!({"name": "X", "class_name": "Fighter"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (json!({"name": "  "}), StatusCode::UNPROCESSABLE_ENTITY),
        (
            json!({"name": "X", "ability_scores": [0, 10, 10, 10, 10, 10]}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ] {
        let (status, _) = f
            .send(Method::POST, "/characters", Some(body.clone()))
            .await;
        assert_eq!(status, want, "{body}");
    }
}

#[tokio::test]
async fn npcs_moves_and_delete() {
    let f = fixture().await;
    // An NPC in no campaign, with its role.
    let npc = f
        .ok(
            Method::POST,
            "/characters",
            Some(json!({"name": "Sister Garaele", "is_npc": true, "role": "Priestess", "location": "Phandalin", "faction": ""})),
        )
        .await;
    let id = npc["id"].as_str().unwrap().to_string();
    assert_eq!(npc["is_npc"], true);
    assert_eq!(npc["role"], "Priestess");
    assert!(npc["faction"].is_null());
    assert!(npc["campaign_id"].is_null());
    let un = f.ok(Method::GET, "/characters/unassigned", None).await;
    assert!(un
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["id"] == id.as_str()));

    // Edit the NPC fields.
    let s = f
        .ok(
            Method::PATCH,
            &format!("/characters/{id}"),
            Some(json!({"faction": "Harpers", "location": null})),
        )
        .await;
    assert_eq!(s["faction"], "Harpers");
    assert!(s["location"].is_null());
    assert_eq!(s["role"], "Priestess");

    // Into the campaign, then out again.
    let path = format!("/characters/{id}/campaign");
    let s = f
        .ok(Method::PUT, &path, Some(json!({"campaign_id": f.campaign})))
        .await;
    assert_eq!(s["campaign_id"], f.campaign.as_str());
    let npcs = f
        .ok(
            Method::GET,
            &format!("/campaigns/{}/npcs", f.campaign),
            None,
        )
        .await;
    assert!(npcs
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["id"] == id.as_str()));
    let (status, _) = f
        .send(Method::PUT, &path, Some(json!({"campaign_id": "nope"})))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let s = f
        .ok(Method::PUT, &path, Some(json!({"campaign_id": null})))
        .await;
    assert!(s["campaign_id"].is_null());

    // Delete a PC with a player link: the link stops working.
    let pcs = f
        .ok(Method::GET, &format!("/campaigns/{}/pcs", f.campaign), None)
        .await;
    let pc = pcs[0]["id"].as_str().unwrap().to_string();
    let link = f
        .ok(Method::POST, &format!("/characters/{pc}/link"), None)
        .await;
    let token = link["token"].as_str().unwrap().to_string();
    // A player may not delete, not even their own character.
    let (status, _) = f
        .send_as(&token, Method::DELETE, &format!("/characters/{pc}"), None)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = f
        .send(Method::DELETE, &format!("/characters/{pc}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = f
        .send(Method::GET, &format!("/characters/{pc}"), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = f.send_as(&token, Method::GET, "/session", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

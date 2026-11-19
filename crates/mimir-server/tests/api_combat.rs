//! The combat API (MIMIR-T-0715) on the fixture module, through the real
//! router: start and resume, entries, turn order and turns, HP, conditions,
//! concentration, token links, the players' turn order, and the end.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use mimir_wire as wire;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tower::ServiceExt;

struct F {
    router: axum::Router,
    _dir: tempfile::TempDir,
    campaign: String,
    module: String,
    map: String,
}

async fn fixture() -> F {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().to_path_buf(),
        api_token: None,
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
        module: String::new(),
        map: String::new(),
    };
    let c: Vec<wire::CampaignSummary> = f.ok(Method::GET, "/campaigns", None).await;
    f.campaign = c[0].id.clone();
    let m: Vec<wire::ModuleSummary> = f
        .ok(
            Method::GET,
            &format!("/campaigns/{}/modules", f.campaign),
            None,
        )
        .await;
    f.module = m[0].id.clone();
    let maps: Vec<wire::MapSummary> = f
        .ok(Method::GET, &format!("/modules/{}/maps", f.module), None)
        .await;
    f.map = maps[0].id.clone();
    f
}

impl F {
    async fn send(&self, method: Method, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut req = Request::builder()
            .method(method)
            .uri(format!("/api/v1{path}"));
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

    async fn ok<T: DeserializeOwned>(&self, method: Method, path: &str, body: Option<Value>) -> T {
        let (status, v) = self.send(method.clone(), path, body).await;
        assert!(status.is_success(), "{method} {path}: {status} {v}");
        serde_json::from_value(v).unwrap()
    }
}

fn find<'a>(c: &'a wire::Combat, name: &str) -> &'a wire::CombatEntry {
    c.entries
        .iter()
        .find(|e| e.name == name)
        .unwrap_or_else(|| {
            panic!(
                "{name} in {:?}",
                c.entries.iter().map(|e| &e.name).collect::<Vec<_>>()
            )
        })
}

#[tokio::test]
async fn start_add_order_turns_and_end() {
    let f = fixture().await;
    let base = format!("/modules/{}/combat", f.module);
    let none: Option<wire::Combat> = f.ok(Method::GET, &base, None).await;
    assert!(none.is_none());

    let (status, v) = f.send(Method::POST, &base, None).await;
    assert_eq!(status, StatusCode::CREATED);
    let c: wire::Combat = serde_json::from_value(v).unwrap();
    assert_eq!((c.session.round, c.session.status.as_str()), (1, "active"));
    let again: wire::Combat = f.ok(Method::POST, &base, None).await;
    assert_eq!(
        again.session.id, c.session.id,
        "start resumes the active combat"
    );
    let id = c.session.id.clone();

    // A monster group, a PC, an NPC and a custom entry.
    let monsters: Vec<wire::ModuleMonsterSummary> = f
        .ok(
            Method::GET,
            &format!("/modules/{}/monsters", f.module),
            None,
        )
        .await;
    let klarg = monsters.iter().find(|m| m.name == "Klarg").unwrap();
    let c: wire::Combat = f
        .ok(
            Method::POST,
            &format!("/combat/{id}/entries"),
            Some(json!({"kind": "monster", "module_monster_id": klarg.id})),
        )
        .await;
    assert_eq!(find(&c, "Klarg").source_kind, "module_monster");
    // An SRD group: numbered entries with HP from the stat block.
    let goblins = monsters
        .iter()
        .find(|m| m.monster_name.as_deref() == Some("Goblin"))
        .unwrap();
    let with_goblins: wire::Combat = f
        .ok(
            Method::POST,
            &format!("/combat/{id}/entries"),
            Some(json!({"kind": "monster", "module_monster_id": goblins.id, "count": 2})),
        )
        .await;
    for n in ["Goblin 1", "Goblin 2"] {
        assert!(
            find(&with_goblins, n).max_hp.is_some(),
            "{n}: HP from the stat block"
        );
    }
    for g in ["Goblin 1", "Goblin 2"] {
        let gid = find(&with_goblins, g).id.clone();
        f.ok::<wire::Combat>(Method::DELETE, &format!("/combat-entries/{gid}"), None)
            .await;
    }
    let pcs: Vec<wire::CharacterSummary> = f
        .ok(Method::GET, &format!("/campaigns/{}/pcs", f.campaign), None)
        .await;
    let elara = pcs.iter().find(|p| p.name == "Elara Moonwhisper").unwrap();
    f.ok::<wire::Combat>(
        Method::POST,
        &format!("/combat/{id}/entries"),
        Some(json!({"kind": "character", "character_id": elara.id})),
    )
    .await;
    let npcs: Vec<wire::ModuleNpcSummary> = f
        .ok(Method::GET, &format!("/modules/{}/npcs", f.module), None)
        .await;
    f.ok::<wire::Combat>(
        Method::POST,
        &format!("/combat/{id}/entries"),
        Some(json!({"kind": "npc", "npc_id": npcs[0].id})),
    )
    .await;
    let c: wire::Combat = f
        .ok(
            Method::POST,
            &format!("/combat/{id}/entries"),
            Some(json!({"kind": "custom", "name": "Ghost", "max_hp": 20, "initiative": 12})),
        )
        .await;
    assert_eq!(c.entries.len(), 4);
    let (status, body) = f
        .send(
            Method::POST,
            &format!("/combat/{id}/entries"),
            Some(json!({"kind": "custom", "name": "  "})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    // Initiative orders the entries; an empty one sorts last.
    let ids: Vec<(String, i32)> = vec![
        (find(&c, "Klarg").id.clone(), 15),
        (find(&c, "Elara Moonwhisper").id.clone(), 18),
    ];
    for (eid, init) in &ids {
        f.ok::<wire::Combat>(
            Method::PATCH,
            &format!("/combat-entries/{eid}"),
            Some(json!({"initiative": init})),
        )
        .await;
    }
    let c: wire::Combat = f.ok(Method::GET, &format!("/combat/{id}"), None).await;
    let order: Vec<&str> = c.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(&order[..3], &["Elara Moonwhisper", "Klarg", "Ghost"]);
    assert_eq!(
        c.current_entry_id.as_deref(),
        Some(c.entries[0].id.as_str())
    );

    // Turns wrap into the next round, and back.
    let mut s = c.clone();
    for _ in 0..4 {
        s = f
            .ok(Method::POST, &format!("/combat/{id}/next"), None)
            .await;
    }
    assert_eq!((s.session.round, s.session.turn_index), (2, 0));
    let s: wire::Combat = f
        .ok(Method::POST, &format!("/combat/{id}/previous"), None)
        .await;
    assert_eq!((s.session.round, s.session.turn_index), (1, 3));

    // Clearing an initiative.
    let c: wire::Combat = f
        .ok(
            Method::PATCH,
            &format!("/combat-entries/{}", ids[0].0),
            Some(json!({"initiative": null})),
        )
        .await;
    assert_eq!(find(&c, "Klarg").initiative, None);

    // Remove an entry; end; then changes are refused.
    let ghost = find(&c, "Ghost").id.clone();
    let c: wire::Combat = f
        .ok(Method::DELETE, &format!("/combat-entries/{ghost}"), None)
        .await;
    assert_eq!(c.entries.len(), 3);
    let (status, _) = f.send(Method::DELETE, &format!("/combat/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = f
        .send(Method::POST, &format!("/combat/{id}/next"), None)
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["message"], "this combat has ended");
    let none: Option<wire::Combat> = f.ok(Method::GET, &base, None).await;
    assert!(none.is_none());
}

#[tokio::test]
async fn hp_conditions_and_concentration() {
    let f = fixture().await;
    let c: wire::Combat = f
        .ok(Method::POST, &format!("/modules/{}/combat", f.module), None)
        .await;
    let id = c.session.id;
    let c: wire::Combat = f
        .ok(
            Method::POST,
            &format!("/combat/{id}/entries"),
            Some(json!({"kind": "custom", "name": "Ogre", "max_hp": 30})),
        )
        .await;
    let e = find(&c, "Ogre").id.clone();
    let path = format!("/combat-entries/{e}");

    // Temp HP absorbs damage first.
    f.ok::<wire::Combat>(
        Method::PATCH,
        &path,
        Some(json!({"temp_hp": 5, "concentrating": true})),
    )
    .await;
    let r: wire::DamageResult = f
        .ok(
            Method::POST,
            &format!("{path}/damage"),
            Some(json!({"amount": 24})),
        )
        .await;
    assert_eq!((r.entry.temp_hp, r.entry.current_hp), (0, Some(11)));
    assert_eq!(r.concentration_dc, Some(12), "max(10, 24 / 2)");
    let r: wire::DamageResult = f
        .ok(
            Method::POST,
            &format!("{path}/damage"),
            Some(json!({"amount": 4})),
        )
        .await;
    assert_eq!(r.concentration_dc, Some(10));
    let healed: wire::CombatEntry = f
        .ok(
            Method::POST,
            &format!("{path}/heal"),
            Some(json!({"amount": 100})),
        )
        .await;
    assert_eq!(healed.current_hp, Some(30), "heal caps at max");
    assert_eq!(healed.hp_log.len(), 3, "the log keeps three");
    let (status, _) = f
        .send(
            Method::POST,
            &format!("{path}/damage"),
            Some(json!({"amount": -1})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Conditions.
    let with: wire::CombatEntry = f
        .ok(
            Method::POST,
            &format!("{path}/conditions"),
            Some(json!({"name": "poisoned", "duration_rounds": 2})),
        )
        .await;
    assert_eq!(
        with.conditions,
        vec![wire::Condition {
            name: "poisoned".into(),
            expires_round: Some(2)
        }]
    );
    let (status, _) = f
        .send(
            Method::POST,
            &format!("{path}/conditions"),
            Some(json!({"name": "sleepy"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let without: wire::CombatEntry = f
        .ok(Method::DELETE, &format!("{path}/conditions/poisoned"), None)
        .await;
    assert!(without.conditions.is_empty());

    // Max HP: a monster without HP gets one.
    let c: wire::Combat = f
        .ok(
            Method::POST,
            &format!("/combat/{id}/entries"),
            Some(json!({"kind": "custom", "name": "Shade"})),
        )
        .await;
    let shade = find(&c, "Shade").id.clone();
    let c: wire::Combat = f
        .ok(
            Method::PATCH,
            &format!("/combat-entries/{shade}"),
            Some(json!({"max_hp": 9})),
        )
        .await;
    assert_eq!(
        (find(&c, "Shade").max_hp, find(&c, "Shade").current_hp),
        (Some(9), Some(9))
    );
}

#[tokio::test]
async fn token_links_and_the_players_turn_order() {
    let f = fixture().await;
    let c: wire::Combat = f
        .ok(Method::POST, &format!("/modules/{}/combat", f.module), None)
        .await;
    let id = c.session.id;
    let monsters: Vec<wire::ModuleMonsterSummary> = f
        .ok(
            Method::GET,
            &format!("/modules/{}/monsters", f.module),
            None,
        )
        .await;
    let klarg = monsters.iter().find(|m| m.name == "Klarg").unwrap();
    f.ok::<wire::Combat>(
        Method::POST,
        &format!("/combat/{id}/entries"),
        Some(json!({"kind": "monster", "module_monster_id": klarg.id})),
    )
    .await;
    f.ok::<wire::Combat>(
        Method::POST,
        &format!("/combat/{id}/entries"),
        Some(json!({"kind": "custom", "name": "Robin", "initiative": 5})),
    )
    .await;
    let c: wire::Combat = f
        .ok(
            Method::POST,
            &format!("/combat/{id}/link-tokens"),
            Some(json!({"map_id": f.map})),
        )
        .await;
    let token = find(&c, "Klarg")
        .token_id
        .clone()
        .expect("Klarg is linked to his token");

    // The display shows the map and the order.
    f.ok::<wire::DisplayState>(
        Method::PUT,
        &format!("/campaigns/{}/display", f.campaign),
        Some(json!({"map_id": f.map, "show_initiative": true})),
    )
    .await;
    let view: wire::PlayerView = f
        .ok(Method::GET, &format!("/maps/{}/player-view", f.map), None)
        .await;
    let order = view.initiative.expect("the order is shown");
    let names: Vec<&str> = order.entries.iter().map(|e| e.name.as_str()).collect();
    assert!(
        names.contains(&"Klarg") && names.contains(&"Robin"),
        "{names:?}"
    );
    assert_eq!(order.entries.iter().filter(|e| e.current).count(), 1);

    // Klarg's token hidden: he leaves the players' order; the custom stays.
    f.ok::<wire::Token>(
        Method::PATCH,
        &format!("/tokens/{token}"),
        Some(json!({"hidden": true})),
    )
    .await;
    let view: wire::PlayerView = f
        .ok(Method::GET, &format!("/maps/{}/player-view", f.map), None)
        .await;
    let names: Vec<String> = view
        .initiative
        .unwrap()
        .entries
        .into_iter()
        .map(|e| e.name)
        .collect();
    assert_eq!(names, vec!["Robin".to_string()]);

    // The order not shown: no order in the view.
    f.ok::<wire::DisplayState>(
        Method::PUT,
        &format!("/campaigns/{}/display", f.campaign),
        Some(json!({"map_id": f.map, "show_initiative": false})),
    )
    .await;
    let view: wire::PlayerView = f
        .ok(Method::GET, &format!("/maps/{}/player-view", f.map), None)
        .await;
    assert!(view.initiative.is_none());

    // Unlinking a token.
    let c: wire::Combat = f
        .ok(
            Method::PATCH,
            &format!("/combat-entries/{}", find(&c, "Klarg").id),
            Some(json!({"token_id": null})),
        )
        .await;
    assert_eq!(find(&c, "Klarg").token_id, None);
}

//! The character API (MIMIR-T-0717) on the fixture: the sheet, changes,
//! inventory, spells, level-up, catalog choices, and a player's scope.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use mimir_wire as wire;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tower::ServiceExt;

const DM: &str = "dm-token-for-character-tests";

struct F {
    router: axum::Router,
    _dir: tempfile::TempDir,
    pcs: Vec<wire::CharacterSummary>,
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
        pcs: Vec::new(),
    };
    let c: Value = f.ok(Method::GET, "/campaigns", DM, None).await;
    let campaign = c[0]["id"].as_str().unwrap().to_string();
    f.pcs = f
        .ok(Method::GET, &format!("/campaigns/{campaign}/pcs"), DM, None)
        .await;
    f
}

impl F {
    async fn send(
        &self,
        method: Method,
        path: &str,
        token: &str,
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

    async fn ok<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        token: &str,
        body: Option<Value>,
    ) -> T {
        let (status, v) = self.send(method.clone(), path, token, body).await;
        assert!(status.is_success(), "{method} {path}: {status} {v}");
        serde_json::from_value(v).unwrap()
    }

    fn pc(&self, name: &str) -> String {
        self.pcs
            .iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("{name}"))
            .id
            .clone()
    }
}

#[tokio::test]
async fn the_sheet_and_its_changes() {
    let f = fixture().await;
    let elara = f.pc("Elara Moonwhisper");
    let s: wire::CharacterSheet = f
        .ok(Method::GET, &format!("/characters/{elara}"), DM, None)
        .await;
    assert_eq!(s.name, "Elara Moonwhisper");
    assert!(!s.is_npc);
    assert_eq!(s.classes[0].class_name, "Wizard");
    assert!(s.speed >= 25);

    let path = format!("/characters/{elara}");
    let mut abilities = s.abilities;
    abilities.intelligence = 18;
    let s: wire::CharacterSheet = f
        .ok(
            Method::PATCH,
            &path,
            DM,
            Some(json!({"abilities": abilities, "currency": {"cp": 1, "sp": 2, "ep": 0, "gp": 30, "pp": 1}, "traits": "Curious"})),
        )
        .await;
    assert_eq!(s.abilities.intelligence, 18);
    assert_eq!(s.currency.gp, 30);
    assert_eq!(s.traits.as_deref(), Some("Curious"));
    let s: wire::CharacterSheet = f
        .ok(Method::PATCH, &path, DM, Some(json!({"traits": null})))
        .await;
    assert_eq!(s.traits, None, "null clears");

    abilities.strength = 31;
    for bad in [
        json!({"abilities": abilities}),
        json!({"currency": {"cp": -1, "sp": 0, "ep": 0, "gp": 0, "pp": 0}}),
        json!({"name": "  "}),
    ] {
        let (status, _) = f.send(Method::PATCH, &path, DM, Some(bad.clone())).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{bad}");
    }
    let (status, _) = f.send(Method::GET, "/characters/nope", DM, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn inventory_and_attunement() {
    let f = fixture().await;
    let thorin = f.pc("Thorin Ironforge");
    let base = format!("/characters/{thorin}/inventory");
    let rope: wire::InventoryItem = f
        .ok(
            Method::POST,
            &base,
            DM,
            Some(json!({"item_name": "Rope", "item_source": "PHB", "quantity": 2})),
        )
        .await;
    assert_eq!(rope.quantity, 2);
    let rope: wire::InventoryItem = f
        .ok(
            Method::PATCH,
            &format!("/inventory/{}", rope.id),
            DM,
            Some(json!({"quantity": 3, "equipped": true})),
        )
        .await;
    assert!(rope.equipped && rope.quantity == 3);
    let (status, _) = f
        .send(
            Method::PATCH,
            &format!("/inventory/{}", rope.id),
            DM,
            Some(json!({"quantity": 0})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Attune to three; the fourth is refused.
    let s: wire::CharacterSheet = f
        .ok(Method::GET, &format!("/characters/{thorin}"), DM, None)
        .await;
    let already = s.inventory.iter().filter(|i| i.attuned).count();
    for n in already..3 {
        f.ok::<wire::InventoryItem>(
            Method::POST,
            &base,
            DM,
            Some(json!({"item_name": format!("Ring {n}"), "item_source": "DMG", "attuned": true})),
        )
        .await;
    }
    let (status, body) = f
        .send(
            Method::POST,
            &base,
            DM,
            Some(json!({"item_name": "Ring X", "item_source": "DMG", "attuned": true})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("at most 3"));

    let (status, _) = f
        .send(Method::DELETE, &format!("/inventory/{}", rope.id), DM, None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let s: wire::CharacterSheet = f
        .ok(Method::GET, &format!("/characters/{thorin}"), DM, None)
        .await;
    assert!(s.inventory.iter().all(|i| i.id != rope.id));
}

#[tokio::test]
async fn spells_known_and_prepared() {
    let f = fixture().await;
    let elara = f.pc("Elara Moonwhisper");
    let base = format!("/characters/{elara}/spells");
    // A Wizard spell she does not know yet.
    let sheet: wire::CharacterSheet = f
        .ok(Method::GET, &format!("/characters/{elara}"), DM, None)
        .await;
    let catalog: Vec<wire::Choice> = f
        .ok(
            Method::GET,
            "/catalog/spells?class=Wizard&max_level=1",
            DM,
            None,
        )
        .await;
    let new = catalog
        .iter()
        .find(|c| !sheet.spells.iter().any(|k| k.spell_name == c.name))
        .expect("an unknown spell");
    let body =
        json!({"spell_name": new.name, "spell_source": new.source, "source_class": "Wizard"});
    let s: wire::KnownSpell = f.ok(Method::POST, &base, DM, Some(body.clone())).await;
    assert!(!s.prepared);
    let (status, _) = f.send(Method::POST, &base, DM, Some(body)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "known already");
    let p: wire::KnownSpell = f
        .ok(
            Method::PATCH,
            &format!("{base}/{}", s.id),
            DM,
            Some(json!({"prepared": true})),
        )
        .await;
    assert!(p.prepared);
    let p: wire::KnownSpell = f
        .ok(
            Method::PATCH,
            &format!("{base}/{}", s.id),
            DM,
            Some(json!({"prepared": true})),
        )
        .await;
    assert!(p.prepared, "true stays true");
    let (status, _) = f
        .send(Method::DELETE, &format!("{base}/{}", s.id), DM, None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = f
        .send(Method::DELETE, &format!("{base}/{}", s.id), DM, None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn level_up_takes_the_web_form_and_empty_spell_lists() {
    let f = fixture().await;
    let elara = f.pc("Elara Moonwhisper");
    let before: wire::CharacterSheet = f
        .ok(Method::GET, &format!("/characters/{elara}"), DM, None)
        .await;
    let level = before.classes[0].level;
    // The desktop app failed on an empty spell pick ("missing field").
    let r: wire::LevelUpResult = f
        .ok(
            Method::POST,
            &format!("/characters/{elara}/level-up"),
            DM,
            Some(json!({"class_name": "Wizard", "class_source": "PHB", "hit_points_method": {"type": "Average"}, "spell_changes": {}})),
        )
        .await;
    assert_eq!(r.new_total_level, level + 1);
    assert!(r.hp_gained >= 1);
    assert_eq!(r.sheet.classes[0].level, level + 1);

    // The web client's request type gives the same JSON.
    let req = wire::LevelUpRequest {
        class_name: "Wizard".into(),
        class_source: "PHB".into(),
        hit_points_method: wire::HpMethod::Roll(4),
        subclass: None,
        asi_or_feat: None,
        spell_changes: Some(wire::SpellChanges {
            new_spells: vec![wire::Ref {
                name: "Fly".into(),
                source: "PHB".into(),
            }],
            ..wire::SpellChanges::default()
        }),
        feature_choices: None,
    };
    let r: wire::LevelUpResult = f
        .ok(
            Method::POST,
            &format!("/characters/{elara}/level-up"),
            DM,
            Some(serde_json::to_value(&req).unwrap()),
        )
        .await;
    assert_eq!(r.new_total_level, level + 2);
    assert!(r.sheet.spells.iter().any(|s| s.spell_name == "Fly"));
    // A roll above the hit die is refused, and nothing changes.
    let (status, _) = f
        .send(
            Method::POST,
            &format!("/characters/{elara}/level-up"),
            DM,
            Some(json!({"class_name": "Wizard", "class_source": "PHB", "hit_points_method": {"type": "Roll", "value": 99}})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_player_reaches_only_their_own_character() {
    let f = fixture().await;
    let mine = f.pc("Elara Moonwhisper");
    let other = f.pc("Thorin Ironforge");
    let link: Value = f
        .ok(Method::POST, &format!("/characters/{mine}/link"), DM, None)
        .await;
    let token = link["token"].as_str().unwrap().to_string();

    let s: wire::CharacterSheet = f
        .ok(Method::GET, &format!("/characters/{mine}"), &token, None)
        .await;
    assert_eq!(s.id, mine);
    f.ok::<wire::CharacterSheet>(
        Method::PATCH,
        &format!("/characters/{mine}"),
        &token,
        Some(json!({"bonds": "My spellbook"})),
    )
    .await;
    let item: wire::InventoryItem = f
        .ok(
            Method::POST,
            &format!("/characters/{mine}/inventory"),
            &token,
            Some(json!({"item_name": "Ink", "item_source": "PHB"})),
        )
        .await;
    f.ok::<wire::InventoryItem>(
        Method::PATCH,
        &format!("/inventory/{}", item.id),
        &token,
        Some(json!({"quantity": 2})),
    )
    .await;

    // Another character: refused, by every route.
    let theirs: wire::InventoryItem = f
        .ok(
            Method::POST,
            &format!("/characters/{other}/inventory"),
            DM,
            Some(json!({"item_name": "Axe", "item_source": "PHB"})),
        )
        .await;
    let attempts = [
        (Method::GET, format!("/characters/{other}"), None),
        (
            Method::PATCH,
            format!("/characters/{other}"),
            Some(json!({"bonds": "x"})),
        ),
        (
            Method::POST,
            format!("/characters/{other}/inventory"),
            Some(json!({"item_name": "x", "item_source": "x"})),
        ),
        (
            Method::PATCH,
            format!("/inventory/{}", theirs.id),
            Some(json!({"quantity": 5})),
        ),
        (Method::DELETE, format!("/inventory/{}", theirs.id), None),
        (
            Method::POST,
            format!("/characters/{other}/spells"),
            Some(json!({"spell_name": "x", "spell_source": "x", "source_class": "x"})),
        ),
        (
            Method::POST,
            format!("/characters/{other}/level-up"),
            Some(
                json!({"class_name": "Fighter", "class_source": "PHB", "hit_points_method": {"type": "Average"}}),
            ),
        ),
    ];
    for (method, path, body) in attempts {
        let (status, _) = f.send(method.clone(), &path, &token, body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {path}");
    }
    // The catalog is open to players.
    let classes: Vec<wire::Choice> = f.ok(Method::GET, "/catalog/classes", &token, None).await;
    assert!(classes.iter().any(|c| c.name == "Wizard"));
}

#[tokio::test]
async fn catalog_choices() {
    let f = fixture().await;
    let info: wire::ClassLevelInfo = f
        .ok(
            Method::GET,
            "/catalog/classes/Wizard/PHB/level-info",
            DM,
            None,
        )
        .await;
    assert_eq!(info.hit_die, 6);
    assert_eq!(info.caster.as_deref(), Some("full"));
    assert_eq!(info.spellcasting_ability.as_deref(), Some("int"));
    assert_eq!(info.subclass_level, 2);
    let fighter: wire::ClassLevelInfo = f
        .ok(
            Method::GET,
            "/catalog/classes/Fighter/PHB/level-info",
            DM,
            None,
        )
        .await;
    assert_eq!((fighter.hit_die, fighter.caster), (10, None));
    let (status, _) = f
        .send(
            Method::GET,
            "/catalog/classes/Nope/PHB/level-info",
            DM,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let subs: Vec<wire::Choice> = f
        .ok(Method::GET, "/catalog/classes/Wizard/subclasses", DM, None)
        .await;
    assert!(!subs.is_empty());
    let spells: Vec<wire::Choice> = f
        .ok(
            Method::GET,
            "/catalog/spells?class=Wizard&max_level=1",
            DM,
            None,
        )
        .await;
    assert!(!spells.is_empty());
    assert!(spells.iter().all(|s| s.level.unwrap() <= 1));
    assert!(spells.iter().any(|s| s.level == Some(0)), "cantrips");
    let items: Vec<wire::Choice> = f
        .ok(Method::GET, "/catalog/items?search=rope", DM, None)
        .await;
    assert!(items.iter().all(|i| i.name.to_lowercase().contains("rope")));
    let few: Vec<wire::Choice> = f.ok(Method::GET, "/catalog/items?search=r", DM, None).await;
    assert!(few.is_empty(), "two letters at least");
    let _: Vec<wire::Choice> = f.ok(Method::GET, "/catalog/feats", DM, None).await;
    let _: Vec<wire::Choice> = f
        .ok(Method::GET, "/catalog/optional-features?type=EI", DM, None)
        .await;
}

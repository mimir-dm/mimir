//! MCP over streamable HTTP at `/mcp` (MIMIR-T-0725): the functional
//! sweep of the stdio suite (MIMIR-T-0643) against the server, an active
//! campaign per MCP session, and who may connect (the DM; no token 401, a
//! player link 403; open mode lets anyone in).

use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use mimir_server::config::Config;
use mimir_server::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

const DM: &str = "dm-token-for-mcp-tests";

fn server(token: Option<&str>) -> (Router, tempfile::TempDir, AppState) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir: dir.path().to_path_buf(),
        api_token: token.map(String::from),
        web_dist: None,
        seed_fixture: false,
    };
    mimir_server::prepare(&config).unwrap();
    let state = AppState::new(config);
    (mimir_server::router(state.clone()), dir, state)
}

async fn post(
    router: &Router,
    token: Option<&str>,
    session: Option<&str>,
    body: Value,
) -> (StatusCode, HeaderMap, String) {
    let mut b = Request::builder()
        .method(Method::POST)
        .uri("/mcp")
        // Clients send it; rmcp needs it.
        .header("host", "mimir.test")
        .header("accept", "application/json, text/event-stream")
        .header("content-type", "application/json");
    if let Some(t) = token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    if let Some(s) = session {
        b = b.header("mcp-session-id", s);
    }
    let res = router
        .clone()
        .oneshot(b.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        headers,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

/// The JSON-RPC message of a response: plain JSON, or SSE `data:` lines.
fn rpc(body: &str) -> Value {
    if let Ok(v) = serde_json::from_str::<Value>(body) {
        if v.get("jsonrpc").is_some() {
            return v;
        }
    }
    for line in body.lines() {
        if let Some(data) = line.strip_prefix("data:") {
            if let Ok(v) = serde_json::from_str::<Value>(data.trim()) {
                if v.get("jsonrpc").is_some() {
                    return v;
                }
            }
        }
    }
    panic!("no JSON-RPC message in {body:?}");
}

struct Session<'a> {
    router: &'a Router,
    token: Option<String>,
    id: String,
    next: i64,
}

impl<'a> Session<'a> {
    async fn connect(router: &'a Router, token: Option<&str>) -> (Session<'a>, Value) {
        let (status, headers, body) = post(
            router,
            token,
            None,
            json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
                "protocolVersion": "2025-06-18", "capabilities": {},
                "clientInfo": {"name": "mimir-mcp-http-test", "version": "0"}}}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "initialize: {body}");
        let id = headers
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .expect("a session id")
            .to_string();
        let init = rpc(&body);
        let (status, _, body) = post(
            router,
            token,
            Some(&id),
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED, "initialized: {body}");
        (
            Session {
                router,
                token: token.map(String::from),
                id,
                next: 1,
            },
            init,
        )
    }

    async fn request(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        let (status, _, body) = post(
            self.router,
            self.token.as_deref(),
            Some(&self.id),
            json!({"jsonrpc": "2.0", "id": self.next, "method": method, "params": params}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{method}: {body}");
        rpc(&body)
    }

    /// A tool's result (its text as JSON); a tool error fails.
    async fn call(&mut self, name: &str, args: Value) -> Value {
        let r = self
            .request("tools/call", json!({"name": name, "arguments": args}))
            .await;
        assert!(r.get("error").is_none(), "{name}: {r}");
        assert_ne!(r["result"]["isError"], true, "{name}: {r}");
        serde_json::from_str(r["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
    }

    /// A tool error's text.
    async fn call_error(&mut self, name: &str, args: Value) -> String {
        let r = self
            .request("tools/call", json!({"name": name, "arguments": args}))
            .await;
        assert_eq!(r["result"]["isError"], true, "{name} should fail: {r}");
        r["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string()
    }
}

/// The stdio suite's sweep, over HTTP.
#[tokio::test]
async fn full_interface_exercise_over_http() {
    let (r, _dir, _) = server(Some(DM));
    let (mut s, init) = Session::connect(&r, Some(DM)).await;
    assert_eq!(init["result"]["serverInfo"]["name"], "mimir");
    assert!(init["result"]["instructions"]
        .as_str()
        .unwrap()
        .contains("active campaign"));

    let tools = s.request("tools/list", json!({})).await;
    let list = tools["result"]["tools"].as_array().unwrap();
    assert!(
        list.len() >= 50,
        "the full tool surface, got {}",
        list.len()
    );
    assert!(list.iter().all(|t| t["inputSchema"]["type"] == "object"));

    let res = s
        .call("create_campaign", json!({"name": "E2E Campaign"}))
        .await;
    let campaign_id = res["campaign"]["id"].as_str().unwrap().to_string();
    let res = s.call("get_active_campaign", json!({})).await;
    assert_eq!(res["campaign"]["id"], campaign_id.as_str());

    let res = s.call("create_module", json!({"name": "E2E Module"})).await;
    let module_id = res["module"]["id"].as_str().unwrap().to_string();

    let res = s
        .call(
            "create_homebrew",
            json!({"content_type": "monster", "name": "Winter Wight",
                "data": r#"{"name":"Winter Wight","hp":{"average":90}}"#,
                "cr": "8", "creature_type": "undead", "size": "M"}),
        )
        .await;
    let hb_id = res["monster"]["id"].as_str().unwrap().to_string();

    let res = s
        .call(
            "add_monster_to_module",
            json!({"module_id": module_id, "homebrew_monster_id": hb_id, "count": 2}),
        )
        .await;
    assert_eq!(res["status"], "added");
    let hb_mm = res["module_monster"]["id"].as_str().unwrap().to_string();
    let res = s
        .call(
            "add_monster_to_module",
            json!({"module_id": module_id, "monster_name": "Guard", "monster_source": "MM", "count": 3}),
        )
        .await;
    let cat_mm = res["module_monster"]["id"].as_str().unwrap().to_string();
    let res = s
        .call("get_module_details", json!({"module_id": module_id}))
        .await;
    assert_eq!(res["monsters"].as_array().unwrap().len(), 2);
    for mm in [&hb_mm, &cat_mm] {
        let res = s
            .call(
                "remove_monster_from_module",
                json!({"module_monster_id": mm}),
            )
            .await;
        assert_eq!(res["status"], "removed");
    }

    let res = s
        .call(
            "create_document",
            json!({"module_id": module_id, "title": "Room 1", "document_type": "description", "content": "A cold room."}),
        )
        .await;
    let doc = res["document"]["id"].as_str().unwrap().to_string();
    let res = s
        .call(
            "edit_document",
            json!({"document_id": doc, "search": "cold", "replace": "freezing"}),
        )
        .await;
    assert_eq!(res["status"], "updated");
    let res = s.call("read_document", json!({"document_id": doc})).await;
    assert!(res["document"]["content"]
        .as_str()
        .unwrap()
        .contains("freezing"));

    let res = s
        .call(
            "create_character",
            json!({"name": "E2E NPC", "character_type": "npc"}),
        )
        .await;
    let ch = res["character"]["id"].as_str().unwrap().to_string();
    let res = s
        .call(
            "add_item_to_character",
            json!({"character_id": ch, "item_name": "Dagger", "item_source": "PHB"}),
        )
        .await;
    assert_eq!(res["status"], "added");

    let msg = s
        .call_error(
            "add_monster_to_module",
            json!({"module_id": module_id, "homebrew_monster_id": "no-such-monster"}),
        )
        .await;
    assert!(msg.contains("not found"), "{msg}");
    let _ = s.call_error("get_module_details", json!({})).await;
    // An unknown tool: a JSON-RPC error.
    let r = s
        .request(
            "tools/call",
            json!({"name": "no_such_tool", "arguments": {}}),
        )
        .await;
    assert!(r.get("error").is_some(), "{r}");

    let res = s.call("list_campaigns", json!({})).await;
    assert_eq!(res["campaigns"].as_array().unwrap().len(), 1);
}

/// Each MCP session has its own active campaign.
#[tokio::test]
async fn the_active_campaign_is_the_sessions() {
    let (r, _dir, _) = server(Some(DM));
    let (mut a, _) = Session::connect(&r, Some(DM)).await;
    let (mut b, _) = Session::connect(&r, Some(DM)).await;
    let ra = a.call("create_campaign", json!({"name": "A"})).await;
    let rb = b.call("create_campaign", json!({"name": "B"})).await;
    let active_a = a.call("get_active_campaign", json!({})).await;
    let active_b = b.call("get_active_campaign", json!({})).await;
    assert_eq!(active_a["campaign"]["id"], ra["campaign"]["id"]);
    assert_eq!(active_b["campaign"]["id"], rb["campaign"]["id"]);
}

/// The DM only: no token 401, a player link 403; open mode lets anyone in.
#[tokio::test]
async fn who_may_connect() {
    let (r, _dir, state) = server(Some(DM));
    let init = json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}});
    let (status, _, _) = post(&r, None, None, init.clone()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = post(&r, Some("wrong"), None, init.clone()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // A player link.
    let url = state.config.database_path().to_string_lossy().into_owned();
    let mut conn = mimir_core::db::create_connection(&url).unwrap();
    mimir_core::seed::seed_ui_fixture(&mut conn, &state.config.data_dir).unwrap();
    let pcs = {
        let campaigns = mimir_core::services::CampaignService::new(&mut conn)
            .list(false)
            .unwrap();
        mimir_core::services::CharacterService::new(&mut conn)
            .list_pcs(&campaigns[0].id)
            .unwrap()
    };
    let token = mimir_core::services::PlayerLinkService::new(&mut conn)
        .issue(&pcs[0].id)
        .unwrap();
    let (status, _, _) = post(&r, Some(&token), None, init.clone()).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Open mode.
    let (open, _dir2, _) = server(None);
    let (mut s, _) = Session::connect(&open, None).await;
    let res = s.call("list_campaigns", json!({})).await;
    assert!(res["campaigns"].is_array());
}

/// The adventure tools of the server (MIMIR-T-0739).
#[tokio::test]
async fn an_adventure_becomes_a_campaign_over_mcp() {
    let (r, dir, state) = server(Some(DM));
    let url = state.config.database_path().to_string_lossy().into_owned();
    let mut conn = mimir_core::db::create_connection(&url).unwrap();
    mimir_core::seed::seed_ui_fixture(&mut conn, dir.path()).unwrap();
    let (mut s, _) = Session::connect(&r, Some(DM)).await;
    let tools = s.request("tools/list", json!({})).await;
    let names: Vec<&str> = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"preview_adventure_campaign")
            && names.contains(&"create_campaign_from_adventure")
    );

    let plan = s
        .call("preview_adventure_campaign", json!({"source": "MFA"}))
        .await;
    assert_eq!(plan["parts"].as_array().unwrap().len(), 4);
    let made = s
        .call(
            "create_campaign_from_adventure",
            json!({"source": "MFA", "name": "MCP Fen", "parts": [{"index": 3, "role": "skip"}]}),
        )
        .await;
    assert_eq!(made["modules"], 2);
    assert_eq!(made["maps"], 1);
    let active = s.call("get_active_campaign", json!({})).await;
    assert_eq!(active["campaign"]["id"], made["campaign_id"]);
    // And back out as a homebrew adventure (MIMIR-T-0740).
    let file = s
        .call(
            "export_adventure_file",
            json!({"source": "McpFen", "name": "MCP Fen", "level_start": 1, "level_end": 3}),
        )
        .await;
    assert_eq!(file["adventure"][0]["id"], "McpFen");
    assert_eq!(
        file["adventureData"][0]["data"].as_array().unwrap().len(),
        3
    );
    // A pack out to a folder, and in again as a new campaign (MIMIR-T-0741).
    let out = tempfile::tempdir().unwrap();
    let written = s
        .call("export_adventure_pack", json!({"output_path": out.path().to_string_lossy(), "source": "McpFen", "name": "MCP Fen"}))
        .await;
    let path = written["path"].as_str().unwrap().to_string();
    assert!(path.ends_with("McpFen.mimir-adventure.zip"));
    let imported = s
        .call(
            "import_adventure_pack",
            json!({"path": path, "name": "MCP Fen again"}),
        )
        .await;
    assert_eq!(imported["campaign_name"], "MCP Fen again");
    let active = s.call("get_active_campaign", json!({})).await;
    assert_eq!(active["campaign"]["id"], imported["campaign_id"]);
}

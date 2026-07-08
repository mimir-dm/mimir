//! End-to-end functional test of the MCP server over stdio.
//!
//! Spawns the actual `mimir-mcp` binary and drives it through the real
//! JSON-RPC transport — the same path Claude Desktop uses. This exercises the
//! full stack (transport, dispatch, handlers, SQLite) rather than calling
//! handlers in-process, so a hang or protocol break anywhere in the stack
//! fails the test instead of only surfacing in a live client session.
//!
//! Every request has a hard timeout: a missing response is a failure, never a
//! silent wait.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use serde_json::{json, Value};

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(15);

struct McpClient {
    child: Child,
    stdin: ChildStdin,
    responses: Receiver<Value>,
    next_id: i64,
}

impl McpClient {
    /// Spawn the mimir-mcp binary against a fresh temp database.
    fn spawn() -> Self {
        let db_path = std::env::temp_dir().join(format!(
            "mimir-mcp-e2e-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let mut child = Command::new(env!("CARGO_BIN_EXE_mimir-mcp"))
            .env("MIMIR_DATABASE_PATH", &db_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn mimir-mcp binary");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        // Reader thread: one JSON value per line pushed into a channel so
        // every read can carry a timeout.
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<Value>(&line) {
                    if tx.send(v).is_err() {
                        break;
                    }
                }
            }
        });

        Self {
            child,
            stdin,
            responses: rx,
            next_id: 0,
        }
    }

    fn send_raw(&mut self, msg: Value) {
        let mut line = serde_json::to_string(&msg).unwrap();
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .expect("failed to write to server stdin");
        self.stdin.flush().unwrap();
    }

    /// Send a request and wait (with timeout) for the response with that id.
    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        self.send_raw(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));

        let deadline = std::time::Instant::now() + RESPONSE_TIMEOUT;
        loop {
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .unwrap_or_else(|| {
                    panic!("timed out waiting for response to {} (id {})", method, id)
                });
            let msg = self
                .responses
                .recv_timeout(remaining)
                .unwrap_or_else(|_| {
                    panic!("timed out waiting for response to {} (id {})", method, id)
                });
            if msg.get("id").and_then(|v| v.as_i64()) == Some(id) {
                return msg;
            }
            // Skip notifications / unrelated messages
        }
    }

    /// Call an MCP tool; assert transport-level success and no is_error flag;
    /// return the parsed inner JSON payload.
    fn call_tool(&mut self, name: &str, arguments: Value) -> Value {
        let resp = self.request(
            "tools/call",
            json!({"name": name, "arguments": arguments}),
        );
        assert!(
            resp.get("error").is_none(),
            "tool {} returned JSON-RPC error: {}",
            name,
            resp
        );
        let result = &resp["result"];
        assert_ne!(
            result.get("isError").and_then(|v| v.as_bool()),
            Some(true),
            "tool {} returned isError: {}",
            name,
            result
        );
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("tool {} returned no text content: {}", name, result));
        serde_json::from_str(text)
            .unwrap_or_else(|e| panic!("tool {} returned unparseable payload ({}): {}", name, e, text))
    }

    /// Call a tool expecting a tool-level error (isError: true).
    fn call_tool_expect_error(&mut self, name: &str, arguments: Value) -> String {
        let resp = self.request(
            "tools/call",
            json!({"name": name, "arguments": arguments}),
        );
        let result = &resp["result"];
        assert_eq!(
            result.get("isError").and_then(|v| v.as_bool()),
            Some(true),
            "tool {} should have returned isError: {}",
            name,
            result
        );
        result["content"][0]["text"].as_str().unwrap_or("").to_string()
    }

    fn initialize(&mut self) {
        let resp = self.request(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "functional-test", "version": "0.0.1"}
            }),
        );
        assert!(
            resp["result"]["serverInfo"]["name"] == "mimir-mcp",
            "unexpected initialize response: {}",
            resp
        );
        self.send_raw(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Full campaign-management sweep over real stdio: every step is the same
/// wire traffic an agentic client (Claude Desktop) produces.
#[test]
fn full_interface_exercise_over_stdio() {
    let mut client = McpClient::spawn();
    client.initialize();

    // -- Discovery ----------------------------------------------------------
    let tools = client.request("tools/list", json!({}));
    let tool_list = tools["result"]["tools"].as_array().unwrap();
    assert!(
        tool_list.len() >= 50,
        "expected the full tool surface, got {}",
        tool_list.len()
    );

    // -- Campaign -----------------------------------------------------------
    let res = client.call_tool("create_campaign", json!({"name": "E2E Campaign"}));
    let campaign_id = res["campaign"]["id"].as_str().unwrap().to_string();

    let res = client.call_tool("get_active_campaign", json!({}));
    assert_eq!(res["campaign"]["id"], campaign_id.as_str());

    // -- Module -------------------------------------------------------------
    let res = client.call_tool("create_module", json!({"name": "E2E Module"}));
    let module_id = res["module"]["id"].as_str().unwrap().to_string();

    // -- Homebrew monster ----------------------------------------------------
    let res = client.call_tool(
        "create_homebrew",
        json!({
            "content_type": "monster",
            "name": "Winter Wight",
            "data": r#"{"name":"Winter Wight","hp":{"average":90}}"#,
            "cr": "8",
            "creature_type": "undead",
            "size": "M"
        }),
    );
    let hb_id = res["monster"]["id"].as_str().unwrap().to_string();

    // -- Module monsters: the homebrew path that hung the live session -------
    let res = client.call_tool(
        "add_monster_to_module",
        json!({"module_id": module_id, "homebrew_monster_id": hb_id, "count": 2}),
    );
    assert_eq!(res["status"], "added");
    assert_eq!(res["module_monster"]["homebrew_monster_id"], hb_id.as_str());
    let hb_mm_id = res["module_monster"]["id"].as_str().unwrap().to_string();

    // -- Module monsters: catalog path ----------------------------------------
    let res = client.call_tool(
        "add_monster_to_module",
        json!({"module_id": module_id, "monster_name": "Guard", "monster_source": "MM", "count": 3}),
    );
    assert_eq!(res["status"], "added");
    let cat_mm_id = res["module_monster"]["id"].as_str().unwrap().to_string();

    let res = client.call_tool("get_module_details", json!({"module_id": module_id}));
    assert_eq!(res["monsters"].as_array().unwrap().len(), 2);

    // -- Removals (the operations that worked in the live session) -----------
    for mm_id in [&hb_mm_id, &cat_mm_id] {
        let res = client.call_tool(
            "remove_monster_from_module",
            json!({"module_monster_id": mm_id}),
        );
        assert_eq!(res["status"], "removed");
    }
    let res = client.call_tool("get_module_details", json!({"module_id": module_id}));
    assert_eq!(res["monsters"].as_array().unwrap().len(), 0);

    // -- Documents ------------------------------------------------------------
    let res = client.call_tool(
        "create_document",
        json!({"module_id": module_id, "title": "Room 1", "document_type": "description", "content": "A cold room."}),
    );
    let doc_id = res["document"]["id"].as_str().unwrap().to_string();

    let res = client.call_tool(
        "edit_document",
        json!({"document_id": doc_id, "search": "cold", "replace": "freezing"}),
    );
    assert_eq!(res["status"], "updated");

    let res = client.call_tool("read_document", json!({"document_id": doc_id}));
    assert!(res["document"]["content"].as_str().unwrap().contains("freezing"));

    // -- Characters -------------------------------------------------------------
    let res = client.call_tool(
        "create_character",
        json!({"name": "E2E NPC", "character_type": "npc"}),
    );
    let char_id = res["character"]["id"].as_str().unwrap().to_string();

    let res = client.call_tool(
        "add_item_to_character",
        json!({"character_id": char_id, "item_name": "Dagger", "item_source": "PHB"}),
    );
    assert_eq!(res["status"], "added");

    // -- Tool-level errors come back as isError, not hangs ------------------------
    let msg = client.call_tool_expect_error(
        "add_monster_to_module",
        json!({"module_id": module_id, "homebrew_monster_id": "no-such-monster"}),
    );
    assert!(msg.contains("not found"), "unexpected error text: {}", msg);

    let _ = client.call_tool_expect_error("get_module_details", json!({}));

    // -- Server is still responsive after errors -----------------------------------
    let res = client.call_tool("list_campaigns", json!({}));
    assert_eq!(res["campaigns"].as_array().unwrap().len(), 1);
}

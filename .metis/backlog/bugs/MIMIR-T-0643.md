---
id: mcp-functional-test-suite-desktop
level: task
title: "MCP functional test suite + Desktop hang diagnosis"
short_code: "MIMIR-T-0643"
created_at: 2026-07-07T22:31:24.298334+00:00
updated_at: 2026-07-07T22:31:24.298334+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# MCP functional test suite + Desktop hang diagnosis

## Objective **[REQUIRED]**

Two-part: (1) diagnose the 2026-07-07 Claude Desktop hangs on `add_monster_to_module`
with `homebrew_monster_id`; (2) deliver a functional test suite exercising the full
MCP interface from the server down.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [ ] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: single user (Claude Desktop MCP session against production DB)
- **Reproduction Steps**:
  1. Long-lived Claude Desktop session with mimir-mcp (process spawned 2026-07-07 00:18)
  2. Several tool calls succeed (get_module_details, remove_monster_from_module)
  3. `add_monster_to_module` with `homebrew_monster_id` times out (~4 min), and all subsequent calls also time out
- **Expected vs Actual**: call should return `status: added` in ms; instead client times out and session appears dead

## Diagnosis findings (2026-07-07)

**The repo code is exonerated at every layer:**
- Handler + DAL are plain synchronous Diesel; unit tests of both add paths (catalog + homebrew) pass in ms.
- End-to-end repro with the **deployed binary** (`~/.local/bin/mimir-mcp`, built 00:18 from c871577) over real stdio JSON-RPC against a **copy of the production DB**, using the exact failing arguments (Winter Wight `037532b3` → Module 01 `67899232`): succeeds instantly.
- The deployed binary is current (symlink `/usr/local/bin/mimir-mcp` → `~/.local/bin/mimir-mcp`); Claude Desktop config runs `mimir-mcp` from PATH against the production DB.

**Evidence the failure is in the live Desktop↔server session, not the code:**
- The hung calls never executed: production `module_monsters` has **zero** rows with `homebrew_monster_id` set. Inserts are ms-fast, so a request that reached the handler would have left a row even if the response was lost.
- The live server process (PID 59765, child of Claude.app `disclaimer` helper) has 14h uptime — it was never restarted and never crashed; thread sample is indistinguishable from a healthy idle server (all threads parked; same profile as a fresh idle instance).
- Production DB contains duplicate catalog rows from earlier sessions (Winter Wight ×3 in Module 05 with quantities 4/8/8; Lady Rime ×2, Guard ×2 in Module 06; etc.) — the executed-but-response-lost-then-retried signature. **Request/response loss predates the homebrew calls and is not homebrew-specific.**

**Conclusion (definitive, from `~/Library/Logs/Claude/mcp-server-mimir.log`):** the binary is fully exonerated. The log shows every request that ever reached the server was answered in 3–11 ms across the entire 21h process lifetime, zero crashes. The "hung" `add_monster_to_module` calls have **no "Message from client" log entry at all** — Claude Desktop's client layer never wrote them to the server's stdin after the session went stale (last delivered request 2026-07-07T13:54Z). Desktop's "server failing" banners were misattributed client-side timeouts. After the Desktop restart at 2026-07-08T01:28Z the same calls succeeded in ms, and production now contains the properly linked homebrew row (Winter Wight ×3 → Module 01 via `homebrew_monster_id`). Root cause: Claude Desktop client bug (stale renderer↔MCP-client transport on long-lived sessions), not a Mimir bug.

## Remaining work

- [x] 2026-07-07: Roster cleanup executed via supervised SQL (backup at scratchpad `mimir-backup-pre-roster-cleanup-20260707-215200.db`): 13 placeholder/duplicate rows deleted, 14 rows re-linked in place to homebrew via UPDATE (notes/quantities preserved). Verified: 26 rows, 0 dangling homebrew refs. Six pending placeholders kept (M04 Polar Bear/Specter/Water Elemental, M05 Ghoul/Hobgoblin/Hobgoblin Captain) plus M06 Guard ×4 (stock MM). Open question: M04 still has 4 "Reskin as" placeholders (Dire Wolf ×2, Helmed Horror ×2, Scout ×3, Shield Guardian ×1) duplicating adjacent homebrew-linked entries — omitted from the approved Desktop plan; awaiting DM decision to remove.
- [ ] If hangs recur after a Desktop restart: capture the live session (sample PID during hang, check whether the request reached the server via DB side-effects)
- [ ] Consider adding `PRAGMA busy_timeout` in `mimir-core/src/db.rs` `configure_connection` — currently lock contention returns immediate SQLITE_BUSY errors; a small timeout would make concurrent app+MCP writes robust

## Acceptance Criteria **[REQUIRED]**

- [x] Every published MCP tool is exercised by a functional test (happy path or defined error path)
- [x] `add_monster_to_module` covered for both catalog and homebrew paths, with a 10s hang guard
- [x] End-to-end stdio test spawns the real binary and drives JSON-RPC (transport layer covered)
- [x] Full workspace test suite green (`angreal test unit`: ~550 tests pass)
- [ ] Duplicate production rows cleaned up

## Status Updates **[REQUIRED]**

- 2026-07-07: Diagnosis complete (see findings). Test suite delivered:
  - `crates/mimir-mcp/src/handler.rs` — added functional tests: module monsters (both paths + validation), character inventory lifecycle, character spells lifecycle, level-up, document reorder, campaign sources, add_item_to_module (unimplemented error), mapgen presets/validate/generate, map error paths, export→preview→import roundtrip. 45 lib tests pass.
  - `crates/mimir-mcp/tests/functional_stdio.rs` — new e2e test spawning the real binary over stdio: initialize → tools/list → full campaign sweep including the exact homebrew add that hung live; every request has a 15s timeout so a hang fails the test. Passes in 0.72s.
  - Changes uncommitted on `main` pending review.

---
id: standalone-cli-binary-and-mcp-tool
level: task
title: "Standalone CLI binary and MCP tool integration"
short_code: "MIMIR-T-0577"
created_at: 2026-03-11T21:23:36.497095+00:00
updated_at: 2026-03-11T23:05:43.415564+00:00
parent: MIMIR-I-0058
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0058
---

# Standalone CLI binary and MCP tool integration

## Parent Initiative

[[MIMIR-I-0058]]

## Objective

Create the two user-facing entry points for the mapgen system: a standalone CLI binary (`mimir-mapgen` or similar) and MCP tool integration in `mimir-mcp`. These are the only UX surfaces — there is no Mimir desktop UI for this feature.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Standalone CLI binary: `mimir-mapgen generate <config.yaml> -o <output.dungeondraft_map>`
- [ ] CLI supports `--seed <u64>` override (takes precedence over config file seed)
- [ ] CLI supports `--preset <name>` to generate from a biome preset with no config file
- [ ] CLI supports `--validate <config.yaml>` to check config without generating
- [ ] CLI prints meaningful progress output (stages completed, timing)
- [ ] CLI returns appropriate exit codes (0 success, 1 validation error, 2 generation error)
- [ ] MCP tool `generate_map` accepts config YAML as string input, returns base64-encoded `.dungeondraft_map` or writes to path
- [ ] MCP tool `list_presets` returns available biome presets with descriptions
- [ ] MCP tool `validate_config` checks config and returns structured errors/warnings
- [ ] Binary builds as part of workspace (`cargo build -p mimir-mapgen`)
- [ ] Integration tests: CLI end-to-end with sample configs producing valid `.dungeondraft_map` files

## Implementation Notes

### Technical Approach

**CLI binary** (`crates/mimir-mapgen/src/main.rs` or `src/bin/cli.rs`):
- Use `clap` for argument parsing
- Subcommands: `generate`, `validate`, `list-presets`
- Reads YAML config from file path, calls pipeline from MIMIR-T-0576
- Writes output `.dungeondraft_map` to specified path (default: `./output.dungeondraft_map`)

**MCP integration** (new tool handlers in `crates/mimir-mcp/src/tools/`):
- `generate_map`: Takes `config_yaml` (string) and optional `output_path`. Runs pipeline, returns result.
- `list_presets`: No input, returns JSON array of `{ name, description, default_size }`.
- `validate_config`: Takes `config_yaml`, returns `{ valid: bool, errors: [], warnings: [] }`.

**Binary packaging**:
- Separate `[[bin]]` target in the `mimir-mapgen` crate Cargo.toml
- Not a sidecar of the Tauri app — fully standalone
- Could optionally also be added to `mimir-mcp` as additional tools if the MCP server is running

### Dependencies
- MIMIR-T-0569 (crate scaffold — workspace integration)
- MIMIR-T-0576 (config parsing pipeline — provides the `generate()` entry point)
- MIMIR-T-0570 (map format writer — serializes final output)

### Risk Considerations
- MCP tool returning large base64 blobs may hit message size limits in some MCP clients — offer file output as primary, base64 as fallback
- CLI binary name collision: check that `mimir-mapgen` doesn't conflict with existing tools

## Status Updates

### Session 1 - 2026-03-11
- Wired up CLI main.rs with full implementations for all 3 subcommands:
  - `generate` — accepts config file or `--preset`, `--seed` override, `-o` output path
  - `validate` — checks YAML config and reports errors
  - `list-presets` — shows available biome presets with descriptions
- CLI returns proper exit codes: 0 success, 1 validation/config error, 2 generation error
- CLI prints progress stats (objects placed, paths, water polygons, contours, timing)
- Created MCP mapgen tools module (`crates/mimir-mcp/src/tools/mapgen.rs`):
  - `generate_map` — accepts config_yaml or preset, writes to output_path
  - `list_map_presets` — returns preset names, descriptions, default sizes
  - `validate_map_config` — validates YAML and returns structured errors
- Registered 3 new tools in handler.rs (tool definitions + routing)
- Added mimir-mapgen + serde_yaml dependencies to mimir-mcp Cargo.toml
- Updated EXPECTED_TOOLS list (68 → 71)
- Created sample_config.yaml test fixture
- Created 8 CLI integration tests (all passing):
  - list-presets, generate from preset, generate from YAML, deterministic output, validate valid config, no args fails, both config+preset fails, unknown preset fails
- All tests pass: 88 mapgen (78 unit + 2 format + 8 CLI) + 32 mimir-mcp
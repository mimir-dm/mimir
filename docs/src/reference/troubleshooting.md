# Troubleshooting

Known issues, their causes, and resolutions.

| Symptom | Category | Likely Cause |
|---------|----------|-------------|
| "Database is locked" | [Database](#database-is-locked-errors) | Lingering process holding lock |
| No catalog results | [Database](#missing-catalog-data) | Data not imported or sources not enabled |
| Wrong data showing | [Database](#dev-vs-production-database) | Dev build using dev database |
| Fog doesn't follow walls | [Maps](#fog-of-war-doesnt-follow-walls) | Image file instead of UVTT |
| Tokens misaligned | [Maps](#tokens-dont-snap-to-grid) | Non-standard grid size |
| Map image blank | [Maps](#map-image-doesnt-display) | Asset file missing |
| Player display won't open | [Display](#player-display-window-wont-open) | No map loaded in play mode |
| Player display black | [Display](#player-display-shows-black-screen) | Blackout mode enabled |
| Display on wrong screen | [Display](#display-opens-on-wrong-screen) | Window opened on primary monitor |
| Players see hidden tokens | [Display](#players-see-hidden-tokens) | Token marked visible to players |
| Fog not updating | [Display](#fog-not-updating) | PC tokens hidden or missing from active map |
| No second window | [Display](#no-second-window-appears) | Window failed to open |
| MCP can't find campaigns | [MCP](#claude-code-cant-find-campaigns) | Database path mismatch |
| MCP sidecar won't start | [MCP](#mcp-sidecar-wont-start) | Binary not built |
| Blank character PDF | [PDF](#character-sheet-export-is-blank) | Missing character data |
| App won't launch | [Application](#app-wont-launch) | Check logs for details |

## Database Issues

### "Database is locked" errors

**Cause:** Another process has an exclusive lock on the SQLite database. This can happen if a previous Mimir instance didn't shut down cleanly, or if you're accessing the database with an external tool like `sqlite3` while Mimir is running.

**Resolution:** Close all Mimir windows, quit any lingering `mimir` or `mimir-mcp` processes, then relaunch Mimir.

### Missing catalog data

**Cause:** Source data hasn't been imported, or the desired source books aren't enabled for the campaign.

**Resolution:** Open the Reference browser (header bar) and search for a common monster like "Goblin" to check whether any data exists. If no results at all, import catalog data via Settings. If some results appear but specific content is missing, check Campaign Sources — the source book may not be enabled for this campaign.

### Dev vs production database

**Cause:** Mimir uses separate databases for development and production builds. If you're seeing different data than expected, you may be running a dev build that points to the dev database.

| Build | Database Path (macOS) |
|-------|----------------------|
| Production | `~/Library/Application Support/com.mimir.app/data/mimir.db` |
| Development | `~/Library/Application Support/com.mimir.app/dev/data/mimir.db` |

**Linux paths:**
| Build | Database Path |
|-------|--------------|
| Production | `~/.local/share/com.mimir.app/data/mimir.db` |
| Development | `~/.local/share/com.mimir.app/dev/data/mimir.db` |

**Windows paths:**
| Build | Database Path |
|-------|--------------|
| Production | `%APPDATA%/com.mimir.app/data/mimir.db` |
| Development | `%APPDATA%/com.mimir.app/dev/data/mimir.db` |

**Resolution:** Confirm which build you're running and which database path it uses.

## Maps

### Fog of war controls are missing

**Cause:** The map was uploaded as a plain image (PNG/JPG) rather than a UVTT file. Image maps don't contain wall data, so fog of war is not available at all — the Fog and LOS controls only appear for UVTT maps.

**Resolution:** Re-upload the map in UVTT format (`.dd2vtt` or `.uvtt`) if available; only UVTT files include the wall geometry needed for fog of war. On image maps, use Reveal Map and per-token Hide from Players instead.

### Tokens don't snap to grid

**Cause:** Grid alignment mismatch. Image files default to 70 pixels per grid square. If the map image uses a different grid size, tokens won't align.

**Resolution:** Use UVTT format maps, which embed precise grid data. For image maps, ensure the grid squares are approximately 70 pixels wide.

### Map image doesn't display

**Cause:** The map asset file may have been moved or deleted from the assets directory.

**Resolution:** Check that the file exists at `~/Library/Application Support/com.mimir.app/assets/` (or the platform-equivalent). If the asset is missing, re-upload the map.

## Player Display

### Player display window won't open

**Cause:** The player display is a separate Tauri window. It requires the main application to be running and a map to be loaded in play mode.

**Resolution:** Enter Play Mode from a module (click the play button on a module), load a map in the play view, then click the player display button in the play mode toolbar.

### Player display shows black screen

**Cause:** Blackout mode is enabled, or no map has been sent to the display.

**Resolution:** Check the blackout toggle in the play mode toolbar — if active, toggle it off. Ensure a map is loaded and has been sent to the player display.

### Player display is out of sync

**Cause:** The display viewport hasn't been updated after the DM moved the view.

**Resolution:** The DM viewport updates are sent to the player display via Tauri events. If the display falls behind, toggle the player display closed and reopen it.

### Display opens on wrong screen

**Cause:** The operating system opens the window on the primary (or last-used) monitor rather than the player-facing screen.

**Resolution:** Drag the window to the correct screen, then maximize it after moving.

### Players see hidden tokens

**Cause:** Tokens intended to be hidden are marked visible to players.

**Resolution:** Check token visibility settings; ensure hidden tokens are not marked visible to players.

### Fog not updating

**Cause:** PC tokens are hidden or not placed on the active map, or the map lacks UVTT wall data for wall occlusion.

**Resolution:** Verify PC tokens are visible and placed on the active map. Confirm UVTT wall data loaded for wall occlusion.

### No second window appears

**Cause:** The player display window failed to open.

**Resolution:** Click Player Display again; if it still doesn't appear, restart Play Mode.

## MCP Server

### Claude Code can't find campaigns

**Cause:** The MCP server connects to a database path that may differ from what the main app uses.

**Resolution:** Verify the database path — the MCP server uses `MIMIR_DATABASE_PATH` if set, otherwise platform defaults. If you're using a non-standard database location, set the environment variable (`export MIMIR_DATABASE_PATH=/path/to/your/mimir.db`). Also ensure you've called `set_active_campaign` — most MCP tools require an active campaign.

### MCP sidecar won't start

**Cause:** The sidecar binary may not be built or may be missing from the expected path.

**Resolution:** Check that the binary exists at `crates/mimir/binaries/mimir-mcp-{target-triple}` (e.g., `mimir-mcp-aarch64-apple-darwin`). Rebuild with `bash scripts/build-sidecar.sh`. If running from a release build, the sidecar should be bundled automatically.

## PDF Export

### Character sheet export is blank

**Cause:** The character may not have enough data populated (class, level, ability scores).

**Resolution:** Ensure the character has at least a name, one class with a level, and ability scores set. The PDF template requires these fields to render.

### PDF export is slow

**Cause:** Typst compilation for complex documents (especially those with many monster stat blocks or maps) takes time.

**Resolution:** This is expected for large exports. Campaign document PDFs with many pages take longer than individual character sheets.

## Application

### App won't launch

**Cause:** Could be a missing dependency, corrupted installation, or port conflict.

**Resolution:** Launch from the terminal to see error output, and check logs at:
- **macOS:** `~/Library/Application Support/com.mimir.app/logs/`
- **Linux:** `~/.local/share/com.mimir.app/logs/`
- **Windows:** `%APPDATA%/com.mimir.app/logs/`

## Log Files

Mimir writes logs to platform-specific directories. Default log level is `mimir=info,mimir_core=info`.

Override with the `RUST_LOG` environment variable:

```bash
RUST_LOG=mimir=debug,mimir_core=trace cargo run -p mimir
```

## See Also

- [Development Setup](../developer/DEVELOPMENT.md) — For build and development issues
- [Environment Variables](./environment-variables.md) — All configuration options
- [MCP Server Reference](./mcp-server.md) — MCP setup and tools

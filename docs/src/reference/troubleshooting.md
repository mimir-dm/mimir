# Troubleshooting

Known problems, their causes and their fixes.

| Symptom | Section |
|---------|---------|
| "The server did not answer. Try again." | [Server](#the-server-does-not-answer) |
| "The server refused this token." | [Sign-in](#the-server-refuses-the-token) |
| Anyone can open Mimir without a token | [Sign-in](#mimir-opens-without-a-sign-in-page) |
| "This player link no longer works." | [Players](#a-player-link-no-longer-works) |
| No catalog results | [Catalog](#no-catalog-results) |
| A book is not in the reader | [Catalog](#a-book-is-not-in-the-reader) |
| The map upload fails | [Maps](#the-map-upload-fails) |
| Tokens do not fit the squares | [Maps](#tokens-do-not-fit-the-squares) |
| Fog does not stop at walls | [Maps](#fog-does-not-stop-at-walls) |
| "Waiting for the DM to show a map." | [Display](#the-display-waits-for-a-map) |
| The display shows "Paused" | [Display](#the-display-shows-paused) |
| The display is all covered | [Display](#the-display-is-all-covered) |
| Players see a hidden token | [Display](#players-see-a-token) |
| Claude cannot connect | [MCP](#claude-cannot-connect) |
| A PDF is slow or the fonts look wrong | [PDF](#pdf-export-is-slow-or-the-fonts-look-wrong) |

## Server

### The server does not answer

**Cause:** The server does not run, or the browser cannot reach its address.

**Fix:**
1. Check that the server runs. Open `<server>/healthz`. It must answer `ok`.
2. Open `<server>/readyz`. It must answer `ready`. If not, the database does not open: read the server log.
3. Check the address. A server with the default `MIMIR_BIND` (`127.0.0.1:8740`) accepts connections from its own computer only. Set `MIMIR_BIND=0.0.0.0:8740` to serve other devices.
4. Click **Retry** on the error page.

### The server does not start

**Cause:** A setting is wrong, or the port is in use.

**Fix:** Read the first line of the output.
- "MIMIR_BIND is not an address": fix `MIMIR_BIND`, for example `0.0.0.0:8740`.
- "MIMIR_SEED must be empty or "fixture"": unset `MIMIR_SEED`.
- "cannot listen on …": another program uses the port. Stop it, or choose another port.
- "cannot create …": the server cannot write to `MIMIR_DATA_DIR`. Fix the permissions.

## Sign-in

### The server refuses the token

**Cause:** The token is not the `MIMIR_API_TOKEN` of the server.

**Fix:** Type the token again. Check the value of `MIMIR_API_TOKEN` on the server. A token has no spaces at the start or the end.

### Mimir opens without a sign-in page

**Cause:** The server has no `MIMIR_API_TOKEN`. It is in open mode. **Settings**, **About** shows **Sign-in: Open**.

**Fix:** Set `MIMIR_API_TOKEN` and start the server again. See [Environment Variables](./environment-variables.md#mimir_api_token).

## Players

### A player link no longer works

**Cause:** The DM revoked the link or made a new one.

**Fix:** Open the **PCs** tab, click **Player link** and click **Make a link**. Give the new link to the player.

### A player cannot see another sheet

**Cause:** Its player does not share it.

**Fix:** That player turns on **Share this sheet with the party (read-only)** in **Details**.

## Catalog

### No catalog results

**Cause:** No source data is imported.

**Fix:**
1. Open **Settings**, **Books**. Check the **Sources** table.
2. If it is empty, import a data archive. See [Manage Source Books](../how-to/campaigns/manage-sources.md).
3. If a book is missing, import an archive that has it. Mimir imports only core and supplement books.

### A book is not in the reader

**Cause:** The archive had no text for the book. **Readable** shows **No**.

**Fix:** Import a data archive that has the book text.

## Maps

### The map upload fails

**Cause:** The file is not a supported map, or it is too large.

**Fix:** Read the reason in the dialog. Use a UVTT file (`.dd2vtt`, `.uvtt`) or an image (PNG, JPEG, WebP). A `.dungeondraft_map` file does not work: export it from Dungeondraft as Universal VTT.

### Tokens do not fit the squares

**Cause:** The grid size of an image map is wrong.

**Fix:** Delete the map and upload it again with the right **Grid: pixels a square**. See [Set the Grid](../how-to/maps/configure-grid.md).

### Fog does not stop at walls

**Cause:** The map is an image. It has no walls.

**Fix:** Upload the UVTT version of the map.

### A map image does not show

**Cause:** The image file is missing from `<data dir>/assets/`.

**Fix:** Upload the map again. When you move a server, copy the whole data directory.

## Display

### The display waits for a map

**Cause:** No map is shown. After a server restart, no map is shown.

**Fix:** On the DM map, click **Show to players**.

### The display shows "Paused"

**Cause:** Blackout is on.

**Fix:** On the DM map, click **End blackout**.

### The display is all covered

**Cause:** Fog is on and no PC token is on the map, or the PC tokens are hidden.

**Fix:** Place PC tokens. Check that their **Players see it** is on.

### Players see a token

**Cause:** The token has **Players see it** on, and a PC can see it.

**Fix:** Click the token on the DM map. Turn off **Players see it**.

### The display does not follow the DM

**Cause:** The live connection broke, for example after the device slept.

**Fix:** Reload the display page.

## MCP

### Claude cannot connect

**Cause:** The address or the token is wrong.

**Fix:**
- A 401 answer: the token is missing or wrong. Use the DM token in `Authorization: Bearer <token>`.
- A 403 answer: the token is a player link. Use the DM token.
- Check the address: `<server>/mcp`.
- Most tools need an active campaign: call `set_active_campaign` first.

See [MCP Server](./mcp-server.md).

## PDF

### PDF export is slow or the fonts look wrong

**Cause:** The server renders each PDF with Typst. A large campaign or a tiled map takes time. Typst uses the fonts installed on the server.

**Fix:** Wait for the download. For the best result, install Inter and JetBrains Mono on the server. Without them, Typst falls back to other fonts.

## Logs

The server logs to standard output. Set `RUST_LOG` for more detail, for example `RUST_LOG=debug`. See [Environment Variables](./environment-variables.md#rust_log).

## See Also

- [Run the Server](../how-to/run-server.md)
- [Environment Variables](./environment-variables.md)
- [MCP Server Reference](./mcp-server.md)
- [Development Setup](../developer/DEVELOPMENT.md)

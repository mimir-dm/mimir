# Mimir Architecture

Architecture documentation following the [C4 model](https://c4model.com/): Context, Containers, Components, Code.

## Level 1: System Context

Mimir is a self-hosted web app for D&D 5e campaign management. It has no cloud dependency: all data stays on the server that runs it.

```
   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
   │ Dungeon      │  │ Players      │  │ Claude Code  │  │ Dungeondraft │
   │ Master       │  │ (own devices,│  │ (MCP client) │  │ (map tool)   │
   │ (browser)    │  │  TV display) │  │              │  │              │
   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
          │ DM token        │ player link     │ DM token        │ .dd2vtt /
          ▼                 ▼                 ▼                 │ .uvtt files
   ┌─────────────────────────────────────────────────┐          │
   │                 mimir-server                    │◀─────────┘ (upload)
   │   web app · /api/v1 · /ws · /mcp                │
   └────────────────────────┬────────────────────────┘
                            ▼
                ┌───────────────────────┐
                │ data directory:       │
                │ SQLite DB + assets/   │
                └───────────────────────┘
```

**Users:**
- **The DM** prepares and runs campaigns in the browser, signed in with the DM token.
- **Players** open a player link on their own device. The table TV shows the player display.

**External systems:**
- **Claude Code**: manages campaigns through MCP at `/mcp`, with the DM token.
- **Dungeondraft**: map tool. Mimir uploads UVTT maps; `mimir-mapgen` writes `.dungeondraft_map` files.
- **5etools**: the source of the catalog data, imported by the DM in **Settings** (`crates/mimir-core/src/import/`).

## Level 2: Containers

```
┌──────────────────────────────────────────────────────────────┐
│ Browser                                                       │
│  mimir-web (Leptos CSR, wasm32)  +  mimir-editor.js (TipTap)  │
└───────────────┬───────────────────────────────┬──────────────┘
                │ JSON over HTTPS (/api/v1)     │ WebSocket (/ws)
┌───────────────▼───────────────────────────────▼──────────────┐
│ mimir-server (axum, tokio)                                    │
│  auth · API routes · live hub · MCP (rmcp) · static web app   │
│  uses: mimir-core, mimir-wire, mimir-print, mimir-render,     │
│        mimir-mcp                                              │
└───────────────┬──────────────────────────────────────────────┘
                │ Diesel
┌───────────────▼──────────────────────────────────────────────┐
│ SQLite (WAL) data/mimir.db   ·   assets/ (maps, tokens, images)│
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ mimir-mapgen: standalone CLI and library; no database         │
└──────────────────────────────────────────────────────────────┘
```

### Container Responsibilities

| Container | Technology | Purpose |
|-----------|-----------|---------|
| **mimir-web** | Rust, Leptos 0.8 (CSR), Aurora, trunk | The web app: DM pages, DM map, player display, player page |
| **Editor bundle** | TipTap 3, esbuild | The rich markdown editor, driven from Rust through wasm-bindgen |
| **mimir-server** | Rust, axum 0.8, tokio, rmcp | Auth, REST API, live socket, MCP endpoint, serves the web app |
| **mimir-wire** | Rust, serde | The JSON types between server and web app (builds for wasm) |
| **mimir-core** | Rust, Diesel | Models, services, data access, migrations, catalog import, seed fixture |
| **mimir-print** | Rust, Typst | PDF generation |
| **mimir-render** | Rust | 5etools JSON to HTML, for the catalog, the reader and homebrew previews |
| **mimir-mcp** | Rust | The MCP tool registry, over the core services |
| **mimir-mapgen** | Rust, noise, clap | Procedural map generation |
| **SQLite** | SQLite 3, WAL mode | Campaigns, characters, maps, play state, the D&D catalog |

### Crate Dependency Graph

```
mimir-server ──► mimir-core
     │      ──► mimir-wire
     │      ──► mimir-print
     │      ──► mimir-render
     └────────► mimir-mcp ──► mimir-core
                         └──► mimir-mapgen

mimir-web ──► mimir-wire
          └─► mimir-render

(mimir-core, mimir-wire, mimir-print, mimir-render and mimir-mapgen
 have no other internal dependencies)
```

`mimir-web` depends only on crates that build for `wasm32-unknown-unknown`.

## Level 3: Components

### mimir-server

```
┌──────────────────────────────────────────────────────────────┐
│ mimir-server                                                  │
│                                                               │
│  config.rs   MIMIR_BIND, MIMIR_DATA_DIR, MIMIR_API_TOKEN,     │
│              MIMIR_WEB_DIST, MIMIR_SEED                       │
│  auth.rs     authenticate: DM token (constant-time) or        │
│              player link (hashed) → Caller; dm_only → 403     │
│  api/        route table (routes()) with Access per route;    │
│              handlers per area: campaigns, authoring, maps,   │
│              combat, characters, party, links, catalog,       │
│              reference, homebrew, admin, print, display       │
│  live.rs     /ws: hub of changes; per-client decide()         │
│  mcp.rs      /mcp: streamable HTTP, one handler per session   │
│  web.rs      /api/config; SPA fallback (dist dir, embedded,   │
│              or placeholder)                                  │
│  jobs.rs     background jobs (catalog and image imports)      │
│  print/      PDF assembly for each export                     │
└──────────────────────────────────────────────────────────────┘
```

**Endpoints:**

| Path | Auth | Purpose |
|------|------|---------|
| `/healthz` | none | Liveness |
| `/readyz` | none | Readiness: the database answers |
| `/api/config` | none | Version and sign-in mode, for the app before sign-in |
| `/api/v1/…` | DM or player, per route | The REST API |
| `/ws` | DM or player (`?access_token=`) | Live changes |
| `/mcp` | DM | MCP over streamable HTTP |
| anything else | none | The web app (SPA fallback to `index.html`) |

**Auth** (ADR MIMIR-A-0010). Every API route and the socket name the caller:
- the DM: the DM bearer token, or no token at all in open mode;
- a player: the link token of a player character. Only a SHA-256 hash is stored.

Other tokens get 401. DM routes give a player 403. Player routes check the scope of the player (their campaign, the shown map, their character) in the handler.

**Request flow:**

```
Browser
  │  PATCH /api/v1/tokens/<id>   Authorization: Bearer <token>
  ▼
auth::authenticate → Caller (Dm | Player)
  ▼
auth::dm_only (DM routes) → 403 for a player
  ▼
handler (api/maps.rs)
  │  state.with_db(|conn| MapStateService::new(conn).update_token(…))
  ▼
mimir-core service → DAL → SQLite
  ▼
convert to mimir-wire type → JSON
  ▼
live hub: Change::Map { campaign, map, part: Tokens }
  → each socket decides: DM gets a notice; a player gets the new player view
```

**Live socket.** Each client opens `/ws` and watches a campaign. Handlers publish a change on the hub. Each socket task decides, from the role of its client, what to send: a change notice to the DM, the player view of the shown map to a player, a sign-out when a link ends. The display state (shown map, blackout, turn order on display) lives in the hub's memory.

**MCP** (MIMIR-T-0725). `rmcp` serves the `mimir-mcp` registry over streamable HTTP. Each MCP session has its own handler, so the active campaign belongs to the session. The route sits behind `authenticate` and `dm_only`.

**Web app** (ADR MIMIR-A-0009). With the `embed-web` feature, `rust-embed` bakes `crates/mimir-web/dist` into the binary: one artifact. In development, `MIMIR_WEB_DIST` points at the dist directory.

### mimir-web

See [Web App Architecture](./reference/frontend/README.md).

### mimir-core

```
┌─────────────────────────────────────────────────────────┐
│                        mimir-core                        │
│                                                          │
│  models/      Domain types (campaign, catalog, character)│
│  services/    Rules and transactions: Campaign, Module,  │
│               Document, Character, Map, MapState, Token, │
│               Combat, PlayerLink, Party, Archive, Asset, │
│               Homebrew, Source, and the catalog services │
│  dal/         Diesel queries                             │
│  import/      5etools archive import (data and images)   │
│  seed/        The UI fixture (SRD + test campaign)       │
│  templates/   Starting documents of campaigns and modules│
│  migrations/  Diesel migrations, run at start            │
└─────────────────────────────────────────────────────────┘
```

**Service pattern.** Services are structs that borrow a database connection:

```rust
pub struct CampaignService<'a> {
    pub conn: &'a mut SqliteConnection,
}

impl<'a> CampaignService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }
}
```

The server runs each service call on a blocking thread (`state.with_db`). The MCP tools call the same services.

### mimir-mapgen

```
┌──────────────────────────────────────────────────────────┐
│                      mimir-mapgen                         │
│                                                           │
│  CLI (clap)                    Pipeline                   │
│  generate --preset/--output    YAML config                │
│           --seed                 → noise                  │
│  validate                        → terrain blending       │
│  list-presets                    → objects (trees, clutter)│
│                                  → paths and water        │
│  12 biome presets                → rooms / polygons       │
│                                  → .dungeondraft_map      │
└──────────────────────────────────────────────────────────┘
```

No database dependency. YAML in, file out.

### mimir-print

```
┌──────────────────────────────────────────────────────────┐
│                       mimir-print                         │
│                                                           │
│  DocumentBuilder        Sections                          │
│  assembles sections     character sheet, battle card,     │
│  into one Typst doc     spell cards, equipment cards,     │
│                         monster cards, trap cards,        │
│  Typst World            map preview, tiled map,           │
│  (fonts, in-memory      token cutouts, markdown           │
│   files) → PDF                                            │
└──────────────────────────────────────────────────────────┘
```

The server calls it from `crates/mimir-server/src/print/` and answers with `application/pdf`.

## Level 4: Key Design Decisions

| Decision | Reference | Reason |
|----------|-----------|--------|
| One server binary with the web app embedded | ADR MIMIR-A-0009 | One artifact to deploy; the API and the app share an origin, so no CORS |
| DM token and per-character player links | ADR MIMIR-A-0010 | Players use their own devices without the DM's secret; a link is revoked on its own |
| Leptos CSR in Rust | | One language end to end; `mimir-wire` types make an API change a compile error |
| Server-built player view | | The display gets only what players may see; hidden data never leaves the server |
| Denormalized catalog data | | Child entities (subclasses, subraces) embed parent data to avoid joins on read |
| JSON columns | | Variable fields (spell components, monster traits) stay as 5etools JSON |
| Stateful services | | Services borrow `&mut SqliteConnection` for scoped transactions |

### Database

| Category | Tables |
|----------|--------|
| Campaign | campaigns, modules, documents, campaign_sources, campaign_assets, campaign_homebrew_items / _monsters / _spells |
| Character | characters, character_classes, character_spells, character_inventory, character_feats, character_features, character_proficiencies, character_sources |
| Map | maps, token_placements, light_sources, fog_revealed_areas, map_pois, map_traps, module_monsters, module_npcs |
| Play | combat_sessions, combat_entries, player_links, player_notes, sheet_shares |
| D&D catalog | spells, monsters, items, classes, races, backgrounds, feats, conditions, subclasses and more |
| Catalog support | catalog_sources, books, spell_classes, spell_subclasses, item_attunement_classes |

See the [Data Model](../reference/data-model.md).

### Data Paths

| Path | Contents |
|------|----------|
| `$MIMIR_DATA_DIR/data/mimir.db` | The database (default data dir: `./mimir-data`) |
| `$MIMIR_DATA_DIR/assets/` | Map images, token images, catalog images |
| `target/web-dev/` | The dev data of `angreal web serve` (fixture) |

## Directory Structure

```
mimir/
├── Cargo.toml                 # workspace
├── crates/
│   ├── mimir-server/
│   │   ├── src/
│   │   │   ├── main.rs        # config, prepare (dirs, migrations, seed), serve
│   │   │   ├── lib.rs         # router(): /healthz, /readyz, /api, /ws, /mcp, SPA
│   │   │   ├── api/           # route table and handlers
│   │   │   ├── auth.rs  live.rs  mcp.rs  web.rs  jobs.rs  state.rs  config.rs
│   │   │   └── print/         # PDF exports
│   │   └── tests/             # API, auth, players, live, MCP over HTTP
│   ├── mimir-web/
│   │   ├── src/               # the Leptos app
│   │   ├── style/mimir.css
│   │   ├── editor/            # TipTap bundle
│   │   ├── web-e2e/           # Playwright
│   │   ├── index.html
│   │   └── Trunk.toml
│   ├── mimir-wire/src/        # wire types per area
│   ├── mimir-core/
│   │   ├── src/               # models, services, dal, import, seed, templates
│   │   ├── migrations/
│   │   └── tests/
│   ├── mimir-mcp/
│   │   ├── src/tools/         # tool families
│   │   └── plugin/            # Claude Code plugin
│   ├── mimir-print/src/       # builder, sections, map renderer, Typst world
│   ├── mimir-render/src/      # 5etools JSON → HTML
│   └── mimir-mapgen/          # library, CLI, examples
├── docs/                      # mdBook
└── .angreal/                  # tasks: web, test, docs
```

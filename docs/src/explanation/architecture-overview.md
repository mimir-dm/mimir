# Architecture Overview

How Mimir is built and why.

## The Stack

Mimir is a web app that you run on your own server. One Rust program, `mimir-server`, serves the API, a live socket, the MCP endpoint and the web app. The web app is written in Rust too, with [Leptos](https://leptos.dev/), and runs in the browser as WebAssembly.

```
┌──────────────────────────────────────────────┐
│  Browser: mimir-web (Leptos, WebAssembly)    │  DM pages, DM map, player
│  Aurora design system, TipTap editor         │  display, player page
└───────────────┬──────────────────────────────┘
                │ HTTPS: /api/v1 (JSON), /ws (WebSocket)
┌───────────────┴──────────────────────────────┐   ┌─────────────────┐
│  mimir-server (axum)                         │◀──│ Claude (MCP     │
│  auth, API routes, live hub, /mcp, web app   │   │ over HTTP /mcp) │
├──────────────────────────────────────────────┤   └─────────────────┘
│  mimir-core: models, services, data access   │
│  mimir-print (PDF), mimir-mcp (MCP tools)    │
├──────────────────────────────────────────────┤
│  SQLite (WAL mode) + assets directory        │
└──────────────────────────────────────────────┘
```

## Why a Self-Hosted Web App?

- **Your data stays on your server.** No cloud account, no subscription, no telemetry.
- **Every device at the table.** The DM uses a laptop or a tablet. The TV shows the player display. Each player uses their own phone. A browser is all they need.
- **One place for the data.** All devices work on the same database, so there is nothing to sync.
- **Private by default.** Run it on a private network, for example a Tailscale tailnet.

## The Crates

The Cargo workspace has these crates:

### `mimir-server`: the Server

An [axum](https://github.com/tokio-rs/axum) server. It:

- serves the REST API under `/api/v1`. Each route names who may call it: the DM, or the DM and players.
- checks the caller on each request: the DM token, or the link token of a player. Without a configured token, the server is in open mode.
- runs the live socket at `/ws` and tells each client what changed.
- serves MCP over streamable HTTP at `/mcp`, for Claude. DM only.
- serves the web app: from `MIMIR_WEB_DIST` in development, or built into the binary (`embed-web` feature).

### `mimir-web`: the Web App

A Leptos app, rendered in the browser (client-side rendering) and built with [trunk](https://trunkrs.dev/). It uses the Aurora design system for its components and themes. The rich editor is a small TipTap bundle that the app drives through wasm-bindgen.

### `mimir-wire`: the Wire Types

The JSON types between the server and the web app. Both sides compile against them, so a change to the API breaks the build, not the page. It builds for WebAssembly.

### `mimir-core`: Models, Services and Data Access

The heart of the system. Three layers:

- **Models**: the data structures (Campaign, Character, Module, Map, Document) as Rust structs mapped to tables with Diesel.
- **DAL (Data Access Layer)**: the database operations.
- **Services**: the rules. When you make a character, the character service looks up the race and class in the catalog and writes everything in one transaction. The D&D rules live here.

The API and the MCP tools both call these services.

### `mimir-mcp`: the MCP Tools

The registry of MCP tools for AI assistants, over the core services. The server serves it at `/mcp`. The Claude Code plugin is in `crates/mimir-mcp/plugin`.

### `mimir-print`: PDF Generation

Character sheets, cards, documents and maps with the [Typst](https://typst.app/) typesetting engine, inside the server.

### `mimir-render`: Catalog Rendering

Turns 5etools JSON into HTML for the catalog, the book reader and the homebrew preview. The server and the web app share it.

### `mimir-mapgen`: Procedural Map Generation

Makes Dungeondraft maps from YAML or biome presets. It is also a standalone command-line tool.

## Data Flow

A typical change:

```
Browser (mimir-web)
  → PATCH /api/v1/tokens/<id>  { grid_x, grid_y }   (Authorization: Bearer <token>)
    → auth: who is the caller? (DM, or a player and their character)
      → route handler
        → MapStateService (mimir-core)
          → DAL → SQLite
        → JSON answer (mimir-wire type)
      → live hub: "the tokens of this map changed"
  ← every page that watches the campaign reads the change, or gets the new player view
```

Errors use one JSON shape with an error code and its HTTP status.

## The Database

Mimir uses SQLite with Write-Ahead Logging (WAL):

- **No database server.** The database is one file in the data directory. It is easy to back up.
- **WAL mode** allows reads while a write runs.
- **Migrations run at start**, so the schema is always current.

## Security Model

- The **DM token** (`MIMIR_API_TOKEN`) gives full access. Keep it secret.
- A **player link** gives access to one character: its sheet, its notes, the party and the shown map. The server stores only a hash of each link.
- **Open mode** (no token) gives every caller full access. Use it only where no one else can reach the server.

## See Also

- [Screens and Devices](./screens-and-devices.md)
- [Campaigns vs Modules](./campaign-vs-module.md)
- [The Catalog System](./catalog-system.md)
- [Developer Architecture Guide](../developer/ARCHITECTURE.md)

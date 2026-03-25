# Architecture Overview

How Mimir is built and why its architecture looks the way it does.

## The Stack

Mimir is a desktop application built with [Tauri v2](https://v2.tauri.app/), combining a Rust backend with a Vue 3 frontend. This hybrid approach gives you the performance and reliability of native code for data-heavy operations (database queries, PDF generation, map processing) while using web technologies for the UI layer where rapid iteration and rich interactivity matter.

```
┌────────────────────────────────┐
│        Vue 3 Frontend          │  TypeScript, Tailwind CSS
│   (Campaigns, Maps, Sheets)    │
├────────────────────────────────┤
│        Tauri IPC Bridge        │  invoke() / events
├────────────────────────────────┤
│        Rust Backend            │  Commands, Services, DAL
│   (mimir, mimir-core, ...)     │
├────────────────────────────────┤
│        SQLite (WAL mode)       │  Diesel ORM
└────────────────────────────────┘
         ↕ (shared database)
┌────────────────────────────────┐
│     MCP Sidecar (mimir-mcp)    │  Claude Code integration
└────────────────────────────────┘
```

## Why a Desktop App?

Mimir is intentionally local-first:

- **Your data stays on your machine.** No cloud accounts, no subscriptions, no telemetry.
- **Works offline.** Prep sessions at the coffee shop, run games at a venue with spotty wifi.
- **Performance.** SQLite with WAL mode handles concurrent reads without a server process.
- **Multi-window support.** The DM sees the full prep view while players see only what you reveal on a second display — something web apps struggle with.

## The Five Crates

Mimir's Rust backend is organized as a Cargo workspace with five crates, each with a clear responsibility:

### `mimir` — The Tauri Application

The main application crate. Contains ~200 Tauri command handlers that the frontend calls via IPC. Each command is a thin wrapper that validates input, calls the appropriate service, and returns a structured response.

This crate also manages application state: database connections, file paths, and the active campaign.

### `mimir-core` — Models, Services, and Data Access

The heart of the system. Three layers:

- **Models** define the data structures (Campaign, Character, Module, Map, Document) as Rust structs that map directly to database tables via Diesel.
- **DAL (Data Access Layer)** provides low-level CRUD operations — one function per database operation, using Diesel query builders.
- **Services** implement business logic. When you create a character, the CharacterService looks up racial proficiencies from the catalog, calculates starting equipment, and wraps everything in a database transaction. Services are where the D&D rules live.

This separation means the MCP server and the Tauri app share the same business logic — they both call into `mimir-core` services.

### `mimir-mcp` — MCP Sidecar Server

A standalone binary that implements the [Model Context Protocol](https://modelcontextprotocol.io/). It connects to the same SQLite database as the main app and exposes campaign management tools for AI assistants like Claude Code.

Runs as a Tauri sidecar process — launched automatically alongside the main app, communicating over stdio.

### `mimir-mapgen` — Procedural Map Generation

Generates Dungeondraft-compatible maps from YAML configuration or biome presets. Uses Perlin noise for terrain, supports room/corridor generation, and outputs the `.dungeondraft_map` format.

Can be used as a standalone CLI tool or through the MCP server.

### `mimir-print` — PDF Generation

Produces character sheets, spell cards, monster cards, and campaign document exports using the [Typst](https://typst.app/) typesetting engine. Typst was chosen over LaTeX for its speed and Rust-native integration.

## Data Flow

A typical operation flows through these layers:

```
Frontend (Vue)
  → invoke('create_character', { name: "Gandalf", ... })
    → Tauri IPC
      → Command handler (validates input)
        → CharacterService::create()
          → Look up race/class from catalog
          → Calculate proficiencies
          → Transaction:
            → DAL: insert_character()
            → DAL: insert_character_class()
            → DAL: insert_character_proficiencies()
          → Return CharacterResponse
        → ApiResponse { success: true, data: character }
      → Tauri IPC
    → Frontend receives response
  → Pinia store updates
→ Vue reactivity triggers UI update
```

Every backend response uses the same envelope:

```typescript
interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}
```

## The Database

Mimir uses SQLite with Write-Ahead Logging (WAL mode). This is a deliberate choice:

- **No server process.** The database is a single file — easy to back up, move, or inspect.
- **WAL mode** allows concurrent reads while writing, which matters when the MCP sidecar and the main app access the database simultaneously.
- **Embedded migrations** run automatically on startup, so the schema is always current.

Database connections are created on-demand rather than pooled. With SQLite WAL, this is efficient and avoids the complexity of connection pool management.

### Dev vs Production

In development builds, Mimir uses a separate database directory (`dev/data/`) to protect production campaign data from development experiments. This separation is automatic — debug builds use the dev path, release builds use production.

## Type Safety Across the Stack

Rust model structs are the source of truth. The `ts-rs` crate generates TypeScript type definitions from Rust structs, which are written to `frontend/src/types/generated/`. This means the frontend's TypeScript types always match the backend's Rust types — a mismatch is a compile error, not a runtime bug.

## See Also

- [Multi-Window System](./multi-window-system.md) — How the four application windows work together
- [Campaigns vs Modules](./campaign-vs-module.md) — The data hierarchy
- [The Catalog System](./catalog-system.md) — How reference data works
- [Developer Architecture Guide](../developer/ARCHITECTURE.md) — Detailed technical reference

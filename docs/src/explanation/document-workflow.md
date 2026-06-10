# Document Workflow

Why documents in Mimir are organized the way they are, and how they fit into the life of a campaign.

Documents are markdown-based text entries — your DM notebook. The mechanics of creating, editing, reordering, and exporting them are covered in [Manage Documents](../how-to/campaigns/manage-documents.md); this page explains the design behind them.

## Why Two Scopes

Every document belongs to a campaign, and optionally to one module within it. This mirrors the [Two-Board System](./two-board-system.md): content has a natural lifespan, and the scope encodes it.

**Campaign documents** are the long-lived layer — world lore, faction descriptions, house rules, recurring NPC notes. They are written once and consulted across many adventures, so they live where every module can see them.

**Module documents** are the working layer — encounter plans, read-aloud text, location descriptions for one adventure. They are self-contained within their module, so when you prep or run that module, you see exactly the notes that matter and nothing else. When the module is finished, its documents become a record of what happened rather than clutter in your active workspace.

The alternative — one flat pile of notes — forces you to re-sort your own material every session. Scoping does that sorting once, at creation time.

## The Template Philosophy

A new campaign isn't empty: Mimir generates a set of starter documents (Campaign Pitch, World Primer, House Rules, Safety Tools, and others — see the [Data Model](../reference/data-model.md#campaigns) for the full list). These templates encode the campaign-genesis methodology from the [Campaign Framework](../campaign-framework/README.md): the questions worth answering before play begins.

The templates are ordinary documents — rename, rewrite, or delete them freely. Their purpose is to replace a blank page with a prompt, not to impose structure.

## From Prep to Play

Documents flow through the campaign lifecycle:

**During prep**, you write into campaign documents as the world takes shape and into module documents as a specific session approaches. Because everything auto-saves, prep notes are always current — there is no "publish" step between preparing and running.

**During play**, prepared documents are read-only reference material in spirit: each module's auto-created Play Notes document is the surface for in-the-moment tracking (initiative, HP, events), so your carefully written prep is not overwritten by table chaos.

**After play**, observations migrate upward: session outcomes get folded into campaign documents, turning module-level events into campaign-level history. This is the Module → Campaign information flow described in the Two-Board System.

## See Also

- [Manage Documents](../how-to/campaigns/manage-documents.md) — Creating, editing, reordering, exporting
- [Module Documents](../how-to/modules/module-documents.md) — Module-scoped document tasks
- [The Two-Board System](./two-board-system.md) — The structure documents live in

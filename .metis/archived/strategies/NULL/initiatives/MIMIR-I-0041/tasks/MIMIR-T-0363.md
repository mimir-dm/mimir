---
id: v0-5-ui-architecture-and-views
level: task
title: "v0.5 UI Architecture and Views Design"
short_code: "MIMIR-T-0363"
created_at: 2026-01-20T01:13:57.341058+00:00
updated_at: 2026-01-21T16:38:39.122513+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0041
---

# v0.5 UI Architecture and Views Design

## Parent Initiative
[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective
Design the complete UI architecture: application shell, navigation, views, component structure, and routing. The UI is a thin presentation layer over Pinia stores.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] Application shell layout defined
- [ ] All primary views/routes specified
- [ ] Navigation structure documented
- [ ] Component hierarchy for each view
- [ ] Store-to-view mapping clear

## Design Principles

1. **Stores own state**: Views read from stores, never fetch directly
2. **Thin components**: Logic lives in stores, components just render
3. **Consistent patterns**: Similar views share similar structure
4. **Keyboard navigable**: Power users can work without mouse
5. **Responsive sidebar**: Collapsible for more workspace

## Application Shell

```
┌─────────────────────────────────────────────────────────────────┐
│  Title Bar (Tauri draggable)                      [_][□][X]     │
├─────────┬───────────────────────────────────────────────────────┤
│         │  Breadcrumb: Campaign > Module > Document             │
│  Side   ├───────────────────────────────────────────────────────┤
│  bar    │                                                       │
│         │                                                       │
│  [C]    │                    Main Content                       │
│  [M]    │                                                       │
│  [Ch]   │                    (Router View)                      │
│  [D]    │                                                       │
│  [Ma]   │                                                       │
│  [Ca]   │                                                       │
│         │                                                       │
│         ├───────────────────────────────────────────────────────┤
│  ───    │                                                       │
│  [💬]   │                   Chat Panel                          │
│  [⚙]    │                   (Collapsible)                       │
│         │                                                       │
├─────────┴───────────────────────────────────────────────────────┤
│  Status Bar: Active Campaign | Module Count | Character Count   │
└─────────────────────────────────────────────────────────────────┘
```

### Shell Components

```
AppShell.vue
├── TitleBar.vue (Tauri window controls)
├── Sidebar.vue
│   ├── CampaignSwitcher.vue (dropdown)
│   ├── NavSection.vue (repeated)
│   │   └── NavItem.vue
│   └── SidebarFooter.vue (chat toggle, settings)
├── MainArea.vue
│   ├── Breadcrumb.vue
│   └── <RouterView />
├── ChatPanel.vue (slide-in from right or bottom)
└── StatusBar.vue
```

## Navigation Structure

### Sidebar Navigation

| Icon | Label | Route | Description |
|------|-------|-------|-------------|
| 📚 | Campaigns | `/campaigns` | Campaign list/switcher |
| 📖 | Modules | `/modules` | Module list for active campaign |
| 👥 | Characters | `/characters` | PC and NPC list |
| 📄 | Documents | `/documents` | Document browser |
| 🗺️ | Maps | `/maps` | Map list and editor |
| 🔍 | Catalog | `/catalog` | Monster/item/spell search |

### Routes

```typescript
const routes = [
  // Campaign
  { path: '/', redirect: '/campaigns' },
  { path: '/campaigns', component: CampaignList },
  { path: '/campaigns/:id', component: CampaignDetail },
  
  // Module
  { path: '/modules', component: ModuleList },
  { path: '/modules/:id', component: ModuleDetail },
  
  // Character
  { path: '/characters', component: CharacterList },
  { path: '/characters/:id', component: CharacterDetail },
  { path: '/characters/new', component: CharacterCreate },
  
  // Document
  { path: '/documents', component: DocumentList },
  { path: '/documents/:id', component: DocumentEditor },
  { path: '/documents/new', component: DocumentCreate },
  
  // Map
  { path: '/maps', component: MapList },
  { path: '/maps/:id', component: MapEditor },
  
  // Catalog
  { path: '/catalog', component: CatalogSearch },
  { path: '/catalog/monsters/:name/:source', component: MonsterDetail },
  { path: '/catalog/items/:name/:source', component: ItemDetail },
  
  // Settings
  { path: '/settings', component: Settings },
];
```

## View Designs

### Campaign List (`/campaigns`)

```
┌─────────────────────────────────────────────────────────┐
│  Campaigns                            [+ New Campaign]  │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐    │
│  │ 🏰 Curse of Strahd                              │    │
│  │    3 modules · 12 characters · 45 documents     │    │
│  │    Last active: 2 days ago                      │    │
│  └─────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────┐    │
│  │ 🐉 Dragon Heist                                 │    │
│  │    2 modules · 8 characters · 23 documents      │    │
│  │    Last active: 1 week ago                      │    │
│  └─────────────────────────────────────────────────┘    │
│                                                         │
│  [Show Archived]                                        │
└─────────────────────────────────────────────────────────┘
```

### Module List (`/modules`)

```
┌─────────────────────────────────────────────────────────┐
│  Modules in "Curse of Strahd"           [+ New Module]  │
├─────────────────────────────────────────────────────────┤
│  1. Death House                                         │
│     🐲 4 monsters · 📦 6 items · 👤 2 NPCs              │
│                                                         │
│  2. Village of Barovia                                  │
│     🐲 8 monsters · 📦 12 items · 👤 7 NPCs             │
│                                                         │
│  3. Castle Ravenloft                                    │
│     🐲 24 monsters · 📦 18 items · 👤 5 NPCs            │
└─────────────────────────────────────────────────────────┘
```

### Module Detail (`/modules/:id`)

```
┌─────────────────────────────────────────────────────────┐
│  Module 1: Death House                       [Edit] [⋮] │
├──────────────────────┬──────────────────────────────────┤
│  Encounters          │  Documents                       │
│  ──────────────      │  ──────────────                  │
│  🏷️ basement_fight   │  📄 Overview                     │
│    └─ 3× Ghoul       │  📄 Room Descriptions            │
│    └─ 1× Shambling   │  📄 Play Notes                   │
│                      │                                  │
│  🏷️ attic_haunt      │  NPCs                            │
│    └─ 2× Specter     │  ──────────────                  │
│                      │  👤 Rose (quest_giver)           │
│  Treasure            │  👤 Thorn (neutral)              │
│  ──────────────      │                                  │
│  📦 Cloak of Prot.   │                                  │
│  📦 +1 Shortsword    │                                  │
│                      │  [+ Add Monster] [+ Add Item]    │
└──────────────────────┴──────────────────────────────────┘
```

### Character List (`/characters`)

```
┌─────────────────────────────────────────────────────────┐
│  Characters                    [Filter: All ▾] [+ New]  │
├─────────────────────────────────────────────────────────┤
│  Player Characters                                      │
│  ┌─────────────────────────────────────────────────┐    │
│  │ ⚔️ Kira Stoneheart    Fighter 5   Dwarf         │    │
│  │ 🔮 Elara Moonwhisper  Wizard 5    Elf           │    │
│  │ 🗡️ Shade              Rogue 5     Human         │    │
│  └─────────────────────────────────────────────────┘    │
│                                                         │
│  NPCs                                                   │
│  ┌─────────────────────────────────────────────────┐    │
│  │ 👤 Ireena Kolyana     quest_giver  Human        │    │
│  │ 👤 Strahd von Zarovich antagonist  Vampire      │    │
│  │ 👤 Ismark the Lesser  ally         Human        │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### Character Detail (`/characters/:id`)

```
┌─────────────────────────────────────────────────────────┐
│  Strahd von Zarovich                      [Edit] [PDF]  │
├─────────────────────────────────────────────────────────┤
│  Vampire · Lawful Evil                                  │
│  Role: antagonist · Location: Castle Ravenloft          │
├─────────────────────────────────────────────────────────┤
│  Stats                     │  Personality               │
│  ────────                  │  ────────────              │
│  STR 18 (+4)  INT 20 (+5)  │  Traits: Charming but      │
│  DEX 18 (+4)  WIS 15 (+2)  │  cruel, obsessed with      │
│  CON 18 (+4)  CHA 18 (+4)  │  Ireena...                 │
│                            │                            │
│  HP: 144  AC: 16           │  Backstory                 │
│  Speed: 30 ft              │  ────────────              │
│                            │  Once a noble prince...    │
├────────────────────────────┴────────────────────────────┤
│  Inventory                                              │
│  📦 Icon of Ravenloft                                   │
│  📦 Crystal Ball                                        │
│                                                         │
│  Currency: 1,000 gp                                     │
└─────────────────────────────────────────────────────────┘
```

### Document Editor (`/documents/:id`)

```
┌─────────────────────────────────────────────────────────┐
│  Room Descriptions                    [Save] [Export ▾] │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐    │
│  │                                                 │    │
│  │  # Entry Hall                                   │    │
│  │                                                 │    │
│  │  The heavy oak doors creak open to reveal...   │    │
│  │                                                 │    │
│  │  ## Features                                    │    │
│  │  - Dusty chandelier hangs 20 feet above        │    │
│  │  - Portrait of the Durst family on west wall   │    │
│  │                                                 │    │
│  │  ## Hidden                                      │    │
│  │  DC 15 Perception: Secret door behind portrait │    │
│  │                                                 │    │
│  └─────────────────────────────────────────────────┘    │
│                                                         │
│  Markdown · Auto-saved                                  │
└─────────────────────────────────────────────────────────┘
```

### Map Editor (`/maps/:id`)

```
┌─────────────────────────────────────────────────────────┐
│  Dungeon Level 1            [Tokens ▾] [Fog ▾] [Export] │
├─────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────┬─────────────┐    │
│  │                                   │  Tokens     │    │
│  │     [Map Canvas]                  │  ─────────  │    │
│  │                                   │  🐲 Ghoul   │    │
│  │     - Grid overlay                │  🐲 Ghoul   │    │
│  │     - Tokens placed               │  🐲 Ghoul   │    │
│  │     - Fog of war                  │  👤 Kira    │    │
│  │     - Light sources               │  👤 Elara   │    │
│  │                                   │             │    │
│  │                                   │  [+ Token]  │    │
│  └───────────────────────────────────┴─────────────┘    │
│                                                         │
│  Tools: [Select] [Pan] [Reveal] [Light] [Measure]       │
└─────────────────────────────────────────────────────────┘
```

### Catalog Search (`/catalog`)

```
┌─────────────────────────────────────────────────────────┐
│  Catalog Search              [Monsters] [Items] [Spells]│
├─────────────────────────────────────────────────────────┤
│  🔍 [Search...                    ] [CR: Any ▾] [Go]    │
│                                                         │
│  Results for "dragon"                                   │
│  ┌─────────────────────────────────────────────────┐    │
│  │ 🐲 Adult Red Dragon        CR 17  Dragon  MM    │ [+]│
│  │ 🐲 Young Red Dragon        CR 10  Dragon  MM    │ [+]│
│  │ 🐲 Red Dragon Wyrmling     CR 4   Dragon  MM    │ [+]│
│  │ 🐲 Ancient Red Dragon      CR 24  Dragon  MM    │ [+]│
│  └─────────────────────────────────────────────────┘    │
│                                                         │
│  [+] = Add to active module                             │
└─────────────────────────────────────────────────────────┘
```

## Component Library

### Shared Components

```
components/
├── common/
│   ├── Button.vue
│   ├── Input.vue
│   ├── Select.vue
│   ├── Modal.vue
│   ├── Card.vue
│   ├── Badge.vue
│   ├── Tooltip.vue
│   └── Loading.vue
├── layout/
│   ├── PageHeader.vue
│   ├── SplitPane.vue
│   ├── EmptyState.vue
│   └── ListItem.vue
├── domain/
│   ├── CampaignCard.vue
│   ├── ModuleCard.vue
│   ├── CharacterCard.vue
│   ├── MonsterCard.vue
│   ├── ItemCard.vue
│   ├── DocumentListItem.vue
│   └── EncounterGroup.vue
└── editor/
    ├── MarkdownEditor.vue
    ├── MapCanvas.vue
    ├── TokenPalette.vue
    └── FogControls.vue
```

## View-Store Mapping

| View | Primary Store | Secondary Stores |
|------|--------------|------------------|
| CampaignList | useCampaignStore | - |
| CampaignDetail | useCampaignStore | useModuleStore, useCharacterStore |
| ModuleList | useModuleStore | - |
| ModuleDetail | useModuleStore | useCatalogStore |
| CharacterList | useCharacterStore | - |
| CharacterDetail | useCharacterStore | - |
| DocumentList | useDocumentStore | - |
| DocumentEditor | useDocumentStore | - |
| MapEditor | useMapStore | useCharacterStore, useCatalogStore |
| CatalogSearch | useCatalogStore | useModuleStore (for add-to-module) |
| ChatPanel | useChatStore | (all stores via tool results) |

## Dependencies

- Depends on: [[MIMIR-T-0360]] Pinia Store Design
- Depends on: [[MIMIR-T-0362]] LLM Chat Assistant Design (for ChatPanel)

## Investigation Findings (2026-01-21)

### Current Route Structure

**Existing Routes (mimir-dm-bu/mimir-dm/frontend/src/app/router/):**
```
/campaigns                    → CampaignList
/campaigns/:id/dashboard      → CampaignDashboard (tabbed container)
/modules                      → ModuleList
/modules/:id                  → ModuleDetail
/modules/:id/play             → ModulePlayView
/characters                   → CharacterList
/characters/:id               → CharacterSheet
/settings                     → Settings
/players                      → PlayerList
/templates                    → TemplateViews
/sources                      → SourceSearch (catalog)
```

**Gaps vs. v0.5 Design:**
- No top-level `/documents` route (documents in module context only)
- No top-level `/maps` route (maps in module context only)
- `/catalog` should be primary route (currently `/sources`)
- Dashboard uses intermediate container routing

### Current App Shell

**Existing Structure:**
```
App.vue
├── AppHeader.vue (top bar with campaign switcher)
└── <RouterView /> (full-width content)
```

**Gaps vs. v0.5 Design:**
- Missing left sidebar navigation
- Missing persistent breadcrumb
- Missing collapsible chat panel
- Missing status bar
- Header-based nav instead of sidebar

### Component Organization

**Current (scattered):**
```
src/components/           # Mixed domain-specific
src/shared/components/    # Emerging shared library
  ├── ui/                 # Modal, Loading, Spinner
  └── layout/             # MainLayout, TwoPanelLayout
src/features/*/components/ # Feature-specific
```

**v0.5 Target:**
```
src/components/
├── common/     # Button, Input, Select, Modal, etc.
├── layout/     # PageHeader, SplitPane, EmptyState
├── domain/     # CampaignCard, ModuleCard, etc.
└── editor/     # MarkdownEditor, MapCanvas, etc.
```

### Reusable Components Identified

**Keep as-is:**
- `DmMapViewer.vue` (64KB, complex map rendering)
- `PlayerDisplayWindow.vue` (player-facing display)
- Token/lighting/LOS components in `src/components/`
- Print/export dialogs
- `AppModal.vue`, `LoadingSpinner`, `ThemeSelector`

**Feature-specific worth keeping:**
- `src/features/campaigns/components/dashboard/` tabs
- `src/features/modules/components/` (stage, NPCs, monsters)
- `src/features/characters/components/` (wizard, inventory, level up)
- `src/features/sources/components/` (catalog search, tables)

### View-Store Mapping Assessment

| View | Current Store Usage | Matches v0.5 |
|------|---------------------|--------------|
| CampaignList | useCampaignStore | ✅ Yes |
| CampaignDashboard | campaign + module + character | ✅ Yes |
| ModuleDetail | useModuleStore | ✅ Yes |
| CharacterList | useCharacterStore | ✅ Yes |
| CharacterDetail | useCharacterStore | ✅ Yes |
| SourceSearch | composables (problem) | ❌ Should use useCatalogStore |

### Migration Priorities

**High Priority (structural):**
1. Add sidebar navigation to app shell
2. Restructure router for top-level routes
3. Consolidate shared component library

**Medium Priority (alignment):**
1. Add `/documents` and `/maps` routes
2. Rename `/sources` to `/catalog`
3. Move catalog composables to useCatalogStore

**Low Priority (polish):**
1. Add breadcrumb component
2. Add status bar
3. Implement collapsible chat panel

### Acceptance Criteria Status
- [x] Application shell layout defined (current differs, migration path clear)
- [x] All primary views/routes specified (most exist, gaps identified)
- [ ] Navigation structure documented (needs sidebar implementation)
- [x] Component hierarchy for each view (documented)
- [x] Store-to-view mapping clear (documented above)

## Progress

- 2026-01-21: Investigation complete. Existing UI ~70% aligned, needs shell restructure and route additions.
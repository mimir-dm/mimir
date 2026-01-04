---
id: rewrite-dev-seeder-for-updated
level: task
title: "Rewrite dev_seeder for updated schema and workflows"
short_code: "MIMIR-T-0296"
created_at: 2026-01-03T14:55:41.247966+00:00
updated_at: 2026-01-03T18:57:42.835274+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Rewrite dev_seeder for updated schema and workflows

## Context

The dev_seeder (`crates/mimir-dm-core/src/seed/dev_seeder.rs`) is failing tests after the addition of:
- `module_npcs` table (links NPCs to modules via `character_id`)
- `module_items` table (links catalog items to modules)
- `ModuleFrontmatterService` (document-first sync)

The seeder is tightly coupled to outdated data structures and workflows.

## Issues

1. Tests failing with `Query(NotFound)` errors
2. Seeder doesn't populate new `module_npcs` and `module_items` tables
3. Seeder logic is complex and fragile - needs simplification

## Approach

Rather than patching the existing seeder, consider:

1. **Simplify test data** - Focus on essential entities needed for testing
2. **Use frontmatter-first approach** - Seed module documents with YAML frontmatter, then use `ModuleFrontmatterService` to sync to DB
3. **Decouple from specific templates** - Make seeder resilient to template changes
4. **Add module NPCs** - Create NPC characters and link them to modules
5. **Add module items** - Add sample items to modules

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All dev_seeder tests pass
- [ ] Seeded data includes sample module NPCs (4 NPCs linked to 2 modules)
- [ ] Seeded data includes sample module items (7 items across 2 modules)
- [ ] Minimal catalog data seeded (monsters + items for tests without book imports)
- [ ] Dev tools UI for reseed functionality (Settings > Developer Tools)

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Dev seeder tests failing after module_npcs/module_items schema changes; no way to reset test data from UI
- **Benefits of Fixing**: Tests pass; devs can easily reset test data; catalog data available for tests without book imports
- **Risk Assessment**: Moderate - tests remain broken until fixed, blocking CI/CD

---

## Implementation Plan

### Step 0: Seed Minimal Catalog Data

Add function to seed minimum required catalog entries so tests work without book imports.

**Monsters (from MM):**
| Name | Size | CR | Type | HP | AC |
|------|------|-----|------|-----|-----|
| Goblin | S | 1/4 | Humanoid | 7 | 15 |
| Goblin Boss | S | 1 | Humanoid | 21 | 17 |
| Wolf | M | 1/4 | Beast | 11 | 13 |
| Bugbear | M | 1 | Humanoid | 27 | 16 |
| Adult Black Dragon | H | 14 | Dragon | 195 | 19 |
| Mage | M | 6 | Humanoid | 40 | 12 |

**Items (from DMG/PHB):**
| Name | Source | Type | Rarity |
|------|--------|------|--------|
| Potion of Healing | DMG | Potion | Common |
| +1 Weapon | DMG | Weapon | Uncommon |
| Spell Scroll (2nd Level) | DMG | Scroll | Uncommon |

Uses `ON CONFLICT DO NOTHING` so idempotent with book imports.

### Step 1: Update clear_dev_seed_data()

Add deletion of `module_npcs` and `module_items` tables to cleanup function.

### Step 2: Add seed_module_npcs()

Link existing NPCs to modules with roles and encounter tags:

| Module | NPC | Role | Encounter Tag |
|--------|-----|------|---------------|
| Goblin Ambush | Sildar Hallwinter | captive | ambush_road |
| Cragmaw Hideout | Sildar Hallwinter | captive | boss_chamber |
| Cragmaw Hideout | Gundren Rockseeker | plot_hook | - |
| Cragmaw Hideout | Glasstaff | antagonist | - |

### Step 3: Add seed_module_items()

Add treasure/equipment to modules:

| Module | Item | Source | Qty | Location |
|--------|------|--------|-----|----------|
| Goblin Ambush | Potion of Healing | PHB | 2 | wagon_loot |
| Goblin Ambush | Gold Pieces | PHB | 50 | goblin_bodies |
| Cragmaw Hideout | Potion of Healing | PHB | 3 | store_room |
| Cragmaw Hideout | Supplies | custom | 1 | store_room |
| Cragmaw Hideout | Jade Frog Statuette | custom | 1 | boss_chamber |
| Cragmaw Hideout | +1 Longsword | DMG | 1 | boss_chamber |
| Cragmaw Hideout | Scroll of Augury | PHB | 1 | hidden_cache |

### Step 4: Update seed_dev_data()

Call new functions in order:
1. `seed_catalog_data()` - catalog entries first
2. Create campaign, modules, characters (existing)
3. `seed_module_npcs()` - link NPCs to modules
4. `seed_module_items()` - add items to modules
5. Create maps, tokens (existing)

### Step 5: Update imports

Add `ModuleNpcService`, `ModuleItemService` to imports.

### Step 6: Add test assertions

Verify module NPCs and items are seeded correctly.

### Step 7-9: Dev Tools UI (Bonus)

Add `reseed_dev_data` Tauri command and Developer Tools section to Settings:
- Only visible in dev mode
- "Reset Test Data" button with confirmation
- Clears and reseeds all dev data

---

## Files to Modify

| File | Changes |
|------|---------|
| `crates/mimir-dm-core/src/seed/dev_seeder.rs` | Steps 0-6: catalog seeding, cleanup, NPCs, items, tests |
| `crates/mimir-dm-core/src/seed/mod.rs` | Export `clear_dev_seed_data` |
| `crates/mimir-dm/src/commands/system/dev_tools.rs` | Add `reseed_dev_data` command |
| `crates/mimir-dm/src/main.rs` | Register `reseed_dev_data` command |
| `crates/mimir-dm/frontend/src/views/SettingsView.vue` | Add Developer Tools section |

---

## Test Verification

```bash
cargo test -p mimir-dm-core seed
```

Expected: All dev_seeder tests pass.

## Status Updates

*To be added during implementation*
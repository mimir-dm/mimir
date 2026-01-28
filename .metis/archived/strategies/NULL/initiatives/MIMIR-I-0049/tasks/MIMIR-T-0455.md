---
id: consolidate-formatters
level: task
title: "Consolidate Formatters"
short_code: "MIMIR-T-0455"
created_at: 2026-01-28T03:54:05.458133+00:00
updated_at: 2026-01-28T04:52:58.235016+00:00
parent: MIMIR-I-0049
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0049
---

# Consolidate Formatters

## Parent Initiative

[[MIMIR-I-0049]] - Frontend Organizational Debt Cleanup

## Objective

Analyze and potentially consolidate the 21 formatter files in `src/features/sources/formatters/` to reduce duplication and improve maintainability.

**Note:** This task is optional and lower priority than the other consolidation tasks.

## Current State

21 individual formatter files in `src/features/sources/formatters/`:
- actionFormatter.ts
- backgroundFormatter.ts
- classFormatter.ts
- classFormatterEnhanced.ts
- conditionFormatter.ts
- cultFormatter.ts
- deityFormatter.ts
- featFormatter.ts
- itemFormatter.ts
- languageFormatter.ts
- monsterFormatter.ts
- optionalFeatureFormatter.ts
- raceFormatter.ts
- rewardFormatter.ts
- skillFormatter.ts
- spellFormatter.ts
- subclassFormatter.ts
- subraceFormatter.ts
- tableFormatter.ts
- trapFormatter.ts
- vehicleFormatter.ts

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Analyze formatters for common patterns
- [ ] Document shared patterns/duplication
- [ ] Either consolidate or document decision to keep as-is
- [ ] If consolidated: All imports updated, app builds successfully

## Implementation Notes

### Analysis Questions

1. Do formatters share common structure?
2. Can they use a factory pattern or base class?
3. Is per-entity separation intentional for maintainability?
4. What's the cost/benefit of consolidation?

### Possible Approaches

**Option A: Factory Pattern**
```typescript
// formatterFactory.ts
export const createFormatter = (type: EntityType) => {
  const config = formatterConfigs[type]
  return (entity: unknown) => formatEntity(entity, config)
}
```

**Option B: Grouped Files**
```
formatters/
├── creatures.ts      (monster, npc)
├── items.ts          (item, reward, vehicle)
├── character.ts      (class, subclass, race, subrace, background, feat)
├── rules.ts          (action, condition, skill, language)
└── magic.ts          (spell, optionalFeature)
```

**Option C: Keep As-Is**
If each formatter has significant unique logic, the current separation may be intentional and valuable.

### Recommendation

Defer this task until other consolidation work is complete. The formatters are contained within a single feature and don't impact cross-cutting organization.

## Status Updates

### Analysis Complete - No Action Needed

**Finding:** The 21 formatter files are intentionally separated by D&D entity type, not organizational debt.

**Rationale:**
- Each formatter handles a fundamentally different data structure (Monster vs Spell vs Condition, etc.)
- No meaningful code sharing possible - entities have different fields
- Line counts range from 110 to 1,092 reflecting genuine complexity differences
- Files are already well-organized in a single feature directory
- Consolidation would create a 9,000+ line monolith harder to maintain

**Decision:** Keep current organization. This is proper single-responsibility separation.
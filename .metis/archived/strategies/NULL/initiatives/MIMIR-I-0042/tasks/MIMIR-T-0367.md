---
id: set-up-typify-generation-for
level: task
title: "Set up typify generation for 5etools types"
short_code: "MIMIR-T-0367"
created_at: 2026-01-20T02:43:34.604295+00:00
updated_at: 2026-01-20T14:32:14.174147+00:00
parent: MIMIR-I-0042
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0042
---

# Hand-written 5etools Extraction Types

## Parent Initiative

[[MIMIR-I-0042]] - v0.5 Catalog Implementation

## Objective

Create hand-written Rust extraction types for 5etools JSON data, one entity at a time. After evaluating typify and JSON schema bundlers, we found that circular `$ref` dependencies make automated generation impractical. Following the pattern from mimir-dm-bu, we'll hand-write focused extraction types.

## Approach

1. **One entity at a time** - Complete each entity before moving to the next
2. **Hand-written serde types** - Using vendored schemas as documentation
3. **Extraction-focused** - Only define fields we need to index/display/search
4. **`serde_json::Value` for complex content** - Store `entries` arrays as JSON blobs
5. **Test with real data** - Verify deserialization works before moving on

### Why Not typify?
Evaluated and rejected - 5etools schemas have circular cross-file `$ref` dependencies that cause bundlers to hang. Previous Mimir implementation also used hand-written types.

## Entity Progress

| Entity | Schema | Status | Notes |
|--------|--------|--------|-------|
| Shared Types | util.json | ✅ DONE | CreatureType, Size, Alignment, DamageType, AC, HP, CR, Speed, RaceSpeed, Lineage, HeightAndWeight, StartingEquipment |
| Monster | bestiary.json | ✅ DONE | Core entity - CR, AC, HP, speeds, actions, traits, legendary |
| Spell | spells.json | ✅ DONE | Level, school, components, duration, classes, ritual/concentration |
| Item | items.json | ✅ DONE | Rarity, type, properties, weapons, armor, magic items |
| Class | class.json | ✅ DONE | Features, subclasses, proficiencies, spellcasting, multiclassing |
| Race | races.json | ✅ DONE | Traits, ASI, speed, size, lineage, heightAndWeight, subraces |
| Background | backgrounds.json | ✅ DONE | Features, proficiencies, starting equipment |
| Feat | feats.json | ✅ DONE | Prerequisites, ability bonuses, proficiencies, repeatable |
| Action | actions.json | ✅ DONE | ActionTime (structured/simple), see_also_action |
| Condition | conditionsdiseases.json | ✅ DONE | Condition and Disease types |
| Language | languages.json | ✅ DONE | Standard, exotic, secret; scripts, typical speakers |
| OptionalFeature | optionalfeatures.json | ✅ DONE | Invocations, Metamagic, Fighting Styles; feature_type codes |
| Trap | traps-hazards.json | ✅ DONE | Trap and Hazard types; trap_haz_type codes |
| Object | objects.json | ✅ DONE | Siege weapons, AC/HP variants, immunities |
| Deity | deities.json | ✅ DONE | Pantheon, alignment, domains, symbol |
| Reward | rewards.json | ✅ DONE | Blessings, Boons, Charms; prerequisites |
| Table | tables.json | ✅ DONE | Random tables with rows, columns, TableCell variants |
| Vehicle | vehicles.json | ✅ DONE | Ships, war machines; speed, capacity, weapons |
| Cult | cultsboons.json | ✅ DONE | Cults and demonic Boons; cultists, goals |
| Psionic | psionics.json | ✅ DONE | UA psionics; Talents, Disciplines, modes |
| VariantRule | variantrules.json | ✅ DONE | Optional/variant rules |

## File Structure

```
crates/mimir-core/src/catalog/
├── mod.rs            # Module exports
├── types.rs          # Shared enums and primitives
├── monster.rs        # Monster extraction types
├── spell.rs          # Spell extraction types
├── item.rs           # Item extraction types
├── class.rs          # Class extraction types
├── race.rs           # Race extraction types
├── background.rs     # Background extraction types
├── feat.rs           # Feat extraction types
├── action.rs         # Combat action types
├── condition.rs      # Condition and Disease types
├── language.rs       # Language types
├── optionalfeature.rs # Invocations, Metamagic, Fighting Styles
├── trap.rs           # Trap and Hazard types
├── object.rs         # Siege weapons, objects
├── deity.rs          # Deity types
├── reward.rs         # Blessings, Boons, Charms
├── table.rs          # Random table types
├── vehicle.rs        # Vehicle types
├── cult.rs           # Cult and Boon types
├── psionic.rs        # Psionic types (UA)
└── variantrule.rs    # Variant rule types
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] All listed entities have extraction types
- [x] Types compile without errors
- [x] Each entity type can deserialize sample 5etools JSON (57 tests pass)
- [x] Shared types are reused across entities

## Status Updates

**2026-01-20**: Pivoted from typify to hand-written types after discovering circular schema refs cause bundlers to hang. Reviewed mimir-dm-bu implementation for patterns.

**2026-01-20 (initial)**: 8 extraction type modules complete with 57 passing tests

**2026-01-20 (full coverage)**: Extended to all 21 entity types (122 tests) to match mimir-dm-bu coverage:
- Core (8 modules): types, monster, spell, item, class, race, background, feat
- Rules (5 modules): action, condition, language, optionalfeature, variantrule  
- World (8 modules): deity, cult, reward, trap, object, vehicle, table, psionic

All modules follow consistent patterns: permissive serde deserialization, JSON blobs for entries, helper functions for type codes, unit tests with sample data.
# Characters

Reference for what Mimir tracks on player characters and NPCs. For full table schemas, see the [Data Model](../data-model.md#characters).

## PCs and NPCs

PCs and NPCs share one data model. The `is_npc` flag distinguishes them and controls which fields are relevant:

| Aspect | PC | NPC |
|--------|----|----|
| `player_name` | Real player's name | — |
| `role`, `location`, `faction` | — | NPC organization fields |
| Listing | PC-only campaign lists | NPC-only campaign lists |

Both kinds support the full set of related records below (classes, inventory, spells, and so on).

## Core Fields

| Category | Fields |
|----------|--------|
| Identity | Name, race (catalog reference), background (catalog reference) |
| Ability scores | Strength, Dexterity, Constitution, Intelligence, Wisdom, Charisma (default 10) |
| Currency | `cp`, `sp`, `ep`, `gp`, `pp` |
| Personality | Traits, ideals, bonds, flaws |

## Classes and Multiclassing

Characters can have any number of class entries, one per class:

| Field | Description |
|-------|-------------|
| Class + source | e.g., Fighter (PHB) |
| Level | Levels in this class |
| Subclass + source | Set when the class reaches its subclass level |
| Starting class | Flags the first class, for multiclass proficiency rules |

## Inventory

| Field | Description |
|-------|-------------|
| Item + source | Catalog or homebrew item reference |
| Quantity | Stack count |
| Equipped | Worn/held flag |
| Attuned | Attunement flag; Mimir counts attuned items (the D&D 5e limit of 3 is a game rule, tracked but not enforced) |
| Notes | Free-text notes |

## Spells

| Field | Description |
|-------|-------------|
| Spell + source | Catalog spell reference |
| Source class | Which class grants the spell |
| Prepared | Whether currently prepared (known vs. prepared) |

## Proficiencies

| Field | Description |
|-------|-------------|
| Type | `skill`, `save`, `tool`, `weapon`, `armor`, or `language` |
| Name | e.g., "Perception", "Thieves' Tools" |
| Expertise | Double proficiency bonus flag |

## Features

Selectable class features chosen during level-up:

| Type | Granted by |
|------|-----------|
| `fighting_style` | Fighter 1, Paladin 2, Ranger 2 |
| `metamagic` | Sorcerer 3, 10, 17 |
| `maneuver` | Battle Master 3, 7, 10, 15 |
| `invocation` | Warlock 2, 5, 7, 9, 12, 15, 18 |
| `pact_boon` | Warlock 3 |

Each feature records its source class.

## Feats

| Field | Description |
|-------|-------------|
| Feat + source | Catalog feat reference |
| Source type | How it was acquired: `asi`, `race`, `class`, or `bonus` |

## Per-Character Sources

Characters can carry their own enabled-source list, separate from the campaign's source filter.

## Level-Up Choices

A level-up request specifies:

| Choice | Options |
|--------|---------|
| Class | Any class (adding a new class multiclasses) |
| HP gain | Average (half hit die + 1, rounded up), Roll (enter the die result), or Manual (enter any value) |
| Subclass | Required at the class's subclass level |
| ASI or feat | Ability score improvement (+2 total across one or two abilities) or a feat |
| Spell changes | New spells, new cantrips, and one known-spell swap |
| Feature choices | Fighting style, metamagic, maneuvers (with swap), invocations (with swap), pact boon, expertise skills (Rogue 1/6, Bard 3/10) |

## See Also

- [Data Model](../data-model.md) — Full character table schemas
- [Character How-To Guides](../../how-to/characters/README.md) — Creating, leveling, and managing characters

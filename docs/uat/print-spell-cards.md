# Spell Cards - Print Wireframe

Cards are printed multi-up on a page (3x3 grid = 9 cards per page).
Each card is sized to fit standard trading card sleeves (2.5" x 3.5").

## Single Card Layout

┌─────────────────────────────────────────────────────────┐
│  SPELL NAME                              ◆◆◆ (level)   │
├─────────────────────────────────────────────────────────┤
│  School              Casting Time: _________           │
│  ______________      Range: _________                  │
│                      Duration: _________               │
│  ○ Ritual            Components: V S M (material)     │
│  ○ Concentration                                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Description text goes here. This area contains the    │
│  full spell description, effects, damage dice, saving  │
│  throws, and any scaling information for higher        │
│  level casting.                                         │
│                                                         │
│  At Higher Levels: When you cast this spell using a    │
│  spell slot of Xth level or higher...                  │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  Classes: Wizard, Sorcerer, Cleric                     │
└─────────────────────────────────────────────────────────┘

## Page Layout (3x3 Grid)

┌─────────────────┬─────────────────┬─────────────────┐
│                 │                 │                 │
│   SPELL CARD    │   SPELL CARD    │   SPELL CARD    │
│       1         │       2         │       3         │
│                 │                 │                 │
├─────────────────┼─────────────────┼─────────────────┤
│                 │                 │                 │
│   SPELL CARD    │   SPELL CARD    │   SPELL CARD    │
│       4         │       5         │       6         │
│                 │                 │                 │
├─────────────────┼─────────────────┼─────────────────┤
│                 │                 │                 │
│   SPELL CARD    │   SPELL CARD    │   SPELL CARD    │
│       7         │       8         │       9         │
│                 │                 │                 │
└─────────────────┴─────────────────┴─────────────────┘

## Card Content Fields

| Field | Source | Notes |
|-------|--------|-------|
| Spell Name | spell.name | Bold, prominent |
| Level | spell.level | Diamond icons (◆) or "Cantrip" |
| School | spell.school | Abjuration, Evocation, etc. |
| Casting Time | spell.casting_time | Action, Bonus Action, Reaction, 1 minute, etc. |
| Range | spell.range | Self, Touch, 30 feet, etc. |
| Duration | spell.duration | Instantaneous, 1 minute, Concentration up to 1 hour |
| Components | spell.components | V, S, M with material description |
| Ritual | spell.ritual | Checkbox indicator |
| Concentration | spell.concentration | Checkbox indicator |
| Description | spell.description | Main spell text, may need truncation for long spells |
| At Higher Levels | spell.higher_levels | Upcasting info |
| Classes | spell.classes | Which classes can cast |

## Sorting Options

Cards should be sorted by:
1. **Spell Level** (cantrips first, then 1st through 9th)
2. **Alphabetical** within each level

## Notes

- **Long descriptions**: Decrease font size to fit
- **Two-sided trigger**: When font would shrink below readable threshold (e.g., 6pt), switch to fold-adjacent two-sided layout where front/back are printed side-by-side for a single fold
- Cantrips use "Cantrip" text instead of level diamonds
- No color coding by school
- No border/background distinction for concentration - players pay attention

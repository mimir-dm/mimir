---
name: encounter-balance
description: >-
  This skill should be used when the user asks to "check encounter balance",
  "review CR", "is this too hard", "is this too easy", "balance my encounters",
  "encounter difficulty", "TPK check", "deadly encounter", "party level check",
  or mentions "challenge rating", "encounter math", or "XP budget".
---

# Encounter Balance Review

## Purpose

Analyze encounters in modules against expected party composition to identify deadly, trivial, or unbalanced fights using D&D 5e encounter math.

## Analysis Process

### 1. Establish Party Parameters

Ask for or assume:
- Number of players (default: 4)
- Average party level
- Party composition (optional, for tactical analysis)

### 2. Gather Encounter Data

```
get_campaign_details(campaign_id)
list_modules()
# For each module:
get_module_details(module_id)
# Extract monster list with counts
```

### 3. Calculate Per Encounter

For each encounter:

1. **Sum base XP** — Add XP for each monster by CR (see references/5e-encounter-math.md)
2. **Apply multiplier** — Based on monster count
3. **Compare to thresholds** — Determine difficulty category
4. **Flag concerns** — Deadly, trivial, or resource-draining

### 4. Adventuring Day Analysis

D&D 5e assumes 6-8 medium encounters per long rest. Calculate:
- Total adjusted XP across module
- Expected adventuring days
- Resource pressure (will the party run out of spell slots?)

## Output Format

```markdown
# Encounter Balance Report: [Module Name]
**Party**: [X] players, level [Y]

## Daily XP Budget
- Easy threshold: [X] XP
- Medium threshold: [X] XP
- Hard threshold: [X] XP
- Deadly threshold: [X] XP
- Daily budget: [X] XP (6-8 medium encounters)

## Encounter Analysis

### [Encounter Name/Location]
| Monster | CR | Count | Base XP |
|---------|-----|-------|---------|
| [Name] | [CR] | [N] | [XP] |

- **Total Base XP**: [X]
- **Adjusted XP** (x[multiplier]): [X]
- **Difficulty**: [Easy/Medium/Hard/Deadly]
- **Assessment**: [Notes]

### Summary

| Encounter | Difficulty | Adjusted XP | Concern |
|-----------|------------|-------------|---------|
| [Name] | Deadly | 5,400 | [WARNING] TPK risk |
| [Name] | Easy | 200 | Filler |
| [Name] | Hard | 2,100 | Good challenge |

## Concerns

### Deadly Encounters
- [Encounter]: [Why it's dangerous] -> [Suggestion]

### Trivial Encounters
- [Encounter]: [Why it's too easy] -> [Suggestion]

### Resource Pressure
- Module contains [X] adjusted XP
- Expected adventuring days: [Y]
- Assessment: [Over/under tuned]

## Recommendations
1. [Specific adjustment]
2. [Specific adjustment]
```

## Interactive Mode

1. Ask for party composition
2. Present module-by-module analysis
3. For deadly encounters, offer alternatives:
   - "This encounter is deadly. Should I suggest monster substitutions?"
   - Search catalog for CR-appropriate alternatives
4. For trivial encounters, suggest enhancements

## Reference Data

For XP thresholds, CR-to-XP tables, and encounter multipliers, see references/5e-encounter-math.md.

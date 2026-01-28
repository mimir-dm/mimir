# Encounter Balance Review

This skill should be used when the user asks to "check encounter balance", "review CR", "is this too hard", "is this too easy", "balance my encounters", "encounter difficulty", "TPK check", "deadly encounter", "party level check", or mentions "challenge rating", "encounter math", or "XP budget".

## Purpose

Analyze encounters in modules against expected party composition to identify potentially deadly, trivial, or unbalanced fights. Uses D&D 5e encounter building math.

## D&D 5e Encounter Math Reference

### XP Thresholds by Level (Per Character)

| Level | Easy | Medium | Hard | Deadly |
|-------|------|--------|------|--------|
| 1 | 25 | 50 | 75 | 100 |
| 2 | 50 | 100 | 150 | 200 |
| 3 | 75 | 150 | 225 | 400 |
| 4 | 125 | 250 | 375 | 500 |
| 5 | 250 | 500 | 750 | 1,100 |
| 6 | 300 | 600 | 900 | 1,400 |
| 7 | 350 | 750 | 1,100 | 1,700 |
| 8 | 450 | 900 | 1,400 | 2,100 |
| 9 | 550 | 1,100 | 1,600 | 2,400 |
| 10 | 600 | 1,200 | 1,900 | 2,800 |

### Encounter Multipliers

| # Monsters | Multiplier |
|------------|------------|
| 1 | ×1 |
| 2 | ×1.5 |
| 3-6 | ×2 |
| 7-10 | ×2.5 |
| 11-14 | ×3 |
| 15+ | ×4 |

## Analysis Process

### 1. Establish Party Parameters

Ask for or assume:
- Number of players (default: 4)
- Average party level
- Party composition (optional: for tactical analysis)

### 2. Gather Encounter Data

```
get_campaign_details(campaign_id)
list_modules()
# For each module:
get_module_details(module_id)
# Extract monster list with counts
```

### 3. Calculate Per-Encounter

For each encounter:

1. **Sum Base XP**: Add XP for each monster by CR
2. **Apply Multiplier**: Based on monster count
3. **Compare to Thresholds**: Determine difficulty category
4. **Flag Concerns**: Deadly, trivial, or resource-draining

### CR to XP Reference

| CR | XP | CR | XP |
|----|-----|----|----|
| 0 | 10 | 5 | 1,800 |
| 1/8 | 25 | 6 | 2,300 |
| 1/4 | 50 | 7 | 2,900 |
| 1/2 | 100 | 8 | 3,900 |
| 1 | 200 | 9 | 5,000 |
| 2 | 450 | 10 | 5,900 |
| 3 | 700 | 11+ | varies |
| 4 | 1,100 | | |

### 4. Adventuring Day Analysis

D&D 5e assumes 6-8 medium encounters per long rest.

Calculate:
- Total adjusted XP across module
- Expected adventuring days
- Resource pressure (will they run out of spell slots?)

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
- **Adjusted XP** (×[multiplier]): [X]
- **Difficulty**: [Easy/Medium/Hard/Deadly]
- **Assessment**: [Notes]

### Summary

| Encounter | Difficulty | Adjusted XP | Concern |
|-----------|------------|-------------|---------|
| [Name] | Deadly | 5,400 | ⚠️ TPK risk |
| [Name] | Easy | 200 | Filler |
| [Name] | Hard | 2,100 | Good challenge |

## Concerns

### ⚠️ Deadly Encounters
- [Encounter]: [Why it's dangerous] → [Suggestion]

### 😴 Trivial Encounters
- [Encounter]: [Why it's too easy] → [Suggestion]

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
   - "This encounter is deadly. Would you like me to suggest monster substitutions?"
   - Search catalog for CR-appropriate alternatives
4. For trivial encounters, suggest enhancements

---
# Template metadata
id: module_overview
title: Module Overview
type: module_overview
level: module
purpose: Complete runnable adventure module with catalog references and playable content
author: Mimir Team

# Module-specific metadata
# These fields are filled in when creating a module from this template
module_number: 1
theme: "[Theme]"
tone: "[Tone]"
estimated_hours: 4

# Catalog references (machine-readable)
# The system parses these arrays and syncs to database tables
# Full stat blocks are generated in separate monsters.md, npcs.md, items.md files

monsters:
  # Group monsters by encounter tag for organization
  # - encounter: Tag name matching Adventure Content encounters
  # - name: Exact catalog name
  # - source: Source book abbreviation (MM, PHB, DMG, etc.)
  # - quantity: Number of creatures
  # - notes: Context for this encounter
  - encounter: entrance_fight
    name: "[Monster Name]"
    source: MM
    quantity: 2
    notes: "Guards at the entrance"
  - encounter: boss_fight
    name: "[Boss Monster]"
    source: MM
    quantity: 1
    notes: "Main antagonist"

npcs:
  # NPCs can reference catalog entries or campaign-specific characters
  # - role: quest_giver, ally, antagonist, informant, wild_card
  # - source: "campaign" for custom NPCs, or source book for catalog NPCs
  - role: quest_giver
    name: "[NPC Name]"
    source: campaign
    location: "[Location]"
    notes: "Provides the hook"
  - role: antagonist
    name: "[Antagonist Name]"
    source: campaign
    location: "[Location]"
    notes: "Primary opposition"

items:
  # Magic items and significant treasure
  # - location: Where found (matches encounter tags or location names)
  - location: boss_chamber
    name: "[Magic Item]"
    source: DMG
    quantity: 1
    notes: "Primary reward"
  - location: hidden_cache
    name: Potion of Healing
    source: PHB
    quantity: 3
    notes: "Optional discovery"

variables:
  - name: module_name
    type: string
    description: Name of the module
    default: "[Module Name]"
    required: true
  - name: module_number
    type: number
    description: Module number in campaign sequence
    default: 1
    required: true
---

# Module {{module_number}}: {{module_name}}

---

## 1. Overview

**Pitch:** [One sentence that captures the essence of this module]

**Theme:** [What this module is really about - the deeper meaning]

**Tone:** [Serious / Light / Dark / Adventure / Horror / Mystery]

**Estimated Play Time:** [X hours]

### The Hook

**Inciting Incident:** [What starts this module - the event that pulls players in]

**Why Now:** [The urgency factor - why this can't wait]

**Personal Stakes:** [Why the PCs specifically care about this]

### Module Structure

<!-- Visual diagram of the module flow -->
```
[START] → [LOCATION 1] → [LOCATION 2] → [CLIMAX] → [RESOLUTION]
 (hook)     (challenge)    (challenge)    (boss)     (aftermath)
```

[Brief description of the structure - linear, branching, hub-and-spoke, etc.]

---

## 2. Locations

**Hub:** [Where PCs operate from or return to]

### Challenge Sites

| # | Location | What Happens Here | Key Features |
|---|----------|-------------------|--------------|
| 1 | [Location Name] | [Primary activity/challenge] | [Notable features] |
| 2 | [Location Name] | [Primary activity/challenge] | [Notable features] |
| 3 | [Location Name] | [Primary activity/challenge] | [Notable features] |

---

## 3. Critical Path

### Must Happen
<!-- Essential plot beats - the module fails without these -->
1. [Essential event or discovery]
2. [Essential event or discovery]
3. [Essential event or discovery]

### Should Happen
<!-- Important but flexible - can be skipped if needed -->
- [Important but flexible event]
- [Important but flexible event]

### Could Happen
<!-- Bonus content for thorough or creative players -->
- [Bonus content or easter egg]
- [Bonus content or easter egg]

---

## 4. Information Architecture

### Essential Clues
<!-- Each clue must have multiple sources (Three Clue Rule) -->

| Clue | Source A | Source B | Source C |
|------|----------|----------|----------|
| [What players learn] | [Where/how] | [Where/how] | [Where/how] |
| [What players learn] | [Where/how] | [Where/how] | [Where/how] |

### Bonus Information
<!-- Rewards for thoroughness - nice to know but not essential -->

| Info | How to Find It |
|------|----------------|
| [Interesting detail] | [Method/location] |
| [Interesting detail] | [Method/location] |

---

## 5. Adventure Content

<!--
GUIDANCE FOR AUTHORS:
- Each Part is a major scene or location
- Include read-aloud text in blockquotes for key moments
- Reference encounters by tag (matches front matter monsters)
- Provide multiple outcomes where player choice matters
-->

### Part 1: [Scene Name]

**Setup:** [Context for this scene - what's happening, why players are here]

> **Read Aloud:**
> "[Evocative description of what players see, hear, and sense. Write in second person present tense. Include sensory details - sights, sounds, smells. Set the mood.]"

**Features:**
- [Environmental detail or interactive element]
- [Environmental detail or interactive element]
- [Notable object or NPC present]

**Encounter:** `entrance_fight`
<!-- Full stats synced to monsters.md from front matter -->
- **Tactics:** [How enemies fight - their strategy and behavior]
- **Terrain:** [Environmental factors affecting combat]
- **Complications:** [What might go wrong or change mid-fight]

**Outcomes:**
- **Success:** [What happens if players succeed]
- **Failure:** [Consequences if players fail - should not end the adventure]
- **Transition:** [How this leads to the next scene]

---

### Part 2: [Scene Name]

**Setup:** [Context for this scene]

> **Read Aloud:**
> "[Description for players]"

**Features:**
- [Environmental details]
- [Interactive elements]

**Challenge:** [Puzzle, social encounter, or exploration challenge]
- **Objective:** [What players need to accomplish]
- **Approach Options:** [Different ways to succeed]

**Outcomes:**
- **Success:** [What happens]
- **Partial Success:** [Alternative outcome]
- **Transition:** [To next scene]

---

### Part 3: [Climax Scene Name]

**Setup:** [Building to the climax]

> **Read Aloud:**
> "[Dramatic description setting up the final challenge]"

**Features:**
- [Key environmental features]
- [Dramatic elements]

**Encounter:** `boss_fight`
<!-- Full stats synced to monsters.md from front matter -->
- **Tactics:** [Boss behavior and phases]
- **Legendary Actions:** [If applicable]
- **Lair Effects:** [Environmental dangers]

**Outcomes:**
- **Victory:** [Rewards and consequences]
- **Defeat:** [What happens - avoid TPK unless appropriate]
- **Resolution:** [Wrapping up the module]

---

## 6. Puzzles & Challenges

### [Puzzle Name]

**Location:** [Where this puzzle appears]

**Setup:** [What players see when they encounter the puzzle]

> **Read Aloud:**
> "[Description of the puzzle environment]"

**The Puzzle:**
- **Objective:** [What players need to accomplish]
- **Components:** [Interactive elements]
- **Solution:** [How to solve it]

**Hints (Progressive):**
1. [Subtle hint - free or low DC]
2. [Medium hint - moderate DC or cost]
3. [Direct hint - if they're really stuck]

**Consequences:**
- **Solved:** [Reward or progression]
- **Failed Attempt:** [What happens on wrong answer]
- **Brute Force:** [Alternative solution if applicable]

---

## 7. DM Notes

### Pacing

[Tips for managing time and momentum through the module. When to speed up, when to slow down, natural break points.]

### Tone Calibration

[How to adjust the mood for different groups. What elements to emphasize or downplay.]

### Scaling

- **Weaker Party:** [Adjustments for struggling players - reduce HP, remove abilities, add NPC help]
- **Stronger Party:** [Adjustments for experienced players - add enemies, buff abilities, add complications]

### Common Pitfalls

[Things that might derail the module and how to get back on track]

- [Pitfall]: [How to handle it]
- [Pitfall]: [How to handle it]

---

## 8. Connections

### From Previous Module
[What carries forward - plot threads, NPC relationships, consequences of prior choices]

### To Next Module
[What this sets up - hooks planted, information revealed, new threats introduced]

### Campaign Themes
[How this module reinforces the overarching campaign themes and narrative]

---

## 9. Post-Module Notes

<!-- Fill this out after running the module -->

### What Happened
- [ ] [Key decision point - which way did they go?]
- [ ] [Key decision point - what choice did they make?]
- [ ] [Important NPC interaction - how did it go?]
- [ ] [Unexpected player action worth noting]

### Continuity Notes
[Important facts for future modules - what players know, what they missed, relationships changed]

### Player Feedback
[What worked well, what to adjust for future sessions]

---

**Status:** [Planning / Ready / Active / Complete]
**Started:** [Date]
**Completed:** [Date]

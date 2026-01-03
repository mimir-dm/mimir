---
# Template metadata
id: module_dungeon
title: Dungeon Module
type: module_dungeon
level: module
purpose: Exploration and combat-focused adventure with room-by-room content
author: Mimir Team

# Module-specific metadata
module_number: 1
theme: "[Theme]"
tone: "Adventure"
estimated_hours: 8

# Catalog references (machine-readable)
monsters:
  # Dungeon modules are combat-heavy - organize by area/encounter
  - encounter: entrance_guards
    name: "[Guardian Type]"
    source: MM
    quantity: 2
    notes: "First line of defense"
  - encounter: patrol
    name: "[Patrol Creature]"
    source: MM
    quantity: 4
    notes: "Roaming encounters"
  - encounter: miniboss
    name: "[Mini-boss]"
    source: MM
    quantity: 1
    notes: "Guards key area"
  - encounter: boss_fight
    name: "[Boss Monster]"
    source: MM
    quantity: 1
    notes: "Final guardian"
  - encounter: boss_fight
    name: "[Minion Type]"
    source: MM
    quantity: 3
    notes: "Boss support"

npcs:
  # Dungeons typically have fewer NPCs
  - role: prisoner
    name: "[Captive]"
    source: campaign
    location: "Cells/Prison Area"
    notes: "Can provide information or need rescue"
  - role: informant
    name: "[Survivor/Ghost]"
    source: campaign
    location: "Hidden area"
    notes: "Knows dungeon secrets"

items:
  # Treasure distribution through the dungeon
  - location: hidden_cache
    name: "[Minor Magic Item]"
    source: DMG
    quantity: 1
    notes: "Reward for exploration"
  - location: miniboss_treasure
    name: "[Medium Magic Item]"
    source: DMG
    quantity: 1
    notes: "Mini-boss reward"
  - location: boss_hoard
    name: "[Major Magic Item]"
    source: DMG
    quantity: 1
    notes: "Primary treasure"
  - location: boss_hoard
    name: "[Gold/Gems]"
    source: DMG
    quantity: 1
    notes: "Monetary treasure"

variables:
  - name: dungeon_name
    type: string
    description: Name of the dungeon
    default: "[Dungeon Name]"
    required: true
  - name: module_number
    type: number
    description: Module number in campaign sequence
    default: 1
    required: true
---

# Module {{module_number}}: {{dungeon_name}}

*Exploration and combat adventure*

---

## 1. Overview

**Pitch:** [One sentence describing the dungeon adventure]

**Dungeon Name:** [What locals call this place]

**Core Question:** [What mystery does the dungeon hold?]

**Threat Level:** [Low / Medium / High / Extreme]

**Estimated Play Time:** [X hours]

### The Hook

**How PCs Learn About It:** [Rumor, treasure map, plea for help, etc.]

**What They're Told:** [The public story about this place]

**The Real Danger:** [What they don't know]

**Time Pressure:** [Why go now? What happens if they wait?]

### Dungeon Structure

```
[ENTRANCE] → [LEVEL 1] → [LEVEL 2] → [BOSS CHAMBER]
  (guards)    (explore)   (challenges)    (climax)
```

[Brief description - linear, branching, hub-and-spoke, etc.]

---

## 2. Dungeon Background

### History

**Original Purpose:** [Why was it built?]

**The Catastrophe:** [What went wrong?]

**Current State:** [What is it now?]

**Local Knowledge:** [What people think they know]

### Theme and Atmosphere

**Visual Theme:** [Architecture style, decay state]

**Mood:** [Oppressive, mysterious, ancient, corrupted]

**Unique Features:** [What makes this dungeon special]

**Environmental Challenges:** [Light, air, temperature, water]

---

## 3. Faction Overview

**Primary Inhabitants:** [Who lives here now - their goal]

**Secondary Group:** [Rivals, prisoners, or opportunists]

**Ancient Guardians:** [What still protects the original purpose]

**Wild Cards:** [Neutral parties players might ally with or fight]

---

## 4. Dungeon Map & Key Areas

### Level 1: [Entry Level Name]

**Theme:** [First impression]

**Difficulty:** Easy-Medium

**Key Rooms:** 5-8

| # | Room Name | Contents | Exits |
|---|-----------|----------|-------|
| 1 | [Entrance] | [What's here] | [Where it leads] |
| 2 | [Room Name] | [What's here] | [Where it leads] |
| 3 | [Room Name] | [What's here] | [Where it leads] |

### Level 2: [Deeper Level Name]

**Theme:** [How it changes from Level 1]

**Difficulty:** Medium-Hard

**Key Rooms:** 6-10

| # | Room Name | Contents | Exits |
|---|-----------|----------|-------|
| 1 | [Room Name] | [What's here] | [Where it leads] |
| 2 | [Room Name] | [What's here] | [Where it leads] |

### Level 3: [Final Level / Boss Area]

**Theme:** [Core dungeon purpose revealed]

**Difficulty:** Hard-Deadly

**Key Rooms:** 4-6

| # | Room Name | Contents | Exits |
|---|-----------|----------|-------|
| 1 | [Antechamber] | [What's here] | [Where it leads] |
| 2 | [Boss Chamber] | [What's here] | [Escape routes] |

---

## 5. Treasure Distribution

### By Location

| Location | Treasure | Value | Notes |
|----------|----------|-------|-------|
| [Room #] | [Item/coins] | [GP value] | [How hidden] |
| [Room #] | [Item/coins] | [GP value] | [How hidden] |
| [Boss Chamber] | [Major items] | [GP value] | [Primary reward] |

### Magic Items

- **Minor Items:** [Consumables, common items - locations]
- **Uncommon Items:** [+1 weapons, useful items - locations]
- **Major Item:** [Signature piece - boss reward]

---

## 6. Adventure Content

### Part 1: Approach & Entrance

**Setup:** [How players arrive at the dungeon]

> **Read Aloud:**
> "[Description of the dungeon exterior - the entrance looms before them. What do they see? What sounds or smells? What hints at the danger within?]"

**Features:**
- [Visible defenses or warnings]
- [Environmental details that set the mood]
- [Signs of previous adventurers]

**Encounter:** `entrance_guards`
<!-- Full stats synced to monsters.md from front matter -->
- **Tactics:** [How the guards defend the entrance]
- **Terrain:** [Cover, elevation, hazards]
- **Alert:** [What happens if guards sound alarm]

**Outcomes:**
- **Stealth Entry:** Guards bypassed, interior unaware
- **Quick Victory:** Guards defeated before alarm
- **Alarm Raised:** Dungeon inhabitants prepare
- **Transition:** Into the dungeon proper

---

### Room: [Room Name] (Room #)

**Purpose:** [Original and current function]

> **Read Aloud:**
> "[Description of what players see when they enter - architecture, lighting, occupants, notable features]"

**Features:**
- [Interactive element or searchable area]
- [Environmental hazard or benefit]
- [Clue to dungeon's history or layout]

**Encounter/Challenge:** [Combat, trap, puzzle, or empty]
- [Details appropriate to the challenge type]

**Treasure:** [If any - reference items from front matter]

**Exits:** [Where doors/passages lead, any locked or hidden]

---

### Room: [Hub Room Name] (Room #)

**Purpose:** [Central junction with multiple paths]

> **Read Aloud:**
> "[Description emphasizing the choices players have]"

**Features:**
- [Each exit has distinctive hints about what lies beyond]
- [Possible rest spot if secured]

**Branching Paths:**
| Exit | Hints | Leads To |
|------|-------|----------|
| North | [What players notice] | [Destination] |
| East | [What players notice] | [Destination] |
| Down | [What players notice] | [Destination] |

---

### Part 2: The Mini-Boss

**Setup:** [This guardian protects something valuable or blocks progress]

> **Read Aloud:**
> "[Description of the mini-boss's lair - show their power through the environment]"

**Features:**
- [Tactical terrain elements]
- [Environmental hazards or benefits]
- [Evidence of the mini-boss's nature]

**Encounter:** `miniboss`
<!-- Full stats synced to monsters.md from front matter -->
- **Tactics:** [How the mini-boss fights - phases, abilities]
- **Terrain:** [How the room affects combat]
- **Retreat:** [When/if the mini-boss flees]

**Treasure:** [Mini-boss reward - reference items from front matter]

**Outcomes:**
- **Victory:** [What they gain access to]
- **Defeat:** [Consequences - capture? Death? Escape?]
- **Transition:** [Path to deeper levels or boss]

---

### Part 3: The Boss Chamber

**Setup:** [The climax of the dungeon - what the boss guards]

> **Read Aloud:**
> "[Dramatic description of the final chamber - this should feel climactic. The architecture, the boss, any minions, the treasure they protect]"

**Features:**
- [Lair features that affect combat]
- [The prize that makes this worthwhile]
- [Escape routes if things go badly]

**Encounter:** `boss_fight`
<!-- Full stats synced to monsters.md from front matter -->
- **Phase 1:** [Initial tactics and abilities]
- **Phase 2:** [When bloodied - what changes]
- **Legendary Actions:** [If applicable]
- **Lair Actions:** [Environmental effects each round]
- **Minions:** [Support creatures and their role]

**Victory:**
- [How the boss can be defeated]
- [What happens when they fall]
- [Access to treasure hoard]

**Defeat:**
- [What happens if party falls]
- [Escape or rescue possibilities]

---

## 7. Traps & Puzzles

### Trap: [Trap Name]

**Location:** [Room #]

**Trigger:** [What sets it off]

**Effect:** [Damage and effects]

**Detection:** Perception DC [X]

**Disarm:** [Thieves' tools DC X, or alternative methods]

**Clues:** [What hints at its presence]

### Puzzle: [Puzzle Name]

**Location:** [Room #]

**Presentation:** [What players see]

> **Read Aloud:**
> "[Description of the puzzle elements]"

**Solution:** [How to solve it]

**Hints:**
1. [Subtle hint - DC 15 Investigation]
2. [Medium hint - found elsewhere in dungeon]
3. [Direct hint - if stuck, NPC or inscription]

**Reward:** [What solving grants - access, treasure, shortcut]

**Failure:** [Consequences of wrong answer - damage, alarm, reset]

---

## 8. Random Encounters (d12)

Roll every 30 minutes of in-game exploration:

| Roll | Encounter |
|------|-----------|
| 1-3 | No encounter |
| 4-5 | Evidence of creatures (tracks, sounds, smells) |
| 6-7 | [Common inhabitant - patrol] |
| 8-9 | [Uncommon inhabitant - hunting] |
| 10 | [Environmental hazard - ceiling collapse, gas pocket] |
| 11 | [Hostile patrol - searching for intruders] |
| 12 | [Special encounter - prisoner, rival adventurers, unique creature] |

---

## 9. DM Notes

### Pacing

- First level should move quickly, establish danger
- Middle levels allow for exploration and discovery
- Build tension as they approach the boss
- Boss fight should feel like a true climax

### Resource Management

- Track light sources, rations, spell slots
- Short rests only in secured areas
- Long rests extremely difficult in hostile dungeon
- Tension from dwindling resources

### Common Pitfalls

- **Party retreats to rest:** Dungeon resets partially, enemies fortify
- **Party gets lost:** Use sound, air flow, tracks as navigation aids
- **TPK imminent:** Capture scenario, rescue mission follows

### Scaling

- **Easier:** Reduce enemy numbers, lower HP, add helpful NPC
- **Harder:** Add enemies, use tactics, enforce resource scarcity

---

## 10. Connections

### From Previous Module
[What leads them here - map found, NPC request, rumor followed]

### To Next Module
[What they discover - hooks, items, information leading forward]

### Campaign Themes
[How this dungeon reinforces larger narrative themes]

---

## 11. Post-Module Notes

### What Happened
- [ ] Which path did they take through the dungeon?
- [ ] Did they find the secret areas?
- [ ] How did the boss fight go?
- [ ] What treasure did they claim?

### Continuity Notes
[Enemies escaped, allies made, secrets learned]

### Dungeon Status
[Cleared? Partially explored? New inhabitants moving in?]

---

**Status:** [Planning / Ready / Active / Complete]
**Started:** [Date]
**Completed:** [Date]

---
# Template metadata
id: module_political
title: Political Module
type: module_political
level: module
purpose: Social maneuvering adventure with factions, intrigue, and reputation tracking
author: Mimir Team

# Module-specific metadata
module_number: 1
theme: "[Theme]"
tone: "Intrigue"
estimated_hours: 8

# Catalog references (machine-readable)
monsters:
  # Political modules may have minimal combat
  - encounter: assassination_attempt
    name: Assassin
    source: MM
    quantity: 2
    notes: "If things turn violent"
  - encounter: faction_enforcers
    name: "[Faction Muscle]"
    source: MM
    quantity: 4
    notes: "Faction's armed supporters"
  - encounter: coup_forces
    name: "[Military Unit]"
    source: MM
    quantity: 6
    notes: "If civil conflict erupts"

npcs:
  # Political modules are extremely NPC-heavy
  - role: faction_leader
    name: "[Faction 1 Leader]"
    source: campaign
    location: "[Their power base]"
    notes: "Goal: [What they want]"
  - role: faction_leader
    name: "[Faction 2 Leader]"
    source: campaign
    location: "[Their power base]"
    notes: "Goal: [What they want]"
  - role: faction_leader
    name: "[Faction 3 Leader]"
    source: campaign
    location: "[Their power base]"
    notes: "Goal: [What they want]"
  - role: power_broker
    name: "[The Kingmaker]"
    source: campaign
    location: "Neutral ground"
    notes: "Everyone wants their support"
  - role: wild_card
    name: "[The Unpredictable One]"
    source: campaign
    location: "Various"
    notes: "Shifts loyalty for advantage"
  - role: idealist
    name: "[The True Believer]"
    source: campaign
    location: "[Their cause's base]"
    notes: "Won't compromise principles"

items:
  - location: reward
    name: "[Political Favor/Title]"
    source: campaign
    quantity: 1
    notes: "Power is the reward"
  - location: secret_document
    name: "[Damaging Evidence]"
    source: campaign
    quantity: 1
    notes: "Leverage over a faction"

variables:
  - name: conflict_name
    type: string
    description: Name of the political conflict
    default: "[Conflict Name]"
    required: true
  - name: module_number
    type: number
    description: Module number in campaign sequence
    default: 1
    required: true
---

# Module {{module_number}}: {{conflict_name}}

*Social maneuvering and faction-based adventure*

---

## 1. Overview

**Pitch:** [One sentence describing the political conflict]

**Central Conflict:** [What power struggle is happening]

**The Prize:** [What everyone wants - throne, treaty, position, etc.]

**The Deadline:** [When decision must be made - coronation, vote, festival]

**Estimated Play Time:** [X hours - highly variable based on player choices]

### The Hook

**Why PCs Are Involved:** [Hired, caught up, personal stake]

**What They're Asked To Do:** [Initial mission or role]

**Hidden Agendas:** [What they don't know yet]

### Module Structure

```
[ENTERING THE GAME] → [MANEUVERING] → [CRISIS] → [RESOLUTION]
    (factions meet)      (missions)     (betrayals)  (new order)
```

---

## 2. The Political Landscape

### The Situation

**Current State:** [Who has power now]

**The Crisis:** [What destabilized the status quo]

**The Opportunity:** [Why change is suddenly possible]

**The Deadline:** [When a decision will be forced]

### Power Centers

| Power Type | Who | Strength |
|------------|-----|----------|
| **Legitimate Authority** | [Who should rule by law/tradition] | [Strong/Weak] |
| **Actual Power** | [Who really controls things] | [Strong/Weak] |
| **Rising Power** | [Who wants to take over] | [Growing] |
| **Shadow Power** | [Who manipulates from behind] | [Hidden] |

---

## 3. Major Factions

### Faction 1: [Name]

| Aspect | Details |
|--------|---------|
| **Leader** | [Name and title] |
| **Public Goal** | [What they claim to want] |
| **Secret Goal** | [What they really want] |
| **Resources** | [Money, troops, influence, magic] |
| **Weakness** | [Exploitable flaw] |
| **Key Members** | [Important supporters] |

**What They Offer PCs:**
- [Reward or position]
- [Access or information]
- [Protection or resources]

**What They Want From PCs:**
- [Service or task]
- [Information or access]
- [Elimination of rival]

---

### Faction 2: [Name]

| Aspect | Details |
|--------|---------|
| **Leader** | [Name and title] |
| **Public Goal** | [What they claim to want] |
| **Secret Goal** | [What they really want] |
| **Resources** | [Money, troops, influence, magic] |
| **Weakness** | [Exploitable flaw] |
| **Key Members** | [Important supporters] |

**What They Offer PCs:**
- [Reward or position]
- [Access or information]
- [Protection or resources]

**What They Want From PCs:**
- [Service or task]
- [Information or access]
- [Elimination of rival]

---

### Faction 3: [Name]

| Aspect | Details |
|--------|---------|
| **Leader** | [Name and title] |
| **Public Goal** | [What they claim to want] |
| **Secret Goal** | [What they really want] |
| **Resources** | [Money, troops, influence, magic] |
| **Weakness** | [Exploitable flaw] |
| **Key Members** | [Important supporters] |

**What They Offer PCs:**
- [Reward or position]
- [Access or information]
- [Protection or resources]

**What They Want From PCs:**
- [Service or task]
- [Information or access]
- [Elimination of rival]

---

## 4. Key NPCs

### The Power Broker: [Name]

**Role:** [Position that makes them influential]

**Appears:** [How they present publicly]

**Actually:** [Their true nature and agenda]

**Pressure Points:** [How to influence them]

**Value:** [Why factions court them]

### The Wild Card: [Name]

**Role:** [Their official position]

**Loyalty:** [Shifts based on advantage]

**Price:** [What buys their support]

**Danger:** [What makes them unpredictable]

### The Idealist: [Name]

**Role:** [Their position and cause]

**Believes:** [Their unshakeable principles]

**Blind Spot:** [What they can't see]

**Breaking Point:** [What would change them]

---

## 5. Reputation Tracking

### Faction Standing

Track PC reputation with each faction (-5 to +5):

| Faction | Starting | Current | Recent Action |
|---------|----------|---------|---------------|
| [Faction 1] | 0 | ___ | [What changed it] |
| [Faction 2] | 0 | ___ | [What changed it] |
| [Faction 3] | 0 | ___ | [What changed it] |

### Reputation Effects

| Standing | Status | Effects |
|----------|--------|---------|
| -5 to -3 | **Enemies** | Assassination risk, active opposition |
| -2 to -1 | **Distrusted** | Watched, excluded from inner circle |
| 0 | **Neutral** | Unknown or cautiously regarded |
| +1 to +2 | **Useful Ally** | Access to resources, minor missions |
| +3 to +5 | **Trusted Insider** | Major missions, secret information |

### Changing Standing

**Gain Standing:**
- Complete faction mission: +1
- Embarrass their rivals: +1
- Provide valuable intelligence: +1
- Public victory for faction: +2
- Save leader's life: +3

**Lose Standing:**
- Fail a mission: -1
- Work for rivals openly: -2
- Betray their trust: -3 to -5

---

## 6. Adventure Content

### Part 1: Entering the Game

**Setup:** The PCs become involved in the political conflict

> **Read Aloud:**
> "[Description of their entry into the political arena - a grand event, a secret meeting, being approached by a faction. The atmosphere of power and danger. The sense that words here are as deadly as swords.]"

**The Introduction:**
- Which faction approaches them first
- What they're asked to do
- What they're offered in return

**Initial Choices:**
| Option | Faction Reaction | Consequences |
|--------|-----------------|--------------|
| Accept Faction 1 | F1 +2, F2 -1 | [What happens] |
| Accept Faction 2 | F2 +2, F1 -1 | [What happens] |
| Play neutral | All 0 | [What happens] |

**Outcomes:**
- **Commit to a faction:** Clear path, clear enemies
- **Stay neutral:** Both court them, neither trusts them
- **Transition:** First faction mission or event

---

### Part 2: Social Battleground Events

#### Event 1: [The Grand Ball / Council Meeting / Festival]

**Setup:** A major social event where all factions gather

> **Read Aloud:**
> "[Description of the event - the grandeur, the tension beneath the surface, the key players present. Every smile hides a knife.]"

**Present Factions:**
- [Faction 1 attendees and their goals for the evening]
- [Faction 2 attendees and their goals]
- [Faction 3 attendees and their goals]

**Opportunities:**
| Opportunity | Skill Check | Success | Failure |
|-------------|-------------|---------|---------|
| [Gather intelligence] | Investigation DC 14 | [Info gained] | [Noticed] |
| [Win over neutral] | Persuasion DC 15 | [Ally gained] | [Offended] |
| [Embarrass rival] | Deception DC 16 | [Standing shift] | [Backfires] |

**Complications:**
- [What could go wrong at this event]
- [Secret meeting that might be discovered]
- [Assassination attempt or confrontation]

---

#### Event 2: [The Secret Meeting / Conspiracy]

**Setup:** A clandestine gathering with high stakes

> **Read Aloud:**
> "[Description of the secret location - the shadows, the whispered words, the paranoia. Everyone here has something to hide.]"

**What's Being Planned:**
- [The conspiracy or deal being made]
- [Who's involved]
- [What they want PCs to do]

**Moral Choice:**
- [Ethical dilemma presented]
- [What each choice costs]

---

#### Event 3: [The Public Crisis / Assassination / Revelation]

**Setup:** A dramatic event that forces action

> **Read Aloud:**
> "[Description of the crisis - chaos erupts, masks fall, true allegiances revealed. The moment everything changes.]"

**What Happens:**
- [The triggering incident]
- [Immediate reactions from each faction]
- [How PCs can influence the outcome]

**Encounter:** `assassination_attempt` (if violence occurs)
<!-- Full stats synced to monsters.md from front matter -->

**Consequences:**
- [How the political landscape shifts]
- [New opportunities created]
- [New dangers emerging]

---

### Part 3: The Crisis Point

**Setup:** All maneuvering comes to a head

> **Read Aloud:**
> "[Description of the moment when decision can no longer be delayed. The factions are at each other's throats. The PCs' choices will determine the outcome.]"

**The Situation:**
- [Current state of each faction]
- [What each faction needs to win]
- [What role PCs can play]

**Resolution Paths:**

| Path | Requirements | PC Role | Outcome |
|------|--------------|---------|---------|
| **Faction 1 Victory** | [What it takes] | [What PCs do] | [What changes] |
| **Faction 2 Victory** | [What it takes] | [What PCs do] | [What changes] |
| **Faction 3 Victory** | [What it takes] | [What PCs do] | [What changes] |
| **Compromise** | [Balance achieved] | [Negotiators] | [Uneasy peace] |
| **Chaos** | [All fail] | [Survivors] | [Power vacuum] |

---

## 7. Information Warfare

### Secrets to Discover

| Secret | About | Impact | How to Find |
|--------|-------|--------|-------------|
| [Damaging info] | [Faction 1] | [Leverage/destruction] | [Method] |
| [Hidden scandal] | [Leader X] | [Leverage] | [Method] |
| [True conspiracy] | [Hidden faction] | [Major revelation] | [Method] |
| [Planned betrayal] | [Alliance] | [Prevent/exploit] | [Method] |

### Using Information

| Action | Effect | Risk |
|--------|--------|------|
| **Blackmail** | Force cooperation | Target becomes enemy |
| **Expose publicly** | Destroy reputation | All-out war |
| **Trade to rival** | Gain favor elsewhere | Original holder's revenge |
| **Protect secret** | Earn loyalty | Miss opportunity |

---

## 8. DM Notes

### Pacing

- Let factions approach PCs, don't force choices
- Build tension through escalating events
- Reveal secrets gradually
- Climax should feel earned by player choices

### Running Social Combat

- Track NPC attitudes like HP
- Social "damage" shifts attitudes
- Recovery possible through effort
- Some NPCs immune to certain approaches

### Common Pitfalls

- **Analysis paralysis:** Factions act without PCs, forcing response
- **Murder-hobos:** Violence has severe political consequences
- **Indecision:** Deadline forces choice regardless
- **Lost in complexity:** Focus on faction leaders, not all members

### Scaling

- **Simpler:** Reduce to 2 factions, clearer goals
- **Complex:** Add subfactions, hidden agendas within factions

---

## 9. Connections

### From Previous Module
[What connections or debts brought them into this political arena]

### To Next Module
[How the new political order creates new adventures]

### Campaign Themes
[How political maneuvering reflects larger campaign themes]

---

## 10. Post-Module Notes

### What Happened
- [ ] Which faction did they support?
- [ ] What secrets did they learn/expose?
- [ ] What enemies did they make?
- [ ] Who now rules?

### Reputation Final State

| Faction | Final Standing | Relationship |
|---------|---------------|--------------|
| [Faction 1] | [+/- X] | [Ally/Enemy/Neutral] |
| [Faction 2] | [+/- X] | [Ally/Enemy/Neutral] |
| [Faction 3] | [+/- X] | [Ally/Enemy/Neutral] |

### New Political Reality
[What the world looks like now]

---

**Status:** [Planning / Ready / Active / Complete]
**Started:** [Date]
**Completed:** [Date]

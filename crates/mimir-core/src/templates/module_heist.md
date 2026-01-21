---
# Template metadata
id: module_heist
title: Heist Module
type: module_heist
level: module
purpose: Planning and execution adventure with infiltration, complications, and getaway
author: Mimir Team

# Module-specific metadata
module_number: 1
theme: "[Theme]"
tone: "Thriller"
estimated_hours: 6

# Catalog references (machine-readable)
monsters:
  # Heists focus on guards and security, not monsters
  - encounter: perimeter_guards
    name: Guard
    source: MM
    quantity: 4
    notes: "Exterior patrols"
  - encounter: interior_security
    name: "[Elite Guard Type]"
    source: MM
    quantity: 2
    notes: "Protect the prize"
  - encounter: alarm_response
    name: Guard
    source: MM
    quantity: 6
    notes: "Reinforcements if alarm raised"
  - encounter: getaway_pursuit
    name: "[Pursuit Force]"
    source: MM
    quantity: 4
    notes: "Chase sequence enemies"

npcs:
  # The mark, crew contacts, and complications
  - role: mark
    name: "[The Mark]"
    source: campaign
    location: "[Their Territory]"
    notes: "The target - wealthy, powerful, dangerous"
  - role: contact
    name: "[The Fixer]"
    source: campaign
    location: "Underworld"
    notes: "Provides the job and intel"
  - role: specialist
    name: "[Hired Expert]"
    source: campaign
    location: "Available for hire"
    notes: "Specialty: [Their skill]"
  - role: wild_card
    name: "[Rival or Insider]"
    source: campaign
    location: "[Location]"
    notes: "Complicates the job"

items:
  - location: target_vault
    name: "[The Prize]"
    source: campaign
    quantity: 1
    notes: "What they're stealing"
  - location: bonus_loot
    name: "[Unexpected Treasure]"
    source: DMG
    quantity: 1
    notes: "Found during heist"

variables:
  - name: target_name
    type: string
    description: Name of the heist target
    default: "[Target Name]"
    required: true
  - name: module_number
    type: number
    description: Module number in campaign sequence
    default: 1
    required: true
---

# Module {{module_number}}: The {{target_name}} Job

*Planning and execution adventure*

---

## 1. Overview

**Pitch:** [One sentence describing the heist]

**The Score:** [What they're stealing]

**The Mark:** [Who they're stealing from]

**The Stakes:** [What happens if caught]

**Estimated Play Time:** [X hours - 2 planning, 2-3 execution, 1 aftermath]

### The Hook

**Who's Hiring:** [The fixer/contact who brings the job]

**Why These PCs:** [Why they were chosen for this job]

**The Payout:** [What they get if successful]

**The Catch:** [Complication or hidden agenda]

### Module Structure

```
[PROPOSITION] → [PLANNING] → [EXECUTION] → [GETAWAY] → [AFTERMATH]
   (hook)        (intel)      (heist)      (escape)    (consequences)
```

---

## 2. The Target

### The Prize

**What It Is:** [Specific item, information, or person]

**Current Location:** [Where it's kept - vault, safe, display, etc.]

**Why It's Valuable:** [To PCs, to employer, to others]

**Who Else Wants It:** [Competing interests that might interfere]

### The Mark

| Aspect | Details |
|--------|---------|
| **Name** | [Full name and title] |
| **Public Face** | [How they appear to society] |
| **True Nature** | [What they're really like] |
| **Resources** | [Wealth, guards, political power] |
| **Weakness** | [Exploitable flaw - schedule, vice, blind spot] |
| **If Crossed** | [How they retaliate - ruthless? Legal? Criminal?] |

---

## 3. The Location

### Overview

**Type:** [Mansion, vault, fortress, museum, etc.]

**Neighborhood:** [Surrounding area - rich district, industrial, etc.]

**Public Access:** [When/if civilians can enter]

**Private Hours:** [When the mark is present]

### Security Layers

#### Layer 1: Perimeter

| Element | Details | Weakness |
|---------|---------|----------|
| **Physical Barriers** | [Walls, fences, gates] | [Gap or vulnerability] |
| **Guards** | [Number, equipment, schedule] | [Shift change, blind spot] |
| **Detection** | [Alarms, dogs, magic] | [Method to bypass] |

#### Layer 2: Building Exterior

| Element | Details | Weakness |
|---------|---------|----------|
| **Entry Points** | [Doors, windows, roof] | [Least defended option] |
| **Surveillance** | [Magical, mechanical, patrols] | [Coverage gaps] |
| **Alarms** | [Type and triggers] | [Disarm method] |

#### Layer 3: Interior

| Element | Details | Weakness |
|---------|---------|----------|
| **Staff** | [Servants, guests, family] | [Routines and blind spots] |
| **Inner Guards** | [Elite security near prize] | [Vulnerability] |
| **Final Barrier** | [Vault, safe, wards] | [Key, code, or bypass] |

---

## 4. Approach Options

### Plan A: Stealth

**Entry:** [How they get in unseen]

**Path to Prize:** [Route through building]

**Acquisition:** [How they get past final barriers]

**Exit:** [How they escape undetected]

**Requirements:** [Skills and resources needed]

**Risk:** [What could go wrong]

### Plan B: Deception

**Cover Story:** [Their pretense for being there]

**Entry:** [How they're invited in]

**Access to Prize:** [How the cover gets them close]

**Exit:** [How they leave naturally]

**Requirements:** [Disguises, forged documents, allies]

**Risk:** [What could expose them]

### Plan C: Smash and Grab

**Entry Point:** [Direct assault method]

**Speed Required:** [How fast they must move]

**Opposition:** [Expected resistance]

**Exit:** [Fighting retreat route]

**Requirements:** [Combat readiness, escape vehicle]

**Risk:** [Casualties, witnesses, pursuit]

---

## 5. Adventure Content

### Part 1: The Proposition

**Setup:** The PCs are approached with the job

> **Read Aloud:**
> "[Description of where and how they're contacted. The fixer's demeanor. The tension of being offered something dangerous and lucrative.]"

**The Pitch:**
- What they're asked to steal
- Why the employer wants it
- What they'll be paid
- The deadline

**Red Flags:** [Warning signs about the job - if any]

**Negotiation:** [What terms can be adjusted]

**Outcomes:**
- **Accept:** Move to planning phase
- **Decline:** [What happens - employer angry? Job goes to rivals?]
- **Counter-offer:** [Modified terms possible]

---

### Part 2: Planning Phase

**Setup:** Gathering intel and resources for the job

> **Read Aloud:**
> "[Description of their planning space - safehouse, tavern back room, etc. The tools and maps spread out. The clock ticking toward the deadline.]"

#### Intelligence Gathering

| Method | What They Learn | DC/Cost |
|--------|-----------------|---------|
| **Surveillance** | Guard schedules, routines | Time + Stealth DC 14 |
| **Social Engineering** | Staff habits, alarm codes | Deception DC 15 |
| **Research** | Building plans, history | Investigation DC 12 |
| **Bribery** | Insider information | 100+ GP |
| **Magic** | Scrying, divination | Spell slots |

#### Key Intelligence Checklist
- [ ] Guard numbers and schedules
- [ ] Alarm systems and triggers
- [ ] Exact location of prize
- [ ] Emergency response protocols
- [ ] Hidden defenses

#### Recruiting Specialists

| Specialist | Skill | Cost | Loyalty |
|------------|-------|------|---------|
| [Name] | [Locks, forgery, etc.] | [GP or favor] | [Reliable? Might betray?] |
| [Name] | [Skill] | [Cost] | [Loyalty] |

---

### Part 3: The Heist

**Setup:** Execution night - everything they planned comes together (or falls apart)

> **Read Aloud:**
> "[Description of the moment before they commit - the target location at night, the last deep breath, then they move.]"

#### Phase 1: Infiltration

**Encounter:** `perimeter_guards`
<!-- Full stats synced to monsters.md from front matter -->

**Approach Execution:**
- [How their chosen plan plays out]
- [First obstacle and how to overcome it]
- [Entering the building proper]

**Heat Level:** Start at 0

#### Phase 2: Navigation

**Moving Through the Building:**
- [Obstacles between entry and prize]
- [Staff to avoid]
- [Checkpoints or barriers]

**Potential Complications:**
| Trigger | Complication | Heat Increase |
|---------|--------------|---------------|
| Failed stealth | Guard alerted | +1 |
| Wrong door | Encounter unexpected occupant | +1 |
| Time pressure | Rushed action, disadvantage | — |

#### Phase 3: Acquisition

**Encounter:** `interior_security`
<!-- Full stats synced to monsters.md from front matter -->

**The Final Barrier:**
> **Read Aloud:**
> "[Description of the vault/safe/display - the prize within reach. The last obstacle between them and success.]"

**Overcoming the Barrier:**
- [Lock/puzzle/ward to bypass]
- [DC or requirements]
- [Time required]

**The Moment of Truth:**
- [Taking the prize]
- [Any alarms triggered]
- [Discovery of bonus loot or complications]

---

### Part 4: The Getaway

**Setup:** They have the prize - now they have to escape

> **Read Aloud:**
> "[The moment of success - prize in hand. But the night isn't over. The building looms behind them. Safety is still far away.]"

#### If Undetected:

**Clean Exit:**
- [Retracing their entry path]
- [Avoiding discovery of the theft]
- [Getting clear before sunrise]

#### If Alarm Raised:

**Encounter:** `alarm_response`
<!-- Full stats synced to monsters.md from front matter -->

**Fighting Retreat:**
- [Combat in confined spaces]
- [Emergency exit routes]
- [Sacrifices and hard choices]

#### The Chase

**Encounter:** `getaway_pursuit`
<!-- If pursuit occurs -->
- [Urban chase mechanics]
- [Obstacles and shortcuts]
- [Losing the pursuers]

---

### Part 5: Aftermath

**Setup:** The immediate aftermath and long-term consequences

#### Immediate

> **Read Aloud:**
> "[The safehouse after the job. The prize on the table. Catching their breath - did they really do it?]"

**Fencing the Goods:**
- [Meeting the buyer]
- [Getting paid]
- [Any complications with payment]

**Laying Low:**
- [How long they need to hide]
- [What the mark is doing in response]

#### Long-Term Consequences

| Outcome | If Clean Heist | If Messy Heist |
|---------|---------------|----------------|
| **Mark's Response** | Doesn't know who | Hunting them |
| **Law Enforcement** | No investigation | Active search |
| **Underworld Rep** | Skilled professionals | Dangerous but sloppy |
| **Future Jobs** | Better offers | Might be blacklisted |

---

## 6. Complication Table

Roll when things go too smoothly (d12):

| Roll | Complication |
|------|--------------|
| 1-2 | Nothing - smooth sailing |
| 3-4 | Minor delay - small obstacle |
| 5-6 | Unexpected guard - wrong place/time |
| 7 | Rival crew - someone else is here! |
| 8 | Changed layout - intel was wrong |
| 9 | Double security - special event tonight |
| 10 | Alarm triggered - time pressure! |
| 11 | Betrayal - inside man flips |
| 12 | It's a trap - they knew you were coming |

---

## 7. Heat Tracking

**Heat Level:** Track growing awareness during the heist

| Heat | Status | Effects |
|------|--------|---------|
| 0 | Undetected | Normal security |
| 1-2 | Something's off | Extra patrols, locked doors |
| 3-4 | Active searching | Guards investigating areas |
| 5-6 | Full alarm | All guards mobilized |
| 7+ | Lockdown | No escape, reinforcements called |

**Actions That Increase Heat:**
- Failed stealth check: +1
- Guard eliminated but not hidden: +1
- Alarm triggered: +2
- Explosion or loud combat: +3
- Witness escapes: +2

---

## 8. DM Notes

### Pacing

- Planning should feel thorough but not endless
- Execution should be tense - dice rolls matter
- Complications should challenge, not destroy plans
- Getaway should be thrilling climax

### Player Agency

- Let their plans work (mostly)
- Reward preparation with easier DCs
- Complications from dice, not arbitrary "gotchas"
- Multiple solutions to every obstacle

### Tone Variants

- **Ocean's Eleven:** Stylish, clever, always a twist
- **Heat:** Professional, intense, things go wrong
- **Pink Panther:** Comedic chaos, everything goes wrong hilariously
- **Mission Impossible:** High-tech, setpieces, last-second saves

### Common Pitfalls

- **Over-planning:** Set a deadline, call for action
- **Too easy:** Add complications, security surprises
- **Too hard:** Let clever plans bypass obstacles
- **Murder-hobos:** Heat system punishes violence

---

## 9. Connections

### From Previous Module
[How they got connected to this job - past contacts, reputation, favors owed]

### To Next Module
[Consequences of the heist - new enemies, new opportunities, what the prize reveals]

### Campaign Themes
[How this heist fits larger narrative - factions affected, power shifted]

---

## 10. Post-Module Notes

### What Happened
- [ ] Which approach did they take?
- [ ] What complications occurred?
- [ ] Clean getaway or messy escape?
- [ ] Full payment or complications?

### Continuity Notes
[New enemies, allies impressed, reputation changes]

### Heat Status
[Are they being hunted? By whom? For how long?]

---

**Status:** [Planning / Ready / Active / Complete]
**Started:** [Date]
**Completed:** [Date]

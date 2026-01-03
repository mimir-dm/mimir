# Artifact Templates

## The Tools That Make the System Work

This chapter provides practical templates for every artifact in the campaign management system. These aren't just forms to fill out—they're tools refined through years of actual play, designed to capture exactly what you need and nothing more.

Each template follows the principle of Minimum Viable Documentation: capture what you'll use, organize it for quick access, and maintain it with minimal effort.

> **Note**: Full templates are available as separate files in the `templates/` directory for easy copying and customization.

## Key Terminology

To ensure clarity throughout these templates:

- **Module**: A complete, runnable adventure unit with encounters, NPCs, locations, and narrative structure. The atomic unit of campaign content.
- **Session**: The actual play time when you and your players meet (3-4 hours typically). A module may span multiple sessions.
- **Arc**: A narrative thread that may span multiple modules or run within a single module
- **Campaign**: The overarching story containing multiple modules
- **Artifact**: Any document, template, or tool used in campaign management
- **Front Matter**: YAML metadata at the top of module files that links to catalog entries (monsters, NPCs, items)

## Quick Reference Table

| Template | Purpose | Level | Link |
|----------|---------|-------|------|
| **Campaign Pitch** | One-page pitch to excite players | Campaign | [Download](./templates/campaign-pitch.md) |
| **Campaign Bible** | Master reference document | Campaign | [Download](./templates/campaign-bible.md) |
| **Starting Scenario** | Player-facing introduction | Campaign | [Download](./templates/starting-scenario.md) |
| **Quick Start Kit** | Launch campaign in 8 weeks | Campaign | [Download](./templates/quick-start-kit.md) |
| **World Overview** | Campaign setting foundation | Campaign | [Download](./templates/world-overview.md) |
| **Region Overview** | Detailed area description | Campaign | [Download](./templates/region-overview.md) |
| **Faction Template** | Power group details | Campaign | [Download](./templates/faction-template.md) |
| **Major NPC Tracker** | Detailed NPC profiles | Campaign | [Download](./templates/major-npc-tracker.md) |
| **PC Arc Tracker** | Character development | Campaign | [Download](./templates/pc-arc-tracker.md) |
| **Character Integration** | Session Zero party building | Campaign | [Download](./templates/character-integration.md) |
| **Module Overview** | General runnable adventure | Module | [Download](./templates/module_overview.md) |
| **Mystery Module** | Investigation adventures | Module | [Download](./templates/module_mystery.md) |
| **Dungeon Module** | Exploration & combat | Module | [Download](./templates/module_dungeon.md) |
| **Heist Module** | Planning & execution | Module | [Download](./templates/module_heist.md) |
| **Political Module** | Intrigue & factions | Module | [Download](./templates/module_political.md) |
| **Horror Module** | Suspense & fear | Module | [Download](./templates/module_horror.md) |
| **Play Notes** | Capture what happened | Session | [Download](./templates/play_notes.md) |
| **Quick NPC Reference** | Tonight's NPCs | Session | [Download](./templates/quick-npc-reference.md) |
| **Clue Tracker** | Information flow | Module | [Download](./templates/clue-tracker.md) |
| **Document Tracker** | Master documentation index | All | [Download](./templates/document-tracker.md) |

> **Note**: For guidance on scaling this system to different game types (one-shots, mini-campaigns, etc.), see [Scaling the System](../05-scaling/README.md).

## Template Categories

### Campaign Level

These artifacts track the big picture across your entire campaign:

#### Campaign Bible
**Purpose**: Master document tracking essential campaign truths
**Update Frequency**: Monthly or after major events
**Key Sections**:
- Core concept and Big Three
- Major themes and timeline
- Power structures and factions
- Established facts and mysteries

[View Full Template](./templates/campaign-bible.md)

#### Starting Scenario
**Purpose**: Player-facing prologue that sets up character creation context
**Update Frequency**: Created once during campaign genesis
**Key Sections**:
- Current situation (what players observe)
- Starting location and circumstances
- Common knowledge and rumors
- Character consideration prompts
- Practical starting details

[View Full Template](./templates/starting-scenario.md)

#### Major NPC Tracker
**Purpose**: Detailed profiles for campaign-shaping NPCs
**Update Frequency**: After each appearance
**Key Sections**:
- Identity and characterization
- Motivations and resources
- Relationships and campaign role
- Play notes and interaction history

[View Full Template](./templates/major-npc-tracker.md)

#### Player Character Arc Tracker
**Purpose**: Monitor long-term character development
**Update Frequency**: Every 3-4 sessions
**Key Sections**:
- Character core and personal arcs
- Mechanical milestones
- Player engagement patterns
- Arc progress checklist

[View Full Template](./templates/pc-arc-tracker.md)

#### Campaign Pitch
**Purpose**: One-page document to excite players about the campaign
**Update Frequency**: Created once, refined during Session Zero
**Key Sections**:
- The Hook and Big Three
- Campaign pillars and tone
- Player buy-in requirements
- Logistics and expectations

[View Full Template](./templates/campaign-pitch.md)

#### Document Tracker
**Purpose**: Master index of all campaign documentation
**Update Frequency**: Weekly review, monthly deep update
**Key Sections**:
- Document status tracking
- Update schedules
- Quick status dashboard
- Works great as a spreadsheet!

[View Full Template](./templates/document-tracker.md)

### Module Level

Modules are complete, runnable adventures. Each module template includes:

- **YAML Front Matter**: Machine-readable references to monsters, NPCs, and items from your catalog
- **Adventure Content**: Read-aloud text, encounters, and scene descriptions
- **DM Notes**: Pacing, scaling, and troubleshooting guidance
- **Post-Module Notes**: Checklists for tracking what happened

#### Module Overview
**Purpose**: General-purpose runnable adventure template
**Update Frequency**: Created during planning, updated after completion
**Key Sections**:
- Front matter with catalog references
- Overview, hook, and structure
- Locations and critical path
- Adventure content with encounters
- Puzzles, challenges, and DM notes

[View Full Template](./templates/module_overview.md)

#### Module Type Templates
**Purpose**: Specialized templates for different adventure styles
**Available Types**:
- [Mystery](./templates/module_mystery.md) - Investigation with clue structures and suspect tracking
- [Dungeon](./templates/module_dungeon.md) - Exploration with room-by-room content
- [Heist](./templates/module_heist.md) - Planning & execution with heat mechanics
- [Political](./templates/module_political.md) - Intrigue with reputation tracking
- [Horror](./templates/module_horror.md) - Suspense with corruption mechanics

Each type template includes the same front matter schema for catalog integration, plus type-specific mechanics and guidance.

#### Clue Tracker
**Purpose**: Manage information flow through investigation-heavy modules
**Update Frequency**: After each session
**Key Sections**:
- Essential clues with multiple sources
- Supporting information
- Red herrings
- Player theories

[View Full Template](./templates/clue-tracker.md)

#### Character Integration Sheet
**Purpose**: Build connected party during Session Zero
**Update Frequency**: Created at Session Zero, referenced throughout
**Key Sections**:
- Individual character hooks
- Party connection web
- Group identity
- Campaign integration notes

[View Full Template](./templates/character-integration.md)

### Session Support

These artifacts support actual game-night execution:

#### Play Notes
**Purpose**: Capture what happened during play for continuity
**Update Frequency**: After each session
**Key Sections**:
- What happened (key events)
- Player reactions
- Dangling threads
- NPCs and loot
- Setup for next session

[View Full Template](./templates/play_notes.md)

#### Quick NPC Reference
**Purpose**: Tonight's NPC stats and notes
**Update Frequency**: Per session
**Key Sections**:
- Appearance and voice notes
- Immediate goals
- Information they provide
- Combat stats if needed

[View Full Template](./templates/quick-npc-reference.md)

### World Building Templates

Tools for creating your campaign setting:

#### World Overview
**Purpose**: Foundation document for your campaign world
**Update Frequency**: As needed when new regions become relevant
**Key Sections**:
- The Six Truths
- Geography and cosmology
- Magic and divine systems
- Current era details

[View Full Template](./templates/world-overview.md)

#### Region Overview
**Purpose**: Detailed area where adventures happen
**Update Frequency**: When PCs enter new regions
**Key Sections**:
- Regional character and identity
- Settlements and points of interest
- Local conflicts and power structures
- Travel and adventure hooks

[View Full Template](./templates/region-overview.md)

#### Faction Template
**Purpose**: Detailed power group documentation
**Update Frequency**: When factions become plot-relevant
**Key Sections**:
- Goals and methods
- Resources and structure
- Key members
- Interaction opportunities

[View Full Template](./templates/faction-template.md)

### Quick Start Templates

#### Quick Start Kit
**Purpose**: Everything needed to launch a campaign in 8 weeks
**Includes**: Minimal versions of essential templates
**Perfect for**: New DMs or quick campaign starts

[View Full Template](./templates/quick-start-kit.md)

## Using the Templates

### Getting Started

1. **Choose Your Core Set**: Start with just 3-4 templates
   - Campaign Bible (Strategic)
   - Module Overview (Tactical)
   - Play Notes (Operational)
   - Quick NPC Reference (Operational)

2. **Customize Format**: Adapt to your tools
   - Markdown for digital notes
   - Print-friendly for physical binders
   - Form-fillable PDFs
   - Spreadsheet versions

3. **Establish Routine**: When to use each
   - Campaign templates: Monthly review
   - Module templates: Arc planning
   - Session templates: After each game

### Recommended Organization

```
campaigns/
└── {campaign_name}/
    ├── campaign_bible.md        (Master reference)
    ├── pitch.md                 (Campaign pitch)
    ├── document_tracker.md      (Documentation index)
    ├── session_zero/
    │   ├── agenda.md
    │   ├── safety_tools.md
    │   └── character_creation_guide.md
    ├── world/
    │   ├── overview.md
    │   ├── pantheon.md
    │   ├── timeline.md
    │   └── factions/
    ├── regions/
    │   └── {region_name}/
    │       ├── overview.md
    │       ├── map.png
    │       └── settlements/
    ├── modules/
    │   └── {number}_{name}/
    │       ├── overview.md       (The runnable module)
    │       ├── monsters.md       (Synced from front matter)
    │       ├── npcs.md          (Synced from front matter)
    │       ├── items.md         (Synced from front matter)
    │       └── handouts/
    ├── play_notes/
    │   └── session_{number}.md   (What happened each session)
    ├── characters/
    │   ├── pc_arc_tracker.md
    │   └── {character_name}/
    │       ├── backstory.md
    │       ├── notes.md
    │       └── art/
    ├── npcs/
    │   ├── major_npcs.md       (Campaign NPCs)
    │   └── recurring/
    │       └── {npc_name}/
    └── resources/
        ├── maps/
        ├── handouts/
        └── references/
```

## Customization Guidelines

Templates are starting points—adapt them to your needs:

- **Remove** sections you won't use
- **Expand** areas that need more detail
- **Combine** templates for simpler games
- **Create** new templates for your specific genre

## The Minimal Viable Set

If you're overwhelmed, start with just these three:

1. **Module Overview** - Your runnable adventure content
2. **Play Notes** - Track what happened for continuity
3. **Quick NPC Reference** - Game night essentials

You can always expand later as your needs grow.

## Download and Customize

All templates are available as markdown files for easy customization. Copy what works, modify what doesn't, and create your own sustainable documentation practice.

Remember: The best template is the one you actually use. These tools serve your game, not the other way around.

---

*Next: Browse the [template files](./templates/) to find the tools that match your campaign style.*

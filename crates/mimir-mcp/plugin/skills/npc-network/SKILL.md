# NPC Network Analysis

This skill should be used when the user asks to "map NPC relationships", "show character connections", "analyze NPC network", "who knows who", "faction relationships", "visualize NPCs", "NPC web", "character relationship map", or mentions "NPC connections", "social network", or "faction dynamics".

## Purpose

Analyze and visualize the relationships between NPCs, their faction affiliations, knowledge networks, and social connections. Identify isolated NPCs, missing connections, and opportunities for richer storytelling.

## Analysis Process

### 1. Gather NPC Data

```
list_characters(character_type: "npc")
# For each NPC:
get_character(character_id)
```

Extract from each NPC:
- Name, role, location
- Faction affiliation
- Personality (traits, ideals, bonds, flaws)
- Module assignment

### 2. Extract Implicit Relationships

Read documents to find relationship mentions:

```
# For each module:
list_documents(module_id)
read_document(document_id)
```

Look for:
- Direct relationships: "X is Y's brother"
- Implied connections: Characters in same location
- Faction ties: Members of same organization
- Conflict relationships: Enemies, rivals
- Knowledge chains: Who knows what secrets

### 3. Build Relationship Matrix

Categorize relationships:

| Type | Description |
|------|-------------|
| **Family** | Blood relations, marriage |
| **Professional** | Employer/employee, colleagues |
| **Faction** | Same organization membership |
| **Location** | Same place, neighbors |
| **Conflict** | Enemies, rivals, grudges |
| **Secret** | Hidden connections players can discover |
| **Knowledge** | Who knows information about whom |

### 4. Network Analysis

Identify:

#### Hub NPCs
- Characters with many connections
- These are high-value targets for players
- Consider: Are they protected? What if they die?

#### Isolated NPCs
- Characters with no connections
- Missed storytelling opportunities
- Consider: How do players encounter them?

#### Faction Clusters
- Groups of connected NPCs
- Internal faction dynamics
- Cross-faction connections

#### Information Flow
- How does news travel?
- Who would know if X happened?
- Rumor mill paths

## Output Format

### Text Relationship Map

```markdown
# NPC Network: [Campaign Name]

## Faction: [Faction Name]
├── [Leader NPC] (Leader)
│   ├── employs → [NPC]
│   └── rivals → [NPC from other faction]
├── [Member NPC]
│   └── siblings → [NPC]
└── [Member NPC]

## Location: [Location Name]
├── [NPC] - [Role]
├── [NPC] - [Role]
└── [NPC] - [Role]

## Key Relationships
- [NPC] ←secret lovers→ [NPC]
- [NPC] ←owes debt→ [NPC]
- [NPC] ←seeking revenge→ [NPC]

## Isolated NPCs (No Connections)
- [NPC]: Consider connecting to [suggestion]

## Hub NPCs (4+ Connections)
- [NPC]: [List of connections]
  - Risk: High-value target
  - Contingency: [Suggestion]
```

### Mermaid Diagram (if requested)

```
graph TD
    A[Lord Mayor] -->|employs| B[Captain of Guard]
    A -->|rivals| C[Merchant Prince]
    B -->|siblings| D[Innkeeper]
    C -->|secret deal| E[Thieves Guild Leader]
```

## Interactive Mode

1. Present overall network structure
2. Ask: "Which relationships would you like to explore?"
3. Deep dive into specific factions or characters
4. Suggest missing connections: "These NPCs are in the same location but have no relationship defined. Should they know each other?"
5. Offer to update NPC records with discovered relationships

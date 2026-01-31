---
name: npc-network
description: >-
  This skill should be used when the user asks to "map NPC relationships",
  "show character connections", "analyze NPC network", "who knows who",
  "faction relationships", "visualize NPCs", "NPC web",
  "character relationship map", or mentions "NPC connections",
  "social network", or "faction dynamics". Maps relationships between NPCs
  by faction, location, and document references to identify hub characters,
  isolated NPCs, and missing connections.
---

# NPC Network Analysis

## Purpose

Analyze and visualize relationships between NPCs, their faction affiliations, knowledge networks, and social connections. Identify isolated NPCs, missing connections, and opportunities for richer storytelling.

## Analysis Process

### 1. Gather NPC Data

```
list_characters(character_type: "npc")
# Filter by location or faction for focused analysis:
list_characters(character_type: "npc", location: "Waterdeep")
list_characters(character_type: "npc", faction: "Zhentarim")
# For each NPC:
get_character(character_id: character_id)
```

Extract from each NPC: name, role, location, faction affiliation, and module assignment.

### 2. Extract Implicit Relationships

Read documents to find relationship mentions:

```
# Campaign-level documents (world lore, session notes)
list_documents()  # omit module_id for campaign-level docs
read_document(document_id: document_id)
# Module-level documents
# For each module:
list_documents(module_id: module_id)
read_document(document_id: document_id)
```

Look for:
- Direct relationships: "X is Y's brother"
- Implied connections: Characters in the same location
- Faction ties: Members of the same organization
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

**Hub NPCs** — Characters with many connections. These are high-value targets for players. Consider: Are they protected? What happens if they die?

**Isolated NPCs** — Characters with no connections. These represent missed storytelling opportunities. Consider: How do players encounter them?

**Faction Clusters** — Groups of connected NPCs. Map internal faction dynamics and cross-faction connections.

**Information Flow** — How does news travel? Who would know if X happened? Trace rumor mill paths.

## Output Format

```markdown
# NPC Network: [Campaign Name]

## Faction: [Faction Name]
+-- [Leader NPC] (Leader)
|   +-- employs -> [NPC]
|   +-- rivals -> [NPC from other faction]
+-- [Member NPC]
|   +-- siblings -> [NPC]
+-- [Member NPC]

## Location: [Location Name]
+-- [NPC] - [Role]
+-- [NPC] - [Role]
+-- [NPC] - [Role]

## Key Relationships
- [NPC] <-secret lovers-> [NPC]
- [NPC] <-owes debt-> [NPC]
- [NPC] <-seeking revenge-> [NPC]

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
3. Deep-dive into specific factions or characters
4. Suggest missing connections: "These NPCs share a location but have no defined relationship. Should they know each other?"
5. Offer to update NPC records with discovered relationships

---
name: continuity-check
description: >-
  This skill should be used when the user asks to "check continuity",
  "verify plot consistency", "find contradictions", "audit my campaign",
  "check for inconsistencies", "verify NPC references", "timeline check",
  "fact check my module", "continuity audit", or mentions "plot holes",
  "contradictions", or "consistency issues".
---

# Plot Continuity Check

## Purpose

Systematically verify that all references in campaign documents are internally consistent — NPCs exist, locations are established, timelines align, and facts do not contradict each other.

## Continuity Check Process

### 1. Extract All References

Load and analyze all campaign content:

```
get_campaign_details(campaign_id)
# Campaign-level documents (world lore, session notes, etc.)
list_documents()  # omit module_id for campaign-level docs
read_document(document_id)
# Module-level documents
list_modules()
# For each module:
get_module_details(module_id)
list_documents(module_id)
read_document(document_id)
```

### 2. Build Reference Index

Extract and catalog all named entities:

#### Characters
- Extract NPC names mentioned in documents
- Cross-reference with `list_characters(character_type: "npc")`
- Note roles, locations, and relationships

#### Locations
- Extract places mentioned in read-aloud text, backstory, and NPC locations
- Check for consistent spelling and naming

#### Items
- Extract named items in documents, module loot, and character inventories
- Verify catalog references exist via `search_items`

#### Timeline Events
- Extract dates, times, and relative references ("X days ago")
- Check character ages against event dates
- Verify sequence consistency

#### Factions/Organizations
- Extract groups mentioned in documents
- Map NPC faction assignments
- Document inter-faction relationships

### 3. Cross-Reference Check

For each reference, verify:

| Check | Question |
|-------|----------|
| **Existence** | Does this NPC/location actually exist in the campaign? |
| **Spelling** | Is the name spelled consistently everywhere? |
| **Facts** | Are stated facts consistent across documents? |
| **Timeline** | Do dates and sequences make sense? |
| **Relationships** | Are NPC relationships consistently described? |

### 4. Common Continuity Issues

Look specifically for:

- **Orphan References**: NPC mentioned in document but not created as character
- **Name Variations**: "Lord Blackwood" vs "Baron Blackwood" vs "Blackwood"
- **Location Drift**: Inn called "The Rusty Nail" in one doc, "The Bent Nail" in another
- **Timeline Paradoxes**: Event happened "10 years ago" but NPC was "born 5 years ago"
- **Resurrection Issues**: Dead NPC referenced as alive in later content
- **Missing Connections**: NPC has no way to know information they reveal
- **Distance Problems**: Locations described inconsistently (next door vs across town)

## Output Format

Provide a structured continuity report:

```markdown
# Continuity Report: [Campaign Name]

## Summary
- Documents analyzed: X
- NPCs referenced: Y
- Locations referenced: Z
- Issues found: N

## Character Continuity

### Verified NPCs
| Name | Location | Role | Status |
|------|----------|------|--------|
| [Name] | [Location] | [Role] | [OK] Consistent |

### Orphan References
- "[Name]" mentioned in [Document] but no character record exists
- Suggestion: Create NPC or clarify reference

### Inconsistencies
- [NPC] described as [X] in [Doc1] but [Y] in [Doc2]

## Location Continuity

### Verified Locations
- [Location]: Referenced in [X] documents, consistent

### Naming Inconsistencies
- "[Name1]" vs "[Name2]" - likely same location?

## Timeline Continuity

### Verified Events
| Event | When | Referenced In |
|-------|------|---------------|
| [Event] | [Time] | [Documents] |

### Paradoxes
- [Event1] and [Event2] create impossible timeline

## Recommendations
1. [Specific fix]
2. [Specific fix]
```

## Interactive Mode

When checking interactively:

1. Start with a high-level scan
2. Report findings by category
3. Ask: "Should I investigate [specific issue] further?"
4. Offer to create missing NPCs or update documents to fix inconsistencies

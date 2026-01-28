# Plot Continuity Check

This skill should be used when the user asks to "check continuity", "verify plot consistency", "find contradictions", "audit my campaign", "check for inconsistencies", "verify NPC references", "timeline check", "fact check my module", "continuity audit", or mentions "plot holes", "contradictions", or "consistency issues".

## Purpose

Systematically verify that all references in campaign documents are internally consistent - NPCs exist, locations are established, timelines align, and facts don't contradict each other.

## Continuity Check Process

### 1. Extract All References

Load and analyze all campaign content:

```
get_campaign_details(campaign_id)
list_modules()
# For each module:
get_module_details(module_id)
list_documents(module_id)
# Read each document
read_document(document_id)
```

### 2. Build Reference Index

Extract and catalog all named entities:

#### Characters
- NPC names mentioned in documents
- Cross-reference with `list_characters(character_type: "npc")`
- Note: roles, locations, relationships

#### Locations
- Places mentioned in read-aloud text
- Places mentioned in backstory
- Places in NPC locations field
- Check for consistent spelling/naming

#### Items
- Named items in documents
- Module loot items
- Character inventory items
- Verify catalog references exist

#### Timeline Events
- Dates/times mentioned in backstory
- Sequence of events
- "X days ago" references
- Character ages vs event dates

#### Factions/Organizations
- Groups mentioned in documents
- NPC faction assignments
- Relationships between factions

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
| [Name] | [Location] | [Role] | ✓ Consistent |

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

1. Start with high-level scan
2. Report findings by category
3. Ask: "Should I investigate [specific issue] further?"
4. Offer to create missing NPCs or update documents to fix inconsistencies

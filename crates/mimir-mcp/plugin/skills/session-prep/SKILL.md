# Session Prep Review

This skill should be used when the user asks to "prep for session", "review before game night", "session checklist", "am I ready to run this", "pre-session review", "game prep", "DM prep check", or mentions "running tonight", "session tomorrow", or "game day prep".

## Purpose

Comprehensive pre-session review that combines continuity checking, encounter balance, NPC readiness, and identifies gaps that need attention before game time.

## Pre-Session Checklist

### 1. Content Readiness

Load the module being run:

```
get_module_details(module_id)
list_documents(module_id)
```

Verify:
- [ ] Module overview document exists and is complete
- [ ] Read-aloud text prepared for key moments
- [ ] DM notes cover contingencies
- [ ] Maps/handouts referenced are available

### 2. NPC Readiness

```
list_characters(character_type: "npc", module_id: module_id)
```

For each NPC players will likely encounter:
- [ ] Name, role, location defined
- [ ] Personality traits filled in (for roleplay)
- [ ] Motivation/goals clear
- [ ] Key information they know documented
- [ ] Voice/mannerism notes (optional)

### 3. Encounter Readiness

Review module monsters:
- [ ] Monster stat blocks accessible
- [ ] Tactics noted in DM notes
- [ ] Terrain/environmental factors documented
- [ ] Treasure/loot defined

### 4. Plot Thread Check

- [ ] Current plot hooks are clear
- [ ] Player goals acknowledged
- [ ] Clues/information properly placed
- [ ] Multiple paths to success exist

### 5. Contingency Prep

- [ ] "What if they skip X" planned
- [ ] "What if NPC dies" contingency
- [ ] Backup hooks ready
- [ ] Improvisation anchors noted

## Output Format

```markdown
# Session Prep Report: [Module Name]

## ✅ Ready
- [List of complete elements]

## ⚠️ Needs Attention
- [Element]: [What's missing] → [Quick fix suggestion]

## 🚨 Critical Gaps
- [Element]: [Why it's critical] → [Action needed]

## NPCs for This Session

| NPC | Location | Ready? | Missing |
|-----|----------|--------|---------|
| [Name] | [Location] | ✅/⚠️ | [What's missing] |

## Encounters

| Encounter | Difficulty | Ready? | Notes |
|-----------|------------|--------|-------|
| [Name] | [Difficulty] | ✅/⚠️ | [Notes] |

## Quick Reference

### Key NPCs
- **[Name]**: [1-line summary, key info they have]

### Key Locations
- **[Location]**: [What happens here]

### Plot Threads
- [Thread]: [Current state, next beat]

## DM Notes
[Any generated prep notes or reminders]
```

## Interactive Mode

1. Identify the module being run
2. Run through checklist categories
3. For each gap, offer to:
   - Create missing NPC details
   - Add DM notes
   - Generate read-aloud text
   - Suggest encounter adjustments
4. Generate a condensed "cheat sheet" for the session

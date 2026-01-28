# Campaign Pressure Testing

This skill should be used when the user asks to "pressure test my campaign", "stress test this module", "find plot holes", "what if players do X", "test my adventure", "challenge my scenario", "poke holes in my plot", "adversarial review", "player-proof this", or mentions "edge cases", "breaking the campaign", or "unexpected player actions".

## Purpose

Systematically challenge campaign content by adopting an adversarial player mindset. Identify weaknesses, plot holes, and failure points before players discover them.

## Pressure Test Process

### 1. Gather Context

First, load the campaign and module being tested:

```
get_campaign_details(campaign_id)
get_module_details(module_id)
list_characters(character_type: "npc")
```

Read all relevant documents to understand the scenario.

### 2. Apply Pressure Test Categories

Work through each category, asking probing questions:

#### Murder Hobo Test
- What if players kill the quest giver immediately?
- What if they attack every NPC on sight?
- Are there consequences that don't dead-end the campaign?

#### Skip Content Test
- What if players ignore the hook entirely?
- Can they bypass the dungeon and go straight to the BBEG?
- What if they refuse the quest?

#### Clever Solution Test
- Can they solve this with a single spell (Fly, Teleport, Speak with Dead)?
- What if they befriend the villain?
- Can divination magic reveal too much too early?

#### Resource Test
- What if they're out of spell slots when they reach the boss?
- What if they have unlimited money to throw at the problem?
- What if they hire an army of mercenaries?

#### Information Test
- What happens if they miss the key clue?
- Are there backup ways to get critical information?
- What if they get information out of order?

#### Social Test
- What if they intimidate/charm their way past obstacles?
- Can high Persuasion trivialize the challenge?
- What if they take hostages?

### 3. Document Findings

For each issue found, categorize severity:

| Severity | Description |
|----------|-------------|
| **Critical** | Campaign dead-ends, no recovery possible |
| **Major** | Significant content bypassed, story breaks |
| **Minor** | Suboptimal experience but recoverable |
| **Note** | Something to be aware of during play |

### 4. Suggest Mitigations

For each issue, propose solutions:
- Add contingency NPCs/events
- Create backup information sources
- Add consequences instead of blocking
- Prepare improvisational anchors

## Interactive Mode

When pressure testing interactively:

1. Present a scenario: "The players have just arrived at the tavern where the quest giver waits..."
2. Ask: "What's the worst thing your players might do here?"
3. Explore the consequences together
4. Document the failure mode and mitigation

## Output Format

Provide a structured pressure test report:

```markdown
# Pressure Test Report: [Module Name]

## Executive Summary
[1-2 sentence overall assessment]

## Critical Issues
- [Issue]: [Why it's a problem] → [Suggested fix]

## Major Issues
- [Issue]: [Why it's a problem] → [Suggested fix]

## Minor Issues
- [Issue]: [Why it's a problem] → [Suggested fix]

## Strengths
- [What the module does well defensively]

## Recommended Preparations
- [DM notes to add]
- [Contingency NPCs to prepare]
- [Backup plot hooks]
```

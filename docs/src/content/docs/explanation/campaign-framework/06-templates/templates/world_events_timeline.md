---
title: "World Events Timeline"
---

The template's metadata:

```yaml
---
template_type: integration
template_id: world_events_timeline
version: 1.0.0
game_system: dnd_5e
content_type: timeline
status: active
author: Campaign Framework
description: Master timeline of historical and ongoing world events
suggested_stage: integration
tags: 
  - timeline
  - history
  - worldbuilding
  - events
default_context:
  campaign_name: "{{ campaign_name }}"
  world_name: "{{ world_name }}"
  current_year: "{{ current_year }}"
  calendar_system: "{{ calendar_system }}"
---
```

## Current Era: {{ current_year }} {{ calendar_system }}

### Ancient History (1000+ years ago)
- **The Founding Era**: [Key events that shaped the world's foundations]
- **Rise of Civilizations**: [Major cultural developments]
- **The Great Calamity**: [World-changing event that still affects today]

### Recent History (100-1000 years ago)
- **{{ current_year - 500 }}**: [Major historical event]
- **{{ current_year - 250 }}**: [Political change or war]
- **{{ current_year - 100 }}**: [Recent significant event]

### Modern Era (Within 100 years)
- **{{ current_year - 50 }}**: [Event that older NPCs remember]
- **{{ current_year - 20 }}**: [Event that shaped current politics]
- **{{ current_year - 10 }}**: [Recent event affecting the present]
- **{{ current_year - 5 }}**: [Very recent development]

### Current Year Events
- **Spring**: [Seasonal events or developments]
- **Summer**: [Mid-year occurrences]
- **Autumn**: [Harvest time events]
- **Winter**: [Year-end situations]

## Ongoing Conflicts & Tensions
- **Political**: [Current power struggles]
- **Economic**: [Trade disputes or resource conflicts]
- **Religious**: [Faith-based tensions]
- **Territorial**: [Border disputes or expansions]

## Future Events (Planned/Prophesied)
- **Near Future** (within 1 year): [Imminent events]
- **Medium Term** (1-5 years): [Developing situations]
- **Long Term** (5+ years): [Distant possibilities]

## Secret History
*DM Only - Not Known to Players Initially*
- [Hidden historical truth]
- [Concealed event that explains current situation]
- [Secret that could change everything]

## Timeline Hooks for PCs
- **[PC Name]**: How their backstory connects to [specific event]
- **[PC Name]**: Their family/organization's role in [historical event]
- **[PC Name]**: Personal stake in [ongoing conflict]

## Campaign Integration Notes
- Which events can PCs influence?
- What historical mysteries might they uncover?
- How do current events drive the campaign forward?
# Character Print Dialog - UI Wireframe

Modal dialog for selecting which sections to include in character PDF export.

```
┌──────────────────────────────────────────────────────────────────┐
│  Print Character                                            ✕   │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Select sections to include:                                     │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  [✓] Compact Sheet (2-page)                                │  │
│  │      Stats, combat, skills, equipment summary              │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  [ ] Long Form                                             │  │
│  │      Appearance, personality, backstory, RP notes          │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  [✓] Spell Cards                                           │  │
│  │      Printable cards for all spells (if caster)            │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  [ ] Equipment Detail                                      │  │
│  │      Full inventory with descriptions and special rules    │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│                                         [ Cancel ]  [ Export ]   │
└──────────────────────────────────────────────────────────────────┘
```

## Behavior

| Element | Behavior |
|---------|----------|
| Section checkboxes | Toggle inclusion in PDF |
| Default selection | Compact Sheet + Spell Cards |
| Spell Cards option | Always visible - if no spells, silently produces nothing |
| Cancel | Close dialog, no action |
| Export | Generate PDF with selected sections, download |

## Section Order in PDF

Sections appear in PDF in this order (regardless of selection order):
1. Compact Sheet (2 pages)
2. Long Form (1-2 pages)
3. Spell Cards (variable pages)
4. Equipment Detail (variable pages)

## States

- **At least one section required** - Export button disabled if nothing selected
- **Loading state** - Show spinner on Export button while generating PDF
- **Error state** - Toast notification if generation fails

## Notes

- Simple checkbox list - no drag/drop reordering needed
- Descriptions help user understand what each section contains
- Spell Cards always available - silently no-op if spell list is empty

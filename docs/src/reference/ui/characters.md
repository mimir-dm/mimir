# Characters and Character Sheet

The characters page (`/characters`) lists every character. The character sheet (`/characters/<id>`) shows one character.

## Characters Page

The page groups the characters by campaign. Characters in no campaign are under **No campaign**.

| Column | Contents |
|--------|----------|
| **Name** | The name. Click it to open the sheet. |
| **Kind** | PC or NPC |
| **Details** | Classes with levels, and race |
| **Player** | The player of a PC |
| **Campaign** | A menu to move the character to another campaign, or to **No campaign** |
| **Edit** | Open the sheet |
| **PDF…** | Export the sheet to PDF |
| **Level up** | Open the level-up dialog (a PC below level 20) |
| **×** | Delete the character, after a confirmation |

**New character** opens the wizard. It asks for the **Kind** and the **Campaign** first.

## Character Wizard

| Kind | Steps |
|------|-------|
| PC | Basics, Race, Class, Background, Ability scores, Skills, Review. The class starts at level 1. |
| NPC | One form: **Name**, **Race**, **Role**, **Location**, **Faction** |

See [Create a Player Character](../../how-to/characters/create-pc.md).

## Character Sheet

### Header

| Element | Function |
|---------|----------|
| Name | The title |
| "Level … · race · classes" | The summary line |
| **Level up** | Open the level-up dialog. A PC below level 20 only. |
| **PDF…** | Export the sheet to PDF (DM) |
| **Source books…** | Choose the books of this character (DM) |

### Tabs

| Tab | Contents |
|-----|----------|
| **Character** | Abilities, saving throws, skills, armor class, initiative, speed, hit dice, passive Perception, proficiency, attacks |
| **Equipment** | **Coins**, **Equipped**, **Inventory**, **Add an item** (with notes). A name opens the item. See [Manage Inventory](../../how-to/characters/manage-inventory.md). |
| **Spells** | Casters only: spellcasting, **Spell slots**, **Known spells** by level (a name opens the spell), **Learn a spell…**. See [Manage Spells](../../how-to/characters/manage-spells.md). |
| **Details** | An NPC's **Role**, **Location** and **Faction**; personality (traits, ideals, bonds, flaws); the **Background** (a click opens it); classes, proficiencies, languages, feats, chosen features, and the class and subclass features up to the character's level, each opening to its text. **Save** saves the fields. A PC also has **Share this sheet with the party (read-only)**. |
| **Notes** | The player's notes. The DM reads them; the player writes them. |

The sheet follows changes live. When a player changes their sheet, the DM sees it at once.

## Who Can Do What

| Action | DM | The character's player | Another player (shared sheet) |
|--------|----|------------------------|-------------------------------|
| Read the sheet | Yes | Yes | Yes |
| Change the sheet, level up | Yes | Yes | No |
| Read the notes | Yes | Yes | No |
| Write the notes | No | Yes | No |
| PDF, source books | Yes | No | No |

## See Also

- [Create a Player Character](../../how-to/characters/create-pc.md)
- [Level Up](../../how-to/characters/level-up.md)
- [Player Page](./player-page.md)
- [Characters data reference](../characters/README.md)

# Module Page

The module page (`/modules/<id>`) holds one adventure: its documents, NPCs, maps, monsters and the notes of the players. The open tab is in the address (`?section=`).

## Header

| Element | Function |
|---------|----------|
| Module name | The title, with "Module …" (its number) under it. Click it to rename the module in place: **Enter** or leaving the field saves, **Esc** cancels. |
| **Modules** | Back to the **Modules** tab of the campaign |
| **Play** | Open the DM map of the first map of the module (shown when the module has a map) |
| **Edit module** | Change the **Name**, **Type** and **Description** |
| **More** | The module menu (below) |

### More Menu

| Item | Function |
|------|----------|
| **Rename** | Rename the module in place, as a click on its name |
| **PDF…** | Export the module to PDF. See [PDF Export](../pdf-export/README.md). |
| **Monster cards (PDF)** | Download a card for each monster of the module |
| **Delete module…** | Delete the module, its documents, monsters and maps, after a confirmation |

## Tabs

### Documents

The module documents: the list on the left, the open document on the right. New modules start with an overview for their type and **Play Notes**. See [Module Documents](../../how-to/modules/module-documents.md).

### NPCs

A table of the module NPCs: **Name**, **Role**, **Description**. Click a name to see all of the NPC: role, description, appearance, personality, motivation and secrets.

### Maps

The module maps.

| Column | Contents |
|--------|----------|
| **Map** | The name. Click it to open the [DM map](./dm-map.md). |
| **Lighting** | bright, dim or dark |
| **Fog of war** | On or Off |
| **×** | Delete the map, its tokens, lights, traps and points of interest |

**Upload a map** adds a map. See [Upload a Map](../../how-to/maps/upload-map.md).

### Dangers

The monsters of the module, and the traps and points of interest of its maps.

- **Add a monster**: **Monster name**, **Search**, then **Add** next to a result. Results show their source; homebrew shows **Homebrew**. The search keeps to the campaign's source books when the campaign has some.
- **Monsters** table: **Monster**, **Count**, **Source**, **Notes**. Click the monster name to open its stat block in a drawer.
- **Edit**: change **Count**, **Name shown (empty: the monster's name)** and **DM notes**.
- **×** (**Remove …**): remove the monster from the module. Its tokens on the maps stay.
- **Traps** table: **Name**, **Map**, **Players see it**. Click a name: a trap with the name of a catalog trap opens it in a drawer; another trap shows its DC, trigger, effect, notes and state, with **Open the map**.
- **Points of interest** table: **Name** (with its icon), **Map**, **Players see it**. Click a name for its icon, description and **Open the map**.

### Player notes

The notes that players wrote while a map of this module was shown. Each note names its character. Read-only.

## See Also

- [DM Map](./dm-map.md)
- [Create a Module](../../how-to/modules/create-module.md)
- [Add Monsters](../../how-to/modules/add-monsters.md)

# Module Page

The module page (`/modules/<id>`) holds one adventure: its documents, NPCs, maps, monsters and the notes of the players. The open tab is in the address (`?section=`).

## Header

| Element | Function |
|---------|----------|
| Module name | The title, with "Module …" (its number) under it |
| **Modules** | Back to the **Modules** tab of the campaign |
| **Play** | Open the DM map of the first map of the module (shown when the module has a map) |
| **Edit module** | Change the **Name**, **Type** and **Description** |
| **More** | The module menu (below) |

### More Menu

| Item | Function |
|------|----------|
| **PDF…** | Export the module to PDF. See [PDF Export](../pdf-export/README.md). |
| **Monster cards (PDF)** | Download a card for each monster of the module |
| **Delete module…** | Delete the module, its documents, monsters and maps, after a confirmation |

## Tabs

### Documents

The module documents: the list on the left, the open document on the right. New modules start with an overview for their type and **Play Notes**. See [Module Documents](../../how-to/modules/module-documents.md).

### NPCs

A read-only table of the module NPCs: **Name**, **Role**, **Description**.

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

The monsters of the module.

- **Add a monster**: **Monster name**, **Search**, then **Add** next to a result. Results show their source; homebrew shows **Homebrew**.
- **Monsters** table: **Monster**, **Count**, **Source**, **Notes**. Click the monster name to open its stat block in a drawer.
- **Edit**: change **Count**, **Name shown (empty: the monster's name)** and **DM notes**.
- **×** (**Remove …**): remove the monster from the module. Its tokens on the maps stay.

### Player notes

The notes that players wrote while a map of this module was shown. Each note names its character. Read-only.

## See Also

- [DM Map](./dm-map.md)
- [Create a Module](../../how-to/modules/create-module.md)
- [Add Monsters](../../how-to/modules/add-monsters.md)

# Upload a Map

Add battle maps and area maps to a module or to the campaign.

## Supported Formats

| Format | Extensions | Grid | Walls and doors |
|--------|-----------|------|-----------------|
| **UVTT** | `.dd2vtt`, `.uvtt` | In the file | In the file |
| **Image** | `.png`, `.jpg`, `.jpeg`, `.webp` | You set it | None |

## Steps

1. Open the module page and click the **Maps** tab. For a campaign map, open the **Campaign** tab of the dashboard instead.
2. Click **Upload a map**.
3. Drop the map file on the box, or click the box to choose it. Mimir fills in **Name** from the file name.
4. Change the **Name** if you want.
5. For an image, set **Grid: pixels a square**. The default is 70.
6. Choose the **Lighting**: Bright, Dim or Dark.
7. Click **Upload**.

The map appears in the **Maps** table with its lighting and its fog of war state. Click the name to open the [DM map](../../reference/ui/dm-map.md).

To see a map without opening it, click **Preview** in its row. The preview shows the image, the size in squares, the pixels a square and the size in pixels, with **Open** and **PDF…**.

If the file is not a map, Mimir shows the reason in the dialog. Click **Cancel** to close it.

## UVTT Files (Recommended)

UVTT files from tools like [Dungeondraft](https://dungeondraft.net/) hold the grid, the walls and the doors. Fog of war then stops at walls, and doors open and close. See [Map Formats](../../explanation/map-formats.md).

## Image Files

An image works for tokens, lights and fog of war. It has no walls, so sight goes in all directions up to its range.

## Lighting

The lighting of the map is the light where no light source shines:

- **Bright**: daylight. Characters see as far as they can.
- **Dim**: twilight. Characters see their dim-light range.
- **Dark**: night or underground. Only darkvision and light sources give sight.

The player display uses this lighting. You choose it when you upload the map.

## Generated Maps

`mimir-mapgen` makes `.dungeondraft_map` files. Mimir cannot open them. Open the file in Dungeondraft and export it as Universal VTT. Then upload the UVTT file.

## Delete a Map

1. In the **Maps** table, click **×** (**Delete …**) in the row of the map.
2. Click **Delete** to confirm.

Its tokens, lights, traps and points of interest go with it.

## Tips

- Use UVTT when you can.
- Upload one map for each room or area of a dungeon.
- Campaign maps are for places that many modules use, for example a world map or a home town.

## See Also

- [Set the Grid](./configure-grid.md)
- [Place Tokens](./place-tokens.md)
- [Map Formats](../../explanation/map-formats.md)
- [Generate Maps](./generate-map.md)

# Verify Grid Alignment

How to check that your map's grid matches Mimir's grid overlay, and what to do if it doesn't.

## Goal

Ensure that tokens snap to the correct grid squares on your map.

## Steps

### For UVTT Files

1. Upload your UVTT file (see [Upload a Map](./upload-map.md))
2. Open Token Setup by clicking the map card
3. **Verify:** The grid overlay should align with the map's drawn grid lines
4. Place a test token — it should snap to the center of a grid square

UVTT files embed grid data (cell size, offset, dimensions), so alignment is automatic. If the grid looks correct, you're done.

### For Image Files

1. Upload your image file
2. Open Token Setup by clicking the map card
3. **Check alignment:** Mimir defaults to 70 pixels per grid square, starting from the top-left corner
4. If the overlay grid lines align with the map's drawn grid, you're ready to place tokens

### If the Grid Doesn't Align (Image Files)

If your image uses a non-standard grid size (e.g., 100px squares), the overlay won't match the map artwork. Options:

1. **Obtain the UVTT version** of the map if available — this is the best fix
2. **Use the map as a visual backdrop** and position tokens by eye rather than relying on snap-to-grid

## How Token Snapping Works

- Tokens snap to grid centers when placed
- Token size respects the grid — a Large creature occupies a 2×2 area, Huge occupies 3×3
- The grid overlay in Token Setup shows exactly where tokens will land

## See Also

- [Upload a Map](./upload-map.md) — Supported formats and upload steps
- [Place Tokens](./place-tokens.md) — Adding tokens to the grid
- [Fog of War](../play-mode/fog-of-war.md) — How fog interacts with grid and walls
- [Map Formats](../../explanation/map-formats.md) — Understanding UVTT and image formats

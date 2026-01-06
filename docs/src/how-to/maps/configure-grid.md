# Configure Grid

Adjust grid settings so tokens align properly with your map.

> **Note:** UVTT files (from Dungeondraft and similar tools) include grid data automatically. You only need to configure the grid for standard image files (PNG, JPG, WebP).

## When to Configure

You may need to adjust the grid when:
- Using a standard image file (not UVTT)
- The overlay doesn't match the map's grid lines
- Tokens don't snap to the right positions

## Steps

1. Open the Token Setup modal (click a map)
2. Click **Grid** in the canvas controls
3. Adjust settings:
   - **Grid Size** - Pixels per square
   - **X Offset** - Horizontal shift in pixels
   - **Y Offset** - Vertical shift in pixels
4. Watch the grid overlay update in real-time
5. Click **Save** when aligned

## Finding the Right Grid Size

### Method 1: Count Squares
1. Measure the map width in pixels
2. Count the number of squares horizontally
3. Divide: `width ÷ squares = grid size`

### Method 2: Trial and Error
1. Start with common values: 70, 100, or 140
2. Adjust until the overlay matches
3. Fine-tune with offset values

## Grid Offset

Use offset when the grid doesn't start at the top-left corner:
- **X Offset** - Shifts grid right (positive) or left (negative)
- **Y Offset** - Shifts grid down (positive) or up (negative)

## Tips

- UVTT files include grid data - less configuration needed
- Save your settings - they persist with the map
- Zoom in for precise alignment

## See Also

- [Upload a Map](./upload-map.md)
- [Place Tokens](./place-tokens.md)

# Grid Alignment

How Mimir handles grid alignment for maps.

## Automatic Grid Detection

**UVTT files** (from Dungeondraft, Foundry, and similar tools) include grid data embedded in the file. When you upload a UVTT file, Mimir reads the grid size and applies it automatically. No manual configuration is needed.

## Image Files

Standard image files (PNG, JPG, WebP) don't include grid data. Mimir defaults to **70 pixels per grid square** for these files. The grid overlay in Token Setup uses this default for snapping tokens to positions.

For best results with image maps, use files where the grid squares are approximately 70 pixels wide, or use UVTT format which includes precise grid information.

## Tips

- UVTT is the recommended format — it includes grid data, line-of-sight walls, and lighting information
- The grid overlay in Token Setup shows how tokens will snap to positions
- Token placement always snaps to the grid center of each square

## See Also

- [Upload a Map](./upload-map.md)
- [Place Tokens](./place-tokens.md)

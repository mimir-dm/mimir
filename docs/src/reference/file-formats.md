# File Formats

Supported file formats in Mimir.

## Map Formats

### Image Files
Standard image formats for battle maps:

| Format | Extension | Notes |
|--------|-----------|-------|
| PNG | `.png` | Best for maps with transparency |
| JPEG | `.jpg`, `.jpeg` | Good for photographic maps |
| WebP | `.webp` | Efficient compression |

### UVTT Format
Universal Virtual Tabletop format:

| Extension | `.uvtt`, `.dd2vtt` |
|-----------|-------------------|
| Source | Dungeondraft, other VTT tools |
| Contents | Map image + grid data + walls |

UVTT files include:
- Embedded map image
- Grid size and offset
- Wall and door positions
- Light source data (some tools)

## Export Formats

### PDF Export
Available for:
- Character sheets
- Maps (with optional grid)
- Campaign archives
- Token cutout sheets

### Archive Export
Campaign backup format:
- ZIP file containing all campaign data
- Maps, documents, characters
- Use for backup or transfer

## Character Data

Characters are stored in the Mimir database:
- Not exported as separate files
- Use campaign archive for backup
- PDF export for printable sheets

## Campaign Directory

Each campaign creates a directory structure:
```
campaign-name/
├── maps/           # Uploaded map images
├── tokens/         # Token images
├── exports/        # Generated PDFs
└── notes/          # Session note files
```

## Import Sources

### D&D Content
Mimir can import from:
- 5e.tools JSON format
- Official SRD content

Content is stored in the catalog database.

## Recommended Workflows

### Map Creation
1. Create in Dungeondraft
2. Export as UVTT
3. Import to Mimir (grid auto-configured)

### Backup
1. Use Export Archive regularly
2. Store ZIP files externally
3. Import on new installations

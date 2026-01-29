# PDF Export

Mimir generates PDF documents from campaign data using [Typst](https://typst.app/), a modern typesetting system. The PDF export functionality lives in the mimir-print crate and supports character sheets, spell cards, and campaign documents.

## Template System

The export system uses Typst templates that define document layout and styling. Each template is a `.typ` file that receives JSON data from the application and renders it into a formatted PDF. Templates are organized by content type into character/, spells/, and campaign/ directories, with shared components in a components/ directory.

Character templates include a full sheet format for comprehensive character reference, a condensed summary for quick lookup, and a combined sheet-with-spells variant for spellcasters. Spell templates support individual cards, multi-up layouts for printing decks, and compact list formats. Campaign templates handle single documents like session notes as well as combined bundles that compile multiple documents into one PDF.

## Rendering Pipeline

When you request a PDF export, the application serializes the relevant data to JSON and passes it to the PrintService along with the template path. The service writes the JSON to a temporary file, invokes the Typst compiler with the template, and captures the resulting PDF bytes. These bytes can be returned as base64 for display or written directly to a file.

The template accesses its data through Typst's `json()` function, reading from a data.json file that the service creates in the compilation directory. This approach keeps templates portable and testable independent of the application.

## Data Format

Templates expect JSON data matching the Rust structs being exported. A character export includes character_name, level, race, and class strings alongside an abilities object containing the six scores. Hit point data arrives as separate current_hp and max_hp fields. The exact schema varies by template, but follows the patterns established in the domain model.

```json
{
  "character_name": "Thorin Ironforge",
  "level": 5,
  "race": "Dwarf",
  "class": "Cleric",
  "abilities": {
    "strength": 14,
    "dexterity": 10,
    "constitution": 16,
    "intelligence": 12,
    "wisdom": 18,
    "charisma": 8
  },
  "current_hp": 38,
  "max_hp": 45
}
```

## Tauri Commands

| Command | Parameters | Description |
|---------|------------|-------------|
| `generate_pdf` | template_path, data | Render a template to PDF and return base64 |
| `list_templates` | — | List available templates with metadata |
| `save_pdf` | path, pdf_base64 | Write base64 PDF data to a file |

## PrintService API

The Rust PrintService provides the underlying functionality that the Tauri commands wrap. Creating a service requires passing the templates root directory. The `render_to_pdf` method accepts a template path relative to that root and a serializable data reference, returning PDF bytes on success. The `save_pdf` method writes those bytes to a specified path. The `list_templates` method scans the templates directory and returns metadata for each discovered template. The `template_exists` method provides a quick check for template availability.

## Custom Templates

Adding custom templates requires creating a new `.typ` file in the appropriate templates subdirectory. The template should read its data using `json("data.json")` and use Typst markup to define the document structure. Once saved, the template appears automatically in the `list_templates` output and can be used with `generate_pdf`.

Typst documentation at [typst.app/docs](https://typst.app/docs/) covers the markup language, styling options, and available functions. Templates can import shared components and use Typst's full typesetting capabilities including custom fonts, vector graphics, and complex layouts.

## Troubleshooting

PDF generation failures usually stem from template syntax errors or missing data fields. Verify the template compiles independently using the Typst CLI before integrating with the application. Check that all fields referenced in the template exist in the provided data.

Font rendering issues occur when templates reference fonts not available on the system. Typst uses system fonts by default, so either install the required fonts or modify templates to use bundled alternatives. Font names must match exactly including capitalization.

Large PDFs typically result from embedded raster images. Optimize images before including them in exports, prefer vector graphics where possible, and consider splitting very large documents into multiple PDFs.

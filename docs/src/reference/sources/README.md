# Books and Sources

Mimir supports importing D&D 5e source books for reference and catalog population. The source system allows you to browse book content, search the extracted catalog data, and navigate cross-references between game elements.

## Book Data Files

Book data files are available from the [Mimir Resources](https://github.com/mimir-dm/resources/releases) repository. These archives contain structured JSON representations of book content including chapter text, images, and extracted game data like spells and monsters. The repository organizes books into three categories: core rulebooks (PHB, DMG, MM), supplements (XGE, TCE, FTD), and setting guides (ERLW, EGW, VRGR).

Each book archive follows a consistent structure with a metadata.json file describing the book, a book/ directory containing chapter content, and an img/ directory with cover art and interior illustrations. Books that contain game mechanics also include directories for that content type, such as spells/ for spellcasting sourcebooks or bestiary/ for monster compendiums.

## Catalog Population

When you import a book, Mimir extracts game content into the catalog database for searching. The extraction process reads content-specific JSON files and populates the appropriate catalog tables. Spells go into catalog_spells, monsters into catalog_monsters, and items into catalog_items. The system also extracts classes, races, feats, backgrounds, conditions, and actions when present.

The catalog tables store both summary information for search results and complete details for individual entries. This denormalized approach allows fast searching across thousands of entries while still providing full stat blocks and descriptions when viewing a specific item.

## Cross-Reference System

Book content contains tagged references using the `{@type name|source}` syntax. A reference like `{@spell fireball|PHB}` identifies a spell named "fireball" from the Player's Handbook. The content renderer converts these tags into clickable links that display tooltips on hover and full details in a modal on click.

The cross-reference system supports spells, creatures, items, conditions, actions, classes, races, feats, and backgrounds. When you click a reference, the system queries the catalog database for the matching entry and displays its complete information. References to content from books you haven't imported will appear as plain text rather than links.

## Storage Location

Imported books are stored in the application data directory alongside the SQLite database. On macOS this is `~/Library/Application Support/com.mimir.app/`, with equivalent locations on Windows and Linux. Each book occupies its own subdirectory under books/, preserving the original archive structure for image serving and content retrieval.

## Tauri Commands

The source system exposes library and content commands through Tauri IPC.

### Library Commands

| Command | Parameters | Description |
|---------|------------|-------------|
| `upload_book_archive` | file_path | Import a book from a .tar.gz archive |
| `list_library_books` | — | List all imported books with metadata |
| `remove_book_from_library` | book_id | Remove a book and its catalog entries |

### Content Commands

| Command | Parameters | Description |
|---------|------------|-------------|
| `get_book_content` | book_id, section | Retrieve chapter content by section path |
| `serve_book_image` | book_id, image_path | Serve an image from the book's img directory |
| `lookup_reference` | ref_type, ref_name, source | Resolve a cross-reference to catalog data |

## Troubleshooting

Import failures typically result from invalid archive format or missing required files. The archive must be a valid gzip-compressed tar file containing the expected directory structure with metadata.json at the root. Permission errors can occur if the application data directory is not writable.

Missing cross-references usually indicate the referenced content comes from a book that hasn't been imported. Some references point to SRD content that Mimir includes by default, but references to non-SRD material require importing the source book.

Images that fail to load are served on-demand from the book's img/ directory. Verify the directory exists and contains the expected files by checking the book's storage location in the application data directory.

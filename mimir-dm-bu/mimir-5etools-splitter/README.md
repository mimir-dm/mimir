# mimir-5etools-splitter

## Purpose & Boundaries

The `mimir-5etools-splitter` crate is a specialized utility for processing and extracting D&D 5e content from 5etools repositories. It provides tools for splitting large 5etools datasets into book-specific archives and extracting SRD (System Reference Document) content for integration with the Mimir application.

### Responsibilities

- **Repository Processing** - Clone and analyze 5etools repository structure
- **Book Extraction** - Split content into individual book archives
- **SRD Collection** - Extract System Reference Document content
- **Archive Creation** - Generate compressed archives with metadata
- **Data Validation** - Test and validate extracted content
- **Parallel Processing** - Coordinate processing of multiple books

### What This Crate Does NOT Do

- No D&D rules interpretation or validation
- No database operations (output is files)
- No UI or user interaction
- No LLM operations or content generation
- No campaign management functionality

## Architecture

### Binaries

The crate provides two executable binaries:

1. **mimir-5esplit** - Main splitting and extraction tool
2. **mimir-5etest** - Load testing and validation tool

### Core Modules

```
src/
├── lib.rs                 # Public API and main functions
├── main.rs               # mimir-5esplit CLI binary
├── bin/
│   └── load_test.rs      # mimir-5etest binary
├── collector.rs          # Content collection and organization
├── srd_collector.rs      # SRD-specific content extraction
├── filter.rs             # Content filtering and selection
├── srd_filter.rs         # SRD content filtering
├── parser.rs             # JSON parsing and book detection
├── archive.rs            # Archive creation and compression
├── input.rs              # Input source handling (local/remote)
├── repo.rs               # Repository setup and management
├── parallel.rs           # Parallel processing coordination
├── images.rs             # Image asset handling
├── magic_variants.rs     # Magic item variant processing
└── load_tester.rs        # Database load testing utilities
```

## Key Features

### Book Splitting
- **Automatic Detection** - Discovers books from repository structure
- **Content Extraction** - Extracts JSON data, images, and assets
- **Parallel Processing** - Processes multiple books simultaneously using rayon
- **Archive Generation** - Creates compressed tar.gz archives per book

### SRD Extraction
- **Content Collection** - Extracts SRD-licensed content from multiple sources
- **Content Categories** - Classes, spells, items, monsters, and other game content
- **Metadata Generation** - Creates content summaries and statistics
- **Archive Output** - Single compressed archive with all SRD content

### Data Validation
- **Load Testing** - Validates content can be imported into Mimir database
- **Database Integration** - Uses mimir-dm-core services for testing
- **Error Reporting** - Reports validation failures and issues
- **Statistics** - Provides counts and summaries of processed content

## Usage

### CLI: mimir-5esplit

```bash
# Split from local repository
cargo run --bin mimir-5esplit -- split /path/to/5etools-repo /output/directory

# Extract SRD content only
cargo run --bin mimir-5esplit -- srd /path/to/5etools-repo /output/directory
```

### CLI: mimir-5etest

```bash
# Test loading extracted content
cargo run --bin mimir-5etest -- /path/to/archives/

# Test specific book
cargo run --bin mimir-5etest -- /path/to/archives/phb.tar.gz
```

### Library API

```rust
use mimir_5etools_splitter::{split_repository, extract_srd, InputSource};

// Split repository into book archives
let results = split_repository(
    InputSource::Local("/path/to/repo".into()),
    Path::new("/output/dir")
).await?;

// Extract SRD content
let srd_results = extract_srd(
    InputSource::Remote("https://github.com/5etools-mirror-1/5etools-img".to_string()),
    Path::new("/output/dir")
).await?;
```

## Input Sources

### Local Repository
Process an existing local 5etools repository:

```rust
let input = InputSource::Local(PathBuf::from("/path/to/5etools"));
```

### Remote Repository
Clone and process a remote repository:

```rust
let input = InputSource::Remote("https://github.com/5etools-mirror-1/5etools-img".to_string());
```

## Output Format

### Book Archives
Each book is packaged as a compressed tar.gz archive containing:

```
book_name.tar.gz
├── data/              # JSON data files
├── img/              # Book-specific images
└── metadata.json     # Book metadata and statistics
```

### SRD Archive
SRD content is packaged as:

```
srd.tar.gz
├── classes.json      # All SRD classes
├── spells.json       # All SRD spells
├── items.json        # All SRD items
├── monsters.json     # All SRD monsters
└── summary.json      # Content summary and statistics
```

## Dependencies

### Core Dependencies
- `serde` & `serde_json` - JSON parsing and serialization
- `tokio` - Async runtime for I/O operations
- `anyhow` - Error handling
- `walkdir` - Directory traversal
- `regex` - Pattern matching

### Archive Dependencies
- `tar` - Archive creation
- `flate2` - Compression (gzip)
- `tempfile` - Temporary file handling

### Processing Dependencies
- `rayon` - Parallel processing
- `indicatif` - Progress bars and indicators

### Testing Dependencies
- `mimir-dm-core` - Database integration for load testing
- `diesel` - Database operations
- `colored` - Terminal output formatting

## Integration with Mimir

### Data Flow
1. **Extract** - Process 5etools repository with mimir-5esplit
2. **Validate** - Test extraction with mimir-5etest
3. **Import** - Load archives into Mimir via book management commands
4. **Use** - Access content through catalog services in mimir-dm-core

### Book Management
The main Mimir application includes book management commands that:
- List available extracted books
- Import book archives into the database
- Track installation status and metadata

## Testing

Run the test suite:

```bash
cargo test -p mimir-5etools-splitter
```

Integration tests require sample 5etools data and temporary directories for output testing.

## Design Principles

1. **Separation of Concerns** - Clear boundaries between parsing, filtering, and archiving
2. **Error Resilience** - Continue processing even if individual books fail
3. **Resource Management** - Use streaming I/O and parallel processing appropriately
4. **Extensibility** - Easy addition of new content types and formats
5. **Testability** - Comprehensive testing with sample data
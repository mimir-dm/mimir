# mimir-dm

## Purpose & Boundaries

The `mimir-dm` crate is the desktop application layer of Mimir D&D Campaign Assistant. It provides a Tauri-based desktop application with a Vue.js frontend, bridging the gap between the core business logic and the user interface. This crate orchestrates all the other components to deliver a complete D&D campaign management experience.

### Responsibilities

- **Desktop Application Shell**: Tauri framework integration for native desktop experience
- **Command Bridge**: IPC commands connecting frontend to backend services
- **Application Lifecycle**: Initialization, database setup, and resource management
- **Frontend Hosting**: Vue.js single-page application with routing and state management
- **File System Integration**: Campaign directory management and document persistence
- **Database Management**: Connection pooling and migration orchestration
- **Template Seeding**: Initial data population for new installations

### What This Crate Does NOT Do

- No business logic implementation (delegates to mimir-dm-core)
- No direct database operations (uses services from core)
- No LLM operations (future integration point)
- No web server hosting (desktop-only)
- No cloud synchronization

## Architecture

### Backend Structure (`src/`)

```
src/
├── main.rs                      # Application entry point and Tauri setup
├── app_init.rs                 # Application initialization and paths management
├── db_connection.rs            # Database connection pool management
├── seed_templates.rs           # Template seeding for new databases
├── embedded_test_book.rs       # Test book data for development
├── types.rs                    # Shared types (ApiResponse, ApiError)
├── commands/                   # Tauri command handlers (51+ files, 250+ commands)
│   ├── mod.rs                 # Command module exports
│   ├── app_info.rs           # Application information commands
│   ├── campaigns.rs          # Campaign CRUD operations
│   ├── boards.rs             # Board configuration and workflow
│   ├── books.rs              # 5etools book management
│   ├── chat_sessions.rs      # LLM chat session management
│   ├── documents.rs          # Document management
│   ├── stage_transitions.rs  # Campaign stage transitions
│   ├── catalog_*_db.rs       # Database-backed catalog commands (14+ files)
│   │   ├── catalog_background_db.rs # Character backgrounds
│   │   ├── catalog_class_db.rs     # Classes and subclasses
│   │   ├── catalog_cult_db.rs      # Cults and organizations
│   │   ├── catalog_deity_db.rs     # Deities and pantheons
│   │   ├── catalog_feat_db.rs      # Feats and abilities
│   │   ├── catalog_item_db.rs      # Items and equipment
│   │   ├── catalog_language_db.rs  # Languages and scripts
│   │   ├── catalog_monster_db.rs   # Monsters and NPCs
│   │   ├── catalog_object_db.rs    # Objects and structures
│   │   ├── catalog_optional_feature_db.rs # Optional features
│   │   ├── catalog_psionic_db.rs   # Psionic abilities
│   │   ├── catalog_race_db.rs      # Races and lineages
│   │   ├── catalog_reward_db.rs    # Rewards and treasures
│   │   ├── catalog_trap_db.rs      # Traps and hazards
│   │   ├── catalog_variant_rule_db.rs # Variant rules
│   │   └── catalog_vehicle_db.rs   # Vehicles and mounts
│   ├── catalog_*.rs          # Legacy file-based implementations
│   │   ├── catalog_action.rs       # Actions and activities
│   │   ├── catalog_condition.rs    # Conditions and effects
│   │   ├── catalog_deity.rs        # Deities (legacy)
│   │   ├── catalog_spell.rs        # Spells and magic
│   │   └── catalog_table.rs        # Random tables
│   ├── chat_sessions.rs      # LLM chat session management
│   ├── context.rs            # Application context commands
│   ├── dev_tools.rs          # Development and debugging tools
│   ├── modules.rs            # Module management
│   ├── todos.rs              # Todo and task management
│   └── window_manager.rs     # Window state management
└── services/                 # Service layer
    ├── mod.rs               # Service module exports
    ├── database.rs          # Database service wrapper
    ├── context_service.rs   # Context and state management
    ├── llm_service.rs       # LLM integration service
    └── tools/               # LLM tool implementations
        ├── mod.rs
        ├── catalog_tool.rs  # Catalog search tools
        ├── document_tool.rs # Document management tools
        ├── file_tool.rs     # File system tools
        ├── script_tool.rs   # Script execution tools
        └── todo_tool.rs     # Todo management tools
```

### Frontend Structure (`frontend/src/`)

```
frontend/src/
├── app/                  # Application entry points
│   ├── App.vue          # Root Vue component
│   ├── main.ts          # Vue app initialization
│   ├── router.ts        # Vue Router configuration
│   └── stores.ts        # Pinia store setup
├── assets/               # Static assets
│   ├── css/             # Global CSS and themes
│   └── images/          # UI images and icons
├── components/           # Reusable UI components
│   ├── base/            # Base components (buttons, inputs, etc.)
│   ├── boards/          # Campaign board components
│   ├── campaigns/       # Campaign-related components
│   ├── common/          # Common UI components
│   ├── documents/       # Document management components
│   ├── layout/          # Layout and navigation components
│   ├── modals/          # Modal dialogs
│   └── tables/          # Data table components
├── constants/            # Application constants
│   └── routes.ts        # Route constants
├── features/             # Feature-specific modules
│   ├── boards/          # Board workflow features
│   ├── campaigns/       # Campaign management features
│   ├── chat/            # LLM chat features
│   ├── documents/       # Document editing features
│   ├── modules/         # Module management features
│   ├── search/          # Search and catalog features
│   ├── sessions/        # Session management features
│   └── templates/       # Template management features
├── services/             # Frontend services
│   ├── api.ts           # Tauri API wrapper
│   ├── campaigns.ts     # Campaign API service
│   ├── catalog.ts       # Catalog API service
│   ├── documents.ts     # Document API service
│   └── search.ts        # Search API service
├── shared/               # Shared utilities
│   ├── composables/     # Vue composables
│   ├── directives/      # Vue directives
│   ├── guards/          # Route guards
│   └── utils/           # Utility functions
├── stores/               # Pinia state management
│   ├── campaigns.ts     # Campaign state and actions
│   ├── documents.ts     # Document state
│   ├── search.ts        # Search state
│   └── ui.ts            # UI state (theme, modals, etc.)
├── types/                # TypeScript type definitions
│   ├── api.ts           # API response types
│   ├── campaigns.ts     # Campaign domain types
│   ├── catalog.ts       # Catalog item types
│   ├── documents.ts     # Document types
│   ├── search.ts        # Search types
│   └── ui.ts            # UI component types
├── views/                # Page-level components
│   ├── CampaignView.vue # Main campaign workspace
│   ├── HomeView.vue     # Landing page
│   └── SettingsView.vue # Application settings
└── test/                 # Test utilities
    └── setup.ts         # Test environment setup
```

## Key Features

### Application Initialization
1. **Directory Setup**: Creates application directories in platform-specific locations
2. **Database Initialization**: SQLite setup with migration support
3. **Development Mode**: In-memory database option for development
4. **Template Seeding**: Populates initial templates on first run

### Command System (IPC)
The Tauri command system provides type-safe IPC between frontend and backend:

#### Campaign Commands
- `list_campaigns` - Retrieve all campaigns
- `create_campaign` - Create new campaign with directory structure
- `get_campaign` - Get campaign details
- `generate_campaign_document` - Generate document from template

#### Document Commands
- `get_campaign_documents` - List campaign documents
- `get_documents_by_level` - Filter documents by level
- `create_document` - Create new document
- `update_document` - Update existing document
- `complete_document` - Mark document as complete
- `delete_document` - Remove document
- `read_document_file` - Read document from filesystem
- `save_document_file` - Persist document to filesystem

#### Board Commands
- `get_board_configuration` - Get board workflow definition
- `check_campaign_stage_completion` - Check stage requirements
- `transition_campaign_stage` - Move to next stage
- `initialize_stage_documents` - Create required documents for stage
- `get_next_stage` - Determine next valid stage

#### Catalog Commands (16+ categories)
- `search_*` - Search within specific catalog types (classes, spells, items, etc.)
- `get_*_details` - Get detailed information for catalog items
- `get_*_sources` - List available source books for categories
- `get_*_count` - Get total counts for categories
- `get_*_types` - Get classification types (spell schools, item types, etc.)
- `get_*_statistics` - Get statistical summaries for categories

#### LLM & Chat Commands
- `create_chat_session` - Create new LLM chat session
- `send_chat_message` - Send message to LLM with context
- `get_chat_history` - Retrieve conversation history
- `delete_chat_session` - Remove chat session

#### Book Management Commands
- `get_available_books` - List 5etools books available
- `install_book` - Import book data into database
- `get_book_status` - Check installation status

### Frontend Architecture

#### Technology Stack
- **Vue 3**: Composition API with TypeScript
- **Pinia**: State management
- **Vue Router**: Client-side routing
- **TipTap**: Rich text editor with Markdown support
- **Tailwind CSS**: Utility-first styling
- **Vitest**: Unit testing framework

#### State Management
Pinia stores manage application state:
- **Campaign Store**: Campaign CRUD and selection
- **Theme Store**: User theme preferences (dark/light/hyper)

#### Component Organization
- **Views**: Page-level components mapped to routes
- **Components**: Reusable UI components
- **Layouts**: Application structure components
- **Common**: Shared utilities and helpers

## Configuration

### Tauri Configuration (`tauri.conf.json`)
```json
{
  "productName": "Mimir",
  "identifier": "com.mimir.app",
  "build": {
    "frontendDist": "./frontend/dist"
  },
  "app": {
    "windows": [{
      "title": "Mimir - D&D Campaign Assistant",
      "width": 1400,
      "height": 900,
      "minWidth": 1200,
      "minHeight": 700
    }]
  }
}
```

### Development Environment
- **Development Mode**: Automatically uses `mimir-test` directory for persistent test data
- **Production Mode**: Uses `mimir` directory for production data
- **Force Dev Mode**: Set `MIMIR_DEV=1` to force development mode

## Dependencies

### Backend
- `tauri` - Desktop application framework
- `tauri-plugin-shell` - Shell command execution
- `tauri-plugin-dialog` - Native dialogs
- `mimir-dm-core` - Core business logic
- `tokio` - Async runtime
- `tracing` - Structured logging
- `serde` - Serialization
- `directories` - Platform-specific paths

### Frontend
- `@tauri-apps/api` - Tauri frontend API
- `vue` - UI framework
- `vue-router` - Routing
- `pinia` - State management
- `@tiptap` - Rich text editor
- `tailwindcss` - CSS framework

## Building & Running

### Development
```bash
# Install frontend dependencies
cd frontend && npm install

# Run in development mode
cargo tauri dev
```

### Production Build
```bash
# Build frontend
cd frontend && npm run build

# Build Tauri app
cargo tauri build
```

### Testing
```bash
# Frontend tests
cd frontend && npm test

# Backend tests
cargo test -p mimir-dm

# E2E tests (requires built app)
cargo test --features e2e
```

## Development Patterns & Conventions

### Command Implementation Pattern
All Tauri commands follow a consistent pattern:

```rust
#[tauri::command]
pub async fn command_name(
    db_service: State<'_, Arc<DatabaseService>>,
    // other parameters
) -> Result<ResponseType, String> {
    let mut conn = db_service.get_connection()
        .map_err(|e| format!("Database connection failed: {}", e))?;
    
    let service = SomeService::new(&mut conn);
    service.perform_operation(parameters)
}
```

### Database Service Pattern
- All database access goes through `DatabaseService`
- Use connection pooling via `get_connection()`
- Business logic resides in `mimir-dm-core` services
- Commands act as thin adapters between Tauri and core services

### Error Handling Pattern
- Commands return `Result<T, String>` for Tauri compatibility
- Use `ApiResponse<T>` for structured responses when needed
- Convert core errors to user-friendly messages
- Log errors with `tracing` crate for debugging

### Migration Strategy (Legacy → Database)
The codebase is migrating from file-based catalogs to database-backed services:
- **New pattern**: `catalog_*_db.rs` files with database services
- **Legacy pattern**: `catalog_*.rs` files with hardcoded data
- **Transition**: Some commands support both patterns during migration
- **Future**: All catalog commands will use database services

### Frontend Architecture Patterns

#### Service Layer Pattern
- Frontend services in `src/services/` wrap Tauri API calls
- Each domain has its own service (campaigns, documents, catalog)
- Services handle data transformation and error handling
- Use TypeScript for full type safety

#### Feature-Based Organization
- Group related components in `src/features/[domain]/`
- Each feature contains components, composables, and types
- Shared utilities in `src/shared/`
- Domain-specific types in `src/types/[domain].ts`

#### State Management Pattern
- Use Pinia stores for global state
- Keep stores focused on specific domains
- Use composables for component-specific reactive state
- Avoid prop drilling with provide/inject when needed

### Tool Integration Pattern
LLM tools follow a structured pattern for extensibility:

```rust
impl Tool for SomeTool {
    fn name(&self) -> &str { "tool_name" }
    fn description(&self) -> &str { "Clear description" }
    
    async fn execute(&self, args: &Value) -> Result<String, String> {
        // Implementation with error handling
    }
}
```

### Naming Conventions
- **Commands**: `verb_noun` format (e.g., `search_classes`, `get_campaign`)
- **Services**: `NounService` format (e.g., `CampaignService`, `ClassService`)
- **Components**: PascalCase with descriptive names (e.g., `DocumentEditor`, `CampaignBoardView`)
- **Types**: Match domain language (e.g., `Campaign`, `ClassSummary`, `SpellFilters`)

### File Organization Rules
1. **Backend**: Group by functional area (commands/, services/, types.rs)
2. **Frontend**: Group by feature, then by type (components, composables, types)
3. **Tests**: Co-locate with code being tested
4. **Documentation**: Rust doc comments for public APIs

## Design Principles

1. **Separation of Concerns**: Clear boundary between UI and business logic
2. **Type Safety**: TypeScript frontend with Rust backend
3. **Command Pattern**: All IPC through defined commands
4. **State Management**: Centralized state in Pinia stores
5. **Component Reusability**: Modular Vue components
6. **Tool-Based Architecture**: Extensible LLM tool system
7. **Database-First**: Migrate from file-based to database-backed services
8. **Offline First**: Full functionality without internet
9. **Progressive Enhancement**: Features added incrementally

## API Response Pattern

All commands return a standardized response:
```rust
ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>
}
```

This ensures consistent error handling across the application.

## Current Features

- **LLM Integration** - Local LLM support via Ollama with tool-calling capabilities
- **Comprehensive D&D 5e Catalog** - 16+ categories with advanced search and filtering
- **Campaign Workflow Management** - Structured phases and document templates  
- **Rich Text Editing** - TipTap editor with Markdown support and live preview
- **Theme Support** - Dark, light, and hyper theme variants with CSS custom properties
- **Book Management** - Import and manage 5etools data with full SRD support
- **Local-First Design** - SQLite database with connection pooling
- **Type-Safe IPC** - 250+ Tauri commands with standardized error handling
- **Tool-Based Architecture** - LLM tools for catalog search, document management, and file operations
- **Modular Frontend** - Feature-based organization with composables and services
- **Development Mode** - Separate test database and debugging tools
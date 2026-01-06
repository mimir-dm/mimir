# Frontend Architecture

Mimir's frontend is a Vue 3 application built with TypeScript, using Tauri 2.0 for desktop integration. The application follows a feature-based organization pattern where related components, views, and logic are grouped by domain rather than by technical layer.

## Technology Stack

The core framework is Vue 3 using the Composition API exclusively. State management uses Pinia with the setup store pattern, which aligns naturally with the Composition API style. Routing is handled by Vue Router with lazy-loaded route components for code splitting. TypeScript provides static typing throughout the codebase with strict mode enabled.

Desktop integration comes from Tauri 2.0, which provides IPC communication with the Rust backend through the `invoke` function. The frontend calls Tauri commands and receives typed responses, with all data serialization handled automatically.

Rich text editing uses Tiptap 3, a headless editor built on ProseMirror. The configuration includes the starter kit for basic formatting, table extensions for structured data, and markdown serialization for storage. Styling uses Tailwind CSS for utility-first class composition. The build toolchain is Vite, providing fast development server startup and optimized production builds.

## Directory Structure

The src/ directory contains an app/ folder for bootstrap code including the Vue app initialization in main.ts, the root App.vue component, and the router configuration. The features/ folder contains nine feature modules: campaigns, modules, sessions, chat, characters, players, sources, templates, and context. Each feature is self-contained with its own views, components, and composables.

Shared infrastructure lives in the shared/ directory with reusable components organized into layout/, ui/, and catalog/ subdirectories. Shared composables provide common logic patterns, shared types define interfaces used across features, and shared utils contain helper functions.

The stores/ directory contains Pinia stores for global state, with the chat store being particularly complex due to its composition of multiple sub-stores. The services/ directory holds business logic services that coordinate between stores and the Tauri backend. Root-level types/ contains the main type definitions exported for use throughout the application.

## Feature Organization

Each feature follows a consistent internal structure. The views/ subdirectory contains page-level components that correspond to routes. The components/ subdirectory holds feature-specific UI elements that are composed within views. The composables/ subdirectory, when present, contains feature-specific logic hooks.

This organization means that when working on campaign functionality, all relevant code lives within features/campaigns/. The pattern keeps features isolated and makes it clear where new code should be added.

## State Management

Pinia stores use the setup store pattern, which defines stores as functions that return reactive state and methods. This approach mirrors the Composition API and allows stores to use composables directly.

```typescript
export const useCampaignsStore = defineStore('campaigns', () => {
  const campaigns = ref<Campaign[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadCampaigns() {
    loading.value = true
    const response = await invoke<ApiResponse<Campaign[]>>('list_campaigns')
    if (response.success) {
      campaigns.value = response.data
    }
    loading.value = false
  }

  return { campaigns, loading, error, loadCampaigns }
})
```

The chat store demonstrates advanced composition by creating multiple sub-stores for messages, sessions, tokens, todos, and tool confirmations. The parent store coordinates initialization and provides a unified API while keeping internal concerns separated.

## Tauri Integration

Communication with the Rust backend uses Tauri's invoke function with typed responses. All commands return an ApiResponse wrapper containing a success boolean, optional data, and optional error message. This consistent pattern simplifies error handling across the application.

```typescript
interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

const response = await invoke<ApiResponse<Campaign[]>>('list_campaigns')
```

## Layout System

Three layout components handle page structure. MainLayout provides a standard full-page wrapper with header and content areas. TwoPanelLayout adds a collapsible sidebar alongside the main content. ThreePanelLayout extends this with an additional detail panel on the right, useful for master-detail views.

## Sources Feature

The sources feature for D&D catalog browsing is the most architecturally complex. It uses a generic `useCatalogSearch` composable that provides initialization, searching, and detail fetching logic. This base pattern is instantiated 22 times for different content types including spells, monsters, items, classes, races, feats, and backgrounds.

Each catalog type has a dedicated formatter module that transforms raw catalog data into display-ready HTML. These formatters handle the specialized rendering needs of different content types, such as stat block layouts for monsters or component lists for spells.

## Theming

The application supports three themes: light, dark, and hyper. Theme state is managed by a dedicated Pinia store that persists the selection to localStorage and synchronizes across windows through Tauri events. CSS variables define theme colors, allowing components to reference semantic color names that resolve differently based on the active theme.

## Error Handling

Stores and composables follow a consistent error handling pattern. Async operations set a loading flag before starting, execute within a try-catch block, check the response success field, and update an error ref if something fails. The finally block ensures loading is reset regardless of outcome. This pattern provides predictable loading states and error messages throughout the application.

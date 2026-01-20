//! Generic catalog command macros.
//!
//! This module provides macros to reduce boilerplate in catalog command handlers.
//! All catalog services follow the same pattern:
//! - `search_*` - Search with filters, return Vec<Summary>
//! - `get_*_details` - Get by name and source, return Option<Full>
//! - `get_*_sources` - Get available sources, return Vec<String>
//!
//! Using these macros ensures consistent error handling, logging, and API patterns
//! across all 20+ catalog services.

/// Generate standard catalog command handlers for a service that implements CatalogService.
///
/// This macro generates three commands:
/// - `search_{entity}s` - Search with filters
/// - `get_{entity}_details` - Get by name and source
/// - `get_{entity}_sources` - Get available sources
///
/// # Arguments
/// - `$entity` - The entity name (e.g., `background`, `feat`)
/// - `$service` - The service type (e.g., `BackgroundService`)
/// - `$filters` - The filter type (e.g., `BackgroundFilters`)
/// - `$summary` - The summary type (e.g., `BackgroundSummary`)
/// - `$full` - The full entity type (e.g., `Background`)
///
/// # Example
///
/// ```ignore
/// catalog_commands!(
///     entity: background,
///     service: BackgroundService,
///     filters: BackgroundFilters,
///     summary: BackgroundSummary,
///     full: Background
/// );
/// ```
#[macro_export]
macro_rules! catalog_commands {
    (
        entity: $entity:ident,
        service: $service:ty,
        filters: $filters:ty,
        summary: $summary:ty,
        full: $full:ty
    ) => {
        paste::paste! {
            #[doc = "Search the " $entity " catalog with filters."]
            ///
            /// Returns a list of summaries matching the provided filter criteria.
            ///
            /// # Parameters
            /// - `filters` - Filter criteria for the search
            ///
            /// # Returns
            /// List of summary objects with basic information.
            ///
            /// # Errors
            /// Returns an error string if the database connection or query fails.
            #[tauri::command]
            pub async fn [<search_ $entity s>](
                filters: $filters,
                state: tauri::State<'_, $crate::state::AppState>,
            ) -> Result<Vec<$summary>, String> {
                use mimir_dm_core::services::CatalogService;
                tracing::debug!(concat!("Searching ", stringify!($entity), "s with filters: {:?}"), filters);

                let mut conn = state.db.get_connection().map_err(|e| {
                    tracing::error!(concat!("Database connection error during ", stringify!($entity), " search: {}"), e);
                    format!("Database connection failed: {}", e)
                })?;

                let mut service = <$service>::new(&mut conn);
                service.search(filters).map_err(|e| {
                    tracing::error!(concat!("Database ", stringify!($entity), " search failed: {}"), e);
                    format!(concat!("Failed to search ", stringify!($entity), "s: {}"), e)
                })
            }

            #[doc = "Get complete " $entity " details by name and source."]
            ///
            /// Retrieves the full record with all properties.
            ///
            /// # Parameters
            /// - `name` - Exact name (case-sensitive)
            /// - `source` - Source book abbreviation
            ///
            /// # Returns
            /// The complete object if found, or `None` if no match exists.
            ///
            /// # Errors
            /// Returns an error string if the database connection or query fails.
            #[tauri::command]
            pub async fn [<get_ $entity _details>](
                name: String,
                source: String,
                state: tauri::State<'_, $crate::state::AppState>,
            ) -> Result<Option<$full>, String> {
                use mimir_dm_core::services::CatalogService;
                tracing::debug!(concat!("Getting ", stringify!($entity), " details: {} from {}"), name, source);

                let mut conn = state.db.get_connection().map_err(|e| {
                    tracing::error!(concat!("Database connection error during ", stringify!($entity), " details fetch: {}"), e);
                    format!("Database connection failed: {}", e)
                })?;

                let mut service = <$service>::new(&mut conn);
                service.get_by_name_and_source(&name, &source).map_err(|e| {
                    tracing::error!(concat!("Failed to get ", stringify!($entity), " details: {}"), e);
                    format!(concat!("Failed to get ", stringify!($entity), " details: {}"), e)
                })
            }

            #[doc = "Get all unique source books containing " $entity "s."]
            ///
            /// Returns source book abbreviations that contain at least one entry.
            /// Used to populate filter dropdowns in the UI.
            ///
            /// # Returns
            /// List of source abbreviations.
            ///
            /// # Errors
            /// Returns an error string if the database connection or query fails.
            #[tauri::command]
            pub async fn [<get_ $entity _sources>](
                state: tauri::State<'_, $crate::state::AppState>,
            ) -> Result<Vec<String>, String> {
                use mimir_dm_core::services::CatalogService;
                tracing::debug!(concat!("Getting ", stringify!($entity), " sources"));

                let mut conn = state.db.get_connection().map_err(|e| {
                    tracing::error!(concat!("Database connection error during ", stringify!($entity), " sources fetch: {}"), e);
                    format!("Database connection failed: {}", e)
                })?;

                let mut service = <$service>::new(&mut conn);
                service.get_sources().map_err(|e| {
                    tracing::error!(concat!("Failed to get ", stringify!($entity), " sources: {}"), e);
                    format!(concat!("Failed to get ", stringify!($entity), " sources: {}"), e)
                })
            }
        }
    };
}

//! Database service layer for action operations
//!
//! This service provides database-backed action search and retrieval,
//! following the stateful pattern with CatalogService trait implementation.

use crate::error::Result;
use crate::models::catalog::{
    Action, ActionFilters, ActionSummary, CatalogAction, NewCatalogAction,
};
use crate::schema::catalog_actions;
use crate::services::CatalogService;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Service for searching and managing actions in the catalog.
///
/// This service follows the stateful pattern with a borrowed database connection.
/// It implements `CatalogService` for generic catalog operations.
///
/// # Example
///
/// ```ignore
/// use mimir_dm_core::services::ActionService;
/// use mimir_dm_core::models::catalog::ActionFilters;
///
/// let mut service = ActionService::new(&mut conn);
/// let filters = ActionFilters::default();
/// let actions = service.search(filters)?;
/// ```
pub struct ActionService<'a> {
    /// Database connection
    pub conn: &'a mut SqliteConnection,
}

impl<'a> ActionService<'a> {
    /// Create a new ActionService with a database connection.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Search actions with optional filters
    pub fn search_actions(&mut self, filters: ActionFilters) -> Result<Vec<ActionSummary>> {
        debug!("Searching actions with filters: {:?}", filters);

        let mut query = catalog_actions::table.into_boxed();

        // Apply name search filter
        if let Some(name_query) = &filters.name {
            if !name_query.trim().is_empty() {
                query =
                    query.filter(catalog_actions::name.like(format!("%{}%", name_query.trim())));
            }
        }

        // Apply general search filter (searches name and description)
        if let Some(search_query) = &filters.search {
            if !search_query.trim().is_empty() {
                let search_term = format!("%{}%", search_query.trim());
                query = query.filter(
                    catalog_actions::name
                        .like(search_term.clone())
                        .or(catalog_actions::description.like(search_term)),
                );
            }
        }

        // Apply time type filters
        if let Some(time_types) = &filters.time_types {
            if !time_types.is_empty() {
                query = query.filter(catalog_actions::time_type.eq_any(time_types));
            }
        }

        // Apply source filters
        if let Some(sources) = &filters.sources {
            if !sources.is_empty() {
                query = query.filter(catalog_actions::source.eq_any(sources));
            }
        }

        // Execute query and order by name
        let actions: Vec<CatalogAction> = query.order(catalog_actions::name.asc()).load(self.conn)?;

        info!("Found {} actions", actions.len());

        // Convert to summaries
        let summaries: Vec<ActionSummary> = actions.into_iter().map(ActionSummary::from).collect();

        Ok(summaries)
    }

    /// Get a specific action by ID
    pub fn get_action_by_id(&mut self, action_id: i32) -> Result<Option<Action>> {
        debug!("Getting action by ID: {}", action_id);

        let catalog_action: Option<CatalogAction> = catalog_actions::table
            .find(action_id)
            .first(self.conn)
            .optional()?;

        match catalog_action {
            Some(action) => {
                // Parse the full JSON back to Action struct
                let full_action: Action = serde_json::from_str(&action.full_action_json)?;
                Ok(Some(full_action))
            }
            None => Ok(None),
        }
    }

    /// Get a specific action by name and source
    /// Uses case-insensitive name matching since tags use lowercase
    pub fn get_action_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> Result<Option<Action>> {
        debug!("Getting action by name: {} and source: {}", name, source);

        // Use case-insensitive matching for name (tags use lowercase, DB has title case)
        let name_lower = name.to_lowercase();
        let catalog_action: Option<CatalogAction> = catalog_actions::table
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                    "LOWER(name) = '{}' AND source = '{}'",
                    name_lower.replace('\'', "''"),
                    source.replace('\'', "''")
                )),
            )
            .first(self.conn)
            .optional()?;

        match catalog_action {
            Some(action) => {
                let full_action: Action = serde_json::from_str(&action.full_action_json)?;
                Ok(Some(full_action))
            }
            None => Ok(None),
        }
    }

    /// Get all unique time types for filter options
    pub fn get_time_types(&mut self) -> Result<Vec<String>> {
        debug!("Getting unique time types");

        let time_types: Vec<String> = catalog_actions::table
            .select(catalog_actions::time_type)
            .distinct()
            .order(catalog_actions::time_type.asc())
            .load(self.conn)?;

        Ok(time_types)
    }

    /// Get all unique sources for filter options
    pub fn get_action_sources(&mut self) -> Result<Vec<String>> {
        debug!("Getting unique sources");

        let sources: Vec<String> = catalog_actions::table
            .select(catalog_actions::source)
            .distinct()
            .order(catalog_actions::source.asc())
            .load(self.conn)?;

        Ok(sources)
    }

    /// Get action count for statistics
    pub fn get_action_count(&mut self) -> Result<i64> {
        use diesel::dsl::count_star;

        let count: i64 = catalog_actions::table.select(count_star()).first(self.conn)?;

        Ok(count)
    }

    /// Import all action data from an uploaded book directory
    pub fn import_actions_from_book(
        conn: &mut SqliteConnection,
        book_dir: &Path,
        source: &str,
    ) -> Result<usize> {
        info!(
            "Importing actions from book directory: {:?} (source: {})",
            book_dir, source
        );

        let mut total_imported = 0;
        let action_files = Self::find_action_files(book_dir)?;

        if action_files.is_empty() {
            info!("No action files found in book directory");
            return Ok(0);
        }

        info!("Found {} action files to process", action_files.len());

        for action_file in action_files {
            debug!("Processing action file: {:?}", action_file);

            match Self::import_actions_from_file(conn, &action_file, source) {
                Ok(count) => {
                    info!("Imported {} actions from {:?}", count, action_file);
                    total_imported += count;
                }
                Err(e) => {
                    error!("Failed to import actions from {:?}: {}", action_file, e);
                    // Continue processing other files instead of failing completely
                }
            }
        }

        info!(
            "Successfully imported {} total actions from {}",
            total_imported, source
        );
        Ok(total_imported)
    }

    /// Find action JSON files in a book directory
    fn find_action_files(book_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut action_files = Vec::new();

        let actions_dir = book_dir.join("actions");
        if !actions_dir.exists() || !actions_dir.is_dir() {
            debug!("No actions directory found in book: {:?}", book_dir);
            return Ok(action_files);
        }

        debug!("Found actions directory: {:?}", actions_dir);

        // Read all JSON files in the actions directory
        let entries = fs::read_dir(&actions_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip fluff files and non-JSON files
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("fluff") || !filename.ends_with(".json") {
                    continue;
                }
            }

            if path.is_file() {
                debug!("Found action file: {:?}", path);
                action_files.push(path);
            }
        }

        debug!(
            "Found {} action files in book directory",
            action_files.len()
        );
        Ok(action_files)
    }

    /// Import actions from a specific JSON file
    fn import_actions_from_file(
        conn: &mut SqliteConnection,
        file_path: &Path,
        source: &str,
    ) -> Result<usize> {
        debug!("Reading action file: {:?}", file_path);

        let content = fs::read_to_string(file_path)?;

        let json: serde_json::Value = serde_json::from_str(&content)?;

        let mut actions_to_import: Vec<Action> = Vec::new();

        // Handle different JSON structures
        if let Some(action_array) = json.get("action").and_then(|v| v.as_array()) {
            // Structure: { "action": [...] }
            for action_value in action_array {
                match serde_json::from_value::<Action>(action_value.clone()) {
                    Ok(mut action) => {
                        action.source = source.to_string();
                        actions_to_import.push(action);
                    }
                    Err(e) => {
                        warn!("Failed to parse action: {}", e);
                        continue;
                    }
                }
            }
        } else if json.is_array() {
            // Structure: [...]
            if let Some(action_array) = json.as_array() {
                for action_value in action_array {
                    match serde_json::from_value::<Action>(action_value.clone()) {
                        Ok(mut action) => {
                            action.source = source.to_string();
                            actions_to_import.push(action);
                        }
                        Err(e) => {
                            warn!("Failed to parse action: {}", e);
                            continue;
                        }
                    }
                }
            }
        } else if let Some(action_obj) = json.as_object() {
            // Look for nested action arrays in other keys
            for (key, value) in action_obj {
                if key.contains("action") || key == "data" {
                    if let Some(action_array) = value.as_array() {
                        for action_value in action_array {
                            match serde_json::from_value::<Action>(action_value.clone()) {
                                Ok(mut action) => {
                                    action.source = source.to_string();
                                    actions_to_import.push(action);
                                }
                                Err(e) => {
                                    warn!("Failed to parse nested action: {}", e);
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }

        if actions_to_import.is_empty() {
            debug!("No actions found in file: {:?}", file_path);
            return Ok(0);
        }

        debug!(
            "Processing {} actions for database import",
            actions_to_import.len()
        );

        // Transform actions to database format
        let catalog_actions: Vec<NewCatalogAction> = actions_to_import
            .into_iter()
            .map(NewCatalogAction::from)
            .collect();

        // Batch insert actions
        Self::batch_insert_actions(conn, catalog_actions)
    }

    /// Batch insert actions into the database
    fn batch_insert_actions(
        conn: &mut SqliteConnection,
        actions: Vec<NewCatalogAction>,
    ) -> Result<usize> {
        let mut total_inserted = 0;

        debug!(
            "Inserting {} actions individually (SQLite limitation)",
            actions.len()
        );

        for action in actions {
            let inserted = diesel::insert_into(catalog_actions::table)
                .values(&action)
                .on_conflict((catalog_actions::name, catalog_actions::source))
                .do_update()
                .set((
                    catalog_actions::time_type.eq(&action.time_type),
                    catalog_actions::description.eq(&action.description),
                    catalog_actions::see_also.eq(&action.see_also),
                    catalog_actions::full_action_json.eq(&action.full_action_json),
                ))
                .execute(conn)?;

            total_inserted += inserted;
        }

        info!(
            "Successfully imported {} actions into database",
            total_inserted
        );
        Ok(total_inserted)
    }

    /// Remove all actions from a specific source (for book removal)
    pub fn remove_actions_by_source(conn: &mut SqliteConnection, source: &str) -> Result<usize> {
        info!("Removing actions from source: {}", source);

        let deleted =
            diesel::delete(catalog_actions::table.filter(catalog_actions::source.eq(source)))
                .execute(conn)?;

        info!("Removed {} actions from source: {}", deleted, source);
        Ok(deleted)
    }
}

impl<'a> CatalogService for ActionService<'a> {
    type Filters = ActionFilters;
    type Summary = ActionSummary;
    type Full = Action;

    fn search(&mut self, filters: Self::Filters) -> Result<Vec<Self::Summary>> {
        self.search_actions(filters)
    }

    fn get_by_name_and_source(&mut self, name: &str, source: &str) -> Result<Option<Self::Full>> {
        self.get_action_by_name_and_source(name, source)
    }

    fn get_sources(&mut self) -> Result<Vec<String>> {
        self.get_action_sources()
    }
}

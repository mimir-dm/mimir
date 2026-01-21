//! Monster Service
//!
//! Service layer for accessing monster catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Monster, MonsterFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing monster catalog data.
///
/// Provides search, retrieval, and listing operations for monsters.
///
/// # Example
///
/// ```ignore
/// let mut service = MonsterService::new(&mut conn);
///
/// // Search for dragons
/// let filter = MonsterFilter::new().with_creature_type("dragon");
/// let dragons = service.search(&filter)?;
///
/// // Get a specific monster
/// let goblin = service.get_by_name_and_source("Goblin", "MM")?;
/// ```
pub struct MonsterService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> MonsterService<'a> {
    /// Create a new monster service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all monsters from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Monster>> {
        dal::list_monsters_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all monsters (up to DEFAULT_QUERY_LIMIT).
    ///
    /// For large datasets, prefer `search_paginated` with explicit limits.
    pub fn list_all(&mut self) -> ServiceResult<Vec<Monster>> {
        self.search_paginated(&MonsterFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for MonsterService<'a> {
    type Entity = Monster;
    type Filter = MonsterFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        // Use pagination with default limit to prevent memory issues
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_monsters_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_monster_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_monster_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_monster_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_monsters(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_monsters_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_monsters;
    use crate::models::catalog::NewMonster;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_monsters(conn: &mut SqliteConnection) {
        let monsters = vec![
            NewMonster::new("Goblin", "MM", r#"{"name":"Goblin"}"#)
                .with_cr("1/4")
                .with_creature_type("humanoid")
                .with_size("S"),
            NewMonster::new("Orc", "MM", r#"{"name":"Orc"}"#)
                .with_cr("1/2")
                .with_creature_type("humanoid")
                .with_size("M"),
            NewMonster::new("Adult Red Dragon", "MM", r#"{"name":"Adult Red Dragon"}"#)
                .with_cr("17")
                .with_creature_type("dragon")
                .with_size("H"),
            NewMonster::new("Beholder", "MM", r#"{"name":"Beholder"}"#)
                .with_cr("13")
                .with_creature_type("aberration")
                .with_size("L"),
            NewMonster::new("Frost Giant", "VGM", r#"{"name":"Frost Giant"}"#)
                .with_cr("8")
                .with_creature_type("giant")
                .with_size("H"),
        ];
        insert_monsters(conn, &monsters).expect("Failed to insert test monsters");
    }

    #[test]
    fn test_monster_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_monsters(&mut conn);

        let mut service = MonsterService::new(&mut conn);

        // Search all
        let filter = MonsterFilter::default();
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 5);

        // Search by creature type
        let filter = MonsterFilter::new().with_creature_type("humanoid");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 2);

        // Search by name
        let filter = MonsterFilter::new().with_name_contains("dragon");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Adult Red Dragon");
    }

    #[test]
    fn test_monster_service_search_paginated() {
        let mut conn = setup_test_db_with_sources();
        insert_test_monsters(&mut conn);

        let mut service = MonsterService::new(&mut conn);
        let filter = MonsterFilter::default();

        let page1 = service
            .search_paginated(&filter, 2, 0)
            .expect("Search failed");
        assert_eq!(page1.len(), 2);

        let page2 = service
            .search_paginated(&filter, 2, 2)
            .expect("Search failed");
        assert_eq!(page2.len(), 2);

        let page3 = service
            .search_paginated(&filter, 2, 4)
            .expect("Search failed");
        assert_eq!(page3.len(), 1);
    }

    #[test]
    fn test_monster_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_monsters(&mut conn);

        let mut service = MonsterService::new(&mut conn);

        // Get by name and source
        let goblin = service
            .get_by_name_and_source("Goblin", "MM")
            .expect("Query failed")
            .expect("Goblin not found");
        assert_eq!(goblin.name, "Goblin");
        assert_eq!(goblin.cr, Some("1/4".to_string()));

        // Get by ID
        let id = goblin.id.expect("Goblin should have an ID");
        let same_goblin = service
            .get(id)
            .expect("Query failed")
            .expect("Goblin not found by ID");
        assert_eq!(same_goblin.name, "Goblin");

        // Not found
        let not_found = service
            .get_by_name_and_source("NonExistent", "MM")
            .expect("Query failed");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_monster_service_list_sources() {
        let mut conn = setup_test_db_with_sources();
        insert_test_monsters(&mut conn);

        let mut service = MonsterService::new(&mut conn);
        let sources = service.list_sources().expect("List sources failed");

        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&"MM".to_string()));
        assert!(sources.contains(&"VGM".to_string()));
    }

    #[test]
    fn test_monster_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_monsters(&mut conn);

        let mut service = MonsterService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 5);
        assert_eq!(service.count_by_source("MM").expect("Count failed"), 4);
        assert_eq!(service.count_by_source("VGM").expect("Count failed"), 1);
    }

    #[test]
    fn test_monster_service_list_by_source() {
        let mut conn = setup_test_db_with_sources();
        insert_test_monsters(&mut conn);

        let mut service = MonsterService::new(&mut conn);
        let mm_monsters = service.list_by_source("MM").expect("List failed");

        assert_eq!(mm_monsters.len(), 4);
    }
}

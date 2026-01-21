//! Item Service
//!
//! Service layer for accessing item catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Item, ItemFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing item catalog data.
pub struct ItemService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> ItemService<'a> {
    /// Create a new item service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all items from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Item>> {
        dal::list_items_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all items (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Item>> {
        self.search_paginated(&ItemFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }
}

impl<'a> CatalogEntityService for ItemService<'a> {
    type Entity = Item;
    type Filter = ItemFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_items_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_item_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_item_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_item_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_items(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_items_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_items;
    use crate::models::catalog::NewItem;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_items(conn: &mut SqliteConnection) {
        let items = vec![
            NewItem::new("Longsword", "PHB", r#"{"name":"Longsword"}"#)
                .with_type("M")
                .with_rarity("none"),
            NewItem::new("Plate Armor", "PHB", r#"{"name":"Plate Armor"}"#)
                .with_type("HA")
                .with_rarity("none"),
            NewItem::new("Ring of Protection", "DMG", r#"{"name":"Ring of Protection"}"#)
                .with_type("RG")
                .with_rarity("rare"),
            NewItem::new("Bag of Holding", "DMG", r#"{"name":"Bag of Holding"}"#)
                .with_type("W")
                .with_rarity("uncommon"),
        ];
        insert_items(conn, &items).expect("Failed to insert test items");
    }

    #[test]
    fn test_item_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_items(&mut conn);

        let mut service = ItemService::new(&mut conn);

        // Search all
        let results = service.search(&ItemFilter::default()).expect("Search failed");
        assert_eq!(results.len(), 4);

        // Search by type
        let filter = ItemFilter::new().with_type("M");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Longsword");

        // Search by rarity
        let filter = ItemFilter::new().with_rarity("rare");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Ring of Protection");
    }

    #[test]
    fn test_item_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_items(&mut conn);

        let mut service = ItemService::new(&mut conn);

        let longsword = service
            .get_by_name_and_source("Longsword", "PHB")
            .expect("Query failed")
            .expect("Longsword not found");
        assert_eq!(longsword.name, "Longsword");
    }

    #[test]
    fn test_item_service_list_sources() {
        let mut conn = setup_test_db_with_sources();
        insert_test_items(&mut conn);

        let mut service = ItemService::new(&mut conn);
        let sources = service.list_sources().expect("List sources failed");

        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&"PHB".to_string()));
        assert!(sources.contains(&"DMG".to_string()));
    }

    #[test]
    fn test_item_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_items(&mut conn);

        let mut service = ItemService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 4);
        assert_eq!(service.count_by_source("PHB").expect("Count failed"), 2);
        assert_eq!(service.count_by_source("DMG").expect("Count failed"), 2);
    }
}

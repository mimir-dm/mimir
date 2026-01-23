//! Item Data Access Layer
//!
//! Database operations for items.

use crate::models::catalog::{Item, ItemFilter, NewItem};
use crate::schema::items;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new item.
///
/// Returns the ID of the inserted item on success.
pub fn insert_item(conn: &mut SqliteConnection, item: &NewItem) -> QueryResult<i32> {
    diesel::insert_into(items::table)
        .values(item)
        .execute(conn)?;

    // Get the last inserted rowid
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple items in a batch.
pub fn insert_items(conn: &mut SqliteConnection, items: &[NewItem]) -> QueryResult<usize> {
    diesel::insert_into(items::table)
        .values(items)
        .execute(conn)
}

/// Get an item by its ID.
pub fn get_item(conn: &mut SqliteConnection, id: i32) -> QueryResult<Item> {
    items::table
        .filter(items::id.eq(id))
        .first(conn)
}

/// Get an item by its ID, returning None if not found.
pub fn get_item_optional(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Item>> {
    items::table
        .filter(items::id.eq(id))
        .first(conn)
        .optional()
}

/// Get an item by name and source.
pub fn get_item_by_name(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> QueryResult<Option<Item>> {
    items::table
        .filter(items::name.eq(name))
        .filter(items::source.eq(source))
        .first(conn)
        .optional()
}

/// List all items, ordered by name.
pub fn list_items(conn: &mut SqliteConnection) -> QueryResult<Vec<Item>> {
    items::table.order(items::name.asc()).load(conn)
}

/// List items from a specific source.
pub fn list_items_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<Vec<Item>> {
    items::table
        .filter(items::source.eq(source))
        .order(items::name.asc())
        .load(conn)
}

/// Search items with filters.
pub fn search_items(
    conn: &mut SqliteConnection,
    filter: &ItemFilter,
) -> QueryResult<Vec<Item>> {
    // If sources filter is explicitly empty, return no results
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = items::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(items::name.like(pattern));
    }

    // Use effective_sources() to support both single source and sources array
    if let Some(sources) = filter.effective_sources() {
        query = query.filter(items::source.eq_any(sources));
    }

    if let Some(ref item_type) = filter.item_type {
        query = query.filter(items::item_type.eq(item_type));
    }

    if let Some(ref rarity) = filter.rarity {
        query = query.filter(items::rarity.eq(rarity));
    }

    query.order(items::name.asc()).load(conn)
}

/// Search items with pagination.
pub fn search_items_paginated(
    conn: &mut SqliteConnection,
    filter: &ItemFilter,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Item>> {
    // If sources filter is explicitly empty, return no results
    if filter.has_empty_sources_filter() {
        return Ok(vec![]);
    }

    let mut query = items::table.into_boxed();

    if let Some(ref name) = filter.name_contains {
        let pattern = format!("%{}%", name);
        query = query.filter(items::name.like(pattern));
    }

    // Use effective_sources() to support both single source and sources array
    if let Some(sources) = filter.effective_sources() {
        query = query.filter(items::source.eq_any(sources));
    }

    if let Some(ref item_type) = filter.item_type {
        query = query.filter(items::item_type.eq(item_type));
    }

    if let Some(ref rarity) = filter.rarity {
        query = query.filter(items::rarity.eq(rarity));
    }

    query
        .order(items::name.asc())
        .limit(limit)
        .offset(offset)
        .load(conn)
}

/// Delete an item by its ID.
pub fn delete_item(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
    diesel::delete(items::table.filter(items::id.eq(id))).execute(conn)
}

/// Delete all items from a specific source.
pub fn delete_items_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(items::table.filter(items::source.eq(source))).execute(conn)
}

/// Count all items.
pub fn count_items(conn: &mut SqliteConnection) -> QueryResult<i64> {
    items::table.count().get_result(conn)
}

/// Count items from a specific source.
pub fn count_items_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<i64> {
    items::table
        .filter(items::source.eq(source))
        .count()
        .get_result(conn)
}

/// List all distinct sources that have items.
pub fn list_item_sources(conn: &mut SqliteConnection) -> QueryResult<Vec<String>> {
    items::table
        .select(items::source)
        .distinct()
        .order(items::source.asc())
        .load(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db_with_sources;

    #[test]
    fn test_insert_and_get_item() {
        let mut conn = setup_test_db_with_sources();

        let data = r#"{"name":"Longsword","source":"PHB","type":"M","rarity":"none"}"#;
        let item = NewItem::new("Longsword", "PHB", data)
            .with_type("M")
            .with_rarity("none");

        let id = insert_item(&mut conn, &item).expect("Failed to insert");
        assert!(id > 0);

        let retrieved = get_item(&mut conn, id).expect("Failed to get");
        assert_eq!(retrieved.name, "Longsword");
        assert_eq!(retrieved.source, "PHB");
        assert_eq!(retrieved.item_type, Some("M".to_string()));
        assert_eq!(retrieved.rarity, Some("none".to_string()));
    }

    #[test]
    fn test_get_item_by_name() {
        let mut conn = setup_test_db_with_sources();

        let data = r#"{"name":"Longsword"}"#;
        let item = NewItem::new("Longsword", "PHB", data);
        insert_item(&mut conn, &item).expect("Failed to insert");

        let found = get_item_by_name(&mut conn, "Longsword", "PHB")
            .expect("Failed to query")
            .expect("Item not found");
        assert_eq!(found.name, "Longsword");

        let not_found = get_item_by_name(&mut conn, "Greatsword", "PHB").expect("Failed to query");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_list_items_by_source() {
        let mut conn = setup_test_db_with_sources();

        let items = vec![
            NewItem::new("Longsword", "PHB", r#"{"name":"Longsword"}"#),
            NewItem::new("Shortsword", "PHB", r#"{"name":"Shortsword"}"#),
        ];
        insert_items(&mut conn, &items).expect("Failed to insert");

        let list = list_items_by_source(&mut conn, "PHB").expect("Failed to list");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_search_items() {
        let mut conn = setup_test_db_with_sources();

        let items = vec![
            NewItem::new("Longsword", "PHB", r#"{"name":"Longsword"}"#)
                .with_type("M")
                .with_rarity("none"),
            NewItem::new("Plate Armor", "PHB", r#"{"name":"Plate Armor"}"#)
                .with_type("HA")
                .with_rarity("none"),
            NewItem::new("Ring of Protection", "PHB", r#"{"name":"Ring of Protection"}"#)
                .with_type("RG")
                .with_rarity("rare"),
        ];
        insert_items(&mut conn, &items).expect("Failed to insert");

        // Search by item type
        let filter = ItemFilter::new().with_type("M");
        let results = search_items(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Longsword");

        // Search by rarity
        let filter = ItemFilter::new().with_rarity("rare");
        let results = search_items(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Ring of Protection");

        // Search by name
        let filter = ItemFilter::new().with_name_contains("sword");
        let results = search_items(&mut conn, &filter).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Longsword");
    }

    #[test]
    fn test_search_items_paginated() {
        let mut conn = setup_test_db_with_sources();

        // Create owned strings for the item names
        let names: Vec<String> = (1..=10).map(|i| format!("Item {}", i)).collect();
        let items: Vec<_> = names
            .iter()
            .map(|name| NewItem::new(name, "PHB", r#"{"name":"test"}"#))
            .collect();
        insert_items(&mut conn, &items).expect("Failed to insert");

        let filter = ItemFilter::new();
        let page1 = search_items_paginated(&mut conn, &filter, 3, 0).expect("Failed to search");
        assert_eq!(page1.len(), 3);

        let page2 = search_items_paginated(&mut conn, &filter, 3, 3).expect("Failed to search");
        assert_eq!(page2.len(), 3);
    }

    #[test]
    fn test_delete_item() {
        let mut conn = setup_test_db_with_sources();

        let item = NewItem::new("Longsword", "PHB", r#"{"name":"Longsword"}"#);
        let id = insert_item(&mut conn, &item).expect("Failed to insert");

        assert_eq!(count_items(&mut conn).expect("Failed to count"), 1);

        delete_item(&mut conn, id).expect("Failed to delete");

        assert_eq!(count_items(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_delete_items_by_source() {
        let mut conn = setup_test_db_with_sources();

        let items = vec![
            NewItem::new("Longsword", "PHB", r#"{"name":"Longsword"}"#),
            NewItem::new("Shortsword", "PHB", r#"{"name":"Shortsword"}"#),
        ];
        insert_items(&mut conn, &items).expect("Failed to insert");

        assert_eq!(count_items(&mut conn).expect("Failed to count"), 2);

        delete_items_by_source(&mut conn, "PHB").expect("Failed to delete");

        assert_eq!(count_items(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_count_items() {
        let mut conn = setup_test_db_with_sources();

        assert_eq!(count_items(&mut conn).expect("Failed to count"), 0);

        let items = vec![
            NewItem::new("Longsword", "PHB", r#"{"name":"Longsword"}"#),
            NewItem::new("Shortsword", "PHB", r#"{"name":"Shortsword"}"#),
        ];
        insert_items(&mut conn, &items).expect("Failed to insert");

        assert_eq!(count_items(&mut conn).expect("Failed to count"), 2);
        assert_eq!(
            count_items_by_source(&mut conn, "PHB").expect("Failed to count"),
            2
        );
    }
}

//! Book Data Access Layer
//!
//! Database operations for book content.

use crate::models::catalog::{Book, NewBook};
use crate::schema::books;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new book.
///
/// Returns the inserted book ID on success.
pub fn insert_book(conn: &mut SqliteConnection, book: &NewBook) -> QueryResult<i32> {
    diesel::insert_into(books::table)
        .values(book)
        .execute(conn)?;

    // Return the inserted ID
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
        "last_insert_rowid()",
    ))
    .get_result(conn)
}

/// Insert or replace a book (upsert).
///
/// If a book with the same source already exists, it will be replaced.
pub fn upsert_book(conn: &mut SqliteConnection, book: &NewBook) -> QueryResult<i32> {
    diesel::insert_or_ignore_into(books::table)
        .values(book)
        .execute(conn)?;

    // If insert was ignored (already exists), update instead
    diesel::update(books::table.filter(books::source.eq(book.source)))
        .set((
            books::name.eq(book.name),
            books::data.eq(book.data),
            books::contents.eq(book.contents),
            books::cover_path.eq(book.cover_path),
        ))
        .execute(conn)?;

    // Get the book ID
    books::table
        .filter(books::source.eq(book.source))
        .select(books::id)
        .first::<Option<i32>>(conn)
        .map(|id| id.unwrap_or(0))
}

/// Get a book by its source code.
pub fn get_book_by_source(conn: &mut SqliteConnection, source: &str) -> QueryResult<Option<Book>> {
    books::table
        .filter(books::source.eq(source))
        .first(conn)
        .optional()
}

/// List all books.
pub fn list_books(conn: &mut SqliteConnection) -> QueryResult<Vec<Book>> {
    books::table.order(books::name.asc()).load(conn)
}

/// List books by source codes.
pub fn list_books_by_sources(
    conn: &mut SqliteConnection,
    sources: &[String],
) -> QueryResult<Vec<Book>> {
    books::table
        .filter(books::source.eq_any(sources))
        .order(books::name.asc())
        .load(conn)
}

/// Delete a book by its source code.
pub fn delete_book(conn: &mut SqliteConnection, source: &str) -> QueryResult<usize> {
    diesel::delete(books::table.filter(books::source.eq(source))).execute(conn)
}

/// Check if a book exists for a source.
pub fn book_exists(conn: &mut SqliteConnection, source: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(books::table.filter(books::source.eq(source)))).get_result(conn)
}

/// Count all books.
pub fn count_books(conn: &mut SqliteConnection) -> QueryResult<i64> {
    books::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::catalog::insert_source;
    use crate::models::catalog::NewCatalogSource;

    fn setup_test_data(conn: &mut SqliteConnection) {
        let source = NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z");
        insert_source(conn, &source).expect("Failed to insert source");
    }

    #[test]
    fn test_insert_and_get_book() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let book = NewBook::new("PHB", "Player's Handbook", "[{\"type\":\"section\"}]");
        let id = insert_book(&mut conn, &book).expect("Failed to insert");
        assert!(id > 0);

        let retrieved = get_book_by_source(&mut conn, "PHB")
            .expect("Failed to get")
            .expect("Book not found");
        assert_eq!(retrieved.source, "PHB");
        assert_eq!(retrieved.name, "Player's Handbook");
        assert_eq!(retrieved.data, "[{\"type\":\"section\"}]");
    }

    #[test]
    fn test_insert_book_with_contents() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let book = NewBook::new("PHB", "Player's Handbook", "[]")
            .with_contents("[{\"name\":\"Chapter 1\"}]")
            .with_cover_path("assets://covers/phb.webp");

        insert_book(&mut conn, &book).expect("Failed to insert");

        let retrieved = get_book_by_source(&mut conn, "PHB")
            .expect("Failed to get")
            .expect("Book not found");
        assert_eq!(retrieved.contents, Some("[{\"name\":\"Chapter 1\"}]".to_string()));
        assert_eq!(retrieved.cover_path, Some("assets://covers/phb.webp".to_string()));
    }

    #[test]
    fn test_list_books() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        // Add another source
        let dmg_source = NewCatalogSource::new("DMG", "Dungeon Master's Guide", true, "2024-01-20T12:00:00Z");
        insert_source(&mut conn, &dmg_source).expect("Failed to add DMG source");

        let book1 = NewBook::new("PHB", "Player's Handbook", "[]");
        let book2 = NewBook::new("DMG", "Dungeon Master's Guide", "[]");

        insert_book(&mut conn, &book1).expect("Failed to insert");
        insert_book(&mut conn, &book2).expect("Failed to insert");

        let books = list_books(&mut conn).expect("Failed to list");
        assert_eq!(books.len(), 2);
    }

    #[test]
    fn test_delete_book() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        let book = NewBook::new("PHB", "Player's Handbook", "[]");
        insert_book(&mut conn, &book).expect("Failed to insert");

        assert!(book_exists(&mut conn, "PHB").expect("Failed to check"));

        delete_book(&mut conn, "PHB").expect("Failed to delete");

        assert!(!book_exists(&mut conn, "PHB").expect("Failed to check"));
    }

    #[test]
    fn test_count_books() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);

        assert_eq!(count_books(&mut conn).expect("Failed to count"), 0);

        let book = NewBook::new("PHB", "Player's Handbook", "[]");
        insert_book(&mut conn, &book).expect("Failed to insert");

        assert_eq!(count_books(&mut conn).expect("Failed to count"), 1);
    }
}

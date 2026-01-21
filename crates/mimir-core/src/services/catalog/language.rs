//! Language Service
//!
//! Service layer for accessing language catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Language, LanguageFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing language catalog data.
pub struct LanguageService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> LanguageService<'a> {
    /// Create a new language service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all languages from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Language>> {
        dal::list_languages_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all languages (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Language>> {
        self.search_paginated(&LanguageFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }

    /// List languages by type (standard, exotic, secret).
    pub fn list_by_type(&mut self, language_type: &str) -> ServiceResult<Vec<Language>> {
        let filter = LanguageFilter::new().with_type(language_type);
        self.search(&filter)
    }
}

impl<'a> CatalogEntityService for LanguageService<'a> {
    type Entity = Language;
    type Filter = LanguageFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_languages_paginated(self.conn, filter, limit, offset)
            .map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_language_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_language_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_language_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_languages(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_languages_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_languages;
    use crate::models::catalog::NewLanguage;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_languages(conn: &mut SqliteConnection) {
        let languages = vec![
            NewLanguage::new("Common", "PHB", r#"{"name":"Common"}"#).with_type("standard"),
            NewLanguage::new("Elvish", "PHB", r#"{"name":"Elvish"}"#).with_type("standard"),
            NewLanguage::new("Dwarvish", "PHB", r#"{"name":"Dwarvish"}"#).with_type("standard"),
            NewLanguage::new("Druidic", "PHB", r#"{"name":"Druidic"}"#).with_type("secret"),
        ];
        insert_languages(conn, &languages).expect("Failed to insert test languages");
    }

    #[test]
    fn test_language_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_languages(&mut conn);

        let mut service = LanguageService::new(&mut conn);

        let results = service
            .search(&LanguageFilter::default())
            .expect("Search failed");
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_language_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_languages(&mut conn);

        let mut service = LanguageService::new(&mut conn);

        let elvish = service
            .get_by_name_and_source("Elvish", "PHB")
            .expect("Query failed")
            .expect("Elvish not found");
        assert_eq!(elvish.name, "Elvish");
    }

    #[test]
    fn test_language_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_languages(&mut conn);

        let mut service = LanguageService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 4);
    }

    #[test]
    fn test_language_service_by_type() {
        let mut conn = setup_test_db_with_sources();
        insert_test_languages(&mut conn);

        let mut service = LanguageService::new(&mut conn);

        let standard = service.list_by_type("standard").expect("Search failed");
        assert_eq!(standard.len(), 3);

        let secret = service.list_by_type("secret").expect("Search failed");
        assert_eq!(secret.len(), 1);
    }
}

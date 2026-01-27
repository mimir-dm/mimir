//! Spell Service
//!
//! Service layer for accessing spell catalog data.

use diesel::SqliteConnection;

use crate::dal::catalog as dal;
use crate::models::catalog::{Spell, SpellFilter};
use crate::services::{ServiceError, ServiceResult, DEFAULT_QUERY_LIMIT};

use super::CatalogEntityService;

/// Service for accessing spell catalog data.
pub struct SpellService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> SpellService<'a> {
    /// Create a new spell service.
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// List all spells from a specific source.
    pub fn list_by_source(&mut self, source: &str) -> ServiceResult<Vec<Spell>> {
        dal::list_spells_by_source(self.conn, source).map_err(ServiceError::from)
    }

    /// List all spells (up to DEFAULT_QUERY_LIMIT).
    pub fn list_all(&mut self) -> ServiceResult<Vec<Spell>> {
        self.search_paginated(&SpellFilter::default(), DEFAULT_QUERY_LIMIT, 0)
    }

    /// List all cantrips.
    pub fn list_cantrips(&mut self) -> ServiceResult<Vec<Spell>> {
        dal::list_cantrips(self.conn).map_err(ServiceError::from)
    }

    /// List all ritual spells.
    pub fn list_rituals(&mut self) -> ServiceResult<Vec<Spell>> {
        dal::list_ritual_spells(self.conn).map_err(ServiceError::from)
    }

    /// List spells by level.
    pub fn list_by_level(&mut self, level: i32) -> ServiceResult<Vec<Spell>> {
        dal::list_spells_by_level(self.conn, level).map_err(ServiceError::from)
    }

    /// List spells available to a specific class.
    pub fn list_by_class(&mut self, class_name: &str) -> ServiceResult<Vec<Spell>> {
        dal::list_spells_by_class(self.conn, class_name).map_err(ServiceError::from)
    }

    /// List spells available to a specific class at a specific level.
    pub fn list_by_class_and_level(&mut self, class_name: &str, level: i32) -> ServiceResult<Vec<Spell>> {
        dal::list_spells_by_class_and_level(self.conn, class_name, level).map_err(ServiceError::from)
    }
}

impl<'a> CatalogEntityService for SpellService<'a> {
    type Entity = Spell;
    type Filter = SpellFilter;

    fn search(&mut self, filter: &Self::Filter) -> ServiceResult<Vec<Self::Entity>> {
        self.search_paginated(filter, DEFAULT_QUERY_LIMIT, 0)
    }

    fn search_paginated(
        &mut self,
        filter: &Self::Filter,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<Self::Entity>> {
        dal::search_spells_paginated(self.conn, filter, limit, offset).map_err(ServiceError::from)
    }

    fn get(&mut self, id: i32) -> ServiceResult<Option<Self::Entity>> {
        dal::get_spell_optional(self.conn, id).map_err(ServiceError::from)
    }

    fn get_by_name_and_source(
        &mut self,
        name: &str,
        source: &str,
    ) -> ServiceResult<Option<Self::Entity>> {
        dal::get_spell_by_name(self.conn, name, source).map_err(ServiceError::from)
    }

    fn list_sources(&mut self) -> ServiceResult<Vec<String>> {
        dal::list_spell_sources(self.conn).map_err(ServiceError::from)
    }

    fn count(&mut self) -> ServiceResult<i64> {
        dal::count_spells(self.conn).map_err(ServiceError::from)
    }

    fn count_by_source(&mut self, source: &str) -> ServiceResult<i64> {
        dal::count_spells_by_source(self.conn, source).map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::catalog::insert_spells;
    use crate::models::catalog::NewSpell;
    use crate::test_utils::setup_test_db_with_sources;

    fn insert_test_spells(conn: &mut SqliteConnection) {
        let spells = vec![
            NewSpell::new("Fireball", "PHB", 3, r#"{"name":"Fireball"}"#)
                .with_school("V")
                .with_concentration(false),
            NewSpell::new("Fire Bolt", "PHB", 0, r#"{"name":"Fire Bolt"}"#)
                .with_school("V"),
            NewSpell::new("Detect Magic", "PHB", 1, r#"{"name":"Detect Magic"}"#)
                .with_school("D")
                .with_ritual(true)
                .with_concentration(true),
            NewSpell::new("Magic Missile", "PHB", 1, r#"{"name":"Magic Missile"}"#)
                .with_school("V"),
            NewSpell::new("Eldritch Blast", "XGE", 0, r#"{"name":"Eldritch Blast"}"#)
                .with_school("V"),
        ];
        insert_spells(conn, &spells).expect("Failed to insert test spells");
    }

    #[test]
    fn test_spell_service_search() {
        let mut conn = setup_test_db_with_sources();
        insert_test_spells(&mut conn);

        let mut service = SpellService::new(&mut conn);

        // Search all
        let results = service.search(&SpellFilter::default()).expect("Search failed");
        assert_eq!(results.len(), 5);

        // Search by school
        let filter = SpellFilter::new().with_school("V");
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 4);

        // Search by level
        let filter = SpellFilter::new().with_level(0);
        let results = service.search(&filter).expect("Search failed");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_spell_service_get() {
        let mut conn = setup_test_db_with_sources();
        insert_test_spells(&mut conn);

        let mut service = SpellService::new(&mut conn);

        let fireball = service
            .get_by_name_and_source("Fireball", "PHB")
            .expect("Query failed")
            .expect("Fireball not found");
        assert_eq!(fireball.name, "Fireball");
        assert_eq!(fireball.level, 3);
    }

    #[test]
    fn test_spell_service_list_sources() {
        let mut conn = setup_test_db_with_sources();
        insert_test_spells(&mut conn);

        let mut service = SpellService::new(&mut conn);
        let sources = service.list_sources().expect("List sources failed");

        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&"PHB".to_string()));
        assert!(sources.contains(&"XGE".to_string()));
    }

    #[test]
    fn test_spell_service_cantrips_and_rituals() {
        let mut conn = setup_test_db_with_sources();
        insert_test_spells(&mut conn);

        let mut service = SpellService::new(&mut conn);

        let cantrips = service.list_cantrips().expect("List cantrips failed");
        assert_eq!(cantrips.len(), 2);

        let rituals = service.list_rituals().expect("List rituals failed");
        assert_eq!(rituals.len(), 1);
        assert_eq!(rituals[0].name, "Detect Magic");
    }

    #[test]
    fn test_spell_service_count() {
        let mut conn = setup_test_db_with_sources();
        insert_test_spells(&mut conn);

        let mut service = SpellService::new(&mut conn);

        assert_eq!(service.count().expect("Count failed"), 5);
        assert_eq!(service.count_by_source("PHB").expect("Count failed"), 4);
        assert_eq!(service.count_by_source("XGE").expect("Count failed"), 1);
    }
}

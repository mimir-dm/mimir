//! Spell List Data Access Layer
//!
//! Database operations for spell-class and spell-subclass relationships.

use crate::models::catalog::{NewSpellClass, NewSpellSubclass, SpellClass, SpellSubclass};
use crate::schema::{spell_classes, spell_subclasses};
use diesel::prelude::*;
use diesel::SqliteConnection;

// ============================================================================
// Spell Classes
// ============================================================================

/// Insert a spell-class association.
pub fn insert_spell_class(conn: &mut SqliteConnection, assoc: &NewSpellClass) -> QueryResult<i32> {
    diesel::insert_into(spell_classes::table)
        .values(assoc)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple spell-class associations in a batch.
pub fn insert_spell_classes(
    conn: &mut SqliteConnection,
    assocs: &[NewSpellClass],
) -> QueryResult<usize> {
    diesel::insert_into(spell_classes::table)
        .values(assocs)
        .execute(conn)
}

/// Get all class associations for a spell.
pub fn get_spell_classes(conn: &mut SqliteConnection, spell_id: i32) -> QueryResult<Vec<SpellClass>> {
    spell_classes::table
        .filter(spell_classes::spell_id.eq(spell_id))
        .order(spell_classes::class_name.asc())
        .load(conn)
}

/// Get all spell associations for a class.
pub fn get_class_spells(conn: &mut SqliteConnection, class_name: &str) -> QueryResult<Vec<SpellClass>> {
    spell_classes::table
        .filter(spell_classes::class_name.eq(class_name))
        .load(conn)
}

/// Get the class names that have a specific spell.
pub fn get_class_names_for_spell(
    conn: &mut SqliteConnection,
    spell_id: i32,
) -> QueryResult<Vec<String>> {
    spell_classes::table
        .filter(spell_classes::spell_id.eq(spell_id))
        .select(spell_classes::class_name)
        .distinct()
        .order(spell_classes::class_name.asc())
        .load(conn)
}

/// Delete all class associations for a spell.
pub fn delete_spell_classes(conn: &mut SqliteConnection, spell_id: i32) -> QueryResult<usize> {
    diesel::delete(spell_classes::table.filter(spell_classes::spell_id.eq(spell_id))).execute(conn)
}

/// Delete all spell associations from a specific source.
pub fn delete_spell_classes_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(spell_classes::table.filter(spell_classes::source.eq(source))).execute(conn)
}

/// Count spell-class associations.
pub fn count_spell_classes(conn: &mut SqliteConnection) -> QueryResult<i64> {
    spell_classes::table.count().get_result(conn)
}

// ============================================================================
// Spell Subclasses
// ============================================================================

/// Insert a spell-subclass association.
pub fn insert_spell_subclass(
    conn: &mut SqliteConnection,
    assoc: &NewSpellSubclass,
) -> QueryResult<i32> {
    diesel::insert_into(spell_subclasses::table)
        .values(assoc)
        .execute(conn)?;

    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("last_insert_rowid()"))
        .get_result(conn)
}

/// Insert multiple spell-subclass associations in a batch.
pub fn insert_spell_subclasses(
    conn: &mut SqliteConnection,
    assocs: &[NewSpellSubclass],
) -> QueryResult<usize> {
    diesel::insert_into(spell_subclasses::table)
        .values(assocs)
        .execute(conn)
}

/// Get all subclass associations for a spell.
pub fn get_spell_subclasses(
    conn: &mut SqliteConnection,
    spell_id: i32,
) -> QueryResult<Vec<SpellSubclass>> {
    spell_subclasses::table
        .filter(spell_subclasses::spell_id.eq(spell_id))
        .order((
            spell_subclasses::class_name.asc(),
            spell_subclasses::subclass_name.asc(),
        ))
        .load(conn)
}

/// Get all spell associations for a subclass.
pub fn get_subclass_spells(
    conn: &mut SqliteConnection,
    subclass_name: &str,
    class_name: &str,
) -> QueryResult<Vec<SpellSubclass>> {
    spell_subclasses::table
        .filter(spell_subclasses::subclass_name.eq(subclass_name))
        .filter(spell_subclasses::class_name.eq(class_name))
        .load(conn)
}

/// Delete all subclass associations for a spell.
pub fn delete_spell_subclasses(conn: &mut SqliteConnection, spell_id: i32) -> QueryResult<usize> {
    diesel::delete(spell_subclasses::table.filter(spell_subclasses::spell_id.eq(spell_id)))
        .execute(conn)
}

/// Delete all spell-subclass associations from a specific source.
pub fn delete_spell_subclasses_by_source(
    conn: &mut SqliteConnection,
    source: &str,
) -> QueryResult<usize> {
    diesel::delete(spell_subclasses::table.filter(spell_subclasses::source.eq(source)))
        .execute(conn)
}

/// Count spell-subclass associations.
pub fn count_spell_subclasses(conn: &mut SqliteConnection) -> QueryResult<i64> {
    spell_subclasses::table.count().get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;
    use crate::dal::catalog::{insert_source, insert_spell};
    use crate::models::catalog::{NewCatalogSource, NewSpell};

    fn setup_test_data(conn: &mut SqliteConnection) {
        let source = NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z");
        insert_source(conn, &source).expect("Failed to insert source");
    }

    fn insert_test_spell(conn: &mut SqliteConnection, name: &str, level: i32) -> i32 {
        let data = format!(r#"{{"name":"{}"}}"#, name);
        let spell = NewSpell::new(name, "PHB", level, &data);
        insert_spell(conn, &spell).expect("Failed to insert spell")
    }

    #[test]
    fn test_spell_class_crud() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);
        let spell_id = insert_test_spell(&mut conn, "Fireball", 3);

        // Insert class associations
        let classes = vec![
            NewSpellClass::new(spell_id, "Sorcerer", "PHB"),
            NewSpellClass::new(spell_id, "Wizard", "PHB"),
        ];
        insert_spell_classes(&mut conn, &classes).expect("Failed to insert");

        // Get classes for spell
        let found = get_spell_classes(&mut conn, spell_id).expect("Failed to query");
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].class_name, "Sorcerer");
        assert_eq!(found[1].class_name, "Wizard");

        // Get class names
        let names = get_class_names_for_spell(&mut conn, spell_id).expect("Failed to query");
        assert_eq!(names, vec!["Sorcerer", "Wizard"]);

        // Delete
        delete_spell_classes(&mut conn, spell_id).expect("Failed to delete");
        assert_eq!(count_spell_classes(&mut conn).expect("Failed to count"), 0);
    }

    #[test]
    fn test_get_class_spells() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);
        let fireball_id = insert_test_spell(&mut conn, "Fireball", 3);
        let shield_id = insert_test_spell(&mut conn, "Shield", 1);

        // Both spells are on the Wizard list
        let classes = vec![
            NewSpellClass::new(fireball_id, "Wizard", "PHB"),
            NewSpellClass::new(shield_id, "Wizard", "PHB"),
            NewSpellClass::new(fireball_id, "Sorcerer", "PHB"),
        ];
        insert_spell_classes(&mut conn, &classes).expect("Failed to insert");

        // Get spells for Wizard class
        let wizard_spells = get_class_spells(&mut conn, "Wizard").expect("Failed to query");
        assert_eq!(wizard_spells.len(), 2);

        // Get spells for Sorcerer class
        let sorcerer_spells = get_class_spells(&mut conn, "Sorcerer").expect("Failed to query");
        assert_eq!(sorcerer_spells.len(), 1);
    }

    #[test]
    fn test_spell_subclass_crud() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);
        let spell_id = insert_test_spell(&mut conn, "Find Familiar", 1);

        // Insert subclass associations
        let subclasses = vec![
            NewSpellSubclass::new(spell_id, "Arcane Trickster", "Rogue", "PHB"),
            NewSpellSubclass::new(spell_id, "Eldritch Knight", "Fighter", "PHB"),
        ];
        insert_spell_subclasses(&mut conn, &subclasses).expect("Failed to insert");

        // Get subclasses for spell
        let found = get_spell_subclasses(&mut conn, spell_id).expect("Failed to query");
        assert_eq!(found.len(), 2);

        // Get spells for subclass
        let at_spells =
            get_subclass_spells(&mut conn, "Arcane Trickster", "Rogue").expect("Failed to query");
        assert_eq!(at_spells.len(), 1);
        assert_eq!(at_spells[0].spell_id, spell_id);

        // Delete
        delete_spell_subclasses(&mut conn, spell_id).expect("Failed to delete");
        assert_eq!(
            count_spell_subclasses(&mut conn).expect("Failed to count"),
            0
        );
    }

    #[test]
    fn test_delete_by_source() {
        let mut conn = test_connection();
        setup_test_data(&mut conn);
        let spell_id = insert_test_spell(&mut conn, "Fireball", 3);

        // Insert associations
        let classes = vec![NewSpellClass::new(spell_id, "Wizard", "PHB")];
        insert_spell_classes(&mut conn, &classes).expect("Failed to insert");

        let subclasses = vec![NewSpellSubclass::new(
            spell_id,
            "Arcane Trickster",
            "Rogue",
            "PHB",
        )];
        insert_spell_subclasses(&mut conn, &subclasses).expect("Failed to insert");

        // Delete by source
        delete_spell_classes_by_source(&mut conn, "PHB").expect("Failed to delete");
        delete_spell_subclasses_by_source(&mut conn, "PHB").expect("Failed to delete");

        assert_eq!(count_spell_classes(&mut conn).expect("Failed to count"), 0);
        assert_eq!(
            count_spell_subclasses(&mut conn).expect("Failed to count"),
            0
        );
    }
}

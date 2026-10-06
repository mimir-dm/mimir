//! The UI fixture: a complete, disposable campaign for development machines.
//!
//! Development machines hold no real campaign data. Screenshots, the Playwright
//! harness and other UX work run against this fixture instead: the SRD catalog
//! plus the "Lost Mine of Phandelver" dev campaign (modules, PCs with spells and
//! gear, NPCs, homebrew, maps with tokens, lights, traps and POIs).
//!
//! Only available with the `fixtures` feature.

use std::path::Path;

use diesel::SqliteConnection;

use crate::dal::catalog;
use crate::services::ServiceResult;

use super::dev::{seed_dev_data, TEST_CAMPAIGN_NAME};
use super::srd::{seed_srd_catalog, SrdCatalogCounts};

/// Name of the fixture campaign.
pub const FIXTURE_CAMPAIGN_NAME: &str = TEST_CAMPAIGN_NAME;

/// What `seed_ui_fixture` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiFixtureReport {
    /// Catalog rows inserted; `None` when the catalog already had sources.
    pub catalog: Option<SrdCatalogCounts>,
    /// Whether the fixture campaign was created (false if it already existed).
    pub campaign_created: bool,
}

/// Seed the UI fixture into `conn`. Map assets are copied under `app_data_dir`.
///
/// Safe to call again: the catalog is loaded only into an empty catalog and the
/// campaign only if it does not exist yet.
pub fn seed_ui_fixture(
    conn: &mut SqliteConnection,
    app_data_dir: &Path,
) -> ServiceResult<UiFixtureReport> {
    let catalog = if catalog::list_sources(conn)?.is_empty() {
        Some(seed_srd_catalog(conn)?)
    } else {
        None
    };
    let campaign_created = seed_dev_data(conn, app_data_dir)?;
    Ok(UiFixtureReport {
        catalog,
        campaign_created,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign as dal;
    use crate::test_utils::setup_test_db;

    #[test]
    fn seeds_catalog_and_campaign_once() {
        let mut conn = setup_test_db();
        let app_dir = tempfile::tempdir().unwrap();

        let first = seed_ui_fixture(&mut conn, app_dir.path()).expect("seed fixture");
        let catalog = first.catalog.expect("catalog loaded into empty DB");
        assert!(first.campaign_created);
        assert_eq!(catalog.spells, 43);
        assert_eq!(catalog.monsters, 17);
        assert_eq!(catalog.classes, 12);

        let campaigns = dal::list_campaigns(&mut conn, false).unwrap();
        assert_eq!(campaigns.len(), 1);
        assert_eq!(campaigns[0].name, FIXTURE_CAMPAIGN_NAME);

        // Second call changes nothing.
        let again = seed_ui_fixture(&mut conn, app_dir.path()).unwrap();
        assert_eq!(
            again,
            UiFixtureReport {
                catalog: None,
                campaign_created: false
            }
        );
        assert_eq!(dal::list_campaigns(&mut conn, false).unwrap().len(), 1);
    }

    #[test]
    fn seeded_spells_all_resolve_in_the_catalog() {
        let mut conn = setup_test_db();
        let app_dir = tempfile::tempdir().unwrap();
        seed_ui_fixture(&mut conn, app_dir.path()).unwrap();

        let campaign_id = dal::list_campaigns(&mut conn, false).unwrap()[0].id.clone();
        let mut known = 0;
        for character in dal::list_campaign_characters(&mut conn, &campaign_id).unwrap() {
            for spell in dal::list_character_spells(&mut conn, &character.id).unwrap() {
                known += 1;
                assert!(
                    catalog::get_spell_by_name(&mut conn, &spell.spell_name, &spell.spell_source)
                        .unwrap()
                        .is_some(),
                    "{} knows {} ({}), which is not in the SRD catalog",
                    character.name,
                    spell.spell_name,
                    spell.spell_source
                );
            }
        }
        assert!(
            known >= 20,
            "expected the wizard and cleric to know spells, got {known}"
        );
    }

    #[test]
    fn class_spell_lists_cover_the_seeded_casters() {
        let mut conn = setup_test_db();
        let app_dir = tempfile::tempdir().unwrap();
        let report = seed_ui_fixture(&mut conn, app_dir.path()).unwrap();
        assert_eq!(report.catalog.unwrap().spell_classes, 130);

        let campaign_id = dal::list_campaigns(&mut conn, false).unwrap()[0].id.clone();
        for character in dal::list_campaign_characters(&mut conn, &campaign_id).unwrap() {
            for spell in dal::list_character_spells(&mut conn, &character.id).unwrap() {
                let catalog_spell =
                    catalog::get_spell_by_name(&mut conn, &spell.spell_name, &spell.spell_source)
                        .unwrap()
                        .unwrap();
                let classes =
                    catalog::get_class_names_for_spell(&mut conn, catalog_spell.id.unwrap())
                        .unwrap();
                assert!(
                    classes.iter().any(|c| c == &spell.source_class),
                    "{} is on {:?}, not on the {} list",
                    spell.spell_name,
                    classes,
                    spell.source_class
                );
            }
        }
        let cleric = catalog::get_class_spells(&mut conn, "Cleric").unwrap();
        assert!(
            cleric.len() >= 15,
            "cleric list too short: {}",
            cleric.len()
        );
    }

    #[test]
    fn map_assets_land_in_the_scratch_app_dir() {
        let mut conn = setup_test_db();
        let app_dir = tempfile::tempdir().unwrap();
        seed_ui_fixture(&mut conn, app_dir.path()).unwrap();

        let campaign_id = dal::list_campaigns(&mut conn, false).unwrap()[0].id.clone();
        let maps = dal::list_campaign_maps(&mut conn, &campaign_id).unwrap();
        assert!(!maps.is_empty(), "fixture should include maps");

        let files: Vec<_> = walk(app_dir.path());
        assert!(
            files
                .iter()
                .any(|p| p.extension().is_some_and(|e| e == "dd2vtt" || e == "png")),
            "map assets should be copied under the app dir, found {files:?}"
        );
    }

    fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walk(&path));
            } else {
                out.push(path);
            }
        }
        out
    }
}

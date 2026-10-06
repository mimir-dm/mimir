//! SRD catalog fixture loader.
//!
//! Loads the SRD reference data committed under `crates/mimir-core/tests/fixtures`
//! (classes, subclasses, features, backgrounds, races, items, spells, monsters,
//! and which classes can cast each spell) into a database. SRD content is published under the OGL and is safe to commit.
//!
//! The JSON is read at runtime from the source tree, so nothing is embedded in
//! a binary. Only available with the `fixtures` feature: the integration tests
//! and the dev-only UI bridge use it; the app never does.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use diesel::SqliteConnection;
use serde_json::Value;

use crate::dal::catalog;
use crate::models::catalog::{
    NewBackground, NewCatalogSource, NewClass, NewClassFeature, NewItem, NewMonster, NewRace,
    NewSpell, NewSpellClass, NewSubclass, NewSubclassFeature,
};
use crate::services::{ServiceError, ServiceResult};

/// The SRD fixture directory in the source tree.
pub fn srd_fixtures_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))
}

/// How many catalog rows `seed_srd_catalog` inserted, per kind.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SrdCatalogCounts {
    pub sources: usize,
    pub classes: usize,
    pub subclasses: usize,
    pub class_features: usize,
    pub subclass_features: usize,
    pub backgrounds: usize,
    pub races: usize,
    pub items: usize,
    pub spells: usize,
    /// Spell-to-class links (the class spell lists the character sheet uses).
    pub spell_classes: usize,
    pub monsters: usize,
}

/// Class lists per spell (5etools spell entries carry none; the importer gets
/// them from a separate lookup). Entries: `{spell, spellSource, classes: [{name, source}]}`.
const SPELL_CLASSES_FILE: &str = "srd_spell_classes.json";

const FIXTURE_FILES: [&str; 9] = [
    "srd_classes.json",
    "srd_subclasses.json",
    "srd_class_features.json",
    "srd_subclass_features.json",
    "srd_backgrounds.json",
    "srd_races.json",
    "srd_items.json",
    "srd_spells.json",
    "srd_monsters.json",
];

/// Load the SRD catalog fixtures from `srd_fixtures_dir()` into the database.
pub fn seed_srd_catalog(conn: &mut SqliteConnection) -> ServiceResult<SrdCatalogCounts> {
    seed_srd_catalog_from(conn, &srd_fixtures_dir())
}

/// Load the SRD catalog fixtures from `dir` into the database.
pub fn seed_srd_catalog_from(
    conn: &mut SqliteConnection,
    dir: &Path,
) -> ServiceResult<SrdCatalogCounts> {
    let files: HashMap<&str, Vec<Value>> = FIXTURE_FILES
        .iter()
        .map(|f| load(dir, f).map(|v| (*f, v)))
        .collect::<ServiceResult<_>>()?;

    let mut counts = SrdCatalogCounts {
        sources: seed_sources(conn, &files)?,
        ..Default::default()
    };

    for item in &files["srd_classes.json"] {
        let (data, fluff) = blob(item, &["id", "fluff"], true)?;
        catalog::insert_class(
            conn,
            &NewClass {
                name: str_field(item, "name")?,
                source: str_field(item, "source")?,
                data: &data,
                fluff: fluff.as_deref(),
            },
        )?;
        counts.classes += 1;
    }

    for item in &files["srd_subclasses.json"] {
        let (data, fluff) = blob(item, &["id", "fluff", "className"], true)?;
        catalog::insert_subclass(
            conn,
            &NewSubclass {
                name: str_field(item, "name")?,
                class_name: str_field(item, "className")?,
                source: str_field(item, "source")?,
                data: &data,
                fluff: fluff.as_deref(),
            },
        )?;
        counts.subclasses += 1;
    }

    for item in &files["srd_class_features.json"] {
        let (data, _) = blob(item, &["id", "className", "classSource"], false)?;
        catalog::insert_class_feature(
            conn,
            &NewClassFeature {
                name: str_field(item, "name")?,
                source: str_field(item, "source")?,
                class_name: str_field(item, "className")?,
                class_source: item["classSource"].as_str().unwrap_or("PHB"),
                level: int_field(item, "level")?,
                data: &data,
            },
        )?;
        counts.class_features += 1;
    }

    for item in &files["srd_subclass_features.json"] {
        let (data, _) = blob(
            item,
            &[
                "id",
                "className",
                "classSource",
                "subclassName",
                "subclassSource",
            ],
            false,
        )?;
        catalog::insert_subclass_feature(
            conn,
            &NewSubclassFeature {
                name: str_field(item, "name")?,
                source: str_field(item, "source")?,
                class_name: str_field(item, "className")?,
                class_source: item["classSource"].as_str().unwrap_or("PHB"),
                subclass_name: str_field(item, "subclassName")?,
                subclass_source: item["subclassSource"].as_str().unwrap_or("PHB"),
                level: int_field(item, "level")?,
                data: &data,
            },
        )?;
        counts.subclass_features += 1;
    }

    for item in &files["srd_backgrounds.json"] {
        let (data, fluff) = blob(item, &["id", "fluff"], true)?;
        catalog::insert_background(
            conn,
            &NewBackground {
                name: str_field(item, "name")?,
                source: str_field(item, "source")?,
                data: &data,
                fluff: fluff.as_deref(),
            },
        )?;
        counts.backgrounds += 1;
    }

    for item in &files["srd_races.json"] {
        let (data, fluff) = blob(item, &["id", "fluff"], true)?;
        catalog::insert_race(
            conn,
            &NewRace {
                name: str_field(item, "name")?,
                source: str_field(item, "source")?,
                data: &data,
                fluff: fluff.as_deref(),
            },
        )?;
        counts.races += 1;
    }

    for item in &files["srd_items.json"] {
        let (data, fluff) = blob(item, &["id", "fluff", "_itemType", "_rarity"], true)?;
        catalog::insert_item(
            conn,
            &NewItem {
                name: str_field(item, "name")?,
                source: str_field(item, "source")?,
                item_type: item.get("_itemType").and_then(|v| v.as_str()),
                rarity: item.get("_rarity").and_then(|v| v.as_str()),
                data: &data,
                fluff: fluff.as_deref(),
            },
        )?;
        counts.items += 1;
    }

    let mut spell_ids: HashMap<(String, String), i32> = HashMap::new();
    for item in &files["srd_spells.json"] {
        let (data, fluff) = blob(
            item,
            &[
                "id",
                "fluff",
                "_level",
                "_school",
                "_ritual",
                "_concentration",
            ],
            true,
        )?;
        let flag = |k: &str| i32::from(item.get(k).and_then(|v| v.as_bool()).unwrap_or(false));
        let id = catalog::insert_spell(
            conn,
            &NewSpell {
                name: str_field(item, "name")?,
                source: str_field(item, "source")?,
                level: item.get("_level").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                school: item.get("_school").and_then(|v| v.as_str()),
                ritual: flag("_ritual"),
                concentration: flag("_concentration"),
                data: &data,
                fluff: fluff.as_deref(),
            },
        )?;
        spell_ids.insert(
            (
                str_field(item, "name")?.to_string(),
                str_field(item, "source")?.to_string(),
            ),
            id,
        );
        counts.spells += 1;
    }

    for entry in load(dir, SPELL_CLASSES_FILE)? {
        let key = (
            str_field(&entry, "spell")?.to_string(),
            str_field(&entry, "spellSource")?.to_string(),
        );
        let spell_id = *spell_ids.get(&key).ok_or_else(|| {
            ServiceError::Validation(format!(
                "{} lists '{}' ({}), which is not in srd_spells.json",
                SPELL_CLASSES_FILE, key.0, key.1
            ))
        })?;
        for class in entry["classes"].as_array().into_iter().flatten() {
            catalog::insert_spell_class(
                conn,
                &NewSpellClass::new(
                    spell_id,
                    str_field(class, "name")?,
                    str_field(class, "source")?,
                ),
            )?;
            counts.spell_classes += 1;
        }
    }

    for item in &files["srd_monsters.json"] {
        let (data, fluff) = blob(
            item,
            &["id", "fluff", "_cr", "_creatureType", "_size"],
            true,
        )?;
        catalog::insert_monster(
            conn,
            &NewMonster {
                name: str_field(item, "name")?,
                source: str_field(item, "source")?,
                cr: item.get("_cr").and_then(|v| v.as_str()),
                creature_type: item.get("_creatureType").and_then(|v| v.as_str()),
                size: item.get("_size").and_then(|v| v.as_str()),
                token_image_path: None,
                data: &data,
                fluff: fluff.as_deref(),
            },
        )?;
        counts.monsters += 1;
    }

    Ok(counts)
}

/// Insert every source code the fixtures use (source, classSource, subclassSource).
fn seed_sources(
    conn: &mut SqliteConnection,
    files: &HashMap<&str, Vec<Value>>,
) -> ServiceResult<usize> {
    let names: HashMap<&str, &str> = [
        ("PHB", "Player's Handbook"),
        ("MM", "Monster Manual"),
        ("DMG", "Dungeon Master's Guide"),
        ("XGE", "Xanathar's Guide to Everything"),
        ("TCE", "Tasha's Cauldron of Everything"),
        ("XDMG", "Dungeon Master's Guide (2024)"),
        ("VGM", "Volo's Guide to Monsters"),
        ("MPMM", "Mordenkainen Presents: Monsters of the Multiverse"),
    ]
    .into_iter()
    .collect();

    let mut codes = BTreeSet::new();
    for items in files.values() {
        for item in items {
            for key in ["source", "classSource", "subclassSource"] {
                if let Some(code) = item.get(key).and_then(|v| v.as_str()) {
                    codes.insert(code.to_string());
                }
            }
        }
    }

    for code in &codes {
        let name = names.get(code.as_str()).copied().unwrap_or(code.as_str());
        catalog::insert_source(
            conn,
            &NewCatalogSource::new(code, name, true, "2024-01-20T12:00:00Z"),
        )?;
    }
    Ok(codes.len())
}

fn load(dir: &Path, file: &str) -> ServiceResult<Vec<Value>> {
    let path = dir.join(file);
    let text = std::fs::read_to_string(&path)?;
    serde_json::from_str(&text).map_err(|e| {
        ServiceError::Validation(format!(
            "SRD fixture {} is not valid JSON: {}",
            path.display(),
            e
        ))
    })
}

fn str_field<'a>(item: &'a Value, key: &str) -> ServiceResult<&'a str> {
    item.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| missing(item, key))
}

fn int_field(item: &Value, key: &str) -> ServiceResult<i32> {
    item.get(key)
        .and_then(|v| v.as_i64())
        .map(|n| n as i32)
        .ok_or_else(|| missing(item, key))
}

fn missing(item: &Value, key: &str) -> ServiceError {
    ServiceError::Validation(format!(
        "SRD fixture entry {:?} has no '{}'",
        item.get("name").and_then(|v| v.as_str()).unwrap_or("?"),
        key
    ))
}

/// The stored data blob (the entry without `drop` keys) and, if wanted, its fluff.
fn blob(item: &Value, drop: &[&str], with_fluff: bool) -> ServiceResult<(String, Option<String>)> {
    let mut data = item.clone();
    if let Value::Object(ref mut map) = data {
        for key in drop {
            map.remove(*key);
        }
    }
    let to_string = |v: &Value| {
        serde_json::to_string(v)
            .map_err(|e| ServiceError::Validation(format!("SRD fixture serialization: {}", e)))
    };
    let fluff = match (with_fluff, item.get("fluff")) {
        (true, Some(f)) => Some(to_string(f)?),
        _ => None,
    };
    Ok((to_string(&data)?, fluff))
}

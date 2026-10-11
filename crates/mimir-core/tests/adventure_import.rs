//! Adventures in the catalog import (MIMIR-T-0736): `adventures.json` and
//! the adventure files of a 5etools tarball, the `bookData` and
//! `adventureData` wrappers of current 5etools files, and homebrew files.
//! Fixture text only.

use std::io::Write;

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use flate2::{write::GzEncoder, Compression};
use mimir_core::dal::catalog;
use mimir_core::import::{corpus_body, parse_adventures, CatalogImportService};
use serde_json::{json, Value};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn db() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
    conn
}

/// A 5etools tarball with these `data/…` files.
fn tarball(files: &[(&str, Value)]) -> tempfile::NamedTempFile {
    let f = tempfile::Builder::new()
        .suffix(".tar.gz")
        .tempfile()
        .unwrap();
    let gz = GzEncoder::new(f.reopen().unwrap(), Compression::fast());
    let mut tar = tar::Builder::new(gz);
    for (path, v) in files {
        let bytes = serde_json::to_vec(v).unwrap();
        let mut h = tar::Header::new_gnu();
        h.set_size(bytes.len() as u64);
        h.set_mode(0o644);
        h.set_cksum();
        tar.append_data(&mut h, path, &bytes[..]).unwrap();
    }
    tar.into_inner().unwrap().finish().unwrap().flush().unwrap();
    f
}

fn monster(name: &str, source: &str) -> Value {
    json!({"name": name, "source": source, "size": ["M"], "type": "humanoid", "ac": [12],
        "hp": {"average": 9, "formula": "2d8"}, "speed": {"walk": 30},
        "str": 10, "dex": 10, "con": 10, "int": 10, "wis": 10, "cha": 10, "cr": "1/8"})
}

#[test]
fn bodies_of_old_and_current_files() {
    let sections = json!([{"type": "section", "name": "One", "entries": []}]);
    assert_eq!(corpus_body(&json!({"data": sections})), Some(&sections));
    assert_eq!(
        corpus_body(&json!({"bookData": [{"id": "X", "source": "X", "data": sections}]})),
        Some(&sections)
    );
    assert_eq!(
        corpus_body(&json!({"adventureData": [{"id": "X", "source": "X", "data": sections}]})),
        Some(&sections)
    );
    assert_eq!(corpus_body(&json!({"other": 1})), None);
}

#[test]
fn a_tarball_brings_books_adventures_and_adventure_sources() {
    let section = |name: &str| json!([{"type": "section", "name": name, "entries": ["Text."]}]);
    let file = tarball(&[
        (
            "data/books.json",
            json!({"book": [{"id": "TBK", "name": "Test Book", "source": "TBK", "group": "core",
                "contents": [{"name": "Rules"}]}]}),
        ),
        // A current 5etools book file: the `bookData` wrapper.
        (
            "data/book/book-tbk.json",
            json!({"bookData": [{"id": "TBK", "source": "TBK", "data": section("Rules")}]}),
        ),
        (
            "data/adventures.json",
            json!({"adventure": [
                {"name": "Test Adventure", "id": "TAD", "source": "TAD", "group": "supplement",
                 "level": {"start": 1, "end": 4}, "storyline": "Tests",
                 "contents": [{"name": "Chapter 1", "ordinal": {"type": "chapter", "identifier": 1}}]},
                // An adventure inside a book: its id is not a source.
                {"name": "Inner Adventure", "id": "TBK-IN", "source": "TBK", "group": "supplement",
                 "contents": [{"name": "Part 1"}]}
            ]}),
        ),
        (
            "data/adventure/adventure-tad.json",
            json!({"adventureData": [{"id": "TAD", "source": "TAD", "data": section("Chapter 1")}]}),
        ),
        (
            "data/adventure/adventure-tbk-in.json",
            json!({"adventureData": [{"id": "TBK-IN", "source": "TBK", "data": section("Part 1")}]}),
        ),
        (
            "data/bestiary/bestiary-tad.json",
            json!({"monster": [monster("Test Bandit Chief", "TAD")]}),
        ),
    ]);
    let mut conn = db();
    let result = CatalogImportService::new(&mut conn)
        .import_from_tarball(file.path())
        .unwrap();
    assert!(
        result.sources_failed.is_empty(),
        "{:?}",
        result.sources_failed
    );
    assert_eq!(result.sources_imported, vec!["TBK", "TAD"]);
    assert_eq!(result.entity_counts.get("adventure"), Some(&2));

    // The book's text came through the bookData wrapper.
    let book = catalog::get_book_by_source(&mut conn, "TBK")
        .unwrap()
        .unwrap();
    assert_eq!(book.kind, "book");
    assert!(book.data.contains("Rules"));

    // The adventure: kind, index entry (no contents), contents, text.
    let adv = catalog::get_book_by_source(&mut conn, "TAD")
        .unwrap()
        .unwrap();
    assert_eq!(adv.kind, "adventure");
    let meta: Value = serde_json::from_str(adv.meta.as_deref().unwrap()).unwrap();
    assert_eq!(meta["level"], json!({"start": 1, "end": 4}));
    assert!(meta.get("contents").is_none());
    assert!(adv.contents.unwrap().contains("Chapter 1"));
    // Its own monster came with its source.
    assert!(
        catalog::get_monster_by_name(&mut conn, "Test Bandit Chief", "TAD")
            .unwrap()
            .is_some()
    );

    // The adventure inside a book: keyed by its id, with a source of its
    // own for the reader.
    let inner = catalog::get_book_by_source(&mut conn, "TBK-IN")
        .unwrap()
        .unwrap();
    assert_eq!(inner.kind, "adventure");
    assert!(catalog::source_exists(&mut conn, "TBK-IN").unwrap());
}

#[test]
fn a_group_list_without_adventure_leaves_them_out() {
    let file = tarball(&[
        ("data/books.json", json!({"book": []})),
        (
            "data/adventures.json",
            json!({"adventure": [{"name": "Test Adventure", "id": "TAD", "source": "TAD"}]}),
        ),
        (
            "data/adventure/adventure-tad.json",
            json!({"adventureData": [{"id": "TAD", "source": "TAD", "data": []}]}),
        ),
    ]);
    let mut conn = db();
    CatalogImportService::new(&mut conn)
        .with_groups(vec!["core".into()])
        .import_from_tarball(file.path())
        .unwrap();
    assert!(catalog::get_book_by_source(&mut conn, "TAD")
        .unwrap()
        .is_none());
}

#[test]
fn a_homebrew_adventure_file() {
    let file: Value =
        serde_json::from_str(include_str!("../src/seed/assets/fixture-adventure.json")).unwrap();
    assert_eq!(parse_adventures(&file).len(), 1);
    let mut conn = db();
    let result = CatalogImportService::new(&mut conn)
        .import_homebrew(&file)
        .unwrap();
    assert_eq!(result.sources_imported, vec!["MFA"]);
    let adv = catalog::get_book_by_source(&mut conn, "MFA")
        .unwrap()
        .unwrap();
    assert_eq!(adv.name, "The Lantern of Low Fen");
    assert_eq!(adv.kind, "adventure");
    let data: Value = serde_json::from_str(&adv.data).unwrap();
    assert_eq!(data.as_array().unwrap().len(), 4);
    // Marked as homebrew for the Adventures page (MIMIR-T-0745).
    let meta: Value = serde_json::from_str(adv.meta.as_deref().unwrap()).unwrap();
    assert_eq!(meta["homebrew"], true);
    assert!(
        catalog::get_monster_by_name(&mut conn, "Lantern Wisp", "MFA")
            .unwrap()
            .is_some()
    );

    // Again: the source is in the catalog, so the file is refused.
    let err = CatalogImportService::new(&mut conn)
        .import_homebrew(&file)
        .unwrap_err()
        .to_string();
    assert!(err.contains("already in the catalog"), "{err}");

    // Not a homebrew file.
    let err = CatalogImportService::new(&mut conn)
        .import_homebrew(&json!({"monster": []}))
        .unwrap_err()
        .to_string();
    assert!(err.contains("_meta.sources"), "{err}");
}

/// Real adventures from a scratch copy (never committed): put
/// `adventures.json` and `adventure-*.json` in `MIMIR_ADVENTURE_DIR`, then
/// `cargo test -p mimir-core --test adventure_import -- --ignored --nocapture`.
#[test]
#[ignore = "needs MIMIR_ADVENTURE_DIR: a scratch copy of 5etools adventures"]
fn real_adventures_import() {
    let dir = std::path::PathBuf::from(std::env::var("MIMIR_ADVENTURE_DIR").unwrap());
    let index: Value =
        serde_json::from_slice(&std::fs::read(dir.join("adventures.json")).unwrap()).unwrap();
    let mut files = vec![
        ("data/books.json".to_string(), json!({"book": []})),
        ("data/adventures.json".to_string(), index.clone()),
    ];
    let mut present = Vec::new();
    for a in parse_adventures(&index) {
        let p = dir.join(format!("adventure-{}.json", a.id.to_lowercase()));
        if let Ok(bytes) = std::fs::read(&p) {
            files.push((
                format!("data/adventure/adventure-{}.json", a.id.to_lowercase()),
                serde_json::from_slice(&bytes).unwrap(),
            ));
            present.push(a);
        }
    }
    assert!(!present.is_empty());
    let refs: Vec<(&str, Value)> = files.iter().map(|(p, v)| (p.as_str(), v.clone())).collect();
    let file = tarball(&refs);
    let mut conn = db();
    let r = CatalogImportService::new(&mut conn)
        .import_from_tarball(file.path())
        .unwrap();
    for a in present {
        let b = catalog::get_book_by_source(&mut conn, &a.id)
            .unwrap()
            .unwrap();
        let data: Value = serde_json::from_str(&b.data).unwrap();
        let toc: Value = serde_json::from_str(b.contents.as_deref().unwrap()).unwrap();
        let meta: Value = serde_json::from_str(b.meta.as_deref().unwrap()).unwrap();
        println!(
            "{} ({}): {} sections, {} contents entries, levels {}–{}, kind {}",
            b.name,
            b.source,
            data.as_array().unwrap().len(),
            toc.as_array().unwrap().len(),
            meta["level"]["start"],
            meta["level"]["end"],
            b.kind
        );
        assert_eq!(
            data.as_array().unwrap().len(),
            toc.as_array().unwrap().len()
        );
    }
    println!("adventure sources imported: {}", r.sources_imported.len());
}

//! Adventures (COLLIERY-I-0616, ADR MIMIR-A-0011): the catalog entries an
//! adventure needs (MIMIR-T-0738). A pack, an exported adventure and the
//! printed book carry them, so they read with no rule books loaded.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use diesel::SqliteConnection;
use mimir_render::adventure::{refs, Ref};
use serde_json::Value;

use crate::api::reference::entry_named;
use crate::error::ApiError;

/// At most this many entries in a closure: a guard, far above what an
/// adventure names (Curse of Strahd names about 700).
pub const MAX_ENTRIES: usize = 5000;

/// The catalog category (as `/catalog/entry/{category}/…`) of a reference
/// kind; `None` for a kind the catalog does not hold.
pub fn category_of(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "creature" => "monsters",
        "item" => "items",
        "spell" => "spells",
        "condition" | "status" => "conditions",
        "race" => "races",
        "class" => "classes",
        "background" => "backgrounds",
        "feat" => "feats",
        "optfeature" => "optional-features",
        "action" => "actions",
        "deity" => "deities",
        "object" => "objects",
        "trap" => "traps",
        "hazard" => "hazards",
        "language" => "languages",
        "reward" => "rewards",
        "table" => "tables",
        "variantrule" => "variant-rules",
        "vehicle" => "vehicles",
        "cult" | "boon" => "cults",
        "psionic" => "psionics",
        _ => return None,
    })
}

/// The entries a set of references needs, and what is not in the catalog.
#[derive(Debug, Default)]
pub struct Closure {
    /// Each entry found, under its own name and source.
    pub entries: BTreeMap<Ref, Value>,
    /// References with no entry (a kind the catalog does not hold, or a
    /// name it does not have).
    pub missing: BTreeSet<Ref>,
}

/// The entries `start` names, and what those entries name in turn.
pub fn closure(conn: &mut SqliteConnection, start: BTreeSet<Ref>) -> Result<Closure, ApiError> {
    let mut out = Closure::default();
    let mut seen: BTreeSet<Ref> = BTreeSet::new();
    let mut queue: VecDeque<Ref> = start.into_iter().collect();
    while let Some(r) = queue.pop_front() {
        if !seen.insert(r.clone()) || out.entries.len() >= MAX_ENTRIES {
            continue;
        }
        let Some(category) = category_of(&r.kind) else {
            out.missing.insert(r);
            continue;
        };
        match entry_named(conn, category, &r.name, &r.source)? {
            Some(v) => {
                let found = Ref {
                    kind: r.kind.clone(),
                    name: v["name"].as_str().unwrap_or(&r.name).to_string(),
                    source: v["source"].as_str().unwrap_or(&r.source).to_string(),
                };
                if out.entries.contains_key(&found) {
                    continue;
                }
                for next in refs(&v) {
                    if !seen.contains(&next) {
                        queue.push_back(next);
                    }
                }
                out.entries.insert(found, v);
            }
            None => {
                out.missing.insert(r);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(kind: &str, name: &str, source: &str) -> Ref {
        Ref {
            kind: kind.into(),
            name: name.into(),
            source: source.into(),
        }
    }

    fn fixture_db() -> (tempfile::TempDir, SqliteConnection) {
        let dir = tempfile::tempdir().unwrap();
        let url = dir.path().join("mimir.db").to_string_lossy().into_owned();
        let mut conn = mimir_core::db::init_database(&url).unwrap();
        mimir_core::seed::srd::seed_srd_catalog(&mut conn).unwrap();
        (dir, conn)
    }

    #[test]
    fn a_monster_brings_its_spells() {
        let (_dir, mut conn) = fixture_db();
        let c = closure(&mut conn, BTreeSet::from([r("creature", "Lich", "MM")])).unwrap();
        assert!(c.entries.contains_key(&r("creature", "Lich", "MM")));
        // The lich's spellcasting names spells; the fixture holds some of
        // them, and the closure follows them.
        let spells: Vec<&Ref> = c.entries.keys().filter(|k| k.kind == "spell").collect();
        assert!(!spells.is_empty(), "{:?}", c.entries.keys());
        // What the fixture does not hold is reported, not lost.
        assert!(!c.missing.is_empty());
        assert!(c.missing.iter().all(|m| !c.entries.contains_key(m)));
    }

    #[test]
    fn a_wrong_source_falls_back_to_the_name_and_unknown_kinds_are_missing() {
        let (_dir, mut conn) = fixture_db();
        let c = closure(
            &mut conn,
            BTreeSet::from([
                r("creature", "Lich", "LMoP"),
                r("skill", "Perception", "PHB"),
                r("creature", "Nobody At All", "MM"),
            ]),
        )
        .unwrap();
        assert!(c.entries.contains_key(&r("creature", "Lich", "MM")));
        assert!(c.missing.contains(&r("skill", "Perception", "PHB")));
        assert!(c.missing.contains(&r("creature", "Nobody At All", "MM")));
    }
}

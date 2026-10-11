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

// =============================================================================
// An adventure as a campaign (MIMIR-T-0739)
// =============================================================================

/// An adventure in the reader: its book row's data and contents.
pub struct AdventureText {
    pub source: String,
    pub name: String,
    pub data: Vec<Value>,
    pub contents: Vec<Value>,
    /// The catalog source of its own monsters and items.
    pub catalog_source: String,
}

/// Read an adventure from its book row; not found, or a book that is not
/// an adventure, is an error.
pub fn adventure_text(
    conn: &mut SqliteConnection,
    source: &str,
) -> Result<AdventureText, ApiError> {
    use mimir_core::services::ServiceError;
    let b = mimir_core::dal::catalog::get_book_by_source(conn, source)
        .map_err(ServiceError::from)?
        .ok_or_else(|| ServiceError::not_found("Adventure", source))?;
    if b.kind != "adventure" {
        return Err(
            ServiceError::validation(format!("{source} is a book, not an adventure")).into(),
        );
    }
    let parse = |s: Option<&str>| {
        s.and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    };
    let meta: Value = b
        .meta
        .as_deref()
        .and_then(|m| serde_json::from_str(m).ok())
        .unwrap_or(Value::Null);
    Ok(AdventureText {
        catalog_source: meta["source"].as_str().unwrap_or(&b.source).to_string(),
        data: parse(Some(&b.data)),
        contents: parse(b.contents.as_deref()),
        source: b.source,
        name: b.name,
    })
}

/// The plan, as the API shows it.
pub fn plan_view(a: &AdventureText) -> mimir_wire::AdventurePlan {
    use mimir_render::adventure::{plan, PartRole};
    mimir_wire::AdventurePlan {
        source: a.source.clone(),
        name: a.name.clone(),
        parts: plan(&a.data, &a.contents)
            .into_iter()
            .map(|p| mimir_wire::AdventurePart {
                index: p.index,
                role: match p.role {
                    PartRole::Module => "module",
                    _ => "document",
                }
                .into(),
                documents: p.documents.iter().map(|d| d.title.clone()).collect(),
                maps: p.maps.iter().map(|m| m.title.clone()).collect(),
                creatures: p.creatures.len(),
                name: p.name,
            })
            .collect(),
    }
}

/// `{@area text|id|…}` to a link to the document that holds `id`, when that
/// is another document; a place in the same document stays a tag (the
/// viewer goes to it on the page).
pub fn link_areas(markdown: &str, here: &str, anchors: &BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(pos) = rest.find("{@area ") {
        out.push_str(&rest[..pos]);
        let Some(end) = rest[pos..].find('}').map(|e| pos + e) else {
            out.push_str(&rest[pos..]);
            return out;
        };
        let tag = &rest[pos..=end];
        let args: Vec<&str> = rest[pos + 7..end].split('|').collect();
        match (
            args.first(),
            args.get(1)
                .and_then(|id| anchors.get(id.trim()).map(|u| (id, u))),
        ) {
            (Some(text), Some((id, url))) if url != here => {
                out.push_str(&format!("[{}]({url}#{})", text.trim(), id.trim()));
            }
            _ => out.push_str(tag),
        }
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Make the campaign: in one transaction, the campaign, its books (the
/// adventure's source and the sources of what it names), the parts as
/// modules (documents, square-grid maps, module monsters) or campaign
/// documents, and the area links between documents.
pub fn create_campaign(
    conn: &mut SqliteConnection,
    data_dir: &std::path::Path,
    a: &AdventureText,
    req: &mimir_wire::AdventureCampaignRequest,
) -> Result<mimir_wire::AdventureCampaign, ApiError> {
    use diesel::Connection;
    use mimir_core::dal::campaign as dal;
    use mimir_core::models::campaign::{NewCampaign, NewCampaignSource, NewDocument, NewModule};
    use mimir_core::services::{
        AddMonsterInput, CreateMapInput, MapService, ModuleService, MonsterRef, ServiceError,
    };
    use mimir_render::adventure::{plan, refs, PartRole};

    let parts = plan(&a.data, &a.contents);
    let role_of = |p: &mimir_render::adventure::PlanPart| -> (PartRole, bool) {
        match req.parts.iter().find(|c| c.index == p.index) {
            Some(c) => (
                match c.role.as_str() {
                    "module" => PartRole::Module,
                    "skip" => PartRole::Skip,
                    _ => PartRole::Document,
                },
                c.one_document,
            ),
            None => (p.role, false),
        }
    };
    let name = Some(req.name.trim())
        .filter(|n| !n.is_empty())
        .unwrap_or(&a.name)
        .to_string();
    let campaign_id = uuid::Uuid::new_v4().to_string();

    // The documents first, with their ids, so area links can point at them.
    struct Doc {
        id: String,
        module: Option<usize>,
        title: String,
        markdown: String,
        anchors: Vec<String>,
    }
    struct Module {
        id: String,
        name: String,
        part: usize,
    }
    let mut modules: Vec<Module> = Vec::new();
    let mut docs: Vec<Doc> = Vec::new();
    for p in &parts {
        let (role, one) = role_of(p);
        match role {
            PartRole::Skip => {}
            PartRole::Document => docs.push(Doc {
                id: uuid::Uuid::new_v4().to_string(),
                module: None,
                title: p.whole.title.clone(),
                markdown: p.whole.markdown.clone(),
                anchors: p.whole.anchors.clone(),
            }),
            PartRole::Module => {
                let m = modules.len();
                modules.push(Module {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: p.name.clone(),
                    part: p.index,
                });
                let list = if one || p.documents.is_empty() {
                    vec![p.whole.clone()]
                } else {
                    p.documents.clone()
                };
                for d in list {
                    docs.push(Doc {
                        id: uuid::Uuid::new_v4().to_string(),
                        module: Some(m),
                        title: d.title,
                        markdown: d.markdown,
                        anchors: d.anchors,
                    });
                }
            }
        }
    }
    let url_of = |d: &Doc| match d.module {
        Some(m) => format!("/modules/{}?doc={}", modules[m].id, d.id),
        None => format!("/campaigns/{campaign_id}?doc={}", d.id),
    };
    let mut anchors: BTreeMap<String, String> = BTreeMap::new();
    for d in &docs {
        for id in &d.anchors {
            anchors.entry(id.clone()).or_insert_with(|| url_of(d));
        }
    }

    let mut missing: Vec<String> = Vec::new();
    let (mut n_maps, mut n_monsters) = (0, 0);
    conn.transaction(|conn| -> Result<(), ApiError> {
        let description = format!("Made from {}.", a.name);
        dal::insert_campaign(
            conn,
            &NewCampaign::new(&campaign_id, &name).with_description(&description),
        )
        .map_err(ServiceError::from)?;

        // Its books: the adventure's source and those of what it names.
        let mut books: BTreeSet<String> = BTreeSet::from([a.catalog_source.clone()]);
        books.extend(
            refs(&Value::Array(a.data.clone()))
                .into_iter()
                .map(|r| r.source),
        );
        for code in books {
            if mimir_core::dal::catalog::source_exists(conn, &code).map_err(ServiceError::from)? {
                let id = uuid::Uuid::new_v4().to_string();
                dal::insert_campaign_source(
                    conn,
                    &NewCampaignSource::new(&id, &campaign_id, &code),
                )
                .map_err(ServiceError::from)?;
            }
        }

        for (number, m) in modules.iter().enumerate() {
            dal::insert_module(
                conn,
                &NewModule::new(&m.id, &campaign_id, &m.name, number as i32 + 1),
            )
            .map_err(ServiceError::from)?;
        }
        for (order, d) in docs.iter().enumerate() {
            let content = link_areas(&d.markdown, &url_of(d), &anchors);
            let mut nd = match d.module {
                Some(m) => NewDocument::for_module(
                    &d.id,
                    &campaign_id,
                    &modules[m].id,
                    &d.title,
                    "user_document",
                ),
                None => NewDocument::for_campaign(&d.id, &campaign_id, &d.title, "user_document"),
            }
            .with_content(&content);
            nd.sort_order = order as i32;
            dal::insert_document(conn, &nd).map_err(ServiceError::from)?;
        }

        // Each module's maps and monsters.
        for m in &modules {
            let p = &parts[m.part];
            for map in &p.maps {
                let file = data_dir.join("assets").join("catalog").join(&map.path);
                let Ok(bytes) = std::fs::read(&file) else {
                    missing.push(format!("map image {}", map.path));
                    continue;
                };
                let uvtt = mimir_core::services::uvtt_from_image(&bytes, map.grid_px)
                    .map_err(ServiceError::validation)?;
                let filename = format!("{}.uvtt", map.title);
                MapService::new(conn, data_dir).create(CreateMapInput::for_module(
                    &campaign_id,
                    &m.id,
                    &map.title,
                    &filename,
                    uvtt,
                ))?;
                n_maps += 1;
            }
            for (cname, csource) in &p.creatures {
                let Some(v) = entry_named(conn, "monsters", cname, csource)? else {
                    missing.push(format!("creature {cname} ({csource})"));
                    continue;
                };
                ModuleService::new(conn).add_monster(AddMonsterInput {
                    module_id: m.id.clone(),
                    monster: MonsterRef::Catalog {
                        name: v["name"].as_str().unwrap_or(cname).to_string(),
                        source: v["source"].as_str().unwrap_or(csource).to_string(),
                    },
                    quantity: 1,
                    display_name: None,
                    notes: None,
                })?;
                n_monsters += 1;
            }
        }
        Ok(())
    })?;
    Ok(mimir_wire::AdventureCampaign {
        campaign_id,
        modules: modules.len(),
        documents: docs.len(),
        maps: n_maps,
        monsters: n_monsters,
        missing,
    })
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

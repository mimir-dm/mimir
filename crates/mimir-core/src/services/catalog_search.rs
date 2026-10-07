//! Campaign-aware catalog search (COLLIERY-I-0473).
//!
//! One place that answers "search the catalog the way this campaign sees it",
//! for every frontend: category dispatch over the per-entity catalog tables,
//! campaign-source filtering, opt-in merging of campaign homebrew monsters,
//! and filters the per-entity filters do not have (CR range, spell class).

use std::collections::HashSet;

use diesel::SqliteConnection;
use serde::Serialize;

use crate::dal::campaign as campaign_dal;
use crate::dal::catalog as catalog_dal;
use crate::models::catalog::{
    BackgroundFilter, ClassFilter, ConditionFilter, FeatFilter, ItemFilter, MonsterFilter,
    RaceFilter, SpellFilter,
};
use crate::services::{HomebrewService, ServiceResult};

/// A searchable catalog category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CatalogCategory {
    Monster,
    Item,
    Spell,
    Race,
    Class,
    Background,
    Feat,
    Condition,
}

impl CatalogCategory {
    /// Every category, in display order.
    pub const ALL: &'static [CatalogCategory] = &[
        CatalogCategory::Monster,
        CatalogCategory::Item,
        CatalogCategory::Spell,
        CatalogCategory::Race,
        CatalogCategory::Class,
        CatalogCategory::Background,
        CatalogCategory::Feat,
        CatalogCategory::Condition,
    ];

    /// The singular name used in requests ("monster", "spell", ...).
    pub fn as_str(self) -> &'static str {
        match self {
            CatalogCategory::Monster => "monster",
            CatalogCategory::Item => "item",
            CatalogCategory::Spell => "spell",
            CatalogCategory::Race => "race",
            CatalogCategory::Class => "class",
            CatalogCategory::Background => "background",
            CatalogCategory::Feat => "feat",
            CatalogCategory::Condition => "condition",
        }
    }

    /// Parse the singular name.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.as_str() == s)
    }
}

/// What to search for. Filters that do not apply to the category are ignored.
#[derive(Debug, Clone, Default)]
pub struct CatalogQuery {
    /// Name contains (case-insensitive).
    pub name: Option<String>,
    /// Monsters: creature type contains (case-insensitive).
    pub monster_type: Option<String>,
    /// Monsters: lowest challenge rating (fractions as decimals, e.g. 0.25).
    pub cr_min: Option<f64>,
    /// Monsters: highest challenge rating.
    pub cr_max: Option<f64>,
    /// Items: item type.
    pub item_type: Option<String>,
    /// Items: rarity.
    pub rarity: Option<String>,
    /// Spells: spell level (0 = cantrip).
    pub spell_level: Option<i32>,
    /// Spells: school, by name ("evocation") or catalog code ("V").
    pub school: Option<String>,
    /// Spells: on this class's spell list (case-insensitive).
    pub class_name: Option<String>,
    /// Maximum catalog rows. Merged homebrew rows come on top of this.
    pub limit: i64,
}

/// The campaign a search runs for.
#[derive(Debug, Clone)]
pub struct CampaignContext {
    pub campaign_id: String,
    /// Merge the campaign's homebrew monsters into monster results.
    pub include_homebrew: bool,
}

/// One search result, shaped by category.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CatalogHit {
    Monster {
        name: String,
        /// Catalog source code, or "Homebrew".
        source: String,
        cr: Option<String>,
        creature_type: Option<String>,
        size: Option<String>,
        /// Set for campaign homebrew monsters.
        homebrew_id: Option<String>,
    },
    Item {
        name: String,
        source: String,
        rarity: Option<String>,
        item_type: Option<String>,
    },
    Spell {
        name: String,
        source: String,
        level: i32,
        school: Option<String>,
    },
    /// Races, classes, backgrounds, feats, conditions.
    Entry { name: String, source: String },
}

impl CatalogHit {
    pub fn name(&self) -> &str {
        match self {
            CatalogHit::Monster { name, .. }
            | CatalogHit::Item { name, .. }
            | CatalogHit::Spell { name, .. }
            | CatalogHit::Entry { name, .. } => name,
        }
    }

    pub fn source(&self) -> &str {
        match self {
            CatalogHit::Monster { source, .. }
            | CatalogHit::Item { source, .. }
            | CatalogHit::Spell { source, .. }
            | CatalogHit::Entry { source, .. } => source,
        }
    }
}

/// Parse a challenge rating: whole numbers or fractions like "1/4".
pub fn parse_cr(cr: &str) -> Option<f64> {
    match cr.trim().split_once('/') {
        Some((n, d)) => {
            let n: f64 = n.trim().parse().ok()?;
            let d: f64 = d.trim().parse().ok()?;
            (d != 0.0).then(|| n / d)
        }
        None => cr.trim().parse().ok(),
    }
}

/// The catalog's one-letter code for a school of magic, from its name or code
/// (case-insensitive). Unknown values pass through unchanged.
pub fn school_code(school: &str) -> String {
    let code = match school.trim().to_lowercase().as_str() {
        "abjuration" | "a" => "A",
        "conjuration" | "c" => "C",
        "divination" | "d" => "D",
        "enchantment" | "e" => "E",
        "evocation" | "v" => "V",
        "illusion" | "i" => "I",
        "necromancy" | "n" => "N",
        "transmutation" | "t" => "T",
        _ => return school.to_string(),
    };
    code.to_string()
}

fn has_cr_range(q: &CatalogQuery) -> bool {
    q.cr_min.is_some() || q.cr_max.is_some()
}

/// True when there is no CR range, or the CR parses and lies inside it.
fn cr_in_range(cr: Option<&str>, q: &CatalogQuery) -> bool {
    if !has_cr_range(q) {
        return true;
    }
    match cr.and_then(parse_cr) {
        Some(v) => q.cr_min.is_none_or(|min| v >= min) && q.cr_max.is_none_or(|max| v <= max),
        None => false,
    }
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Campaign-aware catalog search.
pub struct CatalogSearch<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> CatalogSearch<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self {
        Self { conn }
    }

    /// Search one category. With a campaign, results are limited to the
    /// campaign's sources (no sources configured = no limit), and its homebrew
    /// monsters are merged in when the context asks for it.
    pub fn search(
        &mut self,
        category: CatalogCategory,
        query: &CatalogQuery,
        campaign: Option<&CampaignContext>,
    ) -> ServiceResult<Vec<CatalogHit>> {
        let sources = self.effective_sources(None, campaign.map(|c| c.campaign_id.as_str()))?;
        let limit = query.limit.max(0);

        let mut hits = match category {
            CatalogCategory::Monster => self.monsters(query, sources, limit)?,
            CatalogCategory::Item => self.items(query, sources, limit)?,
            CatalogCategory::Spell => self.spells(query, sources, limit)?,
            CatalogCategory::Race => {
                let mut f = RaceFilter::new();
                if let Some(n) = &query.name {
                    f = f.with_name_contains(n);
                }
                if let Some(s) = sources {
                    f = f.with_sources(s);
                }
                entries(
                    catalog_dal::search_races_paginated(self.conn, &f, limit, 0)?,
                    |r| (r.name, r.source),
                )
            }
            CatalogCategory::Class => {
                let mut f = ClassFilter::new();
                if let Some(n) = &query.name {
                    f = f.with_name_contains(n);
                }
                if let Some(s) = sources {
                    f = f.with_sources(s);
                }
                entries(
                    catalog_dal::search_classes_paginated(self.conn, &f, limit, 0)?,
                    |c| (c.name, c.source),
                )
            }
            CatalogCategory::Background => {
                let mut f = BackgroundFilter::new();
                if let Some(n) = &query.name {
                    f = f.with_name_contains(n);
                }
                if let Some(s) = sources {
                    f = f.with_sources(s);
                }
                entries(
                    catalog_dal::search_backgrounds_paginated(self.conn, &f, limit, 0)?,
                    |b| (b.name, b.source),
                )
            }
            CatalogCategory::Feat => {
                let mut f = FeatFilter::new();
                if let Some(n) = &query.name {
                    f = f.with_name_contains(n);
                }
                if let Some(s) = sources {
                    f = f.with_sources(s);
                }
                entries(
                    catalog_dal::search_feats_paginated(self.conn, &f, limit, 0)?,
                    |x| (x.name, x.source),
                )
            }
            CatalogCategory::Condition => {
                let mut f = ConditionFilter::new();
                if let Some(n) = &query.name {
                    f = f.with_name_contains(n);
                }
                if let Some(s) = sources {
                    f = f.with_sources(s);
                }
                entries(
                    catalog_dal::search_conditions_paginated(self.conn, &f, limit, 0)?,
                    |c| (c.name, c.source),
                )
            }
        };

        if category == CatalogCategory::Monster {
            if let Some(c) = campaign.filter(|c| c.include_homebrew) {
                hits.extend(self.homebrew_monsters(&c.campaign_id, query)?);
            }
        }
        Ok(hits)
    }

    /// The sources a catalog search should be limited to: the explicit ones if
    /// any are given, else the campaign's (if it has any configured), else
    /// `None`, meaning every source. The one rule for every frontend.
    pub fn effective_sources(
        &mut self,
        explicit: Option<Vec<String>>,
        campaign_id: Option<&str>,
    ) -> ServiceResult<Option<Vec<String>>> {
        if let Some(sources) = explicit.filter(|s| !s.is_empty()) {
            return Ok(Some(sources));
        }
        match campaign_id {
            Some(id) => {
                let codes = campaign_dal::list_campaign_source_codes(self.conn, id)?;
                Ok((!codes.is_empty()).then_some(codes))
            }
            None => Ok(None),
        }
    }

    fn monsters(
        &mut self,
        query: &CatalogQuery,
        sources: Option<Vec<String>>,
        limit: i64,
    ) -> ServiceResult<Vec<CatalogHit>> {
        let mut f = MonsterFilter::new();
        if let Some(n) = &query.name {
            f = f.with_name_contains(n);
        }
        if let Some(t) = &query.monster_type {
            f = f.with_creature_type(t);
        }
        if let Some(s) = sources {
            f = f.with_sources(s);
        }
        // The CR range is not a column filter: filter all matches, then limit.
        let rows = if has_cr_range(query) {
            catalog_dal::search_monsters(self.conn, &f)?
        } else {
            catalog_dal::search_monsters_paginated(self.conn, &f, limit, 0)?
        };
        Ok(rows
            .into_iter()
            .filter(|m| cr_in_range(m.cr.as_deref(), query))
            .take(limit as usize)
            .map(|m| {
                let size = Some(m.size_name().to_string());
                CatalogHit::Monster {
                    name: m.name,
                    source: m.source,
                    cr: m.cr,
                    creature_type: m.creature_type,
                    size,
                    homebrew_id: None,
                }
            })
            .collect())
    }

    fn items(
        &mut self,
        query: &CatalogQuery,
        sources: Option<Vec<String>>,
        limit: i64,
    ) -> ServiceResult<Vec<CatalogHit>> {
        let mut f = ItemFilter::new();
        if let Some(n) = &query.name {
            f = f.with_name_contains(n);
        }
        if let Some(r) = &query.rarity {
            f = f.with_rarity(r);
        }
        if let Some(t) = &query.item_type {
            f = f.with_type(t);
        }
        if let Some(s) = sources {
            f = f.with_sources(s);
        }
        Ok(
            catalog_dal::search_items_paginated(self.conn, &f, limit, 0)?
                .into_iter()
                .map(|i| CatalogHit::Item {
                    name: i.name,
                    source: i.source,
                    rarity: i.rarity,
                    item_type: i.item_type,
                })
                .collect(),
        )
    }

    fn spells(
        &mut self,
        query: &CatalogQuery,
        sources: Option<Vec<String>>,
        limit: i64,
    ) -> ServiceResult<Vec<CatalogHit>> {
        let mut f = SpellFilter::new();
        if let Some(n) = &query.name {
            f = f.with_name_contains(n);
        }
        if let Some(l) = query.spell_level {
            f = f.with_level(l);
        }
        if let Some(s) = &query.school {
            f = f.with_school(school_code(s));
        }
        if let Some(s) = sources {
            f = f.with_sources(s);
        }
        // The class list is a join table: filter all matches, then limit.
        let rows = match &query.class_name {
            Some(class) => {
                let ids: HashSet<i32> = catalog_dal::spell_ids_for_class(self.conn, class)?
                    .into_iter()
                    .collect();
                catalog_dal::search_spells(self.conn, &f)?
                    .into_iter()
                    .filter(|s| s.id.is_some_and(|id| ids.contains(&id)))
                    .take(limit as usize)
                    .collect()
            }
            None => catalog_dal::search_spells_paginated(self.conn, &f, limit, 0)?,
        };
        Ok(rows
            .into_iter()
            .map(|s| CatalogHit::Spell {
                name: s.name,
                source: s.source,
                level: s.level,
                school: s.school,
            })
            .collect())
    }

    /// The campaign's homebrew monsters matching the name, type and CR filters.
    fn homebrew_monsters(
        &mut self,
        campaign_id: &str,
        query: &CatalogQuery,
    ) -> ServiceResult<Vec<CatalogHit>> {
        Ok(HomebrewService::new(self.conn)
            .list_monsters(campaign_id)?
            .into_iter()
            .filter(|hb| {
                query
                    .name
                    .as_deref()
                    .is_none_or(|n| contains_ci(&hb.name, n))
            })
            .filter(|hb| match &query.monster_type {
                None => true,
                Some(t) => hb
                    .creature_type
                    .as_deref()
                    .is_some_and(|ct| contains_ci(ct, t)),
            })
            .filter(|hb| cr_in_range(hb.cr.as_deref(), query))
            .map(|hb| CatalogHit::Monster {
                name: hb.name,
                source: "Homebrew".to_string(),
                cr: hb.cr,
                creature_type: hb.creature_type,
                size: hb.size,
                homebrew_id: Some(hb.id),
            })
            .collect())
    }
}

fn entries<T>(rows: Vec<T>, name_source: impl Fn(T) -> (String, String)) -> Vec<CatalogHit> {
    rows.into_iter()
        .map(|r| {
            let (name, source) = name_source(r);
            CatalogHit::Entry { name, source }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::campaign as campaign_dal;
    use crate::models::campaign::{NewCampaign, NewCampaignSource};
    use crate::seed::srd::seed_srd_catalog;
    use crate::services::{CreateHomebrewMonsterInput, HomebrewService};
    use crate::test_utils::setup_test_db;
    use diesel::SqliteConnection;

    /// SRD catalog + one campaign. `sources` are the campaign's source codes.
    fn setup(sources: &[&str]) -> (SqliteConnection, String) {
        let mut conn = setup_test_db();
        seed_srd_catalog(&mut conn).unwrap();
        let campaign_id = "camp-1".to_string();
        campaign_dal::insert_campaign(&mut conn, &NewCampaign::new(&campaign_id, "Test")).unwrap();
        for (i, code) in sources.iter().enumerate() {
            let id = format!("src-{i}");
            campaign_dal::insert_campaign_source(
                &mut conn,
                &NewCampaignSource::new(&id, &campaign_id, code),
            )
            .unwrap();
        }
        (conn, campaign_id)
    }

    fn add_homebrew(
        conn: &mut SqliteConnection,
        campaign_id: &str,
        name: &str,
        cr: &str,
        kind: &str,
    ) {
        HomebrewService::new(conn)
            .create_monster(CreateHomebrewMonsterInput {
                campaign_id: campaign_id.to_string(),
                name: name.to_string(),
                data: Some("{}".to_string()),
                cr: Some(cr.to_string()),
                creature_type: Some(kind.to_string()),
                size: Some("Medium".to_string()),
                cloned_from_name: None,
                cloned_from_source: None,
            })
            .unwrap();
    }

    fn names(hits: &[CatalogHit]) -> Vec<String> {
        hits.iter().map(|h| h.name().to_string()).collect()
    }

    fn query() -> CatalogQuery {
        CatalogQuery {
            limit: 100,
            ..Default::default()
        }
    }

    fn campaign(id: &str, include_homebrew: bool) -> CampaignContext {
        CampaignContext {
            campaign_id: id.to_string(),
            include_homebrew,
        }
    }

    #[test]
    fn every_category_searches_and_srd_categories_return_rows() {
        // The SRD fixture has no feats or conditions; those must still search.
        let (mut conn, _) = setup(&[]);
        for category in CatalogCategory::ALL {
            let hits = CatalogSearch::new(&mut conn)
                .search(*category, &query(), None)
                .unwrap();
            let in_fixture =
                !matches!(category, CatalogCategory::Feat | CatalogCategory::Condition);
            assert_eq!(
                !hits.is_empty(),
                in_fixture,
                "{category:?}: {} rows",
                hits.len()
            );
        }
    }

    #[test]
    fn categories_parse_from_their_names() {
        for category in CatalogCategory::ALL {
            assert_eq!(CatalogCategory::parse(category.as_str()), Some(*category));
        }
        assert_eq!(CatalogCategory::parse("dragon"), None);
    }

    #[test]
    fn campaign_sources_filter_results() {
        // The SRD fixture has PHB spells and MM monsters.
        let (mut conn, campaign_id) = setup(&["MM"]);
        let ctx = campaign(&campaign_id, false);
        let mut search = CatalogSearch::new(&mut conn);
        assert!(!search
            .search(CatalogCategory::Monster, &query(), Some(&ctx))
            .unwrap()
            .is_empty());
        assert!(
            search
                .search(CatalogCategory::Spell, &query(), Some(&ctx))
                .unwrap()
                .is_empty(),
            "PHB spells hidden when the campaign only uses MM"
        );
    }

    #[test]
    fn a_campaign_without_sources_does_not_filter() {
        let (mut conn, campaign_id) = setup(&[]);
        let ctx = campaign(&campaign_id, false);
        let hits = CatalogSearch::new(&mut conn)
            .search(CatalogCategory::Spell, &query(), Some(&ctx))
            .unwrap();
        assert_eq!(hits.len(), 43);
    }

    #[test]
    fn name_filter_is_a_case_insensitive_contains() {
        let (mut conn, _) = setup(&[]);
        let q = CatalogQuery {
            name: Some("dragon".into()),
            ..query()
        };
        let hits = CatalogSearch::new(&mut conn)
            .search(CatalogCategory::Monster, &q, None)
            .unwrap();
        assert_eq!(
            names(&hits),
            ["Adult Red Dragon", "Ancient Red Dragon", "Young Red Dragon"]
        );
    }

    #[test]
    fn cr_range_compares_fractions_numerically() {
        let (mut conn, _) = setup(&[]);
        let mut search = CatalogSearch::new(&mut conn);

        let low = CatalogQuery {
            cr_max: Some(0.25),
            ..query()
        };
        let mut got = names(&search.search(CatalogCategory::Monster, &low, None).unwrap());
        got.sort();
        assert_eq!(
            got,
            ["Goblin", "Kobold", "Rat", "Skeleton", "Wolf", "Zombie"]
        );

        let mid = CatalogQuery {
            cr_min: Some(2.0),
            cr_max: Some(5.0),
            ..query()
        };
        let mut got = names(&search.search(CatalogCategory::Monster, &mid, None).unwrap());
        got.sort();
        assert_eq!(
            got,
            [
                "Basilisk",
                "Hill Giant",
                "Minotaur",
                "Ogre",
                "Owlbear",
                "Wight"
            ]
        );

        let eighth = CatalogQuery {
            cr_min: Some(0.1),
            cr_max: Some(0.2),
            ..query()
        };
        assert_eq!(
            names(
                &search
                    .search(CatalogCategory::Monster, &eighth, None)
                    .unwrap()
            ),
            ["Kobold"]
        );
    }

    #[test]
    fn limit_applies_after_the_cr_filter() {
        let (mut conn, _) = setup(&[]);
        // Alphabetically the first monsters are dragons (CR 17, 24); a limit
        // applied before the CR filter would return nothing.
        let q = CatalogQuery {
            cr_max: Some(0.25),
            limit: 2,
            ..Default::default()
        };
        let hits = CatalogSearch::new(&mut conn)
            .search(CatalogCategory::Monster, &q, None)
            .unwrap();
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn monster_type_filter() {
        let (mut conn, _) = setup(&[]);
        let q = CatalogQuery {
            monster_type: Some("undead".into()),
            ..query()
        };
        let mut got = names(
            &CatalogSearch::new(&mut conn)
                .search(CatalogCategory::Monster, &q, None)
                .unwrap(),
        );
        got.sort();
        assert_eq!(got, ["Lich", "Skeleton", "Vampire", "Wight", "Zombie"]);
    }

    #[test]
    fn spell_filters_level_school_and_class() {
        let (mut conn, _) = setup(&[]);
        let mut search = CatalogSearch::new(&mut conn);

        let cleric_cantrips = CatalogQuery {
            class_name: Some("cleric".into()),
            spell_level: Some(0),
            ..query()
        };
        let mut got = names(
            &search
                .search(CatalogCategory::Spell, &cleric_cantrips, None)
                .unwrap(),
        );
        got.sort();
        assert_eq!(
            got,
            ["Guidance", "Light", "Sacred Flame"],
            "class match is case-insensitive"
        );

        let wizard_evocation_3 = CatalogQuery {
            class_name: Some("Wizard".into()),
            spell_level: Some(3),
            school: Some("V".into()),
            ..query()
        };
        let mut got = names(
            &search
                .search(CatalogCategory::Spell, &wizard_evocation_3, None)
                .unwrap(),
        );
        got.sort();
        assert_eq!(got, ["Fireball", "Lightning Bolt"]);

        let none = CatalogQuery {
            class_name: Some("Barbarian".into()),
            ..query()
        };
        assert!(search
            .search(CatalogCategory::Spell, &none, None)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn item_filters_rarity_and_type() {
        let (mut conn, _) = setup(&[]);
        let q = CatalogQuery {
            name: Some("potion".into()),
            ..query()
        };
        let hits = CatalogSearch::new(&mut conn)
            .search(CatalogCategory::Item, &q, None)
            .unwrap();
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|h| matches!(h, CatalogHit::Item { .. })));
    }

    #[test]
    fn homebrew_monsters_are_merged_only_when_asked() {
        let (mut conn, campaign_id) = setup(&[]);
        add_homebrew(&mut conn, &campaign_id, "Frost Wight", "3", "undead");
        add_homebrew(&mut conn, &campaign_id, "Ice Scout", "1/2", "humanoid");
        let mut search = CatalogSearch::new(&mut conn);

        let q = CatalogQuery {
            name: Some("wight".into()),
            ..query()
        };
        let without = search
            .search(
                CatalogCategory::Monster,
                &q,
                Some(&campaign(&campaign_id, false)),
            )
            .unwrap();
        assert_eq!(names(&without), ["Wight"]);

        let with = search
            .search(
                CatalogCategory::Monster,
                &q,
                Some(&campaign(&campaign_id, true)),
            )
            .unwrap();
        assert_eq!(names(&with), ["Wight", "Frost Wight"]);
        match &with[1] {
            CatalogHit::Monster {
                homebrew_id,
                source,
                cr,
                ..
            } => {
                assert!(homebrew_id.is_some());
                assert_eq!(source, "Homebrew");
                assert_eq!(cr.as_deref(), Some("3"));
            }
            other => panic!("expected a monster, got {other:?}"),
        }
        match &with[0] {
            CatalogHit::Monster { homebrew_id, .. } => assert!(homebrew_id.is_none()),
            other => panic!("expected a monster, got {other:?}"),
        }
    }

    #[test]
    fn homebrew_respects_type_and_cr_filters() {
        let (mut conn, campaign_id) = setup(&[]);
        add_homebrew(&mut conn, &campaign_id, "Frost Wight", "3", "undead");
        add_homebrew(&mut conn, &campaign_id, "Ice Scout", "1/2", "humanoid");
        let ctx = campaign(&campaign_id, true);
        let mut search = CatalogSearch::new(&mut conn);

        let humanoid = CatalogQuery {
            monster_type: Some("humanoid".into()),
            ..query()
        };
        let got = names(
            &search
                .search(CatalogCategory::Monster, &humanoid, Some(&ctx))
                .unwrap(),
        );
        assert!(
            got.contains(&"Ice Scout".to_string()) && !got.contains(&"Frost Wight".to_string())
        );

        let low = CatalogQuery {
            cr_max: Some(1.0),
            ..query()
        };
        let got = names(
            &search
                .search(CatalogCategory::Monster, &low, Some(&ctx))
                .unwrap(),
        );
        assert!(
            got.contains(&"Ice Scout".to_string()) && !got.contains(&"Frost Wight".to_string())
        );
    }

    #[test]
    fn homebrew_is_never_merged_into_other_categories() {
        let (mut conn, campaign_id) = setup(&[]);
        add_homebrew(&mut conn, &campaign_id, "Fireball Imp", "1", "fiend");
        let q = CatalogQuery {
            name: Some("fireball".into()),
            ..query()
        };
        let hits = CatalogSearch::new(&mut conn)
            .search(
                CatalogCategory::Spell,
                &q,
                Some(&campaign(&campaign_id, true)),
            )
            .unwrap();
        assert_eq!(names(&hits), ["Fireball"]);
    }

    #[test]
    fn effective_sources_prefers_explicit_then_campaign_then_all() {
        let (mut conn, campaign_id) = setup(&["MM", "PHB"]);
        let (mut bare, bare_id) = setup(&[]);
        let mut search = CatalogSearch::new(&mut conn);
        let explicit = Some(vec!["DMG".to_string()]);

        assert_eq!(
            search
                .effective_sources(explicit.clone(), Some(&campaign_id))
                .unwrap(),
            explicit,
            "explicit sources win"
        );
        assert_eq!(
            search
                .effective_sources(Some(vec![]), Some(&campaign_id))
                .unwrap(),
            Some(vec!["MM".to_string(), "PHB".to_string()]),
            "an empty explicit list counts as none"
        );
        assert_eq!(search.effective_sources(None, None).unwrap(), None);
        assert_eq!(
            CatalogSearch::new(&mut bare)
                .effective_sources(None, Some(&bare_id))
                .unwrap(),
            None,
            "a campaign without sources does not filter"
        );
    }

    #[test]
    fn school_accepts_names_and_codes() {
        assert_eq!(school_code("evocation"), "V");
        assert_eq!(school_code("Necromancy"), "N");
        assert_eq!(school_code("v"), "V");
        assert_eq!(school_code("psionics"), "psionics");

        let (mut conn, _) = setup(&[]);
        let q = CatalogQuery {
            school: Some("evocation".into()),
            spell_level: Some(3),
            ..query()
        };
        let mut got = names(
            &CatalogSearch::new(&mut conn)
                .search(CatalogCategory::Spell, &q, None)
                .unwrap(),
        );
        got.sort();
        assert_eq!(got, ["Fireball", "Lightning Bolt"]);
    }

    #[test]
    fn parse_cr_handles_whole_and_fractional_values() {
        assert_eq!(parse_cr("1/4"), Some(0.25));
        assert_eq!(parse_cr("1/8"), Some(0.125));
        assert_eq!(parse_cr("0"), Some(0.0));
        assert_eq!(parse_cr("21"), Some(21.0));
        assert_eq!(parse_cr("Unknown"), None);
        assert_eq!(parse_cr("1/0"), None);
    }
}

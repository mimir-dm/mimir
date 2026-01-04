//! Cross-reference lookup service
//!
//! Provides unified lookup for cross-references in D&D content.
//! Delegates to the appropriate catalog service based on reference type.

use crate::error::Result;
use crate::models::catalog::Race;
use crate::services::{
    ActionService, BackgroundService, ClassService, ConditionService, FeatService, ItemService,
    MonsterService, RaceService, SpellService,
};
use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, warn};

/// Reference data returned from lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceData {
    /// Type of reference (spell, item, creature, etc.)
    pub ref_type: String,
    /// Name of the referenced entity
    pub name: String,
    /// Source book code
    pub source: Option<String>,
    /// Full data as JSON value
    pub data: Value,
    /// Short preview text for tooltips
    pub preview: String,
}

/// Service for looking up cross-references from the catalog database
pub struct ReferenceService;

impl ReferenceService {
    /// Look up a reference by type, name, and optional source
    ///
    /// Delegates to the appropriate catalog service based on ref_type.
    /// Falls back to searching without source if not found with source.
    pub fn lookup(
        conn: &mut SqliteConnection,
        ref_type: &str,
        ref_name: &str,
        ref_source: Option<&str>,
    ) -> Result<Option<ReferenceData>> {
        debug!(
            "Looking up reference: type={}, name={}, source={:?}",
            ref_type, ref_name, ref_source
        );

        // Normalize the reference type
        let ref_type_lower = ref_type.to_lowercase();

        // Try with provided source first, then fall back to common sources
        let sources_to_try: Vec<&str> = if let Some(src) = ref_source {
            vec![src, "PHB", "MM", "DMG", "XGE", "TCE"]
        } else {
            vec!["PHB", "MM", "DMG", "XGE", "TCE", "XPHB"]
        };

        for source in sources_to_try {
            let result = match ref_type_lower.as_str() {
                "spell" => Self::lookup_spell(conn, ref_name, source),
                "item" => Self::lookup_item(conn, ref_name, source),
                "creature" | "monster" => Self::lookup_monster(conn, ref_name, source),
                "condition" => Self::lookup_condition(conn, ref_name, source),
                "action" => Self::lookup_action(conn, ref_name, source),
                "class" => Self::lookup_class(conn, ref_name, source),
                "race" => Self::lookup_race(conn, ref_name, source),
                "feat" => Self::lookup_feat(conn, ref_name, source),
                "background" => Self::lookup_background(conn, ref_name, source),
                _ => {
                    warn!("Unknown reference type: {}", ref_type);
                    Ok(None)
                }
            };

            if let Ok(Some(data)) = result {
                return Ok(Some(data));
            }
        }

        debug!(
            "Reference not found: type={}, name={}",
            ref_type, ref_name
        );
        Ok(None)
    }

    /// Look up a spell
    fn lookup_spell(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<ReferenceData>> {
        if let Some(spell) = SpellService::get_spell_details(conn, name, source)? {
            let casting_time = spell
                .time
                .first()
                .map(|ct| format!("{} {}", ct.number, ct.unit))
                .unwrap_or_else(|| "1 action".to_string());

            let preview = format!(
                "{} {} • {}",
                if spell.level == 0 {
                    "Cantrip".to_string()
                } else {
                    format!("Level {}", spell.level)
                },
                spell.school.as_str(),
                casting_time
            );

            let data = serde_json::to_value(&spell)?;

            return Ok(Some(ReferenceData {
                ref_type: "spell".to_string(),
                name: spell.name,
                source: Some(spell.source),
                data,
                preview,
            }));
        }
        Ok(None)
    }

    /// Look up an item
    fn lookup_item(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<ReferenceData>> {
        let mut service = ItemService::new(conn);
        if let Some(item) = service.get_item_by_name_and_source(name, source)? {
            let preview = format!(
                "{}{}",
                item.item_type.as_deref().unwrap_or("Item"),
                item.rarity
                    .as_ref()
                    .map(|r| format!(" • {}", r))
                    .unwrap_or_default()
            );

            let data = serde_json::to_value(&item)?;

            return Ok(Some(ReferenceData {
                ref_type: "item".to_string(),
                name: item.name,
                source: Some(item.source),
                data,
                preview,
            }));
        }
        Ok(None)
    }

    /// Look up a monster/creature
    fn lookup_monster(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<ReferenceData>> {
        use crate::models::catalog::types::{ChallengeRatingValue, CreatureTypeValue};

        let mut service = MonsterService::new(conn);
        if let Some(monster) = service.get_monster_by_name_and_source(name, source)? {
            let cr_str = match &monster.cr {
                Some(ChallengeRatingValue::Simple(s)) => format!("CR {}", s),
                Some(ChallengeRatingValue::Complex { cr, .. }) => format!("CR {}", cr),
                None => "CR ?".to_string(),
            };

            // Extract creature type
            let creature_type_str = match &monster.creature_type {
                Some(CreatureTypeValue::Simple(s)) => s.clone(),
                Some(CreatureTypeValue::Complex { base_type, .. }) => base_type.clone(),
                None => "creature".to_string(),
            };

            let preview = format!("{} • {}", creature_type_str, cr_str);

            let data = serde_json::to_value(&monster)?;

            return Ok(Some(ReferenceData {
                ref_type: "creature".to_string(),
                name: monster.name,
                source: Some(monster.source),
                data,
                preview,
            }));
        }
        Ok(None)
    }

    /// Look up a condition
    fn lookup_condition(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<ReferenceData>> {
        let mut service = ConditionService::new(conn);
        if let Some(condition) = service.get_condition_by_name_and_source(name, source)? {
            let (cond_name, preview, data) = match &condition {
                crate::models::catalog::ConditionOrDisease::Condition(c) => {
                    let preview = Self::extract_preview_from_entries(&c.entries, "Condition");
                    (c.name.clone(), preview, serde_json::to_value(c)?)
                }
                crate::models::catalog::ConditionOrDisease::Disease(d) => {
                    let preview = Self::extract_preview_from_entries(&d.entries, "Disease");
                    (d.name.clone(), preview, serde_json::to_value(d)?)
                }
            };

            return Ok(Some(ReferenceData {
                ref_type: "condition".to_string(),
                name: cond_name,
                source: Some(source.to_string()),
                data,
                preview,
            }));
        }
        Ok(None)
    }

    /// Look up an action
    fn lookup_action(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<ReferenceData>> {
        let mut service = ActionService::new(conn);
        if let Some(action) = service.get_action_by_name_and_source(name, source)? {
            let preview = Self::extract_preview_from_entries(&action.entries, "Action");

            let data = serde_json::to_value(&action)?;

            return Ok(Some(ReferenceData {
                ref_type: "action".to_string(),
                name: action.name,
                source: Some(action.source),
                data,
                preview,
            }));
        }
        Ok(None)
    }

    /// Look up a class
    fn lookup_class(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<ReferenceData>> {
        let mut service = ClassService::new(conn);
        if let Some(class) = service.get_class_by_name_and_source(name, source)? {
            // Extract hit dice from the hd Value (can be object with "faces" field)
            let hd = class
                .hd
                .as_ref()
                .and_then(|v| v.get("faces").and_then(|f| f.as_u64()))
                .map(|faces| format!("d{}", faces))
                .unwrap_or_else(|| "d?".to_string());

            let preview = format!("Class • {} Hit Die", hd);

            let data = serde_json::to_value(&class)?;

            return Ok(Some(ReferenceData {
                ref_type: "class".to_string(),
                name: class.name,
                source: Some(class.source),
                data,
                preview,
            }));
        }
        Ok(None)
    }

    /// Look up a race
    fn lookup_race(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<ReferenceData>> {
        if let Some(race_json) = RaceService::get_race_details(conn, name, source)? {
            // Parse the JSON string into a Race struct
            let race: Race = serde_json::from_str(&race_json)?;

            // Extract speed from Value (can be number or object with "walk" field)
            let speed = race
                .speed
                .as_ref()
                .and_then(|v| {
                    v.as_u64()
                        .map(|n| format!("{} ft.", n))
                        .or_else(|| v.get("walk").and_then(|w| w.as_u64()).map(|n| format!("{} ft.", n)))
                })
                .unwrap_or_else(|| "30 ft.".to_string());

            let preview = format!("Race • Speed {}", speed);

            let data = serde_json::from_str(&race_json)?;

            return Ok(Some(ReferenceData {
                ref_type: "race".to_string(),
                name: race.name,
                source: Some(race.source),
                data,
                preview,
            }));
        }
        Ok(None)
    }

    /// Look up a feat
    fn lookup_feat(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<ReferenceData>> {
        let mut service = FeatService::new(conn);
        if let Some(feat) = service.get_feat_by_name_and_source(name, source)? {
            let preview = Self::extract_preview_from_entries(&feat.entries, "Feat");

            let data = serde_json::to_value(&feat)?;

            return Ok(Some(ReferenceData {
                ref_type: "feat".to_string(),
                name: feat.name,
                source: Some(feat.source),
                data,
                preview,
            }));
        }
        Ok(None)
    }

    /// Look up a background
    fn lookup_background(
        conn: &mut SqliteConnection,
        name: &str,
        source: &str,
    ) -> Result<Option<ReferenceData>> {
        let mut service = BackgroundService::new(conn);
        if let Some(background) = service.get_background_by_name_and_source(name, source)? {
            // Backgrounds don't have entries, show skill proficiencies instead
            let preview = format!("Background • {}", background.name);

            let data = serde_json::to_value(&background)?;

            return Ok(Some(ReferenceData {
                ref_type: "background".to_string(),
                name: background.name,
                source: Some(background.source),
                data,
                preview,
            }));
        }
        Ok(None)
    }

    /// Extract a preview string from a slice of typed Entry values
    fn extract_preview_from_entries(
        entries: &[crate::models::catalog::types::Entry],
        fallback: &str,
    ) -> String {
        use crate::models::catalog::types::Entry;

        entries
            .first()
            .and_then(|e| match e {
                Entry::Text(s) => Some(s.as_str()),
                Entry::Object(_) => None,
            })
            .map(|s| {
                if s.len() > 100 {
                    format!("{}...", &s[..100])
                } else {
                    s.to_string()
                }
            })
            .unwrap_or_else(|| fallback.to_string())
    }
}

//! Magic variant expansion for 5e-tools item data.
//!
//! This module handles the expansion of magic item variants from base items,
//! following the 5e-tools data format where base items can be combined with
//! magic variants to create specific magic items.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

/// A magic item variant definition from 5e-tools.
///
/// Magic variants define how to transform base items (like "Longsword") into
/// magic items (like "+1 Longsword"). They specify requirements for which base
/// items can be transformed, exclusions, and properties to inherit.
#[derive(Debug, Clone, Deserialize)]
pub struct MagicVariant {
    /// The name of the variant (e.g., "+1 Weapon").
    pub name: String,
    /// The type classification of the variant.
    #[serde(rename = "type")]
    pub variant_type: Option<String>,
    /// Requirements that base items must meet (OR'd together).
    pub requires: Option<Vec<Value>>,
    /// Conditions that exclude base items from this variant.
    pub excludes: Option<Value>,
    /// Properties inherited by the created magic item.
    pub inherits: Option<VariantInherits>,
    /// Additional entries/description for the magic item.
    pub entries: Option<Vec<Value>>,
    /// Whether this variant applies to ammunition.
    pub ammo: Option<bool>,
    /// Whether fluff images are available.
    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
}

/// Properties that a magic item inherits from its variant definition.
///
/// These properties define name transformations, stat bonuses, and other
/// attributes that are applied when creating a magic item from a base item.
#[derive(Debug, Clone, Deserialize)]
pub struct VariantInherits {
    /// Prefix to add to the base item name (e.g., "+1 ").
    #[serde(rename = "namePrefix")]
    pub name_prefix: Option<String>,
    /// Suffix to add to the base item name.
    #[serde(rename = "nameSuffix")]
    pub name_suffix: Option<String>,
    /// Text to remove from the base item name.
    #[serde(rename = "nameRemove")]
    pub name_remove: Option<String>,
    /// Properties to add to the item.
    #[serde(rename = "propertyAdd")]
    pub property_add: Option<Vec<Value>>,
    /// Properties to remove from the item.
    #[serde(rename = "propertyRemove")]
    pub property_remove: Option<Vec<String>>,
    /// Source book for the magic item.
    pub source: Option<String>,
    /// Page number in the source book.
    pub page: Option<u32>,
    /// Whether the item is in the SRD.
    pub srd: Option<bool>,
    /// Whether the item is in the basic rules.
    #[serde(rename = "basicRules")]
    pub basic_rules: Option<bool>,
    /// Magic item tier (minor, major).
    pub tier: Option<String>,
    /// Magic item rarity.
    pub rarity: Option<String>,
    /// Weapon attack and damage bonus (e.g., "+1").
    #[serde(rename = "bonusWeapon")]
    pub bonus_weapon: Option<String>,
    /// Weapon attack bonus only.
    #[serde(rename = "bonusWeaponAttack")]
    pub bonus_weapon_attack: Option<String>,
    /// Weapon damage bonus only.
    #[serde(rename = "bonusWeaponDamage")]
    pub bonus_weapon_damage: Option<String>,
    /// Armor class bonus.
    #[serde(rename = "bonusAc")]
    pub bonus_ac: Option<String>,
    /// Description entries for the magic item.
    pub entries: Option<Vec<Value>>,
    /// Loot tables where this item can appear.
    #[serde(rename = "lootTables")]
    pub loot_tables: Option<Vec<String>>,
}

/// Root structure for the magicvariants.json file.
#[derive(Debug, Clone, Deserialize)]
pub struct MagicVariantsJson {
    /// All magic variant definitions.
    pub magicvariant: Vec<MagicVariant>,
    /// Linked loot table data.
    #[serde(rename = "linkedLootTables")]
    pub linked_loot_tables: Option<Value>,
}

/// Load magic variants from magicvariants.json
pub fn load_magic_variants(data_dir: &std::path::Path) -> Result<Vec<MagicVariant>> {
    let variants_file = data_dir.join("magicvariants.json");
    if !variants_file.exists() {
        return Ok(Vec::new());
    }

    let content =
        std::fs::read_to_string(&variants_file).context("Failed to read magicvariants.json")?;

    let variants_json: MagicVariantsJson =
        serde_json::from_str(&content).context("Failed to parse magicvariants.json")?;

    Ok(variants_json.magicvariant)
}

/// Check if a base item matches the requirements of a magic variant
pub fn item_matches_variant(item: &Value, variant: &MagicVariant) -> bool {
    let Some(requires) = &variant.requires else {
        return false;
    };

    // Check if item meets ANY requirement (requirements are OR'd together)
    let meets_requirements = requires
        .iter()
        .any(|requirement| check_requirement(item, requirement));

    if !meets_requirements {
        return false;
    }

    // Check exclusions
    if let Some(excludes) = &variant.excludes {
        if check_requirement(item, excludes) {
            return false;
        }
    }

    true
}

/// Check if an item meets a specific requirement
/// For requirements: ALL properties in the requirement object must match (AND logic)
fn check_requirement(item: &Value, requirement: &Value) -> bool {
    if let Some(obj) = requirement.as_object() {
        // All properties in the requirement must match
        for (key, value) in obj {
            let item_value = item.get(key);

            // Handle different value types
            match value {
                Value::Bool(req_bool) => {
                    let item_bool = item_value.and_then(|v| v.as_bool()).unwrap_or(false);
                    if *req_bool != item_bool {
                        return false;
                    }
                }
                Value::String(req_str) => {
                    let item_str = item_value.and_then(|v| v.as_str()).unwrap_or("");
                    if req_str != item_str {
                        return false;
                    }
                }
                Value::Array(req_arr) => {
                    // For arrays, check if item value is in the requirement array
                    if let Some(item_val) = item_value {
                        if item_val.is_string() {
                            let item_str = item_val.as_str().unwrap_or("");
                            if !req_arr.iter().any(|v| v.as_str() == Some(item_str)) {
                                return false;
                            }
                        } else if item_val.is_array() {
                            // Check if any item in the item's array matches any in the requirement array
                            let item_arr = item_val.as_array().unwrap();
                            let has_match =
                                item_arr.iter().any(|iv| req_arr.iter().any(|rv| iv == rv));
                            if !has_match {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                Value::Object(_) => {
                    // Recursive check for nested objects
                    if let Some(item_obj) = item_value {
                        if !check_requirement(item_obj, value) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                _ => {
                    // For other types, just check equality
                    if item_value != Some(value) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Create a magic item by combining a base item with a magic variant
pub fn create_magic_item(base_item: &Value, variant: &MagicVariant) -> Result<Value> {
    let mut magic_item = base_item.clone();

    // Apply name transformation
    if let Some(inherits) = &variant.inherits {
        let base_name = base_item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Item");

        let mut new_name = base_name.to_string();

        // Apply nameRemove first (as 5etools does)
        if let Some(remove) = &inherits.name_remove {
            new_name = new_name.replace(remove, "");
        }

        if let Some(prefix) = &inherits.name_prefix {
            new_name = format!("{}{}", prefix, new_name);
        }

        if let Some(suffix) = &inherits.name_suffix {
            new_name = format!("{}{}", new_name, suffix);
        }

        magic_item["name"] = json!(new_name);

        // Add base item reference
        let base_source = base_item
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("PHB");
        magic_item["baseItem"] = json!(format!(
            "{}|{}",
            base_name.to_lowercase(),
            base_source.to_lowercase()
        ));

        // Apply inherited properties
        if let Some(source) = &inherits.source {
            magic_item["source"] = json!(source);
        }

        if let Some(page) = &inherits.page {
            magic_item["page"] = json!(page);
        }

        if let Some(srd) = &inherits.srd {
            magic_item["srd"] = json!(srd);
        }

        if let Some(basic_rules) = &inherits.basic_rules {
            magic_item["basicRules"] = json!(basic_rules);
        }

        if let Some(tier) = &inherits.tier {
            magic_item["tier"] = json!(tier);
        }

        if let Some(rarity) = &inherits.rarity {
            magic_item["rarity"] = json!(rarity);
        }

        if let Some(bonus_weapon) = &inherits.bonus_weapon {
            magic_item["bonusWeapon"] = json!(bonus_weapon);
        }

        if let Some(bonus_weapon_attack) = &inherits.bonus_weapon_attack {
            magic_item["bonusWeaponAttack"] = json!(bonus_weapon_attack);
        }

        if let Some(bonus_weapon_damage) = &inherits.bonus_weapon_damage {
            magic_item["bonusWeaponDamage"] = json!(bonus_weapon_damage);
        }

        if let Some(bonus_ac) = &inherits.bonus_ac {
            magic_item["bonusAc"] = json!(bonus_ac);
        }

        if let Some(loot_tables) = &inherits.loot_tables {
            magic_item["lootTables"] = json!(loot_tables);
        }

        // Handle property modifications
        if let Some(property_add) = &inherits.property_add {
            let existing_props = magic_item
                .get("property")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let mut new_props = existing_props;
            for prop_to_add in property_add {
                // Only add if not already present
                if !new_props.iter().any(|p| p == prop_to_add) {
                    new_props.push(prop_to_add.clone());
                }
            }
            magic_item["property"] = json!(new_props);
        }

        if let Some(property_remove) = &inherits.property_remove {
            if let Some(existing_props) = magic_item.get("property").and_then(|v| v.as_array()) {
                let new_props: Vec<_> = existing_props
                    .iter()
                    .filter(|p| {
                        let prop_str = p.as_str().unwrap_or("");
                        !property_remove.contains(&prop_str.to_string())
                    })
                    .cloned()
                    .collect();

                if new_props.is_empty() {
                    magic_item.as_object_mut().unwrap().remove("property");
                } else {
                    magic_item["property"] = json!(new_props);
                }
            }
        }

        // Apply entries with template variable replacement
        // For specific variants, always use inherits.entries with template processing
        if let Some(entries) = &inherits.entries {
            let processed_entries = process_entries_templates(entries, &magic_item);
            magic_item["entries"] = json!(processed_entries);
        }
    }

    // Also check if the variant itself has entries that need processing
    // This handles cases where entries are defined at the variant level
    if variant.entries.is_some() && magic_item.get("entries").is_none() {
        if let Some(entries) = &variant.entries {
            let processed_entries = process_entries_templates(entries, &magic_item);
            magic_item["entries"] = json!(processed_entries);
        }
    }

    // If entries already exist but might have unprocessed templates, process them
    if let Some(existing_entries) = magic_item.get("entries").cloned() {
        if let Some(entries_array) = existing_entries.as_array() {
            let processed_entries = process_entries_templates(entries_array, &magic_item);
            magic_item["entries"] = json!(processed_entries);
        }
    }

    Ok(magic_item)
}

/// Get the appropriate article ("a" or "an") for a word
fn get_article(word: &str) -> &'static str {
    let word_lower = word.to_lowercase();
    let first_char = word_lower.chars().next().unwrap_or(' ');

    // Check for vowel sounds (simplified English rules)
    match first_char {
        'a' | 'e' | 'i' | 'o' | 'u' => "an",
        _ => "a",
    }
}

/// Process template variables in entry text  
pub fn process_entries_templates(entries: &[Value], item: &Value) -> Vec<Value> {
    entries
        .iter()
        .map(|entry| {
            match entry {
                Value::String(s) => {
                    let mut processed = s.clone();

                    // Replace {=bonusWeapon} with the actual bonus
                    if let Some(bonus) = item.get("bonusWeapon").and_then(|v| v.as_str()) {
                        processed = processed.replace("{=bonusWeapon}", bonus);
                    }

                    // Replace {=bonusWeaponAttack} with the actual bonus
                    if let Some(bonus) = item.get("bonusWeaponAttack").and_then(|v| v.as_str()) {
                        processed = processed.replace("{=bonusWeaponAttack}", bonus);
                    }

                    // Replace {=bonusAc} with the actual bonus
                    if let Some(bonus) = item.get("bonusAc").and_then(|v| v.as_str()) {
                        processed = processed.replace("{=bonusAc}", bonus);
                    }

                    // Replace {=bonusWeaponDamage} with the actual bonus
                    if let Some(bonus) = item.get("bonusWeaponDamage").and_then(|v| v.as_str()) {
                        processed = processed.replace("{=bonusWeaponDamage}", bonus);
                    }

                    // Replace base name templates
                    if let Some(base_item) = item.get("baseItem").and_then(|v| v.as_str()) {
                        // Extract the base name (before the pipe)
                        let base_name = base_item.split('|').next().unwrap_or(base_item);

                        // Clean up common suffixes like "(20)" from "arrows (20)"
                        let clean_name = if let Some(paren_pos) = base_name.find(" (") {
                            &base_name[..paren_pos]
                        } else {
                            base_name
                        };

                        // {=baseName/l} - lowercase base name
                        processed = processed.replace("{=baseName/l}", clean_name);

                        // {=baseName/a} - Just the article "a" or "an" (lowercase), without the name
                        // This is used in patterns like "{=baseName/a} {=baseName/l}" to get "an arrow"
                        let article = get_article(clean_name);
                        processed = processed.replace("{=baseName/a}", article);

                        // {=baseName/at} - Just the article "A" or "An" (capitalized), without the name
                        // This is used in patterns like "{=baseName/at} {=baseName/l}" to get "An arrow"
                        let cap_article = if article == "a" { "A" } else { "An" };
                        processed = processed.replace("{=baseName/at}", cap_article);
                    }

                    // Replace {=dmgType} with the damage type name
                    if let Some(dmg_type) = item.get("dmgType").and_then(|v| v.as_str()) {
                        let dmg_name = match dmg_type {
                            "P" => "piercing".to_string(),
                            "B" => "bludgeoning".to_string(),
                            "S" => "slashing".to_string(),
                            "A" => "acid".to_string(),
                            "C" => "cold".to_string(),
                            "F" => "fire".to_string(),
                            "L" => "lightning".to_string(),
                            "N" => "necrotic".to_string(),
                            "T" => "thunder".to_string(),
                            _ => dmg_type.to_lowercase(),
                        };
                        processed = processed.replace("{=dmgType}", &dmg_name);
                    }

                    // Note: {#itemEntry} references need to be handled at a higher level
                    // where we have access to the full items collection
                    // For now, we'll leave them as-is and handle them separately

                    Value::String(processed)
                }
                // For non-string entries (objects), return as-is
                // Could recursively process if needed
                other => other.clone(),
            }
        })
        .collect()
}

/// Expand all magic variants for a list of base items
pub fn expand_magic_variants(
    base_items: &[Value],
    variants: &[MagicVariant],
) -> Result<Vec<Value>> {
    let mut expanded_items = Vec::new();

    for variant in variants {
        for base_item in base_items {
            if item_matches_variant(base_item, variant) {
                match create_magic_item(base_item, variant) {
                    Ok(magic_item) => expanded_items.push(magic_item),
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to create magic item for variant '{}': {}",
                            variant.name, e
                        );
                    }
                }
            }
        }
    }

    Ok(expanded_items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_requirement_matching() {
        let weapon_item = json!({
            "name": "Longsword",
            "weapon": true,
            "type": "M"
        });

        let weapon_variant = MagicVariant {
            name: "+1 Weapon".to_string(),
            variant_type: None,
            requires: Some(vec![json!({"weapon": true})]),
            excludes: None,
            inherits: None,
            entries: None,
            ammo: None,
            has_fluff_images: None,
        };

        assert!(item_matches_variant(&weapon_item, &weapon_variant));

        let non_weapon = json!({
            "name": "Rope",
            "weapon": false
        });

        assert!(!item_matches_variant(&non_weapon, &weapon_variant));
    }

    #[test]
    fn test_exclusion_matching() {
        let net_item = json!({
            "name": "Net",
            "weapon": true,
            "net": true
        });

        let weapon_variant = MagicVariant {
            name: "+1 Weapon".to_string(),
            variant_type: None,
            requires: Some(vec![json!({"weapon": true})]),
            excludes: Some(json!({"net": true})),
            inherits: None,
            entries: None,
            ammo: None,
            has_fluff_images: None,
        };

        assert!(!item_matches_variant(&net_item, &weapon_variant));
    }
}

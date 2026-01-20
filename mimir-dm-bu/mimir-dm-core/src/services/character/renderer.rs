//! Character sheet markdown renderer
//!
//! Generates human-readable markdown character sheets from CharacterData.

use crate::models::catalog::{Item, Spell};
use crate::models::character::CharacterData;
use std::collections::HashMap;

/// Trait for rendering character sheets in various formats.
pub trait CharacterRenderer {
    /// Renders a basic character sheet.
    fn render(&self, character: &CharacterData) -> String;
    /// Renders a character sheet with spell details.
    fn render_with_spells(
        &self,
        character: &CharacterData,
        spell_details: &HashMap<String, Spell>,
    ) -> String;
    /// Renders a character sheet with spell and item details.
    fn render_with_details(
        &self,
        character: &CharacterData,
        spell_details: &HashMap<String, Spell>,
        item_details: &HashMap<String, Item>,
    ) -> String;
}

/// Markdown renderer for character sheets
pub struct MarkdownRenderer;

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownRenderer {
    /// Creates a new markdown renderer.
    pub fn new() -> Self {
        Self
    }

    fn render_header(&self, character: &CharacterData) -> String {
        format!(
            "# {} - Level {} {}\n\n",
            character.character_name,
            character.level,
            character.class_string()
        )
    }

    fn render_metadata(&self, character: &CharacterData) -> String {
        let mut output = String::new();

        output.push_str(&format!("**Race:** {}", character.race));
        if let Some(subrace) = &character.subrace {
            output.push_str(&format!(" ({})", subrace));
        }
        output.push_str("  \n");

        output.push_str(&format!("**Background:** {}  \n", character.background));

        if let Some(alignment) = &character.alignment {
            output.push_str(&format!("**Alignment:** {}  \n", alignment));
        }

        output.push_str(&format!(
            "**Experience:** {} XP  \n",
            character.experience_points
        ));

        if let Some(reason) = &character.snapshot_reason {
            output.push_str(&format!(
                "**Version:** {} ({})  \n",
                character.version, reason
            ));
        }

        output.push_str(&format!("**Created:** {}\n\n", character.created_at));

        output
    }

    fn render_ability_scores(&self, character: &CharacterData) -> String {
        let mut output = String::from("## Ability Scores\n\n");
        output.push_str("| STR | DEX | CON | INT | WIS | CHA |\n");
        output.push_str("|-----|-----|-----|-----|-----|-----|\n");

        let abilities = &character.abilities;
        output.push_str(&format!(
            "| {} ({:+}) | {} ({:+}) | {} ({:+}) | {} ({:+}) | {} ({:+}) | {} ({:+}) |\n\n",
            abilities.strength,
            abilities.str_modifier(),
            abilities.dexterity,
            abilities.dex_modifier(),
            abilities.constitution,
            abilities.con_modifier(),
            abilities.intelligence,
            abilities.int_modifier(),
            abilities.wisdom,
            abilities.wis_modifier(),
            abilities.charisma,
            abilities.cha_modifier()
        ));

        output
    }

    fn render_combat_stats(&self, character: &CharacterData) -> String {
        let mut output = String::from("## Combat Stats\n\n");

        let dex_mod = character.abilities.dex_modifier();
        let wis_mod = character.abilities.wis_modifier();
        let prof_bonus = character.proficiency_bonus();

        // Calculate AC (base 10 + DEX, note armor if equipped)
        let base_ac = 10 + dex_mod;
        let shield_bonus = if character.equipped.shield.is_some() {
            2
        } else {
            0
        };
        let ac = base_ac + shield_bonus;

        // Calculate Passive Perception
        let perception_prof = character
            .proficiencies
            .skills
            .iter()
            .any(|s| s.to_lowercase() == "perception");
        let passive_perception = 10 + wis_mod + if perception_prof { prof_bonus } else { 0 };

        // Core combat stats in a compact format
        output.push_str("| AC | Initiative | Speed | Passive Perception |\n");
        output.push_str("|:--:|:----------:|:-----:|:------------------:|\n");
        output.push_str(&format!(
            "| {} | {:+} | 30 ft | {} |\n\n",
            ac, dex_mod, passive_perception
        ));

        // Note armor if equipped
        if let Some(armor) = &character.equipped.armor {
            output.push_str(&format!("*Armor: {}*\n\n", armor));
        }

        // HP and Hit Dice
        output.push_str(&format!(
            "**HP:** {} / {}\n",
            character.current_hp, character.max_hp
        ));

        // Render hit dice for each class
        let hit_dice_str = character
            .classes
            .iter()
            .map(|c| format!("{}{}", c.hit_dice_remaining, c.hit_dice_type))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("**Hit Dice:** {}\n", hit_dice_str));
        output.push_str(&format!("**Proficiency Bonus:** +{}\n\n", prof_bonus));

        output
    }

    fn render_attacks(&self, character: &CharacterData) -> String {
        let equipped = &character.equipped;

        // Only render if character has weapons equipped
        if equipped.main_hand.is_none() && equipped.off_hand.is_none() {
            return String::new();
        }

        let mut output = String::from("## Attacks\n\n");
        let prof_bonus = character.proficiency_bonus();
        let str_mod = character.abilities.str_modifier();
        let dex_mod = character.abilities.dex_modifier();

        output.push_str("| Name | Attack Bonus | Damage/Type |\n");
        output.push_str("|------|:------------:|-------------|\n");

        // Main hand weapon
        if let Some(weapon) = &equipped.main_hand {
            let weapon_lower = weapon.to_lowercase();

            // Determine if finesse or ranged (use DEX), otherwise STR
            let is_finesse_or_ranged = weapon_lower.contains("rapier")
                || weapon_lower.contains("dagger")
                || weapon_lower.contains("shortsword")
                || weapon_lower.contains("scimitar")
                || weapon_lower.contains("whip")
                || weapon_lower.contains("bow")
                || weapon_lower.contains("crossbow")
                || weapon_lower.contains("dart")
                || weapon_lower.contains("sling");

            // Ranged weapons always use DEX; finesse weapons use better of STR/DEX
            let is_ranged = weapon_lower.contains("bow")
                || weapon_lower.contains("crossbow")
                || weapon_lower.contains("dart")
                || weapon_lower.contains("sling");
            let ability_mod = if is_ranged || (is_finesse_or_ranged && dex_mod > str_mod) {
                dex_mod
            } else {
                str_mod
            };

            let attack_bonus = prof_bonus + ability_mod;

            // Estimate damage based on common weapons
            let damage = match weapon_lower.as_str() {
                w if w.contains("greatsword") || w.contains("maul") => {
                    format!("2d6{:+}", ability_mod)
                }
                w if w.contains("greataxe") => format!("1d12{:+}", ability_mod),
                w if w.contains("longsword")
                    || w.contains("warhammer")
                    || w.contains("battleaxe") =>
                {
                    format!("1d8{:+}", ability_mod)
                }
                w if w.contains("rapier") => format!("1d8{:+}", ability_mod),
                w if w.contains("shortsword") || w.contains("scimitar") => {
                    format!("1d6{:+}", ability_mod)
                }
                w if w.contains("dagger") => format!("1d4{:+}", ability_mod),
                w if w.contains("quarterstaff") || w.contains("spear") => {
                    format!("1d6{:+}", ability_mod)
                }
                w if w.contains("longbow") => format!("1d8{:+}", ability_mod),
                w if w.contains("shortbow") || w.contains("light crossbow") => {
                    format!("1d6{:+}", ability_mod)
                }
                w if w.contains("heavy crossbow") => format!("1d10{:+}", ability_mod),
                w if w.contains("handaxe") || w.contains("javelin") || w.contains("mace") => {
                    format!("1d6{:+}", ability_mod)
                }
                _ => format!("1d6{:+}", ability_mod), // Default
            };

            output.push_str(&format!(
                "| {} | {:+} | {} |\n",
                weapon, attack_bonus, damage
            ));
        }

        // Off hand weapon (if different from shield)
        if let Some(weapon) = &equipped.off_hand {
            let weapon_lower = weapon.to_lowercase();
            if !weapon_lower.contains("shield") {
                let _ability_mod = str_mod; // Off-hand typically doesn't add ability to damage
                let attack_bonus = prof_bonus + str_mod;

                let damage = match weapon_lower.as_str() {
                    w if w.contains("dagger") => "1d4".to_string(),
                    w if w.contains("shortsword") => "1d6".to_string(),
                    w if w.contains("handaxe") => "1d6".to_string(),
                    _ => "1d6".to_string(),
                };

                output.push_str(&format!(
                    "| {} | {:+} | {} |\n",
                    weapon, attack_bonus, damage
                ));
            }
        }

        output.push('\n');
        output
    }

    fn render_saving_throws(&self, character: &CharacterData) -> String {
        let mut output = String::from("## Saving Throws\n\n");
        let abilities = &character.abilities;
        let prof_bonus = character.proficiency_bonus();
        let saves = &character.proficiencies.saves;

        let save_data = [
            (
                "STR",
                abilities.str_modifier(),
                saves.iter().any(|s| s.to_lowercase().contains("str")),
            ),
            (
                "DEX",
                abilities.dex_modifier(),
                saves.iter().any(|s| s.to_lowercase().contains("dex")),
            ),
            (
                "CON",
                abilities.con_modifier(),
                saves.iter().any(|s| s.to_lowercase().contains("con")),
            ),
            (
                "INT",
                abilities.int_modifier(),
                saves.iter().any(|s| s.to_lowercase().contains("int")),
            ),
            (
                "WIS",
                abilities.wis_modifier(),
                saves.iter().any(|s| s.to_lowercase().contains("wis")),
            ),
            (
                "CHA",
                abilities.cha_modifier(),
                saves.iter().any(|s| s.to_lowercase().contains("cha")),
            ),
        ];

        output.push_str("| Save | Mod | Prof |\n");
        output.push_str("|------|----:|:----:|\n");

        for (name, modifier, is_prof) in save_data {
            let total = modifier + if is_prof { prof_bonus } else { 0 };
            let prof_marker = if is_prof { "●" } else { "○" };
            output.push_str(&format!("| {} | {:+} | {} |\n", name, total, prof_marker));
        }

        output.push('\n');
        output
    }

    fn render_skills(&self, character: &CharacterData) -> String {
        let mut output = String::from("## Skills\n\n");
        let abilities = &character.abilities;
        let prof_bonus = character.proficiency_bonus();
        let skill_profs = &character.proficiencies.skills;

        // All 18 skills with their ability
        let skills = [
            ("Acrobatics", "DEX", abilities.dex_modifier()),
            ("Animal Handling", "WIS", abilities.wis_modifier()),
            ("Arcana", "INT", abilities.int_modifier()),
            ("Athletics", "STR", abilities.str_modifier()),
            ("Deception", "CHA", abilities.cha_modifier()),
            ("History", "INT", abilities.int_modifier()),
            ("Insight", "WIS", abilities.wis_modifier()),
            ("Intimidation", "CHA", abilities.cha_modifier()),
            ("Investigation", "INT", abilities.int_modifier()),
            ("Medicine", "WIS", abilities.wis_modifier()),
            ("Nature", "INT", abilities.int_modifier()),
            ("Perception", "WIS", abilities.wis_modifier()),
            ("Performance", "CHA", abilities.cha_modifier()),
            ("Persuasion", "CHA", abilities.cha_modifier()),
            ("Religion", "INT", abilities.int_modifier()),
            ("Sleight of Hand", "DEX", abilities.dex_modifier()),
            ("Stealth", "DEX", abilities.dex_modifier()),
            ("Survival", "WIS", abilities.wis_modifier()),
        ];

        output.push_str("| Skill | Mod | Prof |\n");
        output.push_str("|-------|----:|:----:|\n");

        for (name, _ability, modifier) in skills {
            let is_prof = skill_profs
                .iter()
                .any(|s| s.to_lowercase() == name.to_lowercase());
            let total = modifier + if is_prof { prof_bonus } else { 0 };
            let prof_marker = if is_prof { "●" } else { "○" };
            output.push_str(&format!("| {} | {:+} | {} |\n", name, total, prof_marker));
        }

        output.push('\n');
        output
    }

    fn render_currency(&self, character: &CharacterData) -> String {
        let currency = &character.currency;

        // Only render if character has any currency
        if currency.copper == 0
            && currency.silver == 0
            && currency.gold == 0
            && currency.platinum == 0
        {
            return String::new();
        }

        let mut output = String::from("## Currency\n\n");
        output.push_str("| PP | GP | SP | CP |\n");
        output.push_str("|---:|---:|---:|---:|\n");
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n\n",
            currency.platinum, currency.gold, currency.silver, currency.copper
        ));

        output
    }

    fn render_proficiencies(&self, character: &CharacterData) -> String {
        let prof = &character.proficiencies;

        // Check if any proficiencies to render (excluding skills/saves which have their own sections)
        if prof.armor.is_empty()
            && prof.weapons.is_empty()
            && prof.tools.is_empty()
            && prof.languages.is_empty()
        {
            return String::new();
        }

        let mut output = String::from("## Other Proficiencies & Languages\n\n");

        if !prof.armor.is_empty() {
            output.push_str(&format!("**Armor:** {}  \n", prof.armor.join(", ")));
        }

        if !prof.weapons.is_empty() {
            output.push_str(&format!("**Weapons:** {}  \n", prof.weapons.join(", ")));
        }

        if !prof.tools.is_empty() {
            output.push_str(&format!("**Tools:** {}  \n", prof.tools.join(", ")));
        }

        if !prof.languages.is_empty() {
            output.push_str(&format!("**Languages:** {}  \n", prof.languages.join(", ")));
        }

        output.push('\n');
        output
    }

    fn render_class_features(&self, character: &CharacterData) -> String {
        if character.class_features.is_empty() {
            return String::new();
        }

        let mut output = String::from("## Class Features\n\n");
        for feature in &character.class_features {
            output.push_str(&format!("- {}\n", feature));
        }
        output.push('\n');

        output
    }

    fn render_feats(&self, character: &CharacterData) -> String {
        if character.feats.is_empty() {
            return String::new();
        }

        let mut output = String::from("## Feats\n\n");
        for feat in &character.feats {
            output.push_str(&format!("- {}\n", feat));
        }
        output.push('\n');

        output
    }

    fn render_spells(
        &self,
        character: &CharacterData,
        spell_details: &HashMap<String, Spell>,
    ) -> String {
        let spells = &character.spells;

        // Only render if character has any spells
        if spells.cantrips.is_empty()
            && spells.known_spells.is_empty()
            && spells.spell_slots.is_empty()
        {
            return String::new();
        }

        let mut output = String::from("## Spells\n\n");

        // Spell slots
        if !spells.spell_slots.is_empty() {
            output.push_str("**Spell Slots:**\n");
            let mut levels: Vec<_> = spells.spell_slots.keys().collect();
            levels.sort();

            for level in levels {
                if let Some(slots) = spells.spell_slots.get(level) {
                    output.push_str(&format!(
                        "- Level {}: {} / {}\n",
                        level, slots.current, slots.max
                    ));
                }
            }
            output.push('\n');
        }

        // Cantrips with full details
        if !spells.cantrips.is_empty() {
            output.push_str("### Cantrips\n\n");
            for spell_ref in &spells.cantrips {
                if let Some(spell) = spell_details.get(&spell_ref.name) {
                    output.push_str(&self.render_spell_detail(spell));
                } else {
                    output.push_str(&format!("**{}** ({})\n\n", spell_ref.name, spell_ref.source));
                }
            }
        }

        // Known spells with full details, grouped by level
        if !spells.known_spells.is_empty() {
            output.push_str("### Known Spells\n\n");

            // Group spells by level
            let mut spells_by_level: HashMap<u8, Vec<&crate::models::character::SpellReference>> = HashMap::new();
            for spell_ref in &spells.known_spells {
                let level = spell_details.get(&spell_ref.name).map(|s| s.level).unwrap_or(1);
                spells_by_level.entry(level).or_default().push(spell_ref);
            }

            // Sort and output by level
            let mut levels: Vec<_> = spells_by_level.keys().collect();
            levels.sort();

            for level in levels {
                output.push_str(&format!("#### {} Level\n\n", Self::ordinal(*level)));
                if let Some(spell_refs) = spells_by_level.get(level) {
                    for spell_ref in spell_refs {
                        if let Some(spell) = spell_details.get(&spell_ref.name) {
                            output.push_str(&self.render_spell_detail(spell));
                        } else {
                            output.push_str(&format!("**{}** ({})\n\n", spell_ref.name, spell_ref.source));
                        }
                    }
                }
            }
        }

        // Prepared spells (just list names since details are above)
        if !spells.prepared_spells.is_empty() {
            output.push_str("### Prepared Spells\n\n");
            let spell_names: Vec<_> = spells.prepared_spells.iter().map(|s| s.name.as_str()).collect();
            output.push_str(&format!("{}  \n\n", spell_names.join(", ")));
        }

        output
    }

    fn render_spell_detail(&self, spell: &Spell) -> String {
        let mut output = String::new();

        // Spell name and level/school
        let level_str = if spell.level == 0 {
            format!("{} cantrip", spell.school.as_str())
        } else {
            format!(
                "{} level {}",
                Self::ordinal(spell.level),
                spell.school.as_str().to_lowercase()
            )
        };
        output.push_str(&format!("**{}**  \n", spell.name));
        output.push_str(&format!("*{}*\n\n", level_str));

        // Casting time
        let casting_time = spell
            .time
            .first()
            .map(|t| {
                let base = format!("{} {}", t.number, t.unit);
                if let Some(condition) = &t.condition {
                    format!("{} ({})", base, condition)
                } else {
                    base
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());
        output.push_str(&format!("**Casting Time:** {}  \n", casting_time));

        // Range
        let range = match &spell.range {
            crate::models::catalog::SpellRange::Point { distance, .. } => {
                if let Some(amount) = distance.amount {
                    format!("{} {}", amount, distance.distance_type)
                } else {
                    distance.distance_type.clone()
                }
            }
            crate::models::catalog::SpellRange::Special { range_type } => range_type.clone(),
        };
        output.push_str(&format!("**Range:** {}  \n", range));

        // Components
        let mut comp_parts = Vec::new();
        if spell.components.v.unwrap_or(false) {
            comp_parts.push("V".to_string());
        }
        if spell.components.s.unwrap_or(false) {
            comp_parts.push("S".to_string());
        }
        if let Some(material) = &spell.components.m {
            let material_text = match material {
                crate::models::catalog::MaterialComponent::Text(text) => format!("M ({})", text),
                crate::models::catalog::MaterialComponent::Object { text, cost, .. } => {
                    if let Some(cost) = cost {
                        format!("M ({}, {} gp)", text, cost)
                    } else {
                        format!("M ({})", text)
                    }
                }
                crate::models::catalog::MaterialComponent::Bool(_) => "M".to_string(),
            };
            comp_parts.push(material_text);
        }
        output.push_str(&format!("**Components:** {}  \n", comp_parts.join(", ")));

        // Duration
        let duration = spell
            .duration
            .first()
            .map(|d| {
                let mut dur_str = String::new();
                if d.concentration.unwrap_or(false) {
                    dur_str.push_str("Concentration, up to ");
                }
                if let Some(value) = &d.duration {
                    if let Some(amount) = value.amount {
                        dur_str.push_str(&format!("{} {}", amount, value.value_type));
                    } else {
                        dur_str.push_str(&value.value_type);
                    }
                } else {
                    dur_str.push_str(&d.duration_type);
                }
                dur_str
            })
            .unwrap_or_else(|| "Instantaneous".to_string());
        output.push_str(&format!("**Duration:** {}  \n\n", duration));

        // Description
        for entry in &spell.entries {
            use crate::models::catalog::types::{Entry, EntryObject};
            match entry {
                Entry::Text(text) => {
                    output.push_str(text);
                    output.push_str("\n\n");
                }
                Entry::Object(obj) => {
                    // Handle structured entries like lists
                    match obj {
                        EntryObject::List { items, .. } => {
                            for item in items {
                                if let Entry::Text(text) = item {
                                    output.push_str(&format!("- {}\n", text));
                                }
                            }
                            output.push('\n');
                        }
                        EntryObject::Entries { entries, .. } => {
                            for e in entries {
                                if let Entry::Text(text) = e {
                                    output.push_str(text);
                                    output.push_str("\n\n");
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        output.push_str("---\n\n");
        output
    }

    fn ordinal(n: u8) -> String {
        match n {
            0 => "Cantrip".to_string(),
            1 => "1st".to_string(),
            2 => "2nd".to_string(),
            3 => "3rd".to_string(),
            _ => format!("{}th", n),
        }
    }

    fn render_equipment(&self, character: &CharacterData) -> String {
        let equipped = &character.equipped;

        // Check if any equipment is present
        if equipped.armor.is_none()
            && equipped.shield.is_none()
            && equipped.main_hand.is_none()
            && equipped.off_hand.is_none()
        {
            return String::new();
        }

        let mut output = String::from("## Equipment\n\n");

        if let Some(armor) = &equipped.armor {
            output.push_str(&format!("- **Armor:** {}\n", armor));
        }

        if let Some(shield) = &equipped.shield {
            output.push_str(&format!("- **Shield:** {}\n", shield));
        }

        if let Some(main_hand) = &equipped.main_hand {
            output.push_str(&format!("- **Main Hand:** {}\n", main_hand));
        }

        if let Some(off_hand) = &equipped.off_hand {
            output.push_str(&format!("- **Off Hand:** {}\n", off_hand));
        }

        output.push('\n');
        output
    }

    fn render_inventory_with_details(
        &self,
        character: &CharacterData,
        item_details: &HashMap<String, Item>,
    ) -> String {
        if character.inventory.is_empty() {
            return String::new();
        }

        let mut output = String::from("## Inventory\n\n");

        for item in &character.inventory {
            // Item header with quantity
            if item.quantity > 1 {
                output.push_str(&format!("### {} (x{})\n\n", item.name, item.quantity));
            } else {
                output.push_str(&format!("### {}\n\n", item.name));
            }

            // Custom notes (flavor text) - displayed prominently
            if let Some(notes) = &item.notes {
                output.push_str(&format!("> **Notes:** {}\n\n", notes));
            }

            // Get item details from catalog
            let key = format!("{}:{}", item.name, item.source.as_deref().unwrap_or("PHB"));
            if let Some(details) = item_details.get(&key) {
                // Item type and rarity
                let mut meta = Vec::new();
                if let Some(item_type) = &details.item_type {
                    meta.push(item_type.clone());
                }
                if let Some(rarity) = &details.rarity {
                    meta.push(rarity.clone());
                }
                if !meta.is_empty() {
                    output.push_str(&format!("*{}*\n\n", meta.join(", ")));
                }

                // Stats
                let mut stats = Vec::new();
                if let Some(ac) = details.ac {
                    stats.push(format!("**AC:** {}", ac));
                }
                if let Some(dmg) = &details.dmg1 {
                    let dmg_str = if let Some(dmg_type) = &details.dmg_type {
                        format!("{} {}", dmg, dmg_type)
                    } else {
                        dmg.clone()
                    };
                    stats.push(format!("**Damage:** {}", dmg_str));
                }
                if let Some(range) = &details.range {
                    stats.push(format!("**Range:** {}", range));
                }
                if let Some(weight) = details.weight {
                    stats.push(format!("**Weight:** {} lb", weight));
                }
                if let Some(value) = details.value {
                    stats.push(format!("**Value:** {} gp", value));
                }
                if !stats.is_empty() {
                    output.push_str(&format!("{}\n\n", stats.join(" | ")));
                }

                // Properties
                if let Some(props) = &details.property {
                    if !props.is_empty() {
                        output.push_str(&format!("**Properties:** {}\n\n", props.join(", ")));
                    }
                }

                // Description entries
                if !details.entries.is_empty() {
                    for entry in &details.entries {
                        output.push_str(&format!("{}\n\n", self.format_entry_typed(entry)));
                    }
                }
            } else {
                // Fallback to basic info if no details found
                output.push_str(&format!(
                    "**Weight:** {:.1} lbs | **Value:** {:.1} gp\n\n",
                    item.weight, item.value
                ));
            }
        }

        output
    }

    /// Format a typed Entry value to a string
    fn format_entry_typed(&self, entry: &crate::models::catalog::types::Entry) -> String {
        use crate::models::catalog::types::{Entry, EntryObject};
        match entry {
            Entry::Text(s) => s.clone(),
            Entry::Object(obj) => match obj {
                EntryObject::Entries { entries, .. } => entries
                    .iter()
                    .map(|e| self.format_entry_typed(e))
                    .collect::<Vec<_>>()
                    .join("\n"),
                EntryObject::List { items, .. } => items
                    .iter()
                    .map(|i| format!("- {}", self.format_entry_typed(i)))
                    .collect::<Vec<_>>()
                    .join("\n"),
                EntryObject::Item { name, entries, .. } => {
                    if let Some(entries) = entries {
                        format!(
                            "**{}:** {}",
                            name,
                            entries
                                .iter()
                                .map(|e| self.format_entry_typed(e))
                                .collect::<Vec<_>>()
                                .join(" ")
                        )
                    } else {
                        format!("**{}**", name)
                    }
                }
                _ => String::new(),
            },
        }
    }

    fn render_personality(&self, character: &CharacterData) -> String {
        let personality = &character.personality;

        // Check if any personality traits are present
        if personality.traits.is_none()
            && personality.ideals.is_none()
            && personality.bonds.is_none()
            && personality.flaws.is_none()
        {
            return String::new();
        }

        let mut output = String::from("## Personality\n\n");

        if let Some(traits) = &personality.traits {
            output.push_str(&format!("**Traits:** {}  \n", traits));
        }

        if let Some(ideals) = &personality.ideals {
            output.push_str(&format!("**Ideals:** {}  \n", ideals));
        }

        if let Some(bonds) = &personality.bonds {
            output.push_str(&format!("**Bonds:** {}  \n", bonds));
        }

        if let Some(flaws) = &personality.flaws {
            output.push_str(&format!("**Flaws:** {}  \n", flaws));
        }

        output.push('\n');
        output
    }

    /// Renders NPC-specific information
    fn render_npc_info(&self, character: &CharacterData) -> String {
        // Only render if character has any NPC fields set
        if character.npc_role.is_none()
            && character.npc_location.is_none()
            && character.npc_faction.is_none()
            && character.npc_notes.is_none()
        {
            return String::new();
        }

        let mut output = String::from("## NPC Information\n\n");

        if let Some(role) = &character.npc_role {
            output.push_str(&format!("**Role:** {}  \n", role));
        }

        if let Some(location) = &character.npc_location {
            output.push_str(&format!("**Location:** {}  \n", location));
        }

        if let Some(faction) = &character.npc_faction {
            output.push_str(&format!("**Faction:** {}  \n", faction));
        }

        if let Some(notes) = &character.npc_notes {
            output.push_str(&format!("\n**Notes:**  \n{}  \n", notes));
        }

        output.push('\n');
        output
    }

    /// Renders legendary actions section for boss NPCs
    fn render_legendary_actions(&self, character: &CharacterData) -> String {
        if character.legendary_actions.is_empty() {
            return String::new();
        }

        let mut output = String::from("## Legendary Actions\n\n");

        let action_count = character.legendary_action_count.unwrap_or(3);
        output.push_str(&format!(
            "{} can take {} legendary action{}, choosing from the options below. \
             Only one legendary action can be used at a time and only at the end of \
             another creature's turn.\n\n",
            character.character_name,
            action_count,
            if action_count == 1 { "" } else { "s" }
        ));

        for action in &character.legendary_actions {
            output.push_str(&format!("**{}**", action.name));
            if action.cost > 1 {
                output.push_str(&format!(" (Costs {} Actions)", action.cost));
            }
            output.push_str(&format!(". {}  \n", action.description));
        }

        output.push('\n');
        output
    }
}

impl CharacterRenderer for MarkdownRenderer {
    fn render(&self, character: &CharacterData) -> String {
        // Use empty spell and item details for backward compatibility
        self.render_with_details(character, &HashMap::new(), &HashMap::new())
    }

    fn render_with_spells(
        &self,
        character: &CharacterData,
        spell_details: &HashMap<String, Spell>,
    ) -> String {
        self.render_with_details(character, spell_details, &HashMap::new())
    }

    fn render_with_details(
        &self,
        character: &CharacterData,
        spell_details: &HashMap<String, Spell>,
        item_details: &HashMap<String, Item>,
    ) -> String {
        let mut output = String::new();

        output.push_str(&self.render_header(character));
        output.push_str(&self.render_metadata(character));
        // Render NPC info after metadata if character has NPC fields
        output.push_str(&self.render_npc_info(character));
        output.push_str(&self.render_legendary_actions(character));
        output.push_str(&self.render_ability_scores(character));
        output.push_str(&self.render_combat_stats(character));
        output.push_str(&self.render_attacks(character));
        output.push_str(&self.render_saving_throws(character));
        output.push_str(&self.render_skills(character));
        output.push_str(&self.render_proficiencies(character));
        output.push_str(&self.render_class_features(character));
        output.push_str(&self.render_feats(character));
        output.push_str(&self.render_spells(character, spell_details));
        output.push_str(&self.render_equipment(character));
        output.push_str(&self.render_inventory_with_details(character, item_details));
        output.push_str(&self.render_currency(character));
        output.push_str(&self.render_personality(character));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::character::data::{ClassLevel, Currency, LegendaryAction};
    use crate::models::character::{
        AbilityScores, Appearance, EquippedItems, FeatureReference, InventoryItem, Personality,
        Proficiencies, RoleplayNotes, SpellData, SpellReference, SpellSlots,
    };

    fn create_sample_fighter() -> CharacterData {
        CharacterData {
            character_name: "Thorin Ironforge".to_string(),
            player_id: Some(1),
            level: 3,
            experience_points: 900,
            version: 1,
            snapshot_reason: Some("Initial creation".to_string()),
            created_at: "2025-01-15T10:30:00Z".to_string(),
            race: "Dwarf".to_string(),
            subrace: Some("Mountain".to_string()),
            classes: vec![ClassLevel {
                class_name: "Fighter".to_string(),
                level: 3,
                subclass: Some("Champion".to_string()),
                hit_dice_type: "d10".to_string(),
                hit_dice_remaining: 3,
            }],
            background: "Soldier".to_string(),
            alignment: Some("Lawful Good".to_string()),
            abilities: AbilityScores {
                strength: 16,
                dexterity: 12,
                constitution: 16,
                intelligence: 10,
                wisdom: 13,
                charisma: 8,
            },
            max_hp: 28,
            current_hp: 28,
            proficiencies: Proficiencies {
                skills: vec!["Athletics".to_string(), "Intimidation".to_string()],
                saves: vec!["Strength".to_string(), "Constitution".to_string()],
                armor: vec!["All armor".to_string(), "Shields".to_string()],
                weapons: vec!["Simple weapons".to_string(), "Martial weapons".to_string()],
                tools: vec!["Smith's tools".to_string()],
                languages: vec!["Common".to_string(), "Dwarvish".to_string()],
            },
            class_features: vec![
                FeatureReference {
                    name: "Fighting Style (Defense)".to_string(),
                    class_name: "Fighter".to_string(),
                    subclass_name: None,
                    source: "PHB".to_string(),
                    level: 1,
                },
                FeatureReference {
                    name: "Second Wind".to_string(),
                    class_name: "Fighter".to_string(),
                    subclass_name: None,
                    source: "PHB".to_string(),
                    level: 1,
                },
                FeatureReference {
                    name: "Action Surge".to_string(),
                    class_name: "Fighter".to_string(),
                    subclass_name: None,
                    source: "PHB".to_string(),
                    level: 2,
                },
            ],
            feats: Vec::new(),
            spells: SpellData::default(),
            inventory: vec![
                InventoryItem {
                    name: "Rations".to_string(),
                    source: None,
                    quantity: 10,
                    weight: 20.0,
                    value: 5.0,
                    notes: None,
                },
                InventoryItem {
                    name: "Healing Potion".to_string(),
                    source: None,
                    quantity: 2,
                    weight: 1.0,
                    value: 50.0,
                    notes: Some("Greater healing".to_string()),
                },
            ],
            currency: Currency::default(),
            speed: 25, // Dwarf speed
            equipped: EquippedItems {
                armor: Some("Chain Mail".to_string()),
                shield: Some("Shield".to_string()),
                main_hand: Some("Warhammer".to_string()),
                off_hand: None,
            },
            personality: Personality {
                traits: Some("I'm always polite and respectful.".to_string()),
                ideals: Some("Responsibility.".to_string()),
                bonds: Some(
                    "I would still lay down my life for the people I served with.".to_string(),
                ),
                flaws: Some("I obey authority without question.".to_string()),
            },
            player_name: None,
            appearance: Appearance::default(),
            backstory: None,
            background_feature: None,
            roleplay_notes: RoleplayNotes::default(),
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: Vec::new(),
            legendary_action_count: None,
        }
    }

    fn create_sample_wizard() -> CharacterData {
        let mut spell_slots = HashMap::new();
        spell_slots.insert(1, SpellSlots::new(4));
        spell_slots.insert(2, SpellSlots::new(2));

        CharacterData {
            character_name: "Elara Moonwhisper".to_string(),
            player_id: Some(2),
            level: 3,
            experience_points: 900,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-15T11:00:00Z".to_string(),
            race: "Elf".to_string(),
            subrace: Some("High".to_string()),
            classes: vec![ClassLevel {
                class_name: "Wizard".to_string(),
                level: 3,
                subclass: Some("School of Evocation".to_string()),
                hit_dice_type: "d6".to_string(),
                hit_dice_remaining: 3,
            }],
            background: "Sage".to_string(),
            alignment: Some("Neutral Good".to_string()),
            abilities: AbilityScores {
                strength: 8,
                dexterity: 14,
                constitution: 12,
                intelligence: 16,
                wisdom: 13,
                charisma: 10,
            },
            max_hp: 15,
            current_hp: 15,
            proficiencies: Proficiencies {
                skills: vec![
                    "Arcana".to_string(),
                    "History".to_string(),
                    "Investigation".to_string(),
                ],
                saves: vec!["Intelligence".to_string(), "Wisdom".to_string()],
                armor: Vec::new(),
                weapons: vec!["Simple weapons".to_string()],
                tools: Vec::new(),
                languages: vec![
                    "Common".to_string(),
                    "Elvish".to_string(),
                    "Draconic".to_string(),
                ],
            },
            class_features: vec![
                FeatureReference {
                    name: "Arcane Recovery".to_string(),
                    class_name: "Wizard".to_string(),
                    subclass_name: None,
                    source: "PHB".to_string(),
                    level: 1,
                },
                FeatureReference {
                    name: "Evocation Savant".to_string(),
                    class_name: "Wizard".to_string(),
                    subclass_name: Some("Evocation".to_string()),
                    source: "PHB".to_string(),
                    level: 2,
                },
                FeatureReference {
                    name: "Sculpt Spells".to_string(),
                    class_name: "Wizard".to_string(),
                    subclass_name: Some("Evocation".to_string()),
                    source: "PHB".to_string(),
                    level: 2,
                },
            ],
            feats: Vec::new(),
            spells: SpellData {
                cantrips: vec![
                    SpellReference::new("Fire Bolt", "PHB"),
                    SpellReference::new("Mage Hand", "PHB"),
                    SpellReference::new("Prestidigitation", "PHB"),
                ],
                known_spells: vec![
                    SpellReference::new("Magic Missile", "PHB"),
                    SpellReference::new("Shield", "PHB"),
                    SpellReference::new("Detect Magic", "PHB"),
                    SpellReference::new("Fireball", "PHB"),
                    SpellReference::new("Counterspell", "PHB"),
                ],
                prepared_spells: vec![
                    SpellReference::new("Magic Missile", "PHB"),
                    SpellReference::new("Shield", "PHB"),
                    SpellReference::new("Fireball", "PHB"),
                ],
                spell_slots,
            },
            inventory: vec![InventoryItem {
                name: "Spellbook".to_string(),
                source: None,
                quantity: 1,
                weight: 3.0,
                value: 50.0,
                notes: Some("Contains all known spells".to_string()),
            }],
            currency: Currency::default(),
            speed: 30, // Elf speed
            equipped: EquippedItems {
                armor: None,
                shield: None,
                main_hand: Some("Quarterstaff".to_string()),
                off_hand: None,
            },
            personality: Personality::default(),
            player_name: None,
            appearance: Appearance::default(),
            backstory: None,
            background_feature: None,
            roleplay_notes: RoleplayNotes::default(),
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: Vec::new(),
            legendary_action_count: None,
        }
    }

    #[test]
    fn test_render_fighter() {
        let renderer = MarkdownRenderer::new();
        let fighter = create_sample_fighter();
        let markdown = renderer.render(&fighter);

        // Check header
        assert!(markdown.contains("# Thorin Ironforge - Level 3 Fighter 3"));

        // Check metadata
        assert!(markdown.contains("**Race:** Dwarf (Mountain)"));
        assert!(markdown.contains("**Background:** Soldier"));

        // Check abilities
        assert!(markdown.contains("| 16 (+3) | 12 (+1) | 16 (+3)"));

        // Check combat stats
        assert!(markdown.contains("**HP:** 28 / 28"));
        assert!(markdown.contains("**Proficiency Bonus:** +2"));

        // Check skills table
        assert!(markdown.contains("| Athletics |"));
        assert!(markdown.contains("| Intimidation |"));

        // Check class features
        assert!(markdown.contains("Fighting Style (Defense)"));

        // Check equipment
        assert!(markdown.contains("**Armor:** Chain Mail"));

        // Check inventory section
        assert!(markdown.contains("## Inventory"));
        assert!(markdown.contains("### Rations (x10)"));
        assert!(markdown.contains("### Healing Potion (x2)"));
        assert!(markdown.contains("> **Notes:** Greater healing"));

        // Check personality
        assert!(markdown.contains("**Traits:** I'm always polite and respectful."));
    }

    #[test]
    fn test_render_wizard_with_spells() {
        let renderer = MarkdownRenderer::new();
        let wizard = create_sample_wizard();
        let markdown = renderer.render(&wizard);

        // Check spells section exists
        assert!(markdown.contains("## Spells"));

        // Check spell slots
        assert!(markdown.contains("Level 1: 4 / 4"));
        assert!(markdown.contains("Level 2: 2 / 2"));

        // Check cantrips
        assert!(markdown.contains("### Cantrips"));
        assert!(markdown.contains("Fire Bolt"));

        // Check known spells
        assert!(markdown.contains("### Known Spells"));
        assert!(markdown.contains("Magic Missile"));

        // Check prepared spells
        assert!(markdown.contains("### Prepared Spells"));
        assert!(markdown.contains("Fireball"));
    }

    #[test]
    fn test_conditional_sections() {
        let renderer = MarkdownRenderer::new();

        // Create minimal character with no spells, feats, or personality
        let minimal = CharacterData {
            character_name: "Test".to_string(),
            player_id: Some(1),
            level: 1,
            experience_points: 0,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-01".to_string(),
            race: "Human".to_string(),
            subrace: None,
            classes: vec![ClassLevel {
                class_name: "Fighter".to_string(),
                level: 1,
                subclass: None,
                hit_dice_type: "d10".to_string(),
                hit_dice_remaining: 1,
            }],
            background: "Folk Hero".to_string(),
            alignment: None,
            abilities: AbilityScores {
                strength: 15,
                dexterity: 14,
                constitution: 13,
                intelligence: 12,
                wisdom: 10,
                charisma: 8,
            },
            max_hp: 12,
            current_hp: 12,
            proficiencies: Proficiencies::default(),
            class_features: Vec::new(),
            feats: Vec::new(),
            spells: SpellData::default(),
            inventory: Vec::new(),
            currency: Currency::default(),
            speed: 30, // Human speed
            equipped: EquippedItems::default(),
            personality: Personality::default(),
            player_name: None,
            appearance: Appearance::default(),
            backstory: None,
            background_feature: None,
            roleplay_notes: RoleplayNotes::default(),
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: Vec::new(),
            legendary_action_count: None,
        };

        let markdown = renderer.render(&minimal);

        // Should not have these sections
        assert!(!markdown.contains("## Spells"));
        assert!(!markdown.contains("## Feats"));
        assert!(!markdown.contains("## Equipment"));
        assert!(!markdown.contains("## Inventory"));
        assert!(!markdown.contains("## Personality"));

        // Should still have core sections
        assert!(markdown.contains("## Ability Scores"));
        assert!(markdown.contains("## Combat Stats"));
    }

    #[test]
    fn test_render_npc_info() {
        let renderer = MarkdownRenderer::new();

        // Create an NPC character
        let npc = CharacterData {
            character_name: "Barkeep Marcus".to_string(),
            player_id: None, // NPCs don't have a player
            level: 1,
            experience_points: 0,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-01".to_string(),
            race: "Human".to_string(),
            subrace: None,
            classes: vec![ClassLevel {
                class_name: "Commoner".to_string(),
                level: 1,
                subclass: None,
                hit_dice_type: "d8".to_string(),
                hit_dice_remaining: 1,
            }],
            background: "Guild Artisan".to_string(),
            alignment: Some("Neutral Good".to_string()),
            abilities: AbilityScores {
                strength: 10,
                dexterity: 10,
                constitution: 12,
                intelligence: 11,
                wisdom: 14,
                charisma: 13,
            },
            max_hp: 8,
            current_hp: 8,
            proficiencies: Proficiencies::default(),
            class_features: Vec::new(),
            feats: Vec::new(),
            spells: SpellData::default(),
            inventory: Vec::new(),
            currency: Currency::default(),
            speed: 30,
            equipped: EquippedItems::default(),
            personality: Personality::default(),
            player_name: None,
            appearance: Appearance::default(),
            backstory: None,
            background_feature: None,
            roleplay_notes: RoleplayNotes::default(),
            npc_role: Some("Tavern Owner".to_string()),
            npc_location: Some("The Rusty Anchor, Waterdeep".to_string()),
            npc_faction: Some("Innkeepers Guild".to_string()),
            npc_notes: Some("Friendly and talkative. Knows local rumors.".to_string()),
            legendary_actions: Vec::new(),
            legendary_action_count: None,
        };

        let markdown = renderer.render(&npc);

        // Check NPC info section
        assert!(markdown.contains("## NPC Information"));
        assert!(markdown.contains("**Role:** Tavern Owner"));
        assert!(markdown.contains("**Location:** The Rusty Anchor, Waterdeep"));
        assert!(markdown.contains("**Faction:** Innkeepers Guild"));
        assert!(markdown.contains("**Notes:**"));
        assert!(markdown.contains("Friendly and talkative"));
    }

    #[test]
    fn test_npc_section_not_rendered_for_pc() {
        let renderer = MarkdownRenderer::new();
        let fighter = create_sample_fighter();
        let markdown = renderer.render(&fighter);

        // Should not have NPC section for player characters
        assert!(!markdown.contains("## NPC Information"));
    }

    #[test]
    fn test_render_legendary_actions() {
        let renderer = MarkdownRenderer::new();

        // Create a boss NPC with legendary actions
        let boss = CharacterData {
            character_name: "Ancient Red Dragon".to_string(),
            player_id: None,
            level: 20,
            experience_points: 0,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-01".to_string(),
            race: "Dragon".to_string(),
            subrace: None,
            classes: vec![ClassLevel {
                class_name: "Monster".to_string(),
                level: 20,
                subclass: None,
                hit_dice_type: "d20".to_string(),
                hit_dice_remaining: 20,
            }],
            background: "Ancient Wyrm".to_string(),
            alignment: Some("Chaotic Evil".to_string()),
            abilities: AbilityScores {
                strength: 30,
                dexterity: 10,
                constitution: 29,
                intelligence: 18,
                wisdom: 15,
                charisma: 23,
            },
            max_hp: 546,
            current_hp: 546,
            proficiencies: Proficiencies::default(),
            class_features: Vec::new(),
            feats: Vec::new(),
            spells: SpellData::default(),
            inventory: Vec::new(),
            currency: Currency::default(),
            speed: 40,
            equipped: EquippedItems::default(),
            personality: Personality::default(),
            player_name: None,
            appearance: Appearance::default(),
            backstory: None,
            background_feature: None,
            roleplay_notes: RoleplayNotes::default(),
            npc_role: Some("Villain".to_string()),
            npc_location: Some("Mount Hotenow".to_string()),
            npc_faction: None,
            npc_notes: None,
            legendary_actions: vec![
                LegendaryAction {
                    name: "Detect".to_string(),
                    cost: 1,
                    description: "The dragon makes a Wisdom (Perception) check.".to_string(),
                },
                LegendaryAction {
                    name: "Tail Attack".to_string(),
                    cost: 1,
                    description: "The dragon makes a tail attack.".to_string(),
                },
                LegendaryAction {
                    name: "Wing Attack".to_string(),
                    cost: 2,
                    description: "The dragon beats its wings. Each creature within 15 feet must succeed on a DC 25 Dexterity saving throw or take 17 bludgeoning damage and be knocked prone.".to_string(),
                },
            ],
            legendary_action_count: Some(3),
        };

        let markdown = renderer.render(&boss);

        // Check legendary actions section
        assert!(markdown.contains("## Legendary Actions"));
        assert!(markdown.contains("Ancient Red Dragon can take 3 legendary actions"));
        assert!(markdown.contains("**Detect**. The dragon makes a Wisdom (Perception) check."));
        assert!(markdown.contains("**Tail Attack**. The dragon makes a tail attack."));
        assert!(markdown.contains("**Wing Attack** (Costs 2 Actions). The dragon beats its wings."));
    }

    #[test]
    fn test_legendary_actions_not_rendered_when_empty() {
        let renderer = MarkdownRenderer::new();
        let fighter = create_sample_fighter();
        let markdown = renderer.render(&fighter);

        // Should not have legendary actions section for regular characters
        assert!(!markdown.contains("## Legendary Actions"));
    }
}

//! The numbers a sheet shows, from the stored character (5e rules). A port
//! of the desktop `characterUtils` and the spell-slot tables of
//! `useSpellManagement`; the stored character has no HP, AC or speed of
//! its own (speed comes with the sheet, from the race).

use mimir_wire::{CharacterSheet, InventoryItem};

pub const ABILITIES: [&str; 6] = [
    "strength",
    "dexterity",
    "constitution",
    "intelligence",
    "wisdom",
    "charisma",
];

/// The 18 skills and their abilities.
pub const SKILLS: [(&str, &str); 18] = [
    ("Acrobatics", "dexterity"),
    ("Animal Handling", "wisdom"),
    ("Arcana", "intelligence"),
    ("Athletics", "strength"),
    ("Deception", "charisma"),
    ("History", "intelligence"),
    ("Insight", "wisdom"),
    ("Intimidation", "charisma"),
    ("Investigation", "intelligence"),
    ("Medicine", "wisdom"),
    ("Nature", "intelligence"),
    ("Perception", "wisdom"),
    ("Performance", "charisma"),
    ("Persuasion", "charisma"),
    ("Religion", "intelligence"),
    ("Sleight of Hand", "dexterity"),
    ("Stealth", "dexterity"),
    ("Survival", "wisdom"),
];

pub fn modifier(score: i32) -> i32 {
    (score - 10).div_euclid(2)
}

/// "+2", "-1".
pub fn signed(n: i32) -> String {
    if n >= 0 {
        format!("+{n}")
    } else {
        n.to_string()
    }
}

pub fn proficiency_bonus(total_level: i32) -> i32 {
    if total_level <= 0 {
        2
    } else {
        (total_level - 1) / 4 + 2
    }
}

pub fn total_level(c: &CharacterSheet) -> i32 {
    c.classes.iter().map(|k| k.level).sum()
}

/// "STR".
pub fn abbrev(ability: &str) -> String {
    ability.chars().take(3).collect::<String>().to_uppercase()
}

pub fn score(c: &CharacterSheet, ability: &str) -> i32 {
    score_of_abilities(&c.abilities, ability)
}

/// A score by its full name ("strength").
pub fn score_of_abilities(a: &mimir_wire::Abilities, ability: &str) -> i32 {
    match ability {
        "strength" => a.strength,
        "dexterity" => a.dexterity,
        "constitution" => a.constitution,
        "intelligence" => a.intelligence,
        "wisdom" => a.wisdom,
        "charisma" => a.charisma,
        _ => 10,
    }
}

fn has(c: &CharacterSheet, kind: &str, name: &str) -> bool {
    c.proficiencies
        .iter()
        .any(|p| p.kind == kind && p.name.eq_ignore_ascii_case(name))
}

fn expert(c: &CharacterSheet, skill: &str) -> bool {
    c.proficiencies
        .iter()
        .any(|p| p.kind == "skill" && p.name.eq_ignore_ascii_case(skill) && p.expertise)
}

/// 0 none, 1 proficient, 2 expertise.
pub fn skill_rank(c: &CharacterSheet, skill: &str) -> i32 {
    if expert(c, skill) {
        2
    } else if has(c, "skill", skill) {
        1
    } else {
        0
    }
}

pub fn skill_bonus(c: &CharacterSheet, skill: &str, ability: &str) -> i32 {
    modifier(score(c, ability)) + skill_rank(c, skill) * proficiency_bonus(total_level(c))
}

pub fn save_proficient(c: &CharacterSheet, ability: &str) -> bool {
    has(c, "save", ability)
}

pub fn save_bonus(c: &CharacterSheet, ability: &str) -> i32 {
    let base = modifier(score(c, ability));
    if save_proficient(c, ability) {
        base + proficiency_bonus(total_level(c))
    } else {
        base
    }
}

pub fn passive_perception(c: &CharacterSheet) -> i32 {
    10 + skill_bonus(c, "Perception", "wisdom")
}

pub fn initiative(c: &CharacterSheet) -> i32 {
    modifier(c.abilities.dexterity)
}

/// AC of a body armor by name (a "+N" in the name adds N).
pub fn armor_ac(name: &str, dex_mod: i32) -> i32 {
    let n = name.to_lowercase();
    let magic = n
        .find('+')
        .and_then(|i| n[i + 1..].chars().next())
        .and_then(|d| d.to_digit(10))
        .map(|d| d as i32)
        .unwrap_or(0);
    let capped = dex_mod.min(2);
    let base = if n.contains("padded") || (n.contains("leather") && !n.contains("studded")) {
        11 + dex_mod
    } else if n.contains("studded leather") {
        12 + dex_mod
    } else if n.contains("hide") {
        12 + capped
    } else if n.contains("chain shirt") {
        13 + capped
    } else if n.contains("scale") || n.contains("breastplate") {
        14 + capped
    } else if n.contains("half plate") {
        15 + capped
    } else if n.contains("ring mail") {
        14
    } else if n.contains("chain mail") {
        16
    } else if n.contains("splint") {
        17
    } else if n.contains("plate") {
        18
    } else {
        11 + dex_mod
    };
    base + magic
}

const ARMOR_WORDS: [&str; 11] = [
    "armor",
    "mail",
    "hide",
    "leather",
    "plate",
    "robe",
    "breastplate",
    "chain shirt",
    "scale",
    "splint",
    "padded",
];

fn is_body_armor(name: &str) -> bool {
    let n = name.to_lowercase();
    !n.contains("shield") && ARMOR_WORDS.iter().any(|w| n.contains(w))
}

fn is_shield(name: &str) -> bool {
    name.to_lowercase().contains("shield")
}

/// AC: the first equipped body armor, else 10 + DEX; +2 for an equipped
/// shield.
pub fn armor_class(c: &CharacterSheet) -> i32 {
    let dex = modifier(c.abilities.dexterity);
    let equipped = c.inventory.iter().filter(|i| i.equipped);
    let body = equipped.clone().find(|i| is_body_armor(&i.item_name));
    let base = match body {
        Some(a) => armor_ac(&a.item_name, dex),
        None => 10 + dex,
    };
    let shield = equipped.into_iter().any(|i| is_shield(&i.item_name));
    base + if shield { 2 } else { 0 }
}

const WEAPONS: [&str; 37] = [
    "club",
    "dagger",
    "greatclub",
    "handaxe",
    "javelin",
    "light hammer",
    "mace",
    "quarterstaff",
    "sickle",
    "spear",
    "light crossbow",
    "dart",
    "shortbow",
    "sling",
    "battleaxe",
    "flail",
    "glaive",
    "greataxe",
    "greatsword",
    "halberd",
    "lance",
    "longsword",
    "maul",
    "morningstar",
    "pike",
    "rapier",
    "scimitar",
    "shortsword",
    "trident",
    "war pick",
    "warhammer",
    "whip",
    "blowgun",
    "hand crossbow",
    "heavy crossbow",
    "longbow",
    "net",
];

pub fn is_weapon(name: &str) -> bool {
    let n = name.to_lowercase();
    WEAPONS
        .iter()
        .any(|w| n == *w || n.starts_with(w) || n.ends_with(w))
}

pub fn is_finesse(name: &str) -> bool {
    let n = name.to_lowercase();
    ["rapier", "dagger", "shortsword", "scimitar", "whip"]
        .iter()
        .any(|w| n.contains(w))
}

pub fn is_ranged(name: &str) -> bool {
    let n = name.to_lowercase();
    ["bow", "crossbow", "dart", "sling", "blowgun", "net"]
        .iter()
        .any(|w| n.contains(w))
}

/// The damage die of a weapon by name.
pub fn weapon_die(name: &str) -> &'static str {
    let w = name.to_lowercase();
    let any = |xs: &[&str]| xs.iter().any(|x| w.contains(x));
    if any(&["greatsword", "maul"]) {
        "2d6"
    } else if any(&["greataxe", "lance"]) {
        "1d12"
    } else if any(&["glaive", "halberd", "pike", "heavy crossbow"]) {
        "1d10"
    } else if any(&[
        "longsword",
        "warhammer",
        "battleaxe",
        "rapier",
        "flail",
        "morningstar",
        "war pick",
        "greatclub",
        "longbow",
    ]) {
        "1d8"
    } else if any(&[
        "dagger",
        "light hammer",
        "sickle",
        "whip",
        "dart",
        "sling",
        "club",
    ]) {
        // Greatclub is matched above.
        "1d4"
    } else if w.contains("blowgun") {
        "1"
    } else if w.contains("net") {
        "0"
    } else {
        "1d6"
    }
}

/// An attack line of an equipped weapon.
#[derive(Debug, Clone, PartialEq)]
pub struct Attack {
    pub name: String,
    pub to_hit: i32,
    pub damage: String,
}

/// Attacks of the equipped weapons: DEX for ranged, the better of STR and
/// DEX for finesse, else STR.
pub fn attacks(c: &CharacterSheet) -> Vec<Attack> {
    let str_m = modifier(c.abilities.strength);
    let dex_m = modifier(c.abilities.dexterity);
    let prof = proficiency_bonus(total_level(c));
    c.inventory
        .iter()
        .filter(|i| i.equipped && is_weapon(&i.item_name))
        .map(|i: &InventoryItem| {
            let m = if is_ranged(&i.item_name) {
                dex_m
            } else if is_finesse(&i.item_name) {
                str_m.max(dex_m)
            } else {
                str_m
            };
            let die = weapon_die(&i.item_name);
            let damage = match die {
                "0" => "0".to_string(),
                d => format!("{d}{}", if m == 0 { String::new() } else { signed(m) }),
            };
            Attack {
                name: i.item_name.clone(),
                to_hit: prof + m,
                damage,
            }
        })
        .collect()
}

/// The spellcasting ability of a class (lower case), if it casts.
pub fn casting_ability(class: &str) -> Option<&'static str> {
    match class.to_lowercase().as_str() {
        "bard" | "paladin" | "sorcerer" | "warlock" => Some("charisma"),
        "cleric" | "druid" | "ranger" => Some("wisdom"),
        "wizard" | "artificer" => Some("intelligence"),
        _ => None,
    }
}

/// Save DC and attack bonus per casting class.
#[derive(Debug, Clone, PartialEq)]
pub struct Casting {
    pub class: String,
    pub ability: &'static str,
    pub save_dc: i32,
    pub attack: i32,
}

pub fn casting(c: &CharacterSheet) -> Vec<Casting> {
    let prof = proficiency_bonus(total_level(c));
    c.classes
        .iter()
        .filter_map(|k| {
            let ability = casting_ability(&k.class_name)?;
            let m = modifier(score(c, ability));
            Some(Casting {
                class: k.class_name.clone(),
                ability,
                save_dc: 8 + prof + m,
                attack: prof + m,
            })
        })
        .collect()
}

/// The caster level for slots (multiclass rules; warlock apart).
pub fn caster_level(c: &CharacterSheet) -> i32 {
    c.classes
        .iter()
        .map(|k| match k.class_name.to_lowercase().as_str() {
            "bard" | "cleric" | "druid" | "sorcerer" | "wizard" => k.level,
            "paladin" | "ranger" if k.level >= 2 => k.level / 2,
            "artificer" => (k.level + 1) / 2,
            _ => 0,
        })
        .sum()
}

const SLOTS: [[i32; 9]; 20] = [
    [2, 0, 0, 0, 0, 0, 0, 0, 0],
    [3, 0, 0, 0, 0, 0, 0, 0, 0],
    [4, 2, 0, 0, 0, 0, 0, 0, 0],
    [4, 3, 0, 0, 0, 0, 0, 0, 0],
    [4, 3, 2, 0, 0, 0, 0, 0, 0],
    [4, 3, 3, 0, 0, 0, 0, 0, 0],
    [4, 3, 3, 1, 0, 0, 0, 0, 0],
    [4, 3, 3, 2, 0, 0, 0, 0, 0],
    [4, 3, 3, 3, 1, 0, 0, 0, 0],
    [4, 3, 3, 3, 2, 0, 0, 0, 0],
    [4, 3, 3, 3, 2, 1, 0, 0, 0],
    [4, 3, 3, 3, 2, 1, 0, 0, 0],
    [4, 3, 3, 3, 2, 1, 1, 0, 0],
    [4, 3, 3, 3, 2, 1, 1, 0, 0],
    [4, 3, 3, 3, 2, 1, 1, 1, 0],
    [4, 3, 3, 3, 2, 1, 1, 1, 0],
    [4, 3, 3, 3, 2, 1, 1, 1, 1],
    [4, 3, 3, 3, 3, 1, 1, 1, 1],
    [4, 3, 3, 3, 3, 2, 1, 1, 1],
    [4, 3, 3, 3, 3, 2, 2, 1, 1],
];

/// Pact magic (warlock level → count, slot level).
fn pact(level: i32) -> Option<(i32, usize)> {
    let (count, at) = match level {
        1 => (1, 1),
        2 => (2, 1),
        3 | 4 => (2, 2),
        5 | 6 => (2, 3),
        7 | 8 => (2, 4),
        9 | 10 => (2, 5),
        11..=16 => (3, 5),
        17..=20 => (4, 5),
        _ => return None,
    };
    Some((count, at))
}

/// Spell slots by level (index 0 = 1st level), multiclass slots plus pact
/// magic.
pub fn spell_slots(c: &CharacterSheet) -> [i32; 9] {
    let mut out = [0; 9];
    let level = caster_level(c);
    if level > 0 {
        out = SLOTS[(level.min(20) - 1) as usize];
    }
    if let Some(w) = c
        .classes
        .iter()
        .find(|k| k.class_name.eq_ignore_ascii_case("warlock"))
    {
        if let Some((count, at)) = pact(w.level) {
            out[at - 1] += count;
        }
    }
    out
}

/// "5d6 + 2d10" by class.
pub fn hit_dice(c: &CharacterSheet) -> String {
    if c.classes.is_empty() {
        return "—".into();
    }
    c.classes
        .iter()
        .map(|k| format!("{}{}", k.level, hit_die(&k.class_name)))
        .collect::<Vec<_>>()
        .join(" + ")
}

pub fn hit_die(class: &str) -> &'static str {
    match class.to_lowercase().as_str() {
        "barbarian" => "d12",
        "fighter" | "paladin" | "ranger" => "d10",
        "sorcerer" | "wizard" => "d6",
        _ => "d8",
    }
}

/// "Wizard (Evocation) 5 / Fighter 1".
pub fn class_line(c: &CharacterSheet) -> String {
    if c.classes.is_empty() {
        return "No class".into();
    }
    c.classes
        .iter()
        .map(|k| match &k.subclass_name {
            Some(s) => format!("{} ({s}) {}", k.class_name, k.level),
            None => format!("{} {}", k.class_name, k.level),
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

/// Gold value of the coins.
pub fn gold_value(c: &CharacterSheet) -> f64 {
    let m = &c.currency;
    f64::from(m.cp) / 100.0
        + f64::from(m.sp) / 10.0
        + f64::from(m.ep) / 2.0
        + f64::from(m.gp)
        + f64::from(m.pp) * 10.0
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use mimir_wire::{Abilities, Currency, Proficiency, SheetClass};

    pub(crate) fn sheet() -> CharacterSheet {
        CharacterSheet {
            id: "c1".into(),
            campaign_id: Some("k1".into()),
            name: "Robin".into(),
            is_npc: false,
            player_name: Some("Sam".into()),
            race_name: Some("Elf".into()),
            race_source: Some("PHB".into()),
            background_name: None,
            background_source: None,
            abilities: Abilities {
                strength: 8,
                dexterity: 14,
                constitution: 12,
                intelligence: 17,
                wisdom: 13,
                charisma: 10,
            },
            currency: Currency {
                cp: 50,
                sp: 5,
                ep: 2,
                gp: 10,
                pp: 1,
            },
            traits: None,
            ideals: None,
            bonds: None,
            flaws: None,
            role: None,
            location: None,
            faction: None,
            classes: vec![class("Wizard", 5)],
            proficiencies: vec![],
            inventory: vec![],
            spells: vec![],
            feats: vec![],
            features: vec![],
            speed: 30,
            updated_at: String::new(),
        }
    }

    pub(crate) fn class(name: &str, level: i32) -> SheetClass {
        SheetClass {
            class_name: name.into(),
            class_source: "PHB".into(),
            level,
            subclass_name: None,
            subclass_source: None,
            starting: true,
        }
    }

    fn prof(kind: &str, name: &str, expertise: bool) -> Proficiency {
        Proficiency {
            kind: kind.into(),
            name: name.into(),
            expertise,
        }
    }

    fn item(name: &str, equipped: bool) -> InventoryItem {
        InventoryItem {
            id: name.into(),
            item_name: name.into(),
            item_source: "PHB".into(),
            quantity: 1,
            equipped,
            attuned: false,
            notes: None,
        }
    }

    #[test]
    fn modifiers_and_proficiency() {
        assert_eq!(modifier(10), 0);
        assert_eq!(modifier(11), 0);
        assert_eq!(modifier(9), -1);
        assert_eq!(modifier(8), -1);
        assert_eq!(modifier(1), -5);
        assert_eq!(modifier(20), 5);
        assert_eq!(
            (signed(2), signed(0), signed(-1)),
            ("+2".into(), "+0".into(), "-1".into())
        );
        assert_eq!(proficiency_bonus(0), 2);
        assert_eq!(proficiency_bonus(4), 2);
        assert_eq!(proficiency_bonus(5), 3);
        assert_eq!(proficiency_bonus(9), 4);
        assert_eq!(proficiency_bonus(17), 6);
        let mut c = sheet();
        c.classes.push(class("Fighter", 3));
        assert_eq!(total_level(&c), 8);
        assert_eq!(abbrev("constitution"), "CON");
    }

    #[test]
    fn skills_saves_and_passive_perception() {
        let mut c = sheet();
        c.proficiencies = vec![
            prof("skill", "Arcana", false),
            prof("skill", "perception", true),
            prof("save", "Intelligence", false),
        ];
        assert_eq!(skill_bonus(&c, "Arcana", "intelligence"), 3 + 3);
        assert_eq!(
            skill_bonus(&c, "Perception", "wisdom"),
            1 + 6,
            "expertise doubles"
        );
        assert_eq!(skill_bonus(&c, "Stealth", "dexterity"), 2);
        assert_eq!(skill_rank(&c, "Perception"), 2);
        assert_eq!(save_bonus(&c, "intelligence"), 6);
        assert_eq!(save_bonus(&c, "strength"), -1);
        assert_eq!(passive_perception(&c), 17);
        assert_eq!(initiative(&c), 2);
    }

    #[test]
    fn armor_class() {
        assert_eq!(armor_ac("Leather Armor", 3), 14);
        assert_eq!(armor_ac("Studded Leather Armor", 3), 15);
        assert_eq!(armor_ac("Hide Armor", 3), 14, "medium caps DEX at 2");
        assert_eq!(armor_ac("Half Plate Armor", 1), 16);
        assert_eq!(armor_ac("Chain Mail", 3), 16, "heavy: no DEX");
        assert_eq!(armor_ac("Plate Armor +1", 0), 19);
        let mut c = sheet();
        assert_eq!(super::armor_class(&c), 12, "no armor: 10 + DEX");
        c.inventory = vec![
            item("Chain Shirt", true),
            item("Plate Armor", false),
            item("Shield", true),
        ];
        assert_eq!(super::armor_class(&c), 13 + 2 + 2, "shirt, DEX 2, shield");
    }

    #[test]
    fn attacks_of_equipped_weapons() {
        let mut c = sheet();
        c.inventory = vec![
            item("Longbow", true),
            item("Dagger", true),
            item("Greataxe", true),
            item("Rope", true),
            item("Rapier", false),
        ];
        let a = attacks(&c);
        assert_eq!(a.len(), 3);
        assert_eq!(
            a[0],
            Attack {
                name: "Longbow".into(),
                to_hit: 3 + 2,
                damage: "1d8+2".into()
            }
        );
        assert_eq!(a[1].damage, "1d4+2", "finesse takes the better (DEX)");
        assert_eq!(
            a[2],
            Attack {
                name: "Greataxe".into(),
                to_hit: 3 - 1,
                damage: "1d12-1".into()
            }
        );
        assert!(is_weapon("Longsword +1") && is_weapon("+1 Longsword") && !is_weapon("Rope"));
    }

    #[test]
    fn spellcasting_and_slots() {
        let mut c = sheet();
        let k = casting(&c);
        assert_eq!(k[0].save_dc, 8 + 3 + 3);
        assert_eq!(k[0].attack, 6);
        assert_eq!(spell_slots(&c)[..3], [4, 3, 2]);
        c.classes = vec![class("Paladin", 4), class("Sorcerer", 3)];
        assert_eq!(caster_level(&c), 2 + 3);
        assert_eq!(spell_slots(&c)[..3], [4, 3, 2]);
        c.classes = vec![class("Warlock", 5)];
        assert_eq!(spell_slots(&c)[..3], [0, 0, 2], "pact slots at 3rd");
        c.classes = vec![class("Fighter", 5)];
        assert!(casting(&c).is_empty());
        assert_eq!(spell_slots(&c), [0; 9]);
    }

    #[test]
    fn hit_dice_lines_and_gold() {
        let mut c = sheet();
        c.classes.push(class("Fighter", 2));
        c.classes[0].subclass_name = Some("Evocation".into());
        assert_eq!(hit_dice(&c), "5d6 + 2d10");
        assert_eq!(class_line(&c), "Wizard (Evocation) 5 / Fighter 2");
        assert!((gold_value(&c) - (0.5 + 0.5 + 1.0 + 10.0 + 10.0)).abs() < 1e-9);
        c.classes.clear();
        assert_eq!(
            (hit_dice(&c), class_line(&c)),
            ("—".into(), "No class".into())
        );
    }
}

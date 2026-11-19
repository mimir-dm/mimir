//! The homebrew item form and its JSON (MIMIR-T-0722). As the desktop's
//! form, with one change: the form writes into the item's data and keeps
//! what it does not model (the desktop rebuilt the data and lost the
//! rest on each edit). Pure, host-tested.

use serde_json::{json, Map, Value};

/// The kinds of item the form knows: (key, label).
pub const KINDS: [(&str, &str); 10] = [
    ("gear", "Adventuring gear"),
    ("weapon", "Weapon"),
    ("armor", "Armor"),
    ("potion", "Potion"),
    ("ring", "Ring"),
    ("rod", "Rod"),
    ("scroll", "Scroll"),
    ("staff", "Staff"),
    ("wand", "Wand"),
    ("wondrous", "Wondrous item"),
];

pub const RARITIES: [(&str, &str); 7] = [
    ("none", "Mundane"),
    ("common", "Common"),
    ("uncommon", "Uncommon"),
    ("rare", "Rare"),
    ("very rare", "Very rare"),
    ("legendary", "Legendary"),
    ("artifact", "Artifact"),
];

pub const ARMOR: [(&str, &str); 4] = [
    ("LA", "Light"),
    ("MA", "Medium"),
    ("HA", "Heavy"),
    ("S", "Shield"),
];

pub const DAMAGE: [(&str, &str); 13] = [
    ("S", "Slashing"),
    ("P", "Piercing"),
    ("B", "Bludgeoning"),
    ("A", "Acid"),
    ("C", "Cold"),
    ("F", "Fire"),
    ("O", "Force"),
    ("L", "Lightning"),
    ("N", "Necrotic"),
    ("I", "Poison"),
    ("Y", "Psychic"),
    ("R", "Radiant"),
    ("T", "Thunder"),
];

pub const PROPERTIES: [(&str, &str); 10] = [
    ("F", "Finesse"),
    ("L", "Light"),
    ("H", "Heavy"),
    ("R", "Reach"),
    ("T", "Thrown"),
    ("2H", "Two-handed"),
    ("V", "Versatile"),
    ("A", "Ammunition"),
    ("LD", "Loading"),
    ("S", "Special"),
];

const WEAPON_KEYS: [&str; 8] = [
    "weapon",
    "weaponCategory",
    "dmg1",
    "dmg2",
    "dmgType",
    "range",
    "property",
    "bonusWeapon",
];
const ARMOR_KEYS: [&str; 5] = ["armor", "ac", "bonusAc", "strength", "stealth"];

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemForm {
    pub kind: String,
    pub rarity: String,
    pub weight: String,
    pub value_gp: String,
    pub attune: bool,
    pub attune_text: String,
    /// The text entries, as paragraphs (a blank line between).
    pub description: String,
    /// The entries hold more than text (lists, tables): the form leaves
    /// them to the JSON.
    pub rich_entries: bool,
    pub weapon_category: String,
    pub bonus_weapon: String,
    pub dmg1: String,
    pub dmg_type: String,
    pub dmg2: String,
    pub range: String,
    pub properties: Vec<String>,
    pub armor_type: String,
    pub ac: String,
    pub bonus_ac: String,
    pub strength: String,
    pub stealth: bool,
}

fn code(v: &Value) -> String {
    v.as_str()
        .unwrap_or("")
        .split('|')
        .next()
        .unwrap_or("")
        .to_string()
}

fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// The kind of an item's data.
pub fn kind_of(d: &Value) -> &'static str {
    let t = code(&d["type"]);
    if d["weapon"].as_bool() == Some(true) || t == "M" || t == "R" {
        return "weapon";
    }
    if d["armor"].as_bool() == Some(true) || matches!(t.as_str(), "LA" | "MA" | "HA" | "S") {
        return "armor";
    }
    match t.as_str() {
        "P" => "potion",
        "RG" => "ring",
        "RD" => "rod",
        "SC" => "scroll",
        "ST" => "staff",
        "WD" => "wand",
        "W" => "wondrous",
        _ if d["wondrous"].as_bool() == Some(true) => "wondrous",
        _ => "gear",
    }
}

/// The form of an item's data.
pub fn form_of(d: &Value) -> ItemForm {
    let entries = d["entries"].as_array().cloned().unwrap_or_default();
    let rich = entries.iter().any(|e| !e.is_string());
    let t = code(&d["type"]);
    ItemForm {
        kind: kind_of(d).to_string(),
        rarity: d["rarity"].as_str().unwrap_or("none").to_string(),
        weight: text(&d["weight"]),
        value_gp: d["value"]
            .as_f64()
            .map(|cp| {
                let gp = cp / 100.0;
                if gp.fract() == 0.0 {
                    format!("{gp:.0}")
                } else {
                    format!("{gp}")
                }
            })
            .unwrap_or_default(),
        attune: !d["reqAttune"].is_null() && d["reqAttune"] != Value::Bool(false),
        attune_text: d["reqAttune"].as_str().unwrap_or("").to_string(),
        description: entries
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n\n"),
        rich_entries: rich,
        weapon_category: d["weaponCategory"].as_str().unwrap_or("simple").to_string(),
        bonus_weapon: text(&d["bonusWeapon"]),
        dmg1: text(&d["dmg1"]),
        dmg_type: d["dmgType"].as_str().unwrap_or("").to_string(),
        dmg2: text(&d["dmg2"]),
        range: text(&d["range"]),
        properties: d["property"]
            .as_array()
            .map(|l| l.iter().map(code).filter(|p| !p.is_empty()).collect())
            .unwrap_or_default(),
        armor_type: if matches!(t.as_str(), "LA" | "MA" | "HA" | "S") {
            t
        } else {
            "LA".into()
        },
        ac: text(&d["ac"]),
        bonus_ac: text(&d["bonusAc"]),
        strength: text(&d["strength"]),
        stealth: d["stealth"].as_bool() == Some(true),
    }
}

fn set_or_remove(m: &mut Map<String, Value>, key: &str, v: Option<Value>) {
    match v {
        Some(v) => {
            m.insert(key.into(), v);
        }
        None => {
            m.remove(key);
        }
    }
}

fn s_opt(s: &str) -> Option<Value> {
    let t = s.trim();
    (!t.is_empty()).then(|| Value::String(t.to_string()))
}

fn n_opt(s: &str) -> Option<Value> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    t.parse::<i64>()
        .map(Value::from)
        .ok()
        .or_else(|| t.parse::<f64>().ok().map(Value::from))
}

/// Write the form into the data (keeping what the form does not model).
pub fn apply(form: &ItemForm, d: &mut Value) {
    if !d.is_object() {
        *d = json!({});
    }
    let m = d.as_object_mut().expect("an object");
    let kind = form.kind.as_str();
    let type_code = match kind {
        "weapon" => {
            if !form.range.trim().is_empty() && !form.properties.iter().any(|p| p == "T") {
                Some("R")
            } else {
                Some("M")
            }
        }
        "armor" => Some(form.armor_type.as_str()),
        "potion" => Some("P"),
        "ring" => Some("RG"),
        "rod" => Some("RD"),
        "scroll" => Some("SC"),
        "staff" => Some("ST"),
        "wand" => Some("WD"),
        "wondrous" => None,
        _ => Some("G"),
    };
    set_or_remove(m, "type", type_code.map(|t| Value::String(t.into())));
    set_or_remove(
        m,
        "wondrous",
        (kind == "wondrous").then_some(Value::Bool(true)),
    );
    set_or_remove(
        m,
        "rarity",
        Some(Value::String(if form.rarity.is_empty() {
            "none".into()
        } else {
            form.rarity.clone()
        })),
    );
    set_or_remove(m, "weight", n_opt(&form.weight));
    set_or_remove(
        m,
        "value",
        form.value_gp
            .trim()
            .parse::<f64>()
            .ok()
            .map(|gp| Value::from((gp * 100.0).round() as i64)),
    );
    let attune = if !form.attune {
        None
    } else if form.attune_text.trim().is_empty() {
        Some(Value::Bool(true))
    } else {
        Some(Value::String(form.attune_text.trim().to_string()))
    };
    set_or_remove(m, "reqAttune", attune);
    if !form.rich_entries {
        let paras: Vec<Value> = form
            .description
            .split("\n\n")
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(|p| Value::String(p.to_string()))
            .collect();
        set_or_remove(
            m,
            "entries",
            (!paras.is_empty()).then_some(Value::Array(paras)),
        );
    }
    if kind == "weapon" {
        m.insert("weapon".into(), Value::Bool(true));
        set_or_remove(m, "weaponCategory", s_opt(&form.weapon_category));
        set_or_remove(m, "bonusWeapon", s_opt(&form.bonus_weapon));
        set_or_remove(m, "dmg1", s_opt(&form.dmg1));
        set_or_remove(m, "dmgType", s_opt(&form.dmg_type));
        set_or_remove(m, "dmg2", s_opt(&form.dmg2));
        set_or_remove(m, "range", s_opt(&form.range));
        set_or_remove(
            m,
            "property",
            (!form.properties.is_empty()).then(|| {
                Value::Array(
                    form.properties
                        .iter()
                        .map(|p| Value::String(p.clone()))
                        .collect(),
                )
            }),
        );
    } else {
        for k in WEAPON_KEYS {
            m.remove(k);
        }
    }
    if kind == "armor" {
        m.insert("armor".into(), Value::Bool(true));
        set_or_remove(m, "ac", n_opt(&form.ac));
        set_or_remove(m, "bonusAc", s_opt(&form.bonus_ac));
        set_or_remove(m, "strength", s_opt(&form.strength));
        set_or_remove(m, "stealth", form.stealth.then_some(Value::Bool(true)));
    } else {
        for k in ARMOR_KEYS {
            m.remove(k);
        }
    }
}

/// The data of a new entry of a kind.
pub fn template(kind: &str) -> Value {
    match kind {
        "items" => json!({"type": "G", "rarity": "common", "entries": ["What it does."]}),
        "monsters" => json!({
            "size": ["M"], "type": "humanoid", "alignment": ["N"],
            "ac": [12], "hp": {"average": 11, "formula": "2d8 + 2"}, "speed": {"walk": 30},
            "str": 10, "dex": 12, "con": 12, "int": 10, "wis": 10, "cha": 10,
            "passive": 10, "languages": ["Common"], "cr": "1/2",
            "action": [{"name": "Club", "entries": ["{@atk mw} {@hit 2} to hit, reach 5 ft., one target. {@h}3 ({@damage 1d4}) bludgeoning damage."]}]
        }),
        _ => json!({
            "level": 1, "school": "V",
            "time": [{"number": 1, "unit": "action"}],
            "range": {"type": "point", "distance": {"type": "feet", "amount": 60}},
            "components": {"v": true, "s": true},
            "duration": [{"type": "instant"}],
            "entries": ["What it does."]
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_of_data() {
        assert_eq!(kind_of(&json!({"type": "M", "weapon": true})), "weapon");
        assert_eq!(kind_of(&json!({"type": "HA"})), "armor");
        assert_eq!(kind_of(&json!({"wondrous": true})), "wondrous");
        assert_eq!(
            kind_of(&json!({"type": "W"})),
            "wondrous",
            "the desktop's code"
        );
        assert_eq!(kind_of(&json!({"type": "RG|DMG"})), "ring");
        assert_eq!(kind_of(&json!({})), "gear");
    }

    #[test]
    fn a_weapon_round_trip() {
        let d = json!({"type": "M", "weapon": true, "weaponCategory": "martial", "dmg1": "2d6", "dmgType": "S",
            "property": ["2H", "H"], "rarity": "very rare", "reqAttune": "by a paladin", "weight": 6,
            "value": 2500000, "bonusWeapon": "+2", "entries": ["A greatsword.", "It hums."]});
        let f = form_of(&d);
        assert_eq!(
            (f.kind.as_str(), f.value_gp.as_str(), f.attune_text.as_str()),
            ("weapon", "25000", "by a paladin")
        );
        assert_eq!(f.description, "A greatsword.\n\nIt hums.");
        let mut out = d.clone();
        apply(&f, &mut out);
        assert_eq!(out, d, "unchanged form, unchanged data");
    }

    #[test]
    fn the_form_keeps_what_it_does_not_model() {
        let mut d = json!({"type": "RG", "rarity": "rare", "charges": 3, "attachedSpells": ["fire bolt"], "entries": ["A ring."]});
        let mut f = form_of(&d);
        f.rarity = "legendary".into();
        apply(&f, &mut d);
        assert_eq!(d["rarity"], "legendary");
        assert_eq!(d["charges"], 3);
        assert_eq!(d["attachedSpells"][0], "fire bolt");
    }

    #[test]
    fn changing_the_kind() {
        let mut d =
            json!({"type": "M", "weapon": true, "dmg1": "1d8", "dmgType": "S", "property": ["V"]});
        let mut f = form_of(&d);
        f.kind = "armor".into();
        f.armor_type = "MA".into();
        f.ac = "14".into();
        f.stealth = true;
        apply(&f, &mut d);
        assert_eq!(d["type"], "MA");
        assert_eq!(d["ac"], 14);
        assert!(d.get("dmg1").is_none() && d.get("weapon").is_none());
        f.kind = "wondrous".into();
        apply(&f, &mut d);
        assert_eq!(d["wondrous"], true);
        assert!(d.get("type").is_none() && d.get("ac").is_none());
    }

    #[test]
    fn ranged_attunement_value_and_rich_entries() {
        let mut d = json!({});
        let f = ItemForm {
            kind: "weapon".into(),
            range: "80/320".into(),
            dmg1: "1d8".into(),
            properties: vec!["A".into(), "2H".into()],
            attune: true,
            value_gp: "0.5".into(),
            description: "One.\n\nTwo.".into(),
            ..ItemForm::default()
        };
        apply(&f, &mut d);
        assert_eq!(d["type"], "R");
        assert_eq!(d["reqAttune"], true);
        assert_eq!(d["value"], 50);
        assert_eq!(d["entries"], json!(["One.", "Two."]));
        // Lists or tables in the entries: the form leaves them.
        let mut rich = json!({"entries": ["Text.", {"type": "list", "items": ["a"]}]});
        let mut f = form_of(&rich);
        assert!(f.rich_entries);
        f.description = "Changed.".into();
        apply(&f, &mut rich);
        assert_eq!(rich["entries"][1]["type"], "list");
    }

    #[test]
    fn templates_render() {
        for (kind, cat) in [
            ("items", "items"),
            ("monsters", "monsters"),
            ("spells", "spells"),
        ] {
            let html = mimir_render::detail::detail(cat, &template(kind));
            assert!(!html.contains("{@"), "{kind}");
        }
    }
}

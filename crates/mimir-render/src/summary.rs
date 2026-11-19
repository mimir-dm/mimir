//! The columns of the catalog's result tables (MIMIR-T-0723): for each
//! kind of entry, the headers and the cells, as plain text, from the
//! 5etools JSON. As the desktop's table configs.

use serde_json::Value;

use crate::tags::plain;

/// The kinds of catalog entry: (key, label, group).
pub const CATEGORIES: [(&str, &str, &str); 21] = [
    ("spells", "Spells", "Reference"),
    ("items", "Items", "Reference"),
    ("monsters", "Bestiary", "Reference"),
    ("conditions", "Conditions & diseases", "Reference"),
    ("actions", "Actions", "Reference"),
    ("deities", "Deities", "Reference"),
    ("languages", "Languages", "Reference"),
    ("rewards", "Rewards", "Reference"),
    ("psionics", "Psionics", "Reference"),
    ("vehicles", "Vehicles", "Reference"),
    ("classes", "Classes", "Player"),
    ("races", "Races", "Player"),
    ("backgrounds", "Backgrounds", "Player"),
    ("feats", "Feats", "Player"),
    ("optional-features", "Other options & features", "Player"),
    ("tables", "Tables", "Rules"),
    ("variant-rules", "Variant rules", "Rules"),
    ("cults", "Cults & boons", "DM tools"),
    ("objects", "Objects", "DM tools"),
    ("traps", "Traps", "DM tools"),
    ("hazards", "Hazards", "DM tools"),
];

/// The cross-reference type of a category ("monsters" → "creature").
pub fn ref_type(category: &str) -> &'static str {
    match category {
        "spells" => "spell",
        "items" => "item",
        "monsters" => "creature",
        "conditions" => "condition",
        "actions" => "action",
        "deities" => "deity",
        "languages" => "language",
        "rewards" => "reward",
        "psionics" => "psionic",
        "vehicles" => "vehicle",
        "classes" => "class",
        "races" => "race",
        "backgrounds" => "background",
        "feats" => "feat",
        "optional-features" => "optfeature",
        "tables" => "table",
        "variant-rules" => "variantrule",
        "cults" => "cult",
        "objects" => "object",
        "traps" => "trap",
        "hazards" => "hazard",
        _ => "",
    }
}

/// The category of a cross-reference type (the reverse of [`ref_type`]).
pub fn category_of(ref_type: &str) -> Option<&'static str> {
    Some(match ref_type {
        "spell" => "spells",
        "item" => "items",
        "creature" | "monster" => "monsters",
        "condition" | "disease" | "status" => "conditions",
        "action" => "actions",
        "deity" => "deities",
        "language" => "languages",
        "reward" => "rewards",
        "psionic" => "psionics",
        "vehicle" => "vehicles",
        "class" => "classes",
        "race" => "races",
        "background" => "backgrounds",
        "feat" => "feats",
        "optfeature" => "optional-features",
        "table" => "tables",
        "variantrule" => "variant-rules",
        "cult" | "boon" => "cults",
        "object" => "objects",
        "trap" => "traps",
        "hazard" => "hazards",
        _ => return None,
    })
}

/// The column headers of a category (after Name and Source).
pub fn headers(category: &str) -> &'static [&'static str] {
    match category {
        "spells" => &["Level", "School", "Time", "Range", "Components"],
        "items" => &["Type", "Rarity", "Value", "Weight"],
        "monsters" => &["CR", "Type", "Size", "HP", "AC"],
        "classes" => &["Hit die", "Primary ability", "Spellcasting"],
        "races" => &["Size", "Speed", "Abilities"],
        "backgrounds" => &["Skills"],
        "feats" => &["Prerequisite"],
        "optional-features" => &["Type", "Prerequisite"],
        "actions" => &["Time"],
        "deities" => &["Title", "Pantheon", "Alignment", "Domains"],
        "objects" => &["Type", "Size", "AC", "HP"],
        "traps" | "hazards" => &["Kind"],
        "languages" => &["Type", "Script", "Typical speakers"],
        "rewards" => &["Type"],
        "variant-rules" => &["Type"],
        "vehicles" => &["Type", "Size"],
        "cults" => &["Kind"],
        "psionics" => &["Type", "Order"],
        _ => &[],
    }
}

fn text(v: &Value) -> String {
    match v {
        Value::String(s) => plain(s),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// A 5etools code without its source ("M|XPHB" → "M").
fn code(v: &Value) -> String {
    v.as_str()
        .unwrap_or("")
        .split('|')
        .next()
        .unwrap_or("")
        .to_string()
}

pub fn size_name(code: &str) -> &str {
    match code {
        "T" => "Tiny",
        "S" => "Small",
        "M" => "Medium",
        "L" => "Large",
        "H" => "Huge",
        "G" => "Gargantuan",
        "V" => "Varies",
        other => other,
    }
}

fn sizes(v: &Value) -> String {
    match v {
        Value::Array(l) => l
            .iter()
            .filter_map(Value::as_str)
            .map(size_name)
            .collect::<Vec<_>>()
            .join(" or "),
        Value::String(s) => size_name(s).to_string(),
        _ => String::new(),
    }
}

pub fn school_name(code: &str) -> &str {
    match code {
        "A" => "Abjuration",
        "C" => "Conjuration",
        "D" => "Divination",
        "E" => "Enchantment",
        "V" => "Evocation",
        "I" => "Illusion",
        "N" => "Necromancy",
        "T" => "Transmutation",
        "P" => "Psionic",
        other => other,
    }
}

/// "Cantrip", "1st", "2nd", …
pub fn spell_level(level: i64) -> String {
    match level {
        0 => "Cantrip".into(),
        1 => "1st".into(),
        2 => "2nd".into(),
        3 => "3rd".into(),
        n => format!("{n}th"),
    }
}

/// A spell's casting time: "1 action", "1 minute".
pub fn spell_time(v: &Value) -> String {
    v.as_array()
        .map(|l| {
            l.iter()
                .map(|t| {
                    let n = t["number"].as_i64().unwrap_or(1);
                    let unit = t["unit"].as_str().unwrap_or("");
                    let unit = match unit {
                        "bonus" => "bonus action",
                        u => u,
                    };
                    if n == 1 {
                        format!("1 {unit}")
                    } else {
                        format!("{n} {unit}s")
                    }
                })
                .collect::<Vec<_>>()
                .join(" or ")
        })
        .unwrap_or_default()
}

/// A spell's range: "60 feet", "Self (15-foot cone)", "Touch".
pub fn spell_range(r: &Value) -> String {
    let kind = r["type"].as_str().unwrap_or("");
    let d = &r["distance"];
    let dtype = d["type"].as_str().unwrap_or("");
    let amount = d["amount"].as_i64();
    let dist = match (dtype, amount) {
        ("self", _) => "Self".to_string(),
        ("touch", _) => "Touch".to_string(),
        ("sight", _) => "Sight".to_string(),
        ("unlimited", _) => "Unlimited".to_string(),
        ("feet", Some(n)) => format!("{n} feet"),
        ("miles", Some(1)) => "1 mile".to_string(),
        ("miles", Some(n)) => format!("{n} miles"),
        (t, Some(n)) => format!("{n} {t}"),
        (t, None) => t.to_string(),
    };
    match kind {
        "point" | "" => dist,
        "special" => "Special".into(),
        shape => match amount {
            Some(n) => format!("Self ({n}-foot {shape})"),
            None => format!("Self ({shape})"),
        },
    }
}

/// A spell's components: "V, S, M".
pub fn spell_components(c: &Value) -> String {
    let mut out = Vec::new();
    for (k, l) in [("v", "V"), ("s", "S"), ("m", "M"), ("r", "R")] {
        if !c[k].is_null() && c[k] != Value::Bool(false) {
            out.push(l);
        }
    }
    out.join(", ")
}

/// The name of an item type code.
pub fn item_type_name(code: &str) -> &str {
    match code {
        "M" => "Melee weapon",
        "R" => "Ranged weapon",
        "A" | "AF" => "Ammunition",
        "LA" => "Light armor",
        "MA" => "Medium armor",
        "HA" => "Heavy armor",
        "S" => "Shield",
        "G" => "Adventuring gear",
        "P" => "Potion",
        "RG" => "Ring",
        "RD" => "Rod",
        "SC" => "Scroll",
        "ST" => "Staff",
        "WD" => "Wand",
        "INS" => "Instrument",
        "AT" => "Artisan's tools",
        "T" => "Tools",
        "GS" => "Gaming set",
        "SCF" => "Spellcasting focus",
        "TAH" => "Tack and harness",
        "TG" => "Trade good",
        "MNT" => "Mount",
        "VEH" | "AIR" | "SHP" => "Vehicle",
        "FD" => "Food and drink",
        "$" | "$C" | "$G" | "$A" => "Treasure",
        "EXP" => "Explosive",
        // The desktop's homebrew form wrote wondrous items as "W".
        "W" => "Wondrous item",
        "OTH" => "Other",
        other => other,
    }
}

/// An item's kind: its type, or "Wondrous item".
pub fn item_kind(i: &Value) -> String {
    let t = code(&i["type"]);
    if !t.is_empty() {
        return item_type_name(&t).to_string();
    }
    if i["wondrous"].as_bool() == Some(true) {
        return "Wondrous item".into();
    }
    if i["weapon"].as_bool() == Some(true) {
        return "Weapon".into();
    }
    String::new()
}

/// A price in copper as coins: "15 gp", "5 sp", "2 cp".
pub fn coins(cp: i64) -> String {
    if cp.rem_euclid(100) == 0 {
        format!("{} gp", group(cp / 100))
    } else if cp.rem_euclid(10) == 0 {
        format!("{} sp", group(cp / 10))
    } else {
        format!("{cp} cp")
    }
}

fn group(n: i64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A rarity as shown: "—" for none.
pub fn rarity(v: &Value) -> String {
    match v.as_str() {
        None | Some("none") | Some("") => "—".into(),
        Some(r) => {
            let mut c = r.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect())
                .unwrap_or_default()
        }
    }
}

/// A challenge rating: "1/4", "17".
pub fn cr(v: &Value) -> String {
    match v {
        Value::Object(o) => o.get("cr").map(text).unwrap_or_default(),
        other => text(other),
    }
}

/// A creature type: "humanoid (goblinoid)".
pub fn creature_type(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(o) => {
            let base = match o.get("type") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Object(c)) => c
                    .get("choose")
                    .and_then(Value::as_array)
                    .map(|l| {
                        l.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(" or ")
                    })
                    .unwrap_or_default(),
                _ => String::new(),
            };
            let tags: Vec<String> = o
                .get("tags")
                .and_then(Value::as_array)
                .map(|l| {
                    l.iter()
                        .filter_map(|t| {
                            t.as_str()
                                .map(String::from)
                                .or_else(|| t["tag"].as_str().map(String::from))
                        })
                        .collect()
                })
                .unwrap_or_default();
            if tags.is_empty() {
                base
            } else {
                format!("{base} ({})", tags.join(", "))
            }
        }
        _ => String::new(),
    }
}

/// The first armor class: "15".
pub fn first_ac(v: &Value) -> String {
    match v.as_array().and_then(|l| l.first()) {
        Some(Value::Number(n)) => n.to_string(),
        Some(o @ Value::Object(_)) => o.get("ac").map(text).unwrap_or_default(),
        _ => match v {
            Value::Number(n) => n.to_string(),
            _ => String::new(),
        },
    }
}

/// Ability bonuses: "Str +2, Cha +1".
pub fn ability_bonuses(v: &Value) -> String {
    let Some(first) = v
        .as_array()
        .and_then(|l| l.first())
        .and_then(Value::as_object)
    else {
        return String::new();
    };
    let mut out = Vec::new();
    for k in ["str", "dex", "con", "int", "wis", "cha"] {
        if let Some(n) = first.get(k).and_then(Value::as_i64) {
            let mut name = k.to_string();
            name[..1].make_ascii_uppercase();
            out.push(format!("{name} {n:+}"));
        }
    }
    if first.contains_key("choose") {
        out.push("choice".into());
    }
    out.join(", ")
}

/// A speed: "30 ft." (walking), or the object's walk.
pub fn speed(v: &Value) -> String {
    match v {
        Value::Number(n) => format!("{n} ft."),
        Value::Object(o) => o
            .get("walk")
            .map(|w| match w {
                Value::Object(x) => format!("{} ft.", text(&x["number"])),
                w => format!("{} ft.", text(w)),
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// The keys of a 5etools proficiency object that are granted (true).
fn granted(v: &Value) -> String {
    v.as_array()
        .and_then(|l| l.first())
        .and_then(Value::as_object)
        .map(|o| {
            o.iter()
                .filter(|(_, x)| x.as_bool() == Some(true))
                .map(|(k, _)| crate::tags::plain(k.split('|').next().unwrap_or(k)))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

/// A prerequisite, short: "Level 4", "Strength 13", "Spellcasting".
pub fn prerequisite(v: &Value) -> String {
    let Some(list) = v.as_array() else {
        return String::new();
    };
    let parts: Vec<String> = list
        .iter()
        .flat_map(|p| {
            let mut out = Vec::new();
            if let Some(l) = p.get("level") {
                match l {
                    Value::Number(n) => out.push(format!("Level {n}")),
                    Value::Object(o) => {
                        let lvl = o.get("level").map(text).unwrap_or_default();
                        match o.get("class").and_then(|c| c["name"].as_str()) {
                            Some(c) => out.push(format!("{c} {lvl}")),
                            None => out.push(format!("Level {lvl}")),
                        }
                    }
                    _ => {}
                }
            }
            if let Some(ab) = p.get("ability").and_then(Value::as_array) {
                for a in ab.iter().filter_map(Value::as_object) {
                    for (k, n) in a {
                        out.push(format!("{} {}", crate::tags::ability_name(k), text(n)));
                    }
                }
            }
            if let Some(r) = p.get("race").and_then(Value::as_array) {
                let names: Vec<String> = r
                    .iter()
                    .filter_map(|x| x["name"].as_str())
                    .map(String::from)
                    .collect();
                if !names.is_empty() {
                    out.push(names.join(" or "));
                }
            }
            if p.get("spellcasting").is_some() || p.get("spellcasting2020").is_some() {
                out.push("Spellcasting".into());
            }
            if let Some(f) = p.get("feat").and_then(Value::as_array) {
                for x in f.iter().filter_map(Value::as_str) {
                    out.push(crate::tags::plain(x.split('|').next().unwrap_or(x)));
                }
            }
            if let Some(o) = p.get("other").and_then(Value::as_str) {
                out.push(plain(o));
            }
            out
        })
        .collect();
    parts.join("; ")
}

/// A feature type code as words ("EI" → "Eldritch Invocation").
pub fn feature_type(code: &str) -> &str {
    match code {
        "EI" => "Eldritch Invocation",
        "FS:F" | "FS:B" | "FS:R" | "FS:P" => "Fighting Style",
        "MM" => "Metamagic",
        "MV:B" => "Battle Master Maneuver",
        "MV:C2-UA" | "MV" => "Maneuver",
        "AI" => "Artificer Infusion",
        "AS" | "AS:V1-UA" | "AS:V2-UA" => "Arcane Shot",
        "ED" => "Elemental Discipline",
        "PB" => "Pact Boon",
        "RN" => "Rune Knight Rune",
        "OTH" => "Other",
        other => other,
    }
}

/// The cells of a row (as [`headers`]).
pub fn cells(category: &str, d: &Value) -> Vec<String> {
    match category {
        "spells" => vec![
            spell_level(d["level"].as_i64().unwrap_or(0)),
            school_name(d["school"].as_str().unwrap_or("")).to_string(),
            spell_time(&d["time"]),
            spell_range(&d["range"]),
            spell_components(&d["components"]),
        ],
        "items" => vec![
            item_kind(d),
            rarity(&d["rarity"]),
            d["value"].as_i64().map(coins).unwrap_or_else(|| "—".into()),
            d["weight"]
                .as_f64()
                .map(|w| format!("{w} lb."))
                .unwrap_or_else(|| "—".into()),
        ],
        "monsters" => vec![
            cr(&d["cr"]),
            creature_type(&d["type"]),
            sizes(&d["size"]),
            text(&d["hp"]["average"]),
            first_ac(&d["ac"]),
        ],
        "classes" => vec![
            d["hd"]["faces"]
                .as_i64()
                .map(|f| format!("d{f}"))
                .unwrap_or_default(),
            d["primaryAbility"]
                .as_array()
                .and_then(|l| l.first())
                .and_then(Value::as_object)
                .map(|o| {
                    o.keys()
                        .map(|k| crate::tags::ability_name(k).to_string())
                        .collect::<Vec<_>>()
                        .join(" or ")
                })
                .unwrap_or_default(),
            d["spellcastingAbility"]
                .as_str()
                .map(|a| crate::tags::ability_name(a).to_string())
                .unwrap_or_else(|| "—".into()),
        ],
        "races" => vec![
            sizes(&d["size"]),
            speed(&d["speed"]),
            ability_bonuses(&d["ability"]),
        ],
        "backgrounds" => vec![granted(&d["skillProficiencies"])],
        "feats" => vec![prerequisite(&d["prerequisite"])],
        "optional-features" => vec![
            d["featureType"]
                .as_array()
                .map(|l| {
                    l.iter()
                        .filter_map(Value::as_str)
                        .map(feature_type)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default(),
            prerequisite(&d["prerequisite"]),
        ],
        "actions" => vec![d["time"]
            .as_array()
            .map(|l| {
                l.iter()
                    .map(|t| match t {
                        Value::String(s) => plain(s),
                        o => spell_time(&Value::Array(vec![o.clone()])),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()],
        "deities" => vec![
            text(&d["title"]),
            text(&d["pantheon"]),
            d["alignment"]
                .as_array()
                .map(|l| {
                    l.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("")
                })
                .unwrap_or_default(),
            d["domains"]
                .as_array()
                .map(|l| {
                    l.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default(),
        ],
        "objects" => vec![
            text(&d["objectType"]),
            sizes(&d["size"]),
            first_ac(&d["ac"]),
            text(&d["hp"]),
        ],
        "traps" | "hazards" => vec![text(&d["trapHazType"])],
        "languages" => vec![
            text(&d["type"]),
            text(&d["script"]),
            d["typicalSpeakers"]
                .as_array()
                .map(|l| {
                    l.iter()
                        .filter_map(Value::as_str)
                        .map(plain)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default(),
        ],
        "rewards" => vec![text(&d["type"])],
        "variant-rules" => vec![text(&d["ruleType"])],
        "vehicles" => vec![text(&d["vehicleType"]), sizes(&d["size"])],
        "cults" => vec![if d["type"].is_string() {
            text(&d["type"])
        } else if d.get("goal").is_some() {
            "Cult".into()
        } else {
            "Boon".into()
        }],
        "psionics" => vec![text(&d["type"]), text(&d["order"])],
        _ => Vec::new(),
    }
    .into_iter()
    .map(|c| if c.is_empty() { "—".into() } else { c })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn every_category_has_its_columns() {
        for (key, _, _) in CATEGORIES {
            assert_eq!(cells(key, &json!({})).len(), headers(key).len(), "{key}");
            assert_eq!(category_of(ref_type(key)), Some(key), "{key}");
        }
    }

    #[test]
    fn spells() {
        let s = json!({"level": 3, "school": "V", "time": [{"number": 1, "unit": "action"}],
            "range": {"type": "point", "distance": {"type": "feet", "amount": 150}},
            "components": {"v": true, "s": true, "m": "a tiny ball of bat guano"}});
        assert_eq!(
            cells("spells", &s),
            ["3rd", "Evocation", "1 action", "150 feet", "V, S, M"]
        );
        let cone = json!({"type": "cone", "distance": {"type": "feet", "amount": 15}});
        assert_eq!(spell_range(&cone), "Self (15-foot cone)");
        assert_eq!(spell_level(0), "Cantrip");
    }

    #[test]
    fn items_and_monsters() {
        let i = json!({"type": "M|XPHB", "rarity": "none", "value": 1500, "weight": 3});
        assert_eq!(cells("items", &i), ["Melee weapon", "—", "15 gp", "3 lb."]);
        assert_eq!(coins(150_000), "1,500 gp");
        assert_eq!(coins(5), "5 cp");
        let w = json!({"wondrous": true, "rarity": "very rare"});
        assert_eq!(cells("items", &w)[..2], ["Wondrous item", "Very rare"]);
        let m = json!({"cr": "1/4", "type": {"type": "humanoid", "tags": ["goblinoid"]}, "size": ["S"],
            "hp": {"average": 7}, "ac": [{"ac": 15, "from": ["leather armor"]}]});
        assert_eq!(
            cells("monsters", &m),
            ["1/4", "humanoid (goblinoid)", "Small", "7", "15"]
        );
    }

    #[test]
    fn player_options() {
        let r = json!({"size": ["M"], "speed": 30, "ability": [{"dex": 2, "int": 1}]});
        assert_eq!(cells("races", &r), ["Medium", "30 ft.", "Dex +2, Int +1"]);
        let f = json!({"prerequisite": [{"ability": [{"str": 13}]}, {"level": 4}]});
        assert_eq!(cells("feats", &f), ["Strength 13; Level 4"]);
        let b = json!({"skillProficiencies": [{"insight": true, "religion": true}]});
        assert_eq!(cells("backgrounds", &b), ["insight, religion"]);
        let o = json!({"featureType": ["EI"], "prerequisite": [{"level": {"level": 5, "class": {"name": "Warlock"}}}]});
        assert_eq!(
            cells("optional-features", &o),
            ["Eldritch Invocation", "Warlock 5"]
        );
    }
}

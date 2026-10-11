//! The detail view of a catalog entry (MIMIR-T-0723): a port of the
//! desktop's per-kind formatters (spell, monster, item, class, race,
//! feat, background, condition, action, optional feature), one set for
//! the catalog and the reader's cross-references (the desktop had two).
//! The other kinds use one generic view: their table columns as
//! properties, then their entries. Images are `data-image-path`
//! placeholders, loaded by the web app as the book's.

use serde_json::Value;

use crate::entries::{entries, s};
use crate::esc;
use crate::summary::{self, cells, headers};
use crate::tags::{ability_name, plain, tags};

/// The detail of an entry of `category` (its 5etools JSON).
pub fn detail(category: &str, v: &Value) -> String {
    let body = match category {
        "spells" => spell(v),
        "monsters" => monster(v),
        "items" => item(v),
        "classes" => class(v),
        "races" => race(v),
        "feats" => feat(v),
        "backgrounds" => background(v),
        "optional-features" => optional_feature(v),
        _ => generic(category, v),
    };
    format!(
        r#"<div class="detail detail--{category}">{body}{}</div>"#,
        footer(v)
    )
}

// ------------------------------------------------------------------ layout

fn subtitle(text_html: &str, badges: &[(&str, String)]) -> String {
    let b: String = badges
        .iter()
        .map(|(class, label)| {
            format!(
                r#"<span class="detail-badge {class}">{}</span>"#,
                esc(label)
            )
        })
        .collect();
    format!(r#"<div class="detail-head"><p class="detail-sub">{text_html}</p>{b}</div>"#)
}

/// Properties: (label, value HTML); empty values are left out.
fn props(list: &[(&str, String)]) -> String {
    let items: String = list
        .iter()
        .filter(|(_, v)| !v.is_empty() && v != "—")
        .map(|(l, v)| format!("<dt>{}</dt><dd>{v}</dd>", esc(l)))
        .collect();
    if items.is_empty() {
        String::new()
    } else {
        format!(r#"<dl class="detail-props">{items}</dl>"#)
    }
}

fn section(title: &str, body_html: &str) -> String {
    if body_html.trim().is_empty() {
        return String::new();
    }
    format!(
        r#"<section class="detail-section"><h4>{}</h4>{body_html}</section>"#,
        esc(title)
    )
}

fn footer(v: &Value) -> String {
    let src = s(&v["source"]);
    if src.is_empty() {
        return String::new();
    }
    let page = v["page"]
        .as_i64()
        .map(|p| format!(", p. {p}"))
        .unwrap_or_default();
    format!(r#"<p class="detail-source">Source: {}{page}</p>"#, esc(src))
}

/// The fluff's images, as placeholders.
fn images(v: &Value) -> String {
    let list = v["fluff"]["images"]
        .as_array()
        .or_else(|| v["fluffImages"].as_array());
    let Some(list) = list else {
        return String::new();
    };
    let imgs: String = list
        .iter()
        .filter_map(|i| i["href"]["path"].as_str())
        .map(|p| {
            format!(
                r#"<div class="image-wrapper detail-image" data-image-path="{}"></div>"#,
                esc(p)
            )
        })
        .collect();
    if imgs.is_empty() {
        imgs
    } else {
        format!(r#"<div class="detail-images">{imgs}</div>"#)
    }
}

/// The fluff's lore entries.
fn lore(v: &Value) -> String {
    let e = &v["fluff"]["entries"];
    let e = if e.is_null() { &v["fluffEntries"] } else { e };
    section("Lore", &entries(e))
}

fn cap(word: &str) -> String {
    let mut c = word.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

fn join_strs(v: &Value, sep: &str) -> String {
    match v {
        Value::Array(l) => l
            .iter()
            .map(|x| match x {
                Value::String(t) => tags(t),
                other => esc(&plain(&other.to_string())),
            })
            .collect::<Vec<_>>()
            .join(sep),
        Value::String(t) => tags(t),
        _ => String::new(),
    }
}

// ------------------------------------------------------------------ spells

fn spell_duration(v: &Value) -> String {
    let Some(d) = v.as_array().and_then(|l| l.first()) else {
        return "Instantaneous".into();
    };
    match d["type"].as_str().unwrap_or("") {
        "instant" => "Instantaneous".into(),
        "permanent" => {
            let ends = d["ends"].as_array().map(|e| {
                e.iter()
                    .filter_map(Value::as_str)
                    .map(|x| match x {
                        "dispel" => "dispelled",
                        "trigger" => "triggered",
                        other => other,
                    })
                    .collect::<Vec<_>>()
                    .join(" or ")
            });
            match ends {
                Some(e) if !e.is_empty() => format!("Until {e}"),
                _ => "Permanent".into(),
            }
        }
        "special" => "Special".into(),
        "timed" => {
            let amount = d["duration"]["amount"].as_i64().unwrap_or(1);
            let unit = d["duration"]["type"].as_str().unwrap_or("");
            let unit = if amount == 1 {
                unit.to_string()
            } else {
                format!("{unit}s")
            };
            let conc = if d["concentration"].as_bool() == Some(true) {
                "Concentration, up to "
            } else {
                ""
            };
            format!("{conc}{amount} {unit}")
        }
        other => cap(other),
    }
}

fn casting_time(v: &Value) -> String {
    let base = summary::spell_time(v);
    match v
        .as_array()
        .and_then(|l| l.first())
        .and_then(|t| t["condition"].as_str())
    {
        Some(c) => format!("{base}, {}", plain(c)),
        None => base,
    }
}

fn components(c: &Value) -> String {
    let mut parts = Vec::new();
    if c["v"].as_bool() == Some(true) {
        parts.push("V".to_string());
    }
    if c["s"].as_bool() == Some(true) {
        parts.push("S".to_string());
    }
    match &c["m"] {
        Value::String(m) => parts.push(format!("M ({})", plain(m))),
        Value::Object(o) => parts.push(format!(
            "M ({})",
            plain(o.get("text").and_then(Value::as_str).unwrap_or(""))
        )),
        Value::Bool(true) => parts.push("M".into()),
        _ => {}
    }
    if parts.is_empty() {
        "None".into()
    } else {
        parts.join(", ")
    }
}

fn misc_tag(code: &str) -> Option<&'static str> {
    Some(match code {
        "SCL" => "Scaling",
        "HEL" => "Healing",
        "SMN" => "Summoning",
        "LGT" => "Light",
        "THP" => "Temp HP",
        "UBA" => "Bonus Action",
        "PRM" => "Permanent",
        "OBJ" => "Affects Objects",
        "FMV" => "Forced Movement",
        "MAC" => "Multiple Attacks",
        _ => return None,
    })
}

fn area(code: &str) -> &str {
    match code {
        "ST" => "Single target",
        "MT" => "Multiple targets",
        "S" => "Sphere",
        "C" => "Cone",
        "L" => "Line",
        "Y" => "Cylinder",
        "H" => "Hemisphere",
        "Q" => "Square",
        "R" => "Rectangle",
        "N" => "Square/Rectangle",
        "W" => "Wall",
        other => other,
    }
}

fn spell_classes(v: &Value) -> String {
    let list = v["classes"]["fromClassList"].as_array();
    list.map(|l| {
        l.iter()
            .filter_map(|c| c["name"].as_str())
            .map(esc)
            .collect::<Vec<_>>()
            .join(", ")
    })
    .unwrap_or_default()
}

fn words(v: &Value) -> String {
    v.as_array()
        .map(|l| {
            l.iter()
                .filter_map(Value::as_str)
                .map(|x| esc(&cap(x)))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

fn spell(v: &Value) -> String {
    let level = v["level"].as_i64().unwrap_or(0);
    let school = summary::school_name(s(&v["school"]));
    let line = if level == 0 {
        format!("{school} cantrip")
    } else {
        format!(
            "{}-level {}",
            summary::spell_level(level),
            school.to_lowercase()
        )
    };
    let mut badges = Vec::new();
    if v["meta"]["ritual"].as_bool() == Some(true) {
        badges.push(("ritual", "Ritual".to_string()));
    }
    let conc = v["duration"]
        .as_array()
        .is_some_and(|l| l.iter().any(|d| d["concentration"].as_bool() == Some(true)));
    if conc {
        badges.push(("concentration", "Concentration".to_string()));
    }
    if let Some(tags_list) = v["miscTags"].as_array() {
        for t in tags_list.iter().filter_map(Value::as_str) {
            if let Some(l) = misc_tag(t) {
                badges.push(("misc", l.to_string()));
            }
        }
    }
    let mut html = subtitle(&esc(&line), &badges);
    html += &props(&[
        ("Casting time", esc(&casting_time(&v["time"]))),
        ("Range", esc(&summary::spell_range(&v["range"]))),
        ("Components", esc(&components(&v["components"]))),
        ("Duration", esc(&spell_duration(&v["duration"]))),
        ("Classes", spell_classes(v)),
    ]);
    // Combat mechanics.
    let attack = v["spellAttack"]
        .as_array()
        .and_then(|l| l.first())
        .and_then(Value::as_str)
        .map(|a| match a {
            "M" => "Melee spell attack",
            "R" => "Ranged spell attack",
            _ => "Spell attack",
        })
        .unwrap_or("")
        .to_string();
    html += &section(
        "Combat",
        &props(&[
            ("Save", words(&v["savingThrow"])),
            ("Attack", attack),
            ("Damage", words(&v["damageInflict"])),
            ("Conditions", words(&v["conditionInflict"])),
            (
                "Area",
                v["areaTags"]
                    .as_array()
                    .map(|l| {
                        l.iter()
                            .filter_map(Value::as_str)
                            .map(area)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default(),
            ),
        ]),
    );
    if level == 0 {
        if let Some(sc) = v["scalingLevelDice"].as_object() {
            let label = sc.get("label").and_then(Value::as_str).unwrap_or("damage");
            let steps: Vec<String> = ["1", "5", "11", "17"]
                .iter()
                .filter_map(|l| {
                    sc.get("scaling")
                        .and_then(|x| x.get(*l))
                        .and_then(Value::as_str)
                        .map(|d| {
                            if *l == "1" {
                                esc(d)
                            } else {
                                format!("{} ({l}th)", esc(d))
                            }
                        })
                })
                .collect();
            html += &section(
                "Cantrip scaling",
                &format!("<p>{}: {}</p>", esc(&cap(label)), steps.join(" → ")),
            );
        }
    }
    html += &section("Description", &entries(&v["entries"]));
    if let Some(h) = v["entriesHigherLevel"].as_array() {
        let inner: String = h.iter().map(|e| entries(&e["entries"])).collect();
        let title = if level == 0 {
            "Cantrip upgrade"
        } else {
            "At higher levels"
        };
        html += &section(title, &inner);
    }
    html
}

// ------------------------------------------------------------------ monsters

fn modifier(score: i64) -> String {
    let m = (score - 10).div_euclid(2);
    if m >= 0 {
        format!("+{m}")
    } else {
        m.to_string()
    }
}

fn alignment_word(a: &str) -> &str {
    match a {
        "L" => "lawful",
        "N" => "neutral",
        "C" => "chaotic",
        "G" => "good",
        "E" => "evil",
        "U" => "unaligned",
        "A" => "any alignment",
        other => other,
    }
}

fn alignment(v: &Value) -> String {
    let map = alignment_word;
    match v {
        Value::Array(l) => {
            let parts: Vec<String> = l
                .iter()
                .map(|x| match x {
                    Value::String(a) => map(a).to_string(),
                    Value::Object(o) => o.get("alignment").map(alignment).unwrap_or_default(),
                    _ => String::new(),
                })
                .collect();
            let joined = parts.join(" ");
            if joined == "neutral neutral" {
                "neutral".into()
            } else {
                joined
            }
        }
        Value::String(a) => map(a).to_string(),
        _ => String::new(),
    }
}

fn armor_class(v: &Value) -> String {
    let Some(list) = v.as_array() else {
        return summary::first_ac(v);
    };
    list.iter()
        .map(|a| match a {
            Value::Number(n) => n.to_string(),
            Value::Object(o) => {
                let ac = o.get("ac").map(|x| x.to_string()).unwrap_or_default();
                let from = o.get("from").and_then(Value::as_array).map(|f| {
                    f.iter()
                        .filter_map(Value::as_str)
                        .map(plain)
                        .collect::<Vec<_>>()
                        .join(", ")
                });
                let cond = o.get("condition").and_then(Value::as_str).map(plain);
                let mut out = ac;
                if let Some(f) = from.filter(|f| !f.is_empty()) {
                    out += &format!(" ({f})");
                }
                if let Some(c) = cond {
                    out += &format!(" {c}");
                }
                out
            }
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn hit_points(v: &Value) -> String {
    match (v["average"].as_i64(), v["formula"].as_str()) {
        (Some(a), Some(f)) => format!("{a} ({f})"),
        (Some(a), None) => a.to_string(),
        _ => v["special"].as_str().map(plain).unwrap_or_default(),
    }
}

fn speeds(v: &Value) -> String {
    let Some(o) = v.as_object() else {
        return summary::speed(v);
    };
    let one = |x: &Value| match x {
        Value::Number(n) => format!("{n} ft."),
        Value::Object(o) => format!(
            "{} ft.{}",
            o.get("number").map(|n| n.to_string()).unwrap_or_default(),
            o.get("condition")
                .and_then(Value::as_str)
                .map(|c| format!(" {}", plain(c)))
                .unwrap_or_default()
        ),
        _ => String::new(),
    };
    let mut parts = Vec::new();
    if let Some(w) = o.get("walk") {
        parts.push(one(w));
    }
    for k in ["burrow", "climb", "fly", "swim"] {
        if let Some(x) = o.get(k) {
            let hover = if k == "fly" && o.get("canHover").and_then(Value::as_bool) == Some(true) {
                " (hover)"
            } else {
                ""
            };
            parts.push(format!("{k} {}{hover}", one(x)));
        }
    }
    parts.join(", ")
}

fn bonuses(v: &Value, title_case: bool) -> String {
    v.as_object()
        .map(|o| {
            o.iter()
                .map(|(k, x)| {
                    let name = if title_case {
                        k.split(' ').map(cap).collect::<Vec<_>>().join(" ")
                    } else {
                        cap(k)
                    };
                    format!(
                        "{name} {}",
                        plain(
                            &x.as_str()
                                .map(String::from)
                                .unwrap_or_else(|| x.to_string())
                        )
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

/// Damage resistances and the like: strings, or objects with a list and a
/// note.
fn damage_list(v: &Value, key: &str) -> String {
    match v {
        Value::Array(l) => l
            .iter()
            .map(|x| match x {
                Value::String(t) => plain(t),
                Value::Object(o) => {
                    let inner = o.get(key).map(|i| damage_list(i, key)).unwrap_or_default();
                    let pre = o
                        .get("preNote")
                        .and_then(Value::as_str)
                        .map(|n| format!("{n} "))
                        .unwrap_or_default();
                    let note = o
                        .get("note")
                        .and_then(Value::as_str)
                        .map(|n| format!(" {n}"))
                        .unwrap_or_default();
                    format!("{pre}{inner}{note}")
                }
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join(", "),
        _ => String::new(),
    }
}

fn named_entries(list: &Value) -> String {
    let Some(items) = list.as_array() else {
        return String::new();
    };
    items
        .iter()
        .map(|t| {
            let name = t["name"].as_str().map(tags).unwrap_or_default();
            let body: Vec<String> = t["entries"]
                .as_array()
                .map(|l| {
                    l.iter()
                        .map(|e| match e.as_str() {
                            Some(x) => tags(x),
                            None => crate::entries::entry(e),
                        })
                        .collect()
                })
                .unwrap_or_default();
            let (first, rest) = body
                .split_first()
                .map(|(f, r)| (f.clone(), r.to_vec()))
                .unwrap_or_default();
            let head = if name.is_empty() {
                String::new()
            } else {
                format!("<strong><em>{name}.</em></strong> ")
            };
            format!(
                r#"<div class="detail-trait"><p>{head}{first}</p>{}</div>"#,
                rest.join("")
            )
        })
        .collect()
}

fn spellcasting(list: &Value) -> String {
    let Some(items) = list.as_array() else {
        return String::new();
    };
    items
        .iter()
        .map(|sc| {
            let mut html = format!(
                "<p><strong><em>{}.</em></strong> {}</p>",
                tags(s(&sc["name"])),
                sc["headerEntries"]
                    .as_array()
                    .map(|l| l
                        .iter()
                        .filter_map(Value::as_str)
                        .map(tags)
                        .collect::<Vec<_>>()
                        .join(" "))
                    .unwrap_or_default()
            );
            let mut lines = Vec::new();
            if let Some(w) = sc["will"].as_array() {
                lines.push(format!(
                    "At will: {}",
                    join_strs(&Value::Array(w.clone()), ", ")
                ));
            }
            if let Some(d) = sc["daily"].as_object() {
                for (k, l) in d {
                    let n = k.trim_end_matches('e');
                    let each = if k.ends_with('e') { " each" } else { "" };
                    lines.push(format!("{n}/day{each}: {}", join_strs(l, ", ")));
                }
            }
            if let Some(spells) = sc["spells"].as_object() {
                for (lvl, x) in spells {
                    let label = if lvl == "0" {
                        "Cantrips (at will)".to_string()
                    } else {
                        let slots = x["slots"]
                            .as_i64()
                            .map(|n| format!(" ({n} slots)"))
                            .unwrap_or_default();
                        format!(
                            "{} level{slots}",
                            summary::spell_level(lvl.parse().unwrap_or(0))
                        )
                    };
                    lines.push(format!("{label}: {}", join_strs(&x["spells"], ", ")));
                }
            }
            if !lines.is_empty() {
                html += &format!(
                    r#"<ul class="detail-spells">{}</ul>"#,
                    lines
                        .iter()
                        .map(|l| format!("<li>{l}</li>"))
                        .collect::<String>()
                );
            }
            if let Some(f) = sc["footerEntries"].as_array() {
                for x in f.iter().filter_map(Value::as_str) {
                    html += &format!("<p>{}</p>", tags(x));
                }
            }
            html
        })
        .collect()
}

fn monster(v: &Value) -> String {
    let size = match &v["size"] {
        Value::Array(l) => l
            .iter()
            .filter_map(Value::as_str)
            .map(summary::size_name)
            .collect::<Vec<_>>()
            .join(" or "),
        Value::String(x) => summary::size_name(x).to_string(),
        _ => String::new(),
    };
    let line = format!(
        "{size} {}, {}",
        summary::creature_type(&v["type"]),
        alignment(&v["alignment"])
    );
    let cr = summary::cr(&v["cr"]);
    let mut html = images(v);
    html += &subtitle(
        &esc(line.trim_end_matches(", ")),
        &[("cr", format!("CR {cr}"))],
    );
    html += &props(&[
        ("Armor class", esc(&armor_class(&v["ac"]))),
        ("Hit points", esc(&hit_points(&v["hp"]))),
        ("Speed", esc(&speeds(&v["speed"]))),
    ]);
    let abilities = ["str", "dex", "con", "int", "wis", "cha"];
    if abilities.iter().any(|a| v[a].is_number()) {
        let head: String = abilities
            .iter()
            .map(|a| format!("<th>{}</th>", a.to_uppercase()))
            .collect();
        let row: String = abilities
            .iter()
            .map(|a| {
                let n = v[a].as_i64().unwrap_or(10);
                format!("<td>{n} ({})</td>", modifier(n))
            })
            .collect();
        html += &format!(
            r#"<table class="detail-abilities"><thead><tr>{head}</tr></thead><tbody><tr>{row}</tr></tbody></table>"#
        );
    }
    let senses = {
        let mut list = match &v["senses"] {
            Value::Array(l) => l
                .iter()
                .filter_map(Value::as_str)
                .map(plain)
                .collect::<Vec<_>>(),
            Value::String(x) => vec![plain(x)],
            _ => Vec::new(),
        };
        if let Some(p) = v["passive"].as_i64() {
            list.push(format!("passive Perception {p}"));
        }
        list.join(", ")
    };
    html += &props(&[
        ("Saving throws", esc(&bonuses(&v["save"], false))),
        ("Skills", esc(&bonuses(&v["skill"], true))),
        (
            "Damage vulnerabilities",
            esc(&damage_list(&v["vulnerable"], "vulnerable")),
        ),
        (
            "Damage resistances",
            esc(&damage_list(&v["resist"], "resist")),
        ),
        (
            "Damage immunities",
            esc(&damage_list(&v["immune"], "immune")),
        ),
        (
            "Condition immunities",
            esc(&damage_list(&v["conditionImmune"], "conditionImmune")),
        ),
        ("Senses", esc(&senses)),
        (
            "Languages",
            esc(&match &v["languages"] {
                Value::Array(l) => l
                    .iter()
                    .filter_map(Value::as_str)
                    .map(plain)
                    .collect::<Vec<_>>()
                    .join(", "),
                Value::String(x) => plain(x),
                _ => String::new(),
            }),
        ),
        ("Challenge", esc(&cr)),
    ]);
    html += &section(
        "Traits",
        &(named_entries(&v["trait"]) + &spellcasting(&v["spellcasting"])),
    );
    html += &section("Actions", &named_entries(&v["action"]));
    html += &section("Bonus actions", &named_entries(&v["bonus"]));
    html += &section("Reactions", &named_entries(&v["reaction"]));
    let legendary_head = v["legendaryHeader"]
        .as_array()
        .map(|l| {
            l.iter()
                .filter_map(Value::as_str)
                .map(|x| format!("<p>{}</p>", tags(x)))
                .collect::<String>()
        })
        .unwrap_or_default();
    let legendary = named_entries(&v["legendary"]);
    if !legendary.is_empty() {
        html += &section("Legendary actions", &(legendary_head + &legendary));
    }
    html += &section("Mythic actions", &named_entries(&v["mythic"]));
    html += &lore(v);
    if let Some(env) = v["environment"].as_array() {
        let list: Vec<String> = env.iter().filter_map(Value::as_str).map(cap).collect();
        html += &section("Environment", &format!("<p>{}</p>", esc(&list.join(", "))));
    }
    html
}

// ------------------------------------------------------------------ items

fn damage_type(code: &str) -> &str {
    match code {
        "A" => "acid",
        "B" => "bludgeoning",
        "C" => "cold",
        "F" => "fire",
        "O" => "force",
        "L" => "lightning",
        "N" => "necrotic",
        "P" => "piercing",
        "I" => "poison",
        "Y" => "psychic",
        "R" => "radiant",
        "S" => "slashing",
        "T" => "thunder",
        other => other,
    }
}

fn property(code: &str) -> &str {
    match code {
        "A" => "ammunition",
        "AF" => "ammunition (futuristic)",
        "BF" => "burst fire",
        "F" => "finesse",
        "H" => "heavy",
        "L" => "light",
        "LD" => "loading",
        "R" => "reach",
        "RLD" => "reload",
        "S" => "special",
        "T" => "thrown",
        "2H" => "two-handed",
        "V" => "versatile",
        other => other,
    }
}

fn attunement(v: &Value) -> Option<String> {
    match &v["reqAttune"] {
        Value::Bool(true) => Some("Requires attunement".into()),
        Value::String(t) => Some(format!("Requires attunement {}", plain(t))),
        _ => None,
    }
}

fn item(v: &Value) -> String {
    let kind = summary::item_kind(v);
    let rarity = summary::rarity(&v["rarity"]);
    let line = if rarity == "—" {
        kind.clone()
    } else {
        format!("{kind}, {}", rarity.to_lowercase())
    };
    let mut badges = Vec::new();
    if let Some(a) = attunement(v) {
        badges.push(("attunement", a));
    }
    if v["curse"].as_bool() == Some(true) {
        badges.push(("curse", "Cursed".into()));
    }
    let mut html = images(v);
    html += &subtitle(&esc(&line), &badges);
    let dmg = |key: &str| {
        v[key].as_str().map(|d| {
            let t = v["dmgType"].as_str().map(damage_type).unwrap_or("");
            format!("{d} {t}").trim().to_string()
        })
    };
    let properties = v["property"]
        .as_array()
        .map(|l| {
            l.iter()
                .map(|p| {
                    let code = p
                        .as_str()
                        .map(String::from)
                        .unwrap_or_else(|| s(&p["uid"]).to_string());
                    let code = code.split('|').next().unwrap_or("").to_string();
                    property(&code).to_string()
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let ac = v["ac"].as_i64().map(|a| {
        let t = v["type"]
            .as_str()
            .unwrap_or("")
            .split('|')
            .next()
            .unwrap_or("")
            .to_string();
        match t.as_str() {
            "LA" => format!("{a} + Dex modifier"),
            "MA" => format!("{a} + Dex modifier (max 2)"),
            "S" => format!("+{a}"),
            _ => a.to_string(),
        }
    });
    html += &props(&[
        (
            "Cost",
            v["value"].as_i64().map(summary::coins).unwrap_or_default(),
        ),
        (
            "Weight",
            v["weight"]
                .as_f64()
                .map(|w| format!("{w} lb."))
                .unwrap_or_default(),
        ),
        ("Armor class", ac.map(|a| esc(&a)).unwrap_or_default()),
        (
            "Strength",
            v["strength"]
                .as_str()
                .map(|x| format!("Str {}", esc(x)))
                .unwrap_or_default(),
        ),
        (
            "Stealth",
            if v["stealth"].as_bool() == Some(true) {
                "Disadvantage".into()
            } else {
                String::new()
            },
        ),
        ("Damage", dmg("dmg1").map(|d| esc(&d)).unwrap_or_default()),
        (
            "Versatile",
            dmg("dmg2").map(|d| esc(&d)).unwrap_or_default(),
        ),
        (
            "Range",
            v["range"]
                .as_str()
                .map(|r| format!("{} ft.", esc(r)))
                .unwrap_or_default(),
        ),
        ("Properties", esc(&properties)),
        (
            "Mastery",
            v["mastery"]
                .as_array()
                .map(|l| {
                    l.iter()
                        .filter_map(Value::as_str)
                        .map(|m| esc(m.split('|').next().unwrap_or(m)))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default(),
        ),
        (
            "Weapon bonus",
            v["bonusWeapon"].as_str().map(esc).unwrap_or_default(),
        ),
        (
            "AC bonus",
            v["bonusAc"].as_str().map(esc).unwrap_or_default(),
        ),
        (
            "Saving throw bonus",
            v["bonusSavingThrow"].as_str().map(esc).unwrap_or_default(),
        ),
        (
            "Spell attack bonus",
            v["bonusSpellAttack"].as_str().map(esc).unwrap_or_default(),
        ),
        (
            "Charges",
            v["charges"]
                .as_i64()
                .map(|c| c.to_string())
                .unwrap_or_default(),
        ),
        (
            "Recharge",
            v["recharge"]
                .as_str()
                .map(|r| esc(&cap(r)))
                .unwrap_or_default(),
        ),
        (
            "Light",
            v["light"]
                .as_array()
                .and_then(|l| l.first())
                .map(|l| {
                    format!(
                        "Bright {} ft., dim {} ft.",
                        l["bright"].as_i64().unwrap_or(0),
                        l["dim"].as_i64().unwrap_or(0)
                    )
                })
                .unwrap_or_default(),
        ),
        (
            "Carrying capacity",
            v["carryingCapacity"]
                .as_i64()
                .map(|c| format!("{c} lb."))
                .unwrap_or_default(),
        ),
        (
            "Speed",
            v["speed"]
                .as_i64()
                .map(|c| format!("{c} ft."))
                .unwrap_or_default(),
        ),
    ]);
    html += &section("Description", &entries(&v["entries"]));
    if let Some(more) = v["additionalEntries"].as_array() {
        html += &section("More", &entries(&Value::Array(more.clone())));
    }
    if let Some(pack) = v["packContents"].as_array() {
        let lis: String = pack
            .iter()
            .map(|p| match p {
                Value::String(x) => format!("<li>{}</li>", tags(&format!("{{@item {x}}}"))),
                Value::Object(o) => {
                    let n = o.get("item").and_then(Value::as_str).unwrap_or("");
                    let q = o.get("quantity").and_then(Value::as_i64).unwrap_or(1);
                    format!("<li>{q} × {}</li>", tags(&format!("{{@item {n}}}")))
                }
                _ => String::new(),
            })
            .collect();
        html += &section("Contents", &format!("<ul>{lis}</ul>"));
    }
    html += &lore(v);
    html
}

// ------------------------------------------------------------------ player options

fn class(v: &Value) -> String {
    let mut html = images(v);
    let hd = v["hd"]["faces"]
        .as_i64()
        .map(|f| format!("d{f}"))
        .unwrap_or_default();
    html += &subtitle(&esc(&format!("Hit die {hd}")), &[]);
    let saves = v["proficiency"]
        .as_array()
        .map(|l| {
            l.iter()
                .filter_map(Value::as_str)
                .map(ability_name)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let sp = &v["startingProficiencies"];
    let skills = sp["skills"]
        .as_array()
        .and_then(|l| l.first())
        .map(|first| {
            if let Some(c) = first.get("choose") {
                let n = c["count"].as_i64().unwrap_or(1);
                let from = c["from"]
                    .as_array()
                    .map(|f| {
                        f.iter()
                            .filter_map(Value::as_str)
                            .map(crate::tags::plain)
                            .map(|x| cap(&x))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                format!("Choose {n} from {from}")
            } else if let Some(n) = first.get("any").and_then(Value::as_i64) {
                format!("Choose any {n}")
            } else {
                String::new()
            }
        })
        .unwrap_or_default();
    html += &props(&[
        ("Primary ability", esc(&cells("classes", v)[1])),
        ("Saving throws", esc(&saves)),
        ("Armor", join_strs(&sp["armor"], ", ")),
        ("Weapons", join_strs(&sp["weapons"], ", ")),
        ("Tools", join_strs(&sp["tools"], ", ")),
        ("Skills", esc(&skills)),
        (
            "Spellcasting",
            v["spellcastingAbility"]
                .as_str()
                .map(|a| esc(ability_name(a)))
                .unwrap_or_default(),
        ),
    ]);
    // The class table.
    let no_groups = Vec::new();
    let groups = v["classTableGroups"].as_array().unwrap_or(&no_groups);
    if !groups.is_empty() || v["classFeatures"].is_array() {
        let mut head = String::from("<th>Level</th><th>Features</th>");
        let mut cols: Vec<&Vec<Value>> = Vec::new();
        for g in groups {
            if let Some(l) = g["colLabels"].as_array() {
                for c in l {
                    head += &format!("<th>{}</th>", tags(s(c)));
                }
            }
            if let Some(r) = g["rows"].as_array() {
                cols.push(r);
            }
        }
        let features = features_by_level(&v["classFeatures"]);
        let rows: String = (1..=20)
            .map(|lvl| {
                let f = features
                    .get(lvl - 1)
                    .cloned()
                    .unwrap_or_default()
                    .join(", ");
                let extra: String = cols
                    .iter()
                    .flat_map(|rows| {
                        rows.get(lvl - 1)
                            .and_then(Value::as_array)
                            .map(|cells| {
                                cells
                                    .iter()
                                    .map(|c| match c {
                                        Value::String(x) => format!("<td>{}</td>", tags(x)),
                                        Value::Number(n) => format!("<td>{n}</td>"),
                                        Value::Object(o) => format!(
                                            "<td>{}</td>",
                                            o.get("value")
                                                .map(|x| x.to_string())
                                                .unwrap_or_default()
                                        ),
                                        _ => "<td></td>".into(),
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default()
                    })
                    .collect();
                format!("<tr><td>{lvl}</td><td>{}</td>{extra}</tr>", esc(&f))
            })
            .collect();
        html += &section(
            "Class table",
            &format!(
                r#"<table class="entry-table"><thead><tr>{head}</tr></thead><tbody>{rows}</tbody></table>"#
            ),
        );
    }
    if let Some(t) = v["subclassTitle"].as_str() {
        html += &props(&[("Subclass", esc(t))]);
    }
    html += &section("Description", &entries(&v["entries"]));
    html += &lore(v);
    html
}

/// Class feature names by level (1..=20) from `classFeatures` references
/// ("Second Wind|Fighter||1" or {"classFeature": …}).
pub fn features_by_level(v: &Value) -> Vec<Vec<String>> {
    let mut out = vec![Vec::new(); 20];
    for f in v.as_array().into_iter().flatten() {
        let r = f
            .as_str()
            .or_else(|| f["classFeature"].as_str())
            .unwrap_or("");
        let parts: Vec<&str> = r.split('|').collect();
        let name = parts.first().copied().unwrap_or("");
        let level = parts
            .get(3)
            .and_then(|l| l.parse::<usize>().ok())
            .unwrap_or(0);
        if (1..=20).contains(&level) && !name.is_empty() {
            out[level - 1].push(name.to_string());
        }
    }
    out
}

fn race(v: &Value) -> String {
    let c = cells("races", v);
    let mut html = images(v);
    html += &props(&[
        ("Size", esc(&c[0])),
        ("Speed", esc(&speeds(&v["speed"]))),
        ("Ability scores", esc(&c[2])),
        (
            "Darkvision",
            v["darkvision"]
                .as_i64()
                .map(|d| format!("{d} ft."))
                .unwrap_or_default(),
        ),
        ("Languages", esc(&granted_list(&v["languageProficiencies"]))),
    ]);
    html += &section("Traits", &entries(&v["entries"]));
    html += &lore(v);
    html
}

fn granted_list(v: &Value) -> String {
    v.as_array()
        .and_then(|l| l.first())
        .and_then(Value::as_object)
        .map(|o| {
            o.iter()
                .filter(|(_, x)| x.as_bool() == Some(true))
                .map(|(k, _)| cap(&plain(k.split('|').next().unwrap_or(k))))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

fn feat(v: &Value) -> String {
    let mut html = props(&[
        (
            "Prerequisite",
            esc(&summary::prerequisite(&v["prerequisite"])),
        ),
        (
            "Category",
            v["category"].as_str().map(esc).unwrap_or_default(),
        ),
    ]);
    html += &section("Description", &entries(&v["entries"]));
    html += &lore(v);
    html
}

fn background(v: &Value) -> String {
    let mut html = props(&[
        ("Skills", esc(&granted_list(&v["skillProficiencies"]))),
        ("Tools", esc(&granted_list(&v["toolProficiencies"]))),
        ("Languages", esc(&granted_list(&v["languageProficiencies"]))),
    ]);
    html += &section("Description", &entries(&v["entries"]));
    html += &lore(v);
    html
}

fn optional_feature(v: &Value) -> String {
    let c = cells("optional-features", v);
    let mut html = props(&[("Type", esc(&c[0])), ("Prerequisite", esc(&c[1]))]);
    html += &section("Description", &entries(&v["entries"]));
    html
}

/// Any other kind: its table columns, then its entries.
fn generic(category: &str, v: &Value) -> String {
    let list: Vec<(&str, String)> = headers(category)
        .iter()
        .zip(cells(category, v))
        .map(|(h, c)| (*h, esc(&c)))
        .collect();
    let mut html = images(v);
    html += &props(&list);
    let body = if v["entries"].is_array() {
        entries(&v["entries"])
    } else if v["entries"].is_object() {
        entries(&Value::Array(vec![v["entries"].clone()]))
    } else if v["rows"].is_array() {
        crate::entries::table(v)
    } else {
        String::new()
    };
    html += &section("Description", &body);
    // A table entry's own table.
    if category == "tables" && v["entries"].is_array() && v["rows"].is_array() {
        html += &crate::entries::table(v);
    }
    html += &lore(v);
    html
}

/// A one-line tooltip for a cross-reference: "Fireball — 3rd-level
/// evocation", "Goblin — CR 1/4".
pub fn tooltip(category: &str, v: &Value) -> String {
    let name = s(&v["name"]);
    let rest = match category {
        "spells" => {
            let level = v["level"].as_i64().unwrap_or(0);
            let school = summary::school_name(s(&v["school"])).to_lowercase();
            if level == 0 {
                format!("{school} cantrip")
            } else {
                format!("{}-level {school}", summary::spell_level(level))
            }
        }
        "monsters" => format!("CR {}", summary::cr(&v["cr"])),
        "items" => summary::item_kind(v),
        other => {
            let c = cells(other, v);
            c.into_iter().find(|x| x != "—").unwrap_or_default()
        }
    };
    if rest.is_empty() {
        name.to_string()
    } else {
        format!("{name} — {rest}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fireball() -> Value {
        json!({
            "name": "Fireball", "source": "PHB", "page": 241, "level": 3, "school": "V",
            "time": [{"number": 1, "unit": "action"}],
            "range": {"type": "point", "distance": {"type": "feet", "amount": 150}},
            "components": {"v": true, "s": true, "m": "a tiny ball of bat guano and sulfur"},
            "duration": [{"type": "instant"}],
            "entries": ["A bright streak flashes... {@damage 8d6} fire damage on a failed save."],
            "entriesHigherLevel": [{"type": "entries", "name": "At Higher Levels", "entries": ["The damage increases by {@scaledamage 8d6|3-9|1d6} for each slot level above 3rd."]}],
            "damageInflict": ["fire"], "savingThrow": ["dexterity"], "areaTags": ["S"],
            "classes": {"fromClassList": [{"name": "Sorcerer", "source": "PHB"}, {"name": "Wizard", "source": "PHB"}]}
        })
    }

    // The desktop's spellFormatting tests, ported (their intents).

    #[test]
    fn spells() {
        let r = detail("spells", &fireball());
        for want in [
            "3rd-level evocation",
            "1 action",
            "150 feet",
            "V, S, M (a tiny ball of bat guano and sulfur)",
            "Instantaneous",
            "Fire",
            "Dexterity",
            "Sphere",
            "Sorcerer, Wizard",
            r#"class="damage-roll""#,
            "At higher levels",
            "Source: PHB, p. 241",
        ] {
            assert!(r.contains(want), "{want} in {r}");
        }
        assert!(!r.contains("Ritual"));
        let detect = json!({"name": "Detect Magic", "level": 1, "school": "D", "meta": {"ritual": true},
            "time": [{"number": 1, "unit": "action"}], "range": {"type": "point", "distance": {"type": "self"}},
            "components": {"v": true, "s": true},
            "duration": [{"type": "timed", "duration": {"type": "minute", "amount": 10}, "concentration": true}]});
        let r = detail("spells", &detect);
        assert!(
            r.contains("Ritual")
                && r.contains("Concentration")
                && r.contains("Concentration, up to 10 minutes")
        );
        assert!(r.contains("Self"));
        let shield = json!({"name": "Shield", "level": 1, "school": "A",
            "time": [{"number": 1, "unit": "reaction", "condition": "which you take when you are hit by an attack"}],
            "duration": [{"type": "timed", "duration": {"type": "round", "amount": 1}}]});
        let r = detail("spells", &shield);
        assert!(r.contains("1 reaction, which you take when you are hit") && r.contains("1 round"));
        let blast = json!({"name": "Eldritch Blast", "level": 0, "school": "V", "spellAttack": ["R"],
            "damageInflict": ["force"], "miscTags": ["SCL"],
            "scalingLevelDice": {"label": "force damage", "scaling": {"1": "1d10", "5": "2d10", "11": "3d10", "17": "4d10"}}});
        let r = detail("spells", &blast);
        assert!(
            r.contains("Evocation cantrip")
                && r.contains("Ranged spell attack")
                && r.contains("Force")
        );
        assert!(
            r.contains("1d10 → 2d10 (5th) → 3d10 (11th) → 4d10 (17th)"),
            "{r}"
        );
        assert!(r.contains("Scaling"));
    }

    fn goblin() -> Value {
        json!({
            "name": "Goblin", "source": "MM", "page": 166, "size": ["S"],
            "type": {"type": "humanoid", "tags": ["goblinoid"]}, "alignment": ["N", "E"],
            "ac": [{"ac": 15, "from": ["{@item leather armor|phb}", "{@item shield|phb}"]}],
            "hp": {"average": 7, "formula": "2d6"}, "speed": {"walk": 30},
            "str": 8, "dex": 14, "con": 10, "int": 10, "wis": 8, "cha": 8,
            "skill": {"stealth": "+6"}, "senses": ["darkvision 60 ft."], "passive": 9,
            "languages": ["Common", "Goblin"], "cr": "1/4",
            "trait": [{"name": "Nimble Escape", "entries": ["The goblin can take the {@action Disengage} or {@action Hide} action as a bonus action."]}],
            "action": [{"name": "Scimitar", "entries": ["{@atk mw} {@hit 4} to hit, reach 5 ft., one target. {@h}5 ({@damage 1d6 + 2}) slashing damage."]}],
            "environment": ["forest", "grassland"],
            "fluff": {"entries": ["Goblins are small, black-hearted humanoids."], "images": [{"href": {"path": "bestiary/MM/Goblin.webp"}}]}
        })
    }

    // The desktop's monsterFormatterEnhanced tests, ported (their intents).

    #[test]
    fn monsters() {
        let r = detail("monsters", &goblin());
        for want in [
            "Small humanoid (goblinoid), neutral evil",
            "CR 1/4",
            "15 (leather armor, shield)",
            "7 (2d6)",
            "30 ft.",
            "<th>STR</th>",
            "8 (-1)",
            "14 (+2)",
            "Stealth +6",
            "darkvision 60 ft., passive Perception 9",
            "Common, Goblin",
            "Nimble Escape.",
            "action-ref",
            "Melee Weapon Attack:",
            r#"class="hit-bonus">+4<"#,
            "Lore",
            "Forest, Grassland",
            r#"data-image-path="bestiary/MM/Goblin.webp""#,
            "Source: MM, p. 166",
        ] {
            assert!(r.contains(want), "{want} in {r}");
        }
        assert!(!r.contains("Reactions"), "no empty sections");
    }

    #[test]
    fn monster_extras() {
        let m = json!({"name": "X", "size": ["H"], "type": "dragon", "alignment": ["C", "E"], "cr": {"cr": "17"},
            "speed": {"walk": 40, "fly": 80, "canHover": true},
            "immune": ["fire"], "resist": [{"resist": ["bludgeoning", "piercing"], "note": "from nonmagical attacks"}],
            "conditionImmune": ["frightened"],
            "legendaryHeader": ["It can take 3 legendary actions."],
            "legendary": [{"name": "Detect", "entries": ["The dragon makes a check."]}],
            "spellcasting": [{"name": "Innate Spellcasting", "headerEntries": ["It casts:"], "will": ["{@spell detect magic}"], "daily": {"1e": ["{@spell fly}"]}}]});
        let r = detail("monsters", &m);
        for want in [
            "fly 80 ft. (hover)",
            "Damage immunities",
            "bludgeoning, piercing from nonmagical attacks",
            "frightened",
            "Legendary actions",
            "3 legendary actions",
            "At will:",
            "1/day each:",
            "spell-ref",
        ] {
            assert!(r.contains(want), "{want} in {r}");
        }
    }

    // The desktop's itemFormatting tests, ported (their intents).

    #[test]
    fn items() {
        let sword = json!({"name": "Longsword", "source": "PHB", "type": "M", "rarity": "none", "value": 1500,
            "weight": 3, "dmg1": "1d8", "dmg2": "1d10", "dmgType": "S", "property": ["V"], "weaponCategory": "martial"});
        let r = detail("items", &sword);
        for want in [
            "Melee weapon",
            "15 gp",
            "3 lb.",
            "1d8 slashing",
            "1d10 slashing",
            "versatile",
        ] {
            assert!(r.contains(want), "{want} in {r}");
        }
        let chain = json!({"name": "Chain Mail", "type": "HA", "ac": 16, "strength": "13", "stealth": true});
        let r = detail("items", &chain);
        assert!(
            r.contains("Heavy armor")
                && r.contains("16")
                && r.contains("Str 13")
                && r.contains("Disadvantage")
        );
        let leather = json!({"name": "Leather", "type": "LA", "ac": 11});
        assert!(detail("items", &leather).contains("11 + Dex modifier"));
        let ring = json!({"name": "Ring of Protection", "type": "RG", "rarity": "rare", "reqAttune": true,
            "bonusAc": "+1", "bonusSavingThrow": "+1", "entries": ["You gain a +1 bonus to AC."]});
        let r = detail("items", &ring);
        assert!(
            r.contains("Ring, rare") && r.contains("Requires attunement") && r.contains("AC bonus")
        );
        let staff = json!({"name": "Staff", "wondrous": true, "rarity": "very rare", "reqAttune": "by a wizard", "charges": 10});
        let r = detail("items", &staff);
        assert!(
            r.contains("Wondrous item, very rare")
                && r.contains("Requires attunement by a wizard")
                && r.contains("Charges")
        );
        let pack = json!({"name": "Explorer's Pack", "type": "G", "packContents": ["backpack|phb", {"item": "torch|phb", "quantity": 10}]});
        let r = detail("items", &pack);
        assert!(r.contains("Contents") && r.contains("10 × ") && r.contains("item-ref"));
    }

    #[test]
    fn player_options() {
        let fighter = json!({"name": "Fighter", "source": "PHB", "hd": {"number": 1, "faces": 10},
            "proficiency": ["str", "con"],
            "startingProficiencies": {"armor": ["light", "medium", "heavy", "{@item shield|phb|shields}"], "weapons": ["simple", "martial"],
                "skills": [{"choose": {"count": 2, "from": ["acrobatics", "athletics"]}}]},
            "classFeatures": ["Fighting Style|Fighter||1", "Second Wind|Fighter||1", {"classFeature": "Martial Archetype|Fighter||3", "gainSubclassFeature": true}],
            "subclassTitle": "Martial Archetype"});
        let r = detail("classes", &fighter);
        assert!(
            r.contains("Hit die d10")
                && r.contains("Strength, Constitution")
                && r.contains("shields</a>")
        );
        assert!(r.contains("Choose 2 from Acrobatics, Athletics"));
        assert!(
            r.contains("<td>1</td><td>Fighting Style, Second Wind</td>")
                && r.contains("<td>3</td><td>Martial Archetype</td>")
        );
        let elf = json!({"name": "Elf", "size": ["M"], "speed": 30, "ability": [{"dex": 2}], "darkvision": 60,
            "languageProficiencies": [{"common": true, "elvish": true}], "entries": [{"type": "entries", "name": "Keen Senses", "entries": ["You have proficiency in the {@skill Perception} skill."]}]});
        let r = detail("races", &elf);
        assert!(
            r.contains("Dex +2")
                && r.contains("60 ft.")
                && r.contains("Common, Elvish")
                && r.contains("Keen Senses")
        );
        let alert = json!({"name": "Grappler", "prerequisite": [{"ability": [{"str": 13}]}], "entries": ["You've developed..."]});
        assert!(detail("feats", &alert).contains("Strength 13"));
        let acolyte = json!({"name": "Acolyte", "skillProficiencies": [{"insight": true, "religion": true}], "languageProficiencies": [{"anyStandard": 2}]});
        assert!(detail("backgrounds", &acolyte).contains("Insight, Religion"));
    }

    #[test]
    fn the_rest_and_tooltips() {
        let deity = json!({"name": "Bane", "source": "PHB", "title": "god of tyranny", "pantheon": "Forgotten Realms",
            "alignment": ["L", "E"], "domains": ["War"], "entries": ["The Black Hand."]});
        let r = detail("deities", &deity);
        assert!(
            r.contains("god of tyranny")
                && r.contains("Forgotten Realms")
                && r.contains("The Black Hand.")
        );
        let cond = json!({"name": "Poisoned", "entries": [{"type": "list", "items": ["A poisoned creature has disadvantage."]}]});
        assert!(detail("conditions", &cond).contains("<li>A poisoned creature"));
        assert_eq!(
            tooltip("spells", &fireball()),
            "Fireball — 3rd-level evocation"
        );
        assert_eq!(tooltip("monsters", &goblin()), "Goblin — CR 1/4");
        // Escaped.
        let bad = json!({"name": "<x>", "source": "<y>", "entries": ["<b>"]});
        let r = detail("actions", &bad);
        assert!(!r.contains("<b>") && !r.contains("<y>"));
    }
}

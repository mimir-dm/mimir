//! The 5etools inline tags (`{@spell fireball|PHB}`, `{@dice 2d6}`, …) to
//! HTML: a port of the desktop's `processFormattingTags`, as a parser
//! (tags may nest; text is escaped).

use crate::{esc, link};

/// A tag that links to a catalog entry: the reference type, and the
/// source when the tag gives none.
fn link_kind(tag: &str) -> Option<(&'static str, &'static str)> {
    Some(match tag {
        "spell" => ("spell", "PHB"),
        "item" => ("item", "PHB"),
        "creature" => ("creature", "MM"),
        "race" => ("race", "PHB"),
        "class" => ("class", "PHB"),
        "feat" => ("feat", "PHB"),
        "background" => ("background", "PHB"),
        "condition" => ("condition", "PHB"),
        "disease" => ("condition", "DMG"),
        "action" => ("action", "PHB"),
        "language" => ("language", "PHB"),
        "trap" => ("trap", "DMG"),
        "hazard" => ("hazard", "DMG"),
        "deity" => ("deity", "PHB"),
        "optfeature" => ("optfeature", "PHB"),
        "variantrule" => ("variantrule", "DMG"),
        "table" => ("table", "DMG"),
        "object" => ("object", "DMG"),
        "reward" => ("reward", "DMG"),
        "vehicle" => ("vehicle", "GoS"),
        "psionic" => ("psionic", "UATheMysticClass"),
        "cult" => ("cult", "MTF"),
        "boon" => ("boon", "MTF"),
        _ => return None,
    })
}

/// The full name of an ability abbreviation ("dex" → "Dexterity").
pub fn ability_name(short: &str) -> &str {
    match short.trim().to_lowercase().as_str() {
        "str" => "Strength",
        "dex" => "Dexterity",
        "con" => "Constitution",
        "int" => "Intelligence",
        "wis" => "Wisdom",
        "cha" => "Charisma",
        _ => short,
    }
}

/// Split tag arguments at `|`, not inside nested braces.
fn split_args(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0usize, 0usize);
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            '|' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out.into_iter().map(str::trim).collect()
}

/// The index of the `}` that closes the `{` at `open`.
fn closing(s: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in s[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Text with 5etools tags, as HTML.
pub fn tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find("{@").into_iter().chain(rest.find("{#")).min() {
        out.push_str(&esc(&rest[..pos]));
        let Some(end) = closing(rest, pos) else {
            out.push_str(&esc(&rest[pos..]));
            return out;
        };
        let inner = &rest[pos + 2..end];
        let (name, args) = match inner.find(char::is_whitespace) {
            Some(i) => (&inner[..i], inner[i..].trim_start()),
            None => (inner, ""),
        };
        let hash = rest[pos + 1..].starts_with('#');
        out.push_str(&render(name, args, hash));
        rest = &rest[end + 1..];
    }
    out.push_str(&esc(rest));
    out
}

/// Text with 5etools tags, as plain text (what the tags show; no
/// markup, not escaped): a name in the contents, a tooltip.
pub fn plain(text: &str) -> String {
    let html = tags(text);
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn span(class: &str, inner_html: &str) -> String {
    format!(r#"<span class="{class}">{inner_html}</span>"#)
}

fn attack(kinds: &str) -> String {
    let words = match kinds.replace(' ', "").as_str() {
        "mw" => "Melee Weapon Attack:",
        "rw" => "Ranged Weapon Attack:",
        "mw,rw" | "rw,mw" => "Melee or Ranged Weapon Attack:",
        "ms" => "Melee Spell Attack:",
        "rs" => "Ranged Spell Attack:",
        "ms,rs" | "rs,ms" => "Melee or Ranged Spell Attack:",
        "m" => "Melee Attack Roll:",
        "r" => "Ranged Attack Roll:",
        "m,r" | "r,m" => "Melee or Ranged Attack Roll:",
        _ => "Attack:",
    };
    format!("<em>{words}</em>")
}

fn render(tag: &str, args: &str, hash: bool) -> String {
    let a = split_args(args);
    let arg = |i: usize| a.get(i).copied().filter(|s| !s.is_empty());
    let first = arg(0).unwrap_or("");

    if hash {
        return match tag {
            "itemEntry" => item_entry(first),
            _ => span("tagged", &tags(first)),
        };
    }
    if let Some((kind, default_source)) = link_kind(tag) {
        let source = arg(1).unwrap_or(default_source).to_uppercase();
        let display = tags(arg(2).unwrap_or(first));
        return link(kind, first, &source, &display);
    }
    match tag {
        "b" | "bold" => format!("<strong>{}</strong>", tags(args)),
        "i" | "italic" => format!("<em>{}</em>", tags(args)),
        "bi" => format!("<strong><em>{}</em></strong>", tags(args)),
        "u" | "underline" => format!("<u>{}</u>", tags(args)),
        "s" | "strike" => format!("<s>{}</s>", tags(args)),
        "sup" => format!("<sup>{}</sup>", tags(args)),
        "sub" => format!("<sub>{}</sub>", tags(args)),
        "code" => format!("<code>{}</code>", esc(args)),
        "kbd" => format!("<kbd>{}</kbd>", esc(args)),
        "highlight" => format!("<mark>{}</mark>", tags(first)),
        "color" | "help" | "font" => tags(first),
        "note" => span("note", &format!("Note: {}", tags(args))),
        "dice" | "autodice" => span("dice-roll", &tags(arg(1).unwrap_or(first))),
        "damage" => span("damage-roll", &tags(arg(1).unwrap_or(first))),
        "scaledice" | "scaledamage" => span("dice-roll", &tags(arg(3).or(arg(2)).unwrap_or(first))),
        "d20" => match arg(1) {
            Some(d) => span("d20-roll", &tags(d)),
            None => span("d20-roll", &esc(&format!("d20{first}"))),
        },
        "hit" => {
            let shown = match first.parse::<i32>() {
                Ok(n) if !first.starts_with(['+', '-']) && n >= 0 => format!("+{n}"),
                _ => first.to_string(),
            };
            span("hit-bonus", &esc(arg(1).unwrap_or(&shown)))
        }
        "dc" => match arg(1) {
            Some(ab) => span("dc-check", &esc(&format!("DC {first} {ab}"))),
            None => span("dc-check", &esc(&format!("DC {first}"))),
        },
        "chance" => match arg(1) {
            Some(t) => span("chance", &tags(t)),
            None => span("chance", &esc(&format!("{first}% chance"))),
        },
        "recharge" => match arg(0) {
            Some(n) => span("recharge", &esc(&format!("(Recharge {n}–6)"))),
            None => span("recharge", "(Recharge)"),
        },
        "atk" | "atkr" => attack(first),
        "h" => "<em>Hit:</em>".into(),
        "m" => "<em>Miss:</em>".into(),
        "hom" => "<em>Hit or Miss:</em>".into(),
        "actSave" => format!("<em>{} Saving Throw:</em>", esc(ability_name(first))),
        "actSaveFail" => match arg(0) {
            Some(n) => format!("<em>Failure by {} or More:</em>", esc(n)),
            None => "<em>Failure:</em>".into(),
        },
        "actSaveSuccess" => "<em>Success:</em>".into(),
        "actSaveSuccessOrFail" => "<em>Failure or Success:</em>".into(),
        "actTrigger" => "<em>Trigger:</em>".into(),
        "actResponse" => "<em>Response:</em>".into(),
        "status" => span("status", &tags(arg(2).unwrap_or(first))),
        "skill" => span("skill", &tags(first)),
        "sense" => span("sense", &tags(first)),
        "classFeature" | "subclassFeature" => span("feature-ref", &tags(first)),
        "filter" => span("filter-ref", &tags(first)),
        "book" | "adventure" => span("book-ref", &tags(first)),
        "5etoolsImg" => {
            let path = arg(1).unwrap_or("");
            if path.ends_with(".pdf") {
                span("pdf-ref", &format!("[PDF: {}]", esc(first)))
            } else {
                span("external-img-ref", &format!("[Image: {}]", esc(first)))
            }
        }
        _ => span("tagged", &tags(arg(2).unwrap_or(first))),
    }
}

fn item_entry(name: &str) -> String {
    let known = match name.to_lowercase().as_str() {
        "armor of resistance" => Some("You have resistance to one type of damage while you wear this armor."),
        "potion of resistance" => Some("When you drink this potion, you gain resistance to one type of damage for 1 hour."),
        "grenade" => Some("As an action, a character can throw a grenade at a point up to 60 feet away. Each creature within 20 feet of an exploding fragmentation grenade must make a DC 15 Dexterity saving throw, taking 5d6 piercing damage on a failed save, or half as much damage on a successful one."),
        _ => None,
    };
    match known {
        Some(t) => span("item-description", t),
        None => span("item-entry-ref", &format!("[See base item: {}]", esc(name))),
    }
}

#[cfg(test)]
mod tests {
    use super::tags as t;

    // The desktop's textFormatting tests (processFormattingTags), ported.

    #[test]
    fn basic_formatting() {
        assert_eq!(t("{@b hello}"), "<strong>hello</strong>");
        assert_eq!(t("{@i hello}"), "<em>hello</em>");
        assert_eq!(t("{@bi hello}"), "<strong><em>hello</em></strong>");
        let r = t("The {@b warrior} was {@i fast}");
        assert!(r.contains("<strong>warrior</strong>") && r.contains("<em>fast</em>"));
    }

    #[test]
    fn dice() {
        let r = t("{@dice 2d6}");
        assert!(r.contains(r#"class="dice-roll""#) && r.contains("2d6"));
        assert!(t("{@dice 2d6|two dice}").contains("two dice"));
        let r = t("{@damage 3d8}");
        assert!(r.contains(r#"class="damage-roll""#) && r.contains("3d8"));
        let r = t("{@hit +5}");
        assert!(r.contains(r#"class="hit-bonus""#) && r.contains("+5"));
        assert!(t("{@hit 7}").contains("+7"), "a bonus shows its sign");
    }

    #[test]
    fn cross_references() {
        let r = t("{@spell fireball}");
        for want in [
            r#"class="cross-ref-link spell-ref""#,
            r#"data-ref-type="spell""#,
            r#"data-ref-name="fireball""#,
            r#"data-ref-source="PHB""#,
            "fireball</a>",
        ] {
            assert!(r.contains(want), "{want} in {r}");
        }
        assert!(t("{@spell eldritch blast|PHB}").contains(r#"data-ref-source="PHB""#));
        let r = t("{@creature goblin}");
        assert!(r.contains(r#"class="cross-ref-link creature-ref""#));
        assert!(r.contains(r#"data-ref-name="goblin""#) && r.contains(r#"data-ref-source="MM""#));
        assert!(t("{@creature goblin|MM|a sneaky goblin}").contains("a sneaky goblin</a>"));
        let r = t("{@item longsword}");
        assert!(
            r.contains(r#"class="cross-ref-link item-ref""#)
                && r.contains(r#"data-ref-name="longsword""#)
        );
        assert!(t("{@condition poisoned}").contains("poisoned</a>"));
        for (tag, kind) in [
            ("class Fighter", "class"),
            ("feat Alert", "feat"),
            ("race Elf", "race"),
            ("background Acolyte", "background"),
            ("action Dodge", "action"),
        ] {
            let r = t(&format!("{{@{tag}}}"));
            assert!(
                r.contains(&format!(r#"class="cross-ref-link {kind}-ref""#)),
                "{r}"
            );
            assert!(r.contains(&format!(r#"data-ref-type="{kind}""#)));
        }
        // An empty source part takes the default.
        assert!(t("{@item dagger||a knife}").contains(r#"data-ref-source="PHB">a knife</a>"#));
    }

    #[test]
    fn skills_and_dc() {
        let r = t("{@skill Perception}");
        assert!(r.contains(r#"class="skill""#) && r.contains("Perception"));
        assert!(t("{@sense darkvision}").contains(r#"class="sense""#));
        let r = t("{@dc 15}");
        assert!(r.contains(r#"class="dc-check""#) && r.contains("DC 15"));
        assert!(t("{@dc 15|Dexterity}").contains("DC 15 Dexterity"));
    }

    #[test]
    fn attacks() {
        assert!(t("{@atk mw}").contains("Melee Weapon Attack:"));
        assert!(t("{@atk rw}").contains("Ranged Weapon Attack:"));
        assert!(t("{@atk mw,rw}").contains("Melee or Ranged Weapon Attack:"));
        assert!(t("{@atk ms}").contains("Melee Spell Attack:"));
        assert!(t("{@atk rs}").contains("Ranged Spell Attack:"));
        assert!(t("{@h}").contains("Hit:"));
        // 2024 stat blocks.
        assert!(t("{@atkr m}").contains("Melee Attack Roll:"));
        assert!(t("{@actSave dex}").contains("Dexterity Saving Throw:"));
        assert!(t("{@actSaveFail}").contains("Failure:"));
    }

    #[test]
    fn special_tags() {
        let r = t("{@recharge 5}");
        assert!(r.contains(r#"class="recharge""#) && r.contains("Recharge 5–6"));
        assert!(t("{@recharge}").contains("(Recharge)"));
        let r = t("{@chance 50}");
        assert!(r.contains(r#"class="chance""#) && r.contains("50% chance"));
        let r = t("{@note important}");
        assert!(r.contains(r#"class="note""#) && r.contains("Note: important"));
        assert!(t("{@scaledamage 8d6|3-9|1d6}").contains(">1d6<"));
        assert!(t("{#itemEntry Grenade}").contains("DC 15 Dexterity"));
        assert!(t("{#itemEntry Ring of X}").contains("[See base item: Ring of X]"));
    }

    #[test]
    fn edge_cases() {
        assert_eq!(t(""), "");
        assert_eq!(t("Hello world"), "Hello world");
        assert!(t("{@unknown something}").contains("something"));
        // Unclosed: left as text.
        assert_eq!(t("a {@b b"), "a {@b b");
    }

    #[test]
    fn escaping_and_nesting() {
        assert_eq!(t("<script>"), "&lt;script&gt;");
        let r = t(r#"{@spell "x"|PHB|<b>}"#);
        assert!(r.contains(r#"data-ref-name="&quot;x&quot;""#) && r.contains("&lt;b&gt;</a>"));
        // A tag inside a tag (the desktop's regexes could not).
        assert_eq!(
            t("{@b {@i deep}} done"),
            "<strong><em>deep</em></strong> done"
        );
        let r = t("{@note See {@spell shield}.}");
        assert!(r.contains("Note: See <a") && r.contains("shield</a>."));
    }

    #[test]
    fn plain_text() {
        assert_eq!(
            super::plain("Cast {@spell fire bolt} & {@b run}"),
            "Cast fire bolt & run"
        );
        assert_eq!(super::plain("{@dc 12} <x>"), "DC 12 <x>");
    }
}

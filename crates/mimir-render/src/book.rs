//! A book in the reader: a section of its data (`data[i]`) as HTML, and
//! the table of contents. A port of the desktop's `contentRenderer`, with
//! one change: an entry without a 5etools id gets a path id
//! (`s{section}-{i}-{j}…`) instead of `entry-{index}`, which repeated
//! within a section; the contents and the anchors use the same ids.

use serde_json::Value;

use crate::entries::{roll_cell, s, statblock};
use crate::esc;
use crate::tags::tags;

/// The anchor of an entry: its 5etools id, else its path.
pub fn anchor(e: &Value, path: &[usize]) -> String {
    match e.get("id").and_then(Value::as_str) {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => format!(
            "s{}",
            path.iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join("-")
        ),
    }
}

/// The name of a section: its name, else the start of its first text.
pub fn section_name(section: &Value, index: usize) -> String {
    if let Some(n) = section.get("name").and_then(Value::as_str) {
        return n.to_string();
    }
    let first = section
        .get("entries")
        .and_then(Value::as_array)
        .and_then(|l| l.iter().find_map(Value::as_str));
    match first {
        Some(t) => crate::tags::plain(t).chars().take(60).collect(),
        None => format!("Section {}", index + 1),
    }
}

/// A heading in the contents.
#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub id: String,
    pub name: String,
    /// 1: under the section; 2: under that.
    pub depth: usize,
}

/// A section of the contents and its headings (two levels, as the
/// desktop).
#[derive(Debug, Clone, PartialEq)]
pub struct ContentsEntry {
    pub index: usize,
    pub name: String,
    pub headings: Vec<Heading>,
}

/// The table of contents of a book's data.
pub fn contents(data: &Value) -> Vec<ContentsEntry> {
    let Some(sections) = data.as_array() else {
        return Vec::new();
    };
    sections
        .iter()
        .enumerate()
        .map(|(i, sec)| {
            let mut headings = Vec::new();
            collect(sec.get("entries"), &[i], 1, &mut headings);
            ContentsEntry {
                index: i,
                name: section_name(sec, i),
                headings,
            }
        })
        .collect()
}

fn collect(list: Option<&Value>, path: &[usize], depth: usize, out: &mut Vec<Heading>) {
    if depth > 2 {
        return;
    }
    let Some(items) = list.and_then(Value::as_array) else {
        return;
    };
    for (i, e) in items.iter().enumerate() {
        let kind = e.get("type").and_then(Value::as_str).unwrap_or("");
        let mut p = path.to_vec();
        p.push(i);
        if matches!(kind, "section" | "entries") {
            if let Some(name) = e.get("name").and_then(Value::as_str) {
                out.push(Heading {
                    id: anchor(e, &p),
                    name: name.to_string(),
                    depth,
                });
                collect(e.get("entries"), &p, depth + 1, out);
            }
        }
    }
}

/// The section (`data[i]`) that holds the entry with this 5etools id: the
/// target of an `{@area}` link (MIMIR-T-0736).
pub fn section_of(data: &Value, id: &str) -> Option<usize> {
    fn holds(v: &Value, id: &str) -> bool {
        match v {
            Value::Object(o) => {
                o.get("id").and_then(Value::as_str) == Some(id) || o.values().any(|x| holds(x, id))
            }
            Value::Array(l) => l.iter().any(|x| holds(x, id)),
            _ => false,
        }
    }
    data.as_array()?.iter().position(|s| holds(s, id))
}

/// Section `index` of a book's data, as HTML.
pub fn section(data: &Value, index: usize) -> String {
    let Some(sec) = data.as_array().and_then(|l| l.get(index)) else {
        return String::new();
    };
    render_section(sec, index)
}

/// A section (`data[index]`) as HTML.
pub fn render_section(sec: &Value, index: usize) -> String {
    if sec.is_null() {
        return String::new();
    }
    match sec.get("entries").and_then(Value::as_array) {
        Some(list) => {
            let head = sec
                .get("name")
                .and_then(Value::as_str)
                .map(|n| format!("<h1>{}</h1>", tags(n)))
                .unwrap_or_default();
            let body: String = list
                .iter()
                .enumerate()
                .map(|(i, e)| render(e, &[index, i], 0))
                .collect();
            format!(r#"<div class="section-content">{head}{body}</div>"#)
        }
        None => render(sec, &[index], 0),
    }
}

fn children(e: &Value, path: &[usize], depth: usize) -> String {
    e.get("entries")
        .and_then(Value::as_array)
        .map(|l| {
            l.iter()
                .enumerate()
                .map(|(i, x)| {
                    let mut p = path.to_vec();
                    p.push(i);
                    render(x, &p, depth)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn render(e: &Value, path: &[usize], depth: usize) -> String {
    if let Some(text) = e.as_str() {
        return format!("<p>{}</p>", tags(text));
    }
    let name = e.get("name").and_then(Value::as_str);
    match e.get("type").and_then(Value::as_str).unwrap_or("") {
        "section" => {
            let h = (depth + 2).min(6);
            format!(
                r#"<div class="section" id="{}"><h{h}>{}</h{h}>{}</div>"#,
                esc(&anchor(e, path)),
                tags(name.unwrap_or("")),
                children(e, path, depth + 1)
            )
        }
        "entries" => {
            let head = name
                .map(|n| {
                    let h = match depth {
                        0 => 2,
                        1 => 3,
                        _ => 4,
                    };
                    format!("<h{h}>{}</h{h}>", tags(n))
                })
                .unwrap_or_default();
            format!(
                r#"<div class="entries" id="{}">{head}{}</div>"#,
                esc(&anchor(e, path)),
                children(e, path, depth + 1)
            )
        }
        "insetReadaloud" => format!(
            r#"<div class="inset-readaloud">{}</div>"#,
            children(e, path, depth + 1)
        ),
        "inset" => format!(
            r#"<div class="inset">{}{}</div>"#,
            name.map(|n| format!("<h4>{}</h4>", tags(n)))
                .unwrap_or_default(),
            children(e, path, depth + 1)
        ),
        "list" => {
            let items: String = e
                .get("items")
                .and_then(Value::as_array)
                .map(|l| {
                    l.iter()
                        .enumerate()
                        .map(|(i, item)| match item.as_str() {
                            Some(t) => format!("<li>{}</li>", tags(t)),
                            None => {
                                let mut p = path.to_vec();
                                p.push(i);
                                format!("<li>{}</li>", render(item, &p, depth + 1))
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();
            format!(r#"<ul class="content-list">{items}</ul>"#)
        }
        "table" => table(e),
        "image" => {
            let p = e
                .get("href")
                .and_then(|h| h.get("path"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let file = p.rsplit('/').next().unwrap_or("image");
            // A map says so, with its grid (MIMIR-T-0736).
            let map = match e.get("imageType").and_then(Value::as_str) {
                Some("map") | Some("mapPlayer") => {
                    let grid = match e.pointer("/grid/type").and_then(Value::as_str) {
                        Some("square") => "map, square grid",
                        Some(g) if g.starts_with("hex") => "map, hex grid",
                        _ => "map",
                    };
                    let who = if e.get("imageType").and_then(Value::as_str) == Some("mapPlayer") {
                        ", for the players"
                    } else {
                        ""
                    };
                    format!(r#" <span class="image-kind">({grid}{who})</span>"#)
                }
                _ => String::new(),
            };
            let caption = e
                .get("title")
                .and_then(Value::as_str)
                .map(|t| format!(r#"<p class="image-caption">{}{map}</p>"#, tags(t)))
                .unwrap_or_default();
            format!(
                r#"<div class="image-container"><div class="image-wrapper" data-image-path="{}"><p class="image-placeholder">Loading image: {}</p></div>{caption}</div>"#,
                esc(p),
                esc(file)
            )
        }
        "quote" => {
            let by = e
                .get("by")
                .and_then(Value::as_str)
                .map(|b| format!("<cite>— {}</cite>", tags(b)))
                .unwrap_or_default();
            format!(
                "<blockquote>{}{by}</blockquote>",
                children(e, path, depth + 1)
            )
        }
        "item" | "itemSub" => {
            let head = name
                .map(|n| {
                    format!(
                        "<strong>{}:</strong> ",
                        tags(n.trim_end_matches([':', '.']))
                    )
                })
                .unwrap_or_default();
            let entry = e
                .get("entry")
                .and_then(Value::as_str)
                .map(tags)
                .unwrap_or_default();
            format!(
                r#"<div class="item-entry">{head}{entry}{}</div>"#,
                children(e, path, depth + 1)
            )
        }
        "options" => format!(
            r#"<div class="options-entry">{}</div>"#,
            children(e, path, depth + 1)
        ),
        "inline" => children(e, path, depth),
        "inlineBlock" => format!(
            r#"<span class="inline-block">{}</span>"#,
            children(e, path, depth)
        ),
        "statblock" => statblock(e),
        "hr" => "<hr>".into(),
        "gallery" => e
            .get("images")
            .and_then(Value::as_array)
            .map(|l| {
                l.iter()
                    .enumerate()
                    .map(|(i, img)| {
                        let mut p = path.to_vec();
                        p.push(i);
                        render(img, &p, depth)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        _ => {
            if e.get("entries").is_some() {
                children(e, path, depth)
            } else if let Some(t) = e.get("entry").and_then(Value::as_str) {
                format!("<p>{}</p>", tags(t))
            } else {
                // Unknown: shown as data, as the desktop (escaped).
                format!(
                    "<pre>{}</pre>",
                    esc(&serde_json::to_string_pretty(e).unwrap_or_default())
                )
            }
        }
    }
}

fn table(t: &Value) -> String {
    let mut html = String::from(r#"<div class="table-wrapper"><table class="content-table">"#);
    if let Some(c) = t.get("caption").and_then(Value::as_str) {
        html += &format!("<caption>{}</caption>", tags(c));
    }
    if let Some(labels) = t
        .get("colLabels")
        .and_then(Value::as_array)
        .filter(|l| !l.is_empty())
    {
        html += "<thead><tr>";
        for l in labels {
            html += &format!("<th>{}</th>", tags(s(l)));
        }
        html += "</tr></thead>";
    }
    if let Some(rows) = t
        .get("rows")
        .and_then(Value::as_array)
        .filter(|r| !r.is_empty())
    {
        html += "<tbody>";
        for row in rows {
            html += "<tr>";
            let cells = row
                .as_array()
                .or_else(|| row.get("row").and_then(Value::as_array));
            if let Some(cells) = cells {
                for cell in cells {
                    html += &format!("<td>{}</td>", cell_html(cell));
                }
            }
            html += "</tr>";
        }
        html += "</tbody>";
    }
    html + "</table></div>"
}

fn cell_html(cell: &Value) -> String {
    if let Some(t) = cell.as_str() {
        return tags(t);
    }
    if let Some(roll) = cell.get("roll") {
        return roll_cell(roll)
            .map(|r| format!(r#"<span class="dice-roll">{r}</span>"#))
            .unwrap_or_default();
    }
    if cell.get("type").and_then(Value::as_str) == Some("entries") {
        return cell
            .get("entries")
            .and_then(Value::as_array)
            .map(|l| {
                l.iter()
                    .map(|x| match x.as_str() {
                        Some(t) => format!("<div>{}</div>", tags(t)),
                        None => crate::entries::entry(x),
                    })
                    .collect()
            })
            .unwrap_or_default();
    }
    if cell.get("type").is_some() {
        return render(cell, &[0], 0);
    }
    match cell {
        Value::Null => String::new(),
        Value::Number(n) => n.to_string(),
        other => tags(&other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn one(e: Value) -> String {
        render_section(&json!({ "entries": [e] }), 0)
    }

    // The desktop's contentRenderer tests, ported.

    #[test]
    fn sections() {
        assert_eq!(render_section(&json!(null), 0), "");
        let r = render_section(
            &json!({"name": "Chapter 1", "entries": ["First paragraph", "Second paragraph"]}),
            0,
        );
        assert!(r.contains("<h1>Chapter 1</h1>"));
        assert!(r.contains("<p>First paragraph</p>") && r.contains("<p>Second paragraph</p>"));
        assert!(r.contains(r#"class="section-content""#));
    }

    #[test]
    fn section_and_entries_types() {
        let r = one(json!({"type": "section", "name": "My Section", "entries": ["x"]}));
        assert!(r.contains(r#"class="section""#) && r.contains("My Section") && r.contains("<h2>"));
        let r = one(json!({"type": "entries", "name": "Subsection", "entries": ["Content"]}));
        assert!(r.contains("Subsection") && r.contains("<p>Content</p>"));
        assert!(one(json!({"type": "entries", "entries": ["Just content"]}))
            .contains("<p>Just content</p>"));
    }

    #[test]
    fn insets_lists_tables() {
        let r = one(json!({"type": "insetReadaloud", "entries": ["Read this aloud"]}));
        assert!(r.contains(r#"class="inset-readaloud""#) && r.contains("Read this aloud"));
        let r = one(json!({"type": "inset", "name": "Sidebar", "entries": ["x"]}));
        assert!(r.contains(r#"class="inset""#) && r.contains("Sidebar"));
        let r = one(json!({"type": "list", "items": ["Item 1", "Item 2", "Item 3"]}));
        assert!(r.contains(r#"<ul class="content-list">"#) && r.contains("<li>Item 3</li>"));
        let r = one(
            json!({"type": "table", "caption": "Monster Table", "colLabels": ["Name", "CR"], "rows": [["Goblin", "1/4"], ["Dragon", "17"]]}),
        );
        assert!(r.contains("<table") && r.contains("<th>CR</th>") && r.contains("<td>Dragon</td>"));
        assert!(r.contains("<caption>Monster Table</caption>"));
        let r = one(json!({"type": "table", "rows": [[{"roll": {"min": 1, "max": 2}}, "x"]]}));
        assert!(r.contains("1-2"));
    }

    #[test]
    fn quotes_images_items_statblocks() {
        let r = one(json!({"type": "quote", "entries": ["Wise words"], "by": "Elminster"}));
        assert!(r.contains("<blockquote>") && r.contains("— Elminster"));
        let r = one(json!({"type": "image", "href": {"path": "img/map.png"}, "title": "Map"}));
        assert!(
            r.contains(r#"class="image-container""#)
                && r.contains(r#"data-image-path="img/map.png""#)
        );
        assert!(r.contains("Map"));
        let r = one(json!({"type": "item", "name": "Str", "entry": "18 (+4)"}));
        assert!(r.contains("<strong>Str:</strong>") && r.contains("18 (+4)"));
        let r = one(json!({"type": "statblock", "name": "Goblin", "source": "MM"}));
        assert!(r.contains(r#"class="cross-ref-link creature-ref""#) && r.contains("Goblin"));
        let r =
            one(json!({"type": "statblock", "name": "Longsword", "source": "PHB", "tag": "item"}));
        assert!(r.contains("See: Longsword") && !r.contains("cross-ref-link"));
        let r = one(json!("A {@creature goblin} rolls {@dice 1d6}"));
        assert!(r.contains("creature-ref") && r.contains(r#"class="dice-roll""#));
    }

    #[test]
    fn contents_and_anchors_agree() {
        let data = json!([
            {"name": "Intro", "entries": [
                {"type": "entries", "name": "Welcome", "entries": [{"type": "entries", "name": "Deeper", "entries": ["x"]}]},
                {"type": "section", "name": "Kept", "id": "abc", "entries": []},
                "text"
            ]},
            {"entries": ["The first words of an unnamed section"]}
        ]);
        let c = contents(&data);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].name, "Intro");
        assert_eq!(c[1].name, "The first words of an unnamed section");
        let names: Vec<_> = c[0]
            .headings
            .iter()
            .map(|h| (h.name.as_str(), h.depth))
            .collect();
        assert_eq!(names, [("Welcome", 1), ("Deeper", 2), ("Kept", 1)]);
        let html = section(&data, 0);
        for h in &c[0].headings {
            assert!(
                html.contains(&format!(r#"id="{}""#, h.id)),
                "{} in the page",
                h.id
            );
        }
        assert_eq!(c[0].headings[2].id, "abc");
    }

    #[test]
    fn unknown_types_are_escaped() {
        let r = one(json!({"type": "mystery", "x": "<b>"}));
        assert!(r.contains("<pre>") && r.contains("&lt;b&gt;"));
    }
}

#[cfg(test)]
mod adventure_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn area_links_find_their_chapter() {
        let data = json!([
            {"type": "section", "name": "One", "entries": ["See {@area the gate|g1|x}."]},
            {"type": "section", "name": "Two", "entries": [
                {"type": "entries", "name": "1. Gate", "id": "g1", "entries": ["x"]}
            ]}
        ]);
        assert_eq!(section_of(&data, "g1"), Some(1));
        assert_eq!(section_of(&data, "nope"), None);
        let html = section(&data, 0);
        assert!(
            html.contains(r##"<a href="#g1" class="area-ref" data-area="g1">the gate</a>"##),
            "{html}"
        );
    }

    #[test]
    fn maps_name_their_grid() {
        let data = json!([{"type": "section", "name": "One", "entries": [
            {"type": "image", "href": {"type": "internal", "path": "adventure/X/map.png"},
             "title": "Tower", "imageType": "map", "grid": {"type": "square", "size": 32}},
            {"type": "image", "href": {"type": "internal", "path": "adventure/X/region.png"},
             "title": "Region", "imageType": "mapPlayer", "grid": {"type": "hexColsOdd"}}
        ]}]);
        let html = section(&data, 0);
        assert!(
            html.contains("Tower <span class=\"image-kind\">(map, square grid)</span>"),
            "{html}"
        );
        assert!(html.contains("(map, hex grid, for the players)"), "{html}");
    }
}

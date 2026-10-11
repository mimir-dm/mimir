//! 5etools adventure entries to Mimir markdown and back (MIMIR-T-0738,
//! ADR MIMIR-A-0011). Mimir documents stay markdown; adventures come in and
//! go out as 5etools entries.
//!
//! The forms (decision 2 of COLLIERY-I-0616):
//!
//! | Entry | Markdown |
//! |---|---|
//! | `section`, named `entries` | a heading, one level deeper for each level of nesting; a kept `id` as `{#id}` |
//! | text | a paragraph; inline `{@…}` tags stay as they are |
//! | `insetReadaloud` | `> [!read-aloud] Name` and the inner blocks quoted |
//! | `inset` | `> [!sidebar] Name` and the inner blocks quoted |
//! | `quote` | a block quote; `by` as a last line `— by` |
//! | `list` | `- item` (`1.` for `list-decimal`); an `item` as `- **Name** text` |
//! | `table` | a GFM table; the caption as a line `Table: caption` before it |
//! | `image`, `gallery` | `![title](5etools:path)`, or the URL of an external image |
//! | `statblock` | the tag on a line of its own: `{@creature Name\|Source}`; lore (`prop: monsterFluff`) as `{@creatureFluff Name\|Source}` |
//! | `wrapper` | its blocks, in place |
//!
//! What a round trip (entries → markdown → entries) loses: `id`s that no
//! `{@area}` names, `page`, list styles other than `list-decimal` and
//! columns, `colStyles` and a cell's `roll` (the roll becomes its text), an
//! image's size, credit and map grid, heading depth below six, `wrapper`
//! groups, and a stand-alone `item` (it comes back as a paragraph that
//! starts with `{@b Name}`). The text, the headings, the boxes, the
//! tables, the stat blocks and the tags come back.

use std::collections::{BTreeSet, HashSet};

use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde_json::{json, Map, Value};

use crate::tags::{closing, link_kind};

/// The URL scheme of a 5etools image (`href.type: internal`): the path
/// under the imported 5etools images.
pub const IMAGE_SCHEME: &str = "5etools:";

/// The line form of a stat block that shows a creature's lore
/// (`prop: monsterFluff`). Mimir only: an export writes it back as a
/// `statblock`.
pub const FLUFF_TAG: &str = "creatureFluff";

/// The tags a stat block line may hold.
const STATBLOCK_TAGS: &[&str] = &[
    "creature",
    FLUFF_TAG,
    "item",
    "spell",
    "background",
    "deck",
    "object",
    "vehicle",
    "trap",
    "hazard",
    "race",
    "feat",
    "optfeature",
    "reward",
    "table",
];

/// Options of [`to_markdown`].
#[derive(Debug, Default, Clone)]
pub struct ToMarkdown {
    /// The `id`s to keep as heading anchors (`{#id}`): those an `{@area}`
    /// names. See [`area_targets`].
    pub keep_ids: HashSet<String>,
}

// =============================================================================
// Entries → markdown
// =============================================================================

/// Entries as markdown. `level`: the heading level of a named entry at the
/// top (1 for a whole chapter, 2 for the inside of a document).
pub fn to_markdown(entries: &[Value], level: usize, opts: &ToMarkdown) -> String {
    let mut blocks = Vec::new();
    for e in entries {
        block(e, level, opts, &mut blocks);
    }
    let mut out = blocks.join("\n\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

fn name_of(o: &Map<String, Value>) -> Option<&str> {
    o.get("name")
        .and_then(Value::as_str)
        .filter(|n| !n.trim().is_empty())
}

fn children(o: &Map<String, Value>, key: &str) -> Vec<Value> {
    o.get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn block(e: &Value, level: usize, opts: &ToMarkdown, out: &mut Vec<String>) {
    let o = match e {
        Value::String(s) => {
            out.push(paragraph(s));
            return;
        }
        Value::Object(o) => o,
        _ => return,
    };
    let kind = o.get("type").and_then(Value::as_str).unwrap_or("entries");
    match kind {
        "section" | "entries" => {
            let mut inner_level = level;
            if let Some(name) = name_of(o) {
                let id = o
                    .get("id")
                    .and_then(Value::as_str)
                    .filter(|id| opts.keep_ids.contains(*id));
                out.push(heading(level, name, id));
                inner_level = level + 1;
            }
            for c in children(o, "entries") {
                block(&c, inner_level, opts, out);
            }
        }
        "insetReadaloud" | "inset" => {
            let marker = if kind == "inset" {
                "sidebar"
            } else {
                "read-aloud"
            };
            let head = match name_of(o) {
                Some(n) => format!("[!{marker}] {}", inline(n)),
                None => format!("[!{marker}]"),
            };
            let inner = to_markdown(&children(o, "entries"), level + 1, opts);
            out.push(quoted(&head, &inner));
        }
        "quote" => {
            let mut inner = to_markdown(&children(o, "entries"), level + 1, opts);
            if let Some(by) = o.get("by").and_then(Value::as_str) {
                inner.push('\n');
                inner.push_str(&format!("— {}\n", inline(by)));
            }
            let mut lines = inner.trim_end().lines();
            let first = lines.next().unwrap_or("").to_string();
            let rest: String = lines.map(|l| format!("{l}\n")).collect();
            out.push(quoted(&first, &rest));
        }
        "list" => {
            // A list right after a list takes the other marker, or the two
            // would read back as one list.
            let ordered = o.get("style").and_then(Value::as_str) == Some("list-decimal");
            let same = if ordered { "1. " } else { "- " };
            let after_list = out.last().is_some_and(|l| l.starts_with(same));
            out.push(list(o, level, opts, after_list));
        }
        "table" => out.extend(table(o)),
        "image" => out.push(image(o)),
        "gallery" => {
            for img in children(o, "images") {
                if let Some(io) = img.as_object() {
                    out.push(image(io));
                }
            }
        }
        "statblock" => {
            if let Some(line) = statblock_line(o) {
                out.push(line);
            }
        }
        "wrapper" => {
            for c in children(o, "wrappeds") {
                block(&c, level, opts, out);
            }
        }
        "item" | "itemSub" | "itemSpell" => out.push(item_text(o)),
        _ => {
            if o.contains_key("entries") {
                for c in children(o, "entries") {
                    block(&c, level, opts, out);
                }
            } else if let Some(t) = o
                .get("entry")
                .or_else(|| o.get("text"))
                .and_then(Value::as_str)
            {
                out.push(paragraph(t));
            }
        }
    }
}

fn heading(level: usize, name: &str, id: Option<&str>) -> String {
    let hashes = "#".repeat(level.clamp(1, 6));
    let mut text = inline_from(name, false);
    // A heading that ends with a tag would read as `{attributes}`.
    if text.ends_with('}') {
        text.insert(text.len() - 1, '\\');
    }
    match id {
        Some(id) => format!("{hashes} {text} {{#{id}}}"),
        None => format!("{hashes} {text}"),
    }
}

/// `> head` then each line of `body` quoted.
fn quoted(head: &str, body: &str) -> String {
    let mut s = format!("> {head}");
    let body = body.trim_end();
    if !body.is_empty() {
        s.push_str("\n>");
        for line in body.lines() {
            if line.is_empty() {
                s.push_str("\n>");
            } else {
                s.push_str("\n> ");
                s.push_str(line);
            }
        }
    }
    s
}

fn paragraph(text: &str) -> String {
    // A text that is only a stat block tag would read back as a stat block:
    // `\{@creature …}` keeps it text.
    let p = inline(text.trim());
    if parse_statblock_line(text.trim()).is_some() {
        format!("\\{p}")
    } else {
        p
    }
}

/// Escape text for markdown, so it reads back the same. Tags stay
/// readable: `{`, `}`, `|` and `@` are not escaped.
pub fn inline(text: &str) -> String {
    inline_from(text, true)
}

/// As [`inline`]; `at_line_start`: whether the text starts a line (a
/// heading's text does not: it follows the `#`s).
fn inline_from(text: &str, at_line_start: bool) -> String {
    let mut at_line_start = at_line_start;
    let mut out = String::with_capacity(text.len() + 8);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            out.push(' ');
            at_line_start = true;
            i += 1;
            continue;
        }
        if at_line_start {
            // Block markers at the start of a line.
            match c {
                '#' | '>' | '-' | '+' | '=' => out.push('\\'),
                d if d.is_ascii_digit() => {
                    let mut j = i;
                    while j < chars.len() && chars[j].is_ascii_digit() {
                        j += 1;
                    }
                    if j < chars.len() && (chars[j] == '.' || chars[j] == ')') {
                        out.extend(&chars[i..j]);
                        out.push('\\');
                        out.push(chars[j]);
                        i = j + 1;
                        at_line_start = false;
                        continue;
                    }
                }
                _ => {}
            }
        }
        if matches!(c, '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '&' | '~') {
            out.push('\\');
        }
        out.push(c);
        at_line_start = false;
        i += 1;
    }
    out
}

fn list(o: &Map<String, Value>, level: usize, opts: &ToMarkdown, other_marker: bool) -> String {
    let ordered = o.get("style").and_then(Value::as_str) == Some("list-decimal");
    let mut lines = Vec::new();
    for (n, item) in children(o, "items").iter().enumerate() {
        let marker = match (ordered, other_marker) {
            (true, false) => format!("{}. ", n + 1),
            (true, true) => format!("{}) ", n + 1),
            (false, false) => "- ".to_string(),
            (false, true) => "* ".to_string(),
        };
        let pad = " ".repeat(marker.len());
        // A list inside a list belongs to the item before it.
        if let (Some("list"), false) = (item.get("type").and_then(Value::as_str), lines.is_empty())
        {
            let inner = to_markdown(std::slice::from_ref(item), level, opts);
            for l in inner.trim_end().lines() {
                lines.push(format!("  {l}"));
            }
            continue;
        }
        let body = match item {
            Value::String(s) => inline(s),
            Value::Object(io) => match io.get("type").and_then(Value::as_str) {
                Some("item" | "itemSub" | "itemSpell") => item_text(io),
                _ => to_markdown(std::slice::from_ref(item), level, opts)
                    .trim_end()
                    .to_string(),
            },
            _ => continue,
        };
        let mut body_lines = body.lines();
        lines.push(format!("{marker}{}", body_lines.next().unwrap_or("")));
        for l in body_lines {
            if l.is_empty() {
                lines.push(String::new());
            } else {
                lines.push(format!("{pad}{l}"));
            }
        }
    }
    lines.join("\n")
}

/// `**Name** text` (a list item with a name).
fn item_text(o: &Map<String, Value>) -> String {
    let name = name_of(o).map(|n| format!("**{}**", inline(n)));
    let body = match o.get("entry").and_then(Value::as_str) {
        Some(t) => inline(t),
        None => children(o, "entries")
            .iter()
            .filter_map(Value::as_str)
            .map(inline)
            .collect::<Vec<_>>()
            .join(" "),
    };
    match name {
        Some(n) if body.is_empty() => n,
        Some(n) => format!("{n} {body}"),
        None => body,
    }
}

/// The text of a table cell, on one line.
fn cell_text(v: &Value) -> String {
    match v {
        Value::String(s) => inline(s),
        Value::Number(n) => n.to_string(),
        Value::Object(o) => {
            if o.get("type").and_then(Value::as_str) == Some("cell") {
                if let Some(e) = o.get("entry") {
                    return cell_text(e);
                }
                if let Some(r) = o.get("roll").and_then(Value::as_object) {
                    let num = |k: &str| r.get(k).and_then(Value::as_i64);
                    return match (num("exact"), num("min"), num("max")) {
                        (Some(x), _, _) => x.to_string(),
                        (_, Some(a), Some(b)) => format!("{a}–{b}"),
                        _ => String::new(),
                    };
                }
            }
            if let Some(t) = o
                .get("entry")
                .or_else(|| o.get("text"))
                .and_then(Value::as_str)
            {
                return inline(t);
            }
            let parts: Vec<String> = ["entries", "items"]
                .iter()
                .flat_map(|k| children(o, k))
                .map(|c| cell_text(&c))
                .filter(|s| !s.is_empty())
                .collect();
            match name_of(o) {
                Some(n) if !parts.is_empty() => format!("**{}** {}", inline(n), parts.join(" ")),
                Some(n) => format!("**{}**", inline(n)),
                None => parts.join(" "),
            }
        }
        _ => String::new(),
    }
    .replace('|', "\\|")
}

fn table(o: &Map<String, Value>) -> Vec<String> {
    let labels: Vec<String> = children(o, "colLabels").iter().map(cell_text).collect();
    let rows: Vec<Vec<String>> = children(o, "rows")
        .iter()
        .map(|r| {
            r.as_array()
                .map(|cells| cells.iter().map(cell_text).collect())
                .unwrap_or_default()
        })
        .collect();
    let width = rows
        .iter()
        .map(Vec::len)
        .chain(std::iter::once(labels.len()))
        .max()
        .unwrap_or(0)
        .max(1);
    let row = |cells: &[String]| {
        let mut s = String::from("|");
        for i in 0..width {
            s.push(' ');
            s.push_str(cells.get(i).map(String::as_str).unwrap_or(""));
            s.push_str(" |");
        }
        s
    };
    let mut lines = vec![row(&labels), format!("|{}", " --- |".repeat(width))];
    lines.extend(rows.iter().map(|r| row(r)));
    let mut out = Vec::new();
    if let Some(c) = o
        .get("caption")
        .and_then(Value::as_str)
        .filter(|c| !c.trim().is_empty())
    {
        out.push(format!("Table: {}", inline(c)));
    }
    out.push(lines.join("\n"));
    for f in children(o, "footnotes") {
        if let Some(t) = f.as_str() {
            out.push(paragraph(t));
        }
    }
    out
}

fn image(o: &Map<String, Value>) -> String {
    let href = o.get("href").and_then(Value::as_object);
    let url = match href {
        Some(h) if h.get("type").and_then(Value::as_str) == Some("external") => h
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        Some(h) => format!(
            "{IMAGE_SCHEME}{}",
            h.get("path").and_then(Value::as_str).unwrap_or("")
        ),
        None => String::new(),
    };
    let title = o.get("title").and_then(Value::as_str).unwrap_or("");
    format!("![{}](<{}>)", inline(title), url.replace('>', "%3E"))
}

fn statblock_line(o: &Map<String, Value>) -> Option<String> {
    let name = o.get("name").and_then(Value::as_str)?;
    let fluff = o.get("prop").and_then(Value::as_str) == Some("monsterFluff");
    let tag = if fluff {
        FLUFF_TAG
    } else {
        o.get("tag").and_then(Value::as_str).unwrap_or("creature")
    };
    Some(match o.get("source").and_then(Value::as_str) {
        Some(src) => format!("{{@{tag} {name}|{src}}}"),
        None => format!("{{@{tag} {name}}}"),
    })
}

// =============================================================================
// Markdown → entries
// =============================================================================

/// One block of parsed markdown, before nesting by headings.
#[derive(Debug)]
enum Block {
    Heading(usize, String, Option<String>),
    Entry(Value),
}

/// Markdown as 5etools entries. A heading opens a named entry that holds
/// what follows, up to the next heading of the same or a higher level; a
/// heading of level `section_level` or higher is a `section`, a deeper one
/// an `entries`.
pub fn from_markdown(md: &str, section_level: usize) -> Vec<Value> {
    let (events, ranges): (Vec<Event>, Vec<std::ops::Range<usize>>) = Parser::new_ext(
        md,
        Options::ENABLE_TABLES | Options::ENABLE_HEADING_ATTRIBUTES,
    )
    .into_offset_iter()
    .unzip();
    let doc = Doc {
        events,
        ranges,
        src: md,
    };
    let mut pos = 0;
    let blocks = parse_blocks(&doc, &mut pos, None);
    let mut out = nest(blocks, section_level);
    out.iter_mut().for_each(one_line);
    out
}

/// Parsed markdown: the events, their source ranges, the source.
struct Doc<'a> {
    events: Vec<Event<'a>>,
    ranges: Vec<std::ops::Range<usize>>,
    src: &'a str,
}

/// Soft line breaks (kept as `\n` while parsing, for a callout's first
/// line) become spaces.
fn one_line(v: &mut Value) {
    match v {
        Value::String(s) if s.contains('\n') => *s = s.replace('\n', " "),
        Value::Array(l) => l.iter_mut().for_each(one_line),
        Value::Object(o) => o.values_mut().for_each(one_line),
        _ => {}
    }
}

/// Nest blocks under their headings.
fn nest(blocks: Vec<Block>, section_level: usize) -> Vec<Value> {
    // A stack of open headings: (level, entry, its children).
    let mut root: Vec<Value> = Vec::new();
    let mut stack: Vec<(usize, Map<String, Value>, Vec<Value>)> = Vec::new();
    fn close(
        stack: &mut Vec<(usize, Map<String, Value>, Vec<Value>)>,
        root: &mut Vec<Value>,
        down_to: usize,
    ) {
        while let Some((lvl, _, _)) = stack.last() {
            if *lvl < down_to {
                break;
            }
            let (_, mut obj, kids) = stack.pop().expect("open heading");
            obj.insert("entries".into(), Value::Array(kids));
            let v = Value::Object(obj);
            match stack.last_mut() {
                Some((_, _, parent)) => parent.push(v),
                None => root.push(v),
            }
        }
    }
    for b in blocks {
        match b {
            Block::Heading(level, name, id) => {
                close(&mut stack, &mut root, level);
                let mut obj = Map::new();
                let kind = if level <= section_level {
                    "section"
                } else {
                    "entries"
                };
                obj.insert("type".into(), kind.into());
                obj.insert("name".into(), name.into());
                if let Some(id) = id {
                    obj.insert("id".into(), id.into());
                }
                stack.push((level, obj, Vec::new()));
            }
            Block::Entry(v) => match stack.last_mut() {
                Some((_, _, kids)) => kids.push(v),
                None => root.push(v),
            },
        }
    }
    close(&mut stack, &mut root, 0);
    root
}

/// Parse blocks until the end tag `until` (or the end).
fn parse_blocks(doc: &Doc, pos: &mut usize, until: Option<TagEnd>) -> Vec<Block> {
    let events = &doc.events[..];
    let mut out = Vec::new();
    // A `Table: caption` paragraph waits for its table.
    let mut caption: Option<String> = None;
    while *pos < events.len() {
        let ev = &events[*pos];
        if let (Event::End(end), Some(u)) = (ev, until) {
            if *end == u {
                *pos += 1;
                break;
            }
        }
        *pos += 1;
        let mut next_caption = None;
        match ev {
            Event::Start(Tag::Heading { level, id, .. }) => {
                let text = inline_until(events, pos, TagEnd::Heading(*level));
                out.push(Block::Heading(
                    heading_number(*level),
                    text,
                    id.as_ref().map(|i| i.to_string()),
                ));
            }
            Event::Start(Tag::Paragraph) => {
                // `\{@…}`: a text, not a stat block.
                let escaped = doc
                    .src
                    .get(doc.ranges[*pos - 1].clone())
                    .is_some_and(|raw| raw.trim_start().starts_with("\\{@"));
                let para = paragraph_until(events, pos);
                match para {
                    Para::Text(t) => {
                        if let Some(c) = t.strip_prefix("Table: ") {
                            next_caption = Some(c.to_string());
                        } else if let Some(sb) = parse_statblock_line(&t).filter(|_| !escaped) {
                            out.push(Block::Entry(sb));
                        } else {
                            out.push(Block::Entry(Value::String(t.trim_end().to_string())));
                        }
                    }
                    Para::Images(imgs) => {
                        out.extend(imgs.into_iter().map(Block::Entry));
                    }
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                out.push(Block::Entry(block_quote(doc, pos)));
            }
            Event::Start(Tag::List(start)) => {
                out.push(Block::Entry(list_entry(doc, pos, *start)));
            }
            Event::Start(Tag::Table(aligns)) => {
                out.push(Block::Entry(table_entry(
                    events,
                    pos,
                    aligns,
                    caption.take(),
                )));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let end = TagEnd::CodeBlock;
                let mut text = String::new();
                while *pos < events.len() {
                    match &events[*pos] {
                        Event::End(e) if *e == end => {
                            *pos += 1;
                            break;
                        }
                        Event::Text(t) => text.push_str(t),
                        _ => {}
                    }
                    *pos += 1;
                }
                let _ = kind as &CodeBlockKind;
                for line in text.lines().filter(|l| !l.trim().is_empty()) {
                    out.push(Block::Entry(Value::String(format!("{{@code {line}}}"))));
                }
            }
            Event::Start(Tag::HtmlBlock) => {
                skip_to(events, pos, TagEnd::HtmlBlock);
            }
            Event::Start(t) => {
                // Anything else: its text as a paragraph.
                let end = t.to_end();
                let text = inline_until(events, pos, end);
                if !text.trim().is_empty() {
                    out.push(Block::Entry(Value::String(text)));
                }
            }
            _ => {}
        }
        if let Some(c) = caption.take() {
            // A caption with no table right after it is text.
            out.push(Block::Entry(Value::String(format!("Table: {c}"))));
        }
        caption = next_caption;
    }
    if let Some(c) = caption {
        out.push(Block::Entry(Value::String(format!("Table: {c}"))));
    }
    out
}

fn heading_number(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn skip_to(events: &[Event], pos: &mut usize, end: TagEnd) {
    while *pos < events.len() {
        let done = matches!(&events[*pos], Event::End(e) if *e == end);
        *pos += 1;
        if done {
            break;
        }
    }
}

enum Para {
    Text(String),
    /// A paragraph of images only.
    Images(Vec<Value>),
}

fn paragraph_until(events: &[Event], pos: &mut usize) -> Para {
    let start = *pos;
    let mut images = Vec::new();
    let mut only_images = true;
    let mut i = start;
    while i < events.len() {
        match &events[i] {
            Event::End(TagEnd::Paragraph) => break,
            Event::Start(Tag::Image { dest_url, .. }) => {
                let title = inline_until(events, &mut (i + 1), TagEnd::Image);
                images.push(image_entry(dest_url, &title));
                // Skip to the end of the image.
                while i < events.len() && !matches!(&events[i], Event::End(TagEnd::Image)) {
                    i += 1;
                }
            }
            Event::SoftBreak | Event::HardBreak => {}
            Event::Text(t) if t.trim().is_empty() => {}
            _ => only_images = false,
        }
        i += 1;
    }
    if only_images && !images.is_empty() {
        *pos = i + 1;
        return Para::Images(images);
    }
    Para::Text(inline_until(events, pos, TagEnd::Paragraph))
}

fn image_entry(url: &str, title: &str) -> Value {
    let href = match url.strip_prefix(IMAGE_SCHEME) {
        Some(path) => json!({"type": "internal", "path": path}),
        None => json!({"type": "external", "url": url}),
    };
    let mut o = Map::new();
    o.insert("type".into(), "image".into());
    o.insert("href".into(), href);
    if !title.is_empty() {
        o.insert("title".into(), title.into());
    }
    Value::Object(o)
}

/// The inline text up to `end`, as 5etools text: emphasis as `{@b}` and
/// `{@i}`, code as `{@code}`, a link as `{@link}`.
fn inline_until(events: &[Event], pos: &mut usize, end: TagEnd) -> String {
    let mut out = String::new();
    while *pos < events.len() {
        let ev = &events[*pos];
        *pos += 1;
        match ev {
            Event::End(e) if *e == end => break,
            Event::Text(t) => out.push_str(t),
            Event::Code(c) => out.push_str(&format!("{{@code {c}}}")),
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            Event::Start(Tag::Strong) => {
                let inner = inline_until(events, pos, TagEnd::Strong);
                out.push_str(&format!("{{@b {inner}}}"));
            }
            Event::Start(Tag::Emphasis) => {
                let inner = inline_until(events, pos, TagEnd::Emphasis);
                out.push_str(&format!("{{@i {inner}}}"));
            }
            Event::Start(Tag::Strikethrough) => {
                let inner = inline_until(events, pos, TagEnd::Strikethrough);
                out.push_str(&format!("{{@s {inner}}}"));
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let inner = inline_until(events, pos, TagEnd::Link);
                out.push_str(&format!("{{@link {inner}|{dest_url}}}"));
            }
            Event::Start(Tag::Image { .. }) => {
                // An image inside text: its title only.
                out.push_str(&inline_until(events, pos, TagEnd::Image));
            }
            Event::Html(h) | Event::InlineHtml(h) => out.push_str(h),
            _ => {}
        }
    }
    out
}

/// A block quote: a read-aloud box, a sidebar, or a quote.
fn block_quote(doc: &Doc, pos: &mut usize) -> Value {
    let mut blocks = parse_blocks(doc, pos, Some(TagEnd::BlockQuote(None)));
    // A callout marker opens the first paragraph: `[!read-aloud] Name`, on
    // its own line.
    let mut marker: Option<(&str, Option<String>, Option<String>)> = None;
    if let Some(Block::Entry(Value::String(first))) = blocks.first() {
        let (line, after) = match first.split_once('\n') {
            Some((l, a)) => (l, Some(a.trim().to_string()).filter(|a| !a.is_empty())),
            None => (first.as_str(), None),
        };
        for (m, kind) in [("[!read-aloud]", "insetReadaloud"), ("[!sidebar]", "inset")] {
            if let Some(rest) = line.strip_prefix(m) {
                let name = rest.trim();
                marker = Some((
                    kind,
                    (!name.is_empty()).then(|| name.to_string()),
                    after.clone(),
                ));
            }
        }
    }
    match marker {
        Some((kind, name, after)) => {
            blocks.remove(0);
            if let Some(a) = after {
                blocks.insert(0, Block::Entry(Value::String(a)));
            }
            let mut o = Map::new();
            o.insert("type".into(), kind.into());
            if let Some(n) = name {
                o.insert("name".into(), n.into());
            }
            o.insert("entries".into(), Value::Array(nest(blocks, 0)));
            Value::Object(o)
        }
        None => {
            let mut entries = nest(blocks, 0);
            let mut o = Map::new();
            o.insert("type".into(), "quote".into());
            if let Some(Value::String(last)) = entries.last() {
                if let Some(by) = last.strip_prefix("— ") {
                    o.insert("by".into(), by.to_string().into());
                    entries.pop();
                }
            }
            o.insert("entries".into(), Value::Array(entries));
            Value::Object(o)
        }
    }
}

fn list_entry(doc: &Doc, pos: &mut usize, start: Option<u64>) -> Value {
    let events = &doc.events[..];
    let end = TagEnd::List(start.is_some());
    let mut items = Vec::new();
    while *pos < events.len() {
        let ev = &events[*pos];
        *pos += 1;
        match ev {
            Event::End(e) if *e == end => break,
            Event::Start(Tag::Item) => items.push(list_item(doc, pos)),
            _ => {}
        }
    }
    let mut o = Map::new();
    o.insert("type".into(), "list".into());
    if start.is_some() {
        o.insert("style".into(), "list-decimal".into());
    }
    o.insert("items".into(), Value::Array(items));
    Value::Object(o)
}

/// One list item: its text, an `item` when it starts with a bold name, or
/// `entries` when it holds more than one block.
fn list_item(doc: &Doc, pos: &mut usize) -> Value {
    let events = &doc.events[..];
    // A tight list item holds inline events directly; a loose one holds
    // paragraphs. Read it as blocks either way.
    let mut blocks: Vec<Value> = Vec::new();
    let mut text = String::new();
    let mut name: Option<String> = None;
    // A named `entries` in a list is a heading in the item.
    let mut heading: Option<(String, Option<String>)> = None;
    let mut first_inline = true;
    while *pos < events.len() {
        let ev = &events[*pos];
        match ev {
            Event::End(TagEnd::Item) => {
                *pos += 1;
                break;
            }
            Event::Start(Tag::Strong) if first_inline && text.is_empty() && blocks.is_empty() => {
                *pos += 1;
                name = Some(inline_until(events, pos, TagEnd::Strong));
                first_inline = false;
                continue;
            }
            Event::Start(Tag::Paragraph) => {
                *pos += 1;
                // The first paragraph may start with the bold name too.
                if blocks.is_empty() && text.is_empty() && name.is_none() {
                    if let Some(Event::Start(Tag::Strong)) = events.get(*pos) {
                        *pos += 1;
                        name = Some(inline_until(events, pos, TagEnd::Strong));
                    }
                }
                let t = inline_until(events, pos, TagEnd::Paragraph);
                if blocks.is_empty() && text.is_empty() {
                    text = t;
                } else {
                    blocks.push(Value::String(t.trim().to_string()));
                }
                first_inline = false;
                continue;
            }
            Event::Start(Tag::Heading { level, id, .. })
                if heading.is_none() && text.is_empty() && blocks.is_empty() =>
            {
                *pos += 1;
                let n = inline_until(events, pos, TagEnd::Heading(*level));
                heading = Some((n, id.as_ref().map(|i| i.to_string())));
                first_inline = false;
                continue;
            }
            Event::Start(Tag::Table(aligns)) => {
                *pos += 1;
                if !text.is_empty() {
                    blocks.push(Value::String(std::mem::take(&mut text).trim().to_string()));
                }
                blocks.push(table_entry(events, pos, aligns, None));
                continue;
            }
            Event::Start(Tag::List(start)) => {
                *pos += 1;
                if !text.is_empty() {
                    blocks.push(Value::String(std::mem::take(&mut text).trim().to_string()));
                }
                blocks.push(list_entry(doc, pos, *start));
                first_inline = false;
                continue;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                *pos += 1;
                if !text.is_empty() {
                    blocks.push(Value::String(std::mem::take(&mut text).trim().to_string()));
                }
                blocks.push(block_quote(doc, pos));
                continue;
            }
            Event::Start(_)
            | Event::Text(_)
            | Event::Code(_)
            | Event::SoftBreak
            | Event::HardBreak => {
                // Inline content of a tight item: read up to the item's end
                // or a nested block.
                let mut t = String::new();
                while *pos < events.len() {
                    match &events[*pos] {
                        Event::End(TagEnd::Item)
                        | Event::Start(Tag::List(_))
                        | Event::Start(Tag::BlockQuote(_))
                        | Event::Start(Tag::Paragraph) => break,
                        Event::Start(Tag::Strong) => {
                            *pos += 1;
                            let inner = inline_until(events, pos, TagEnd::Strong);
                            t.push_str(&format!("{{@b {inner}}}"));
                        }
                        Event::Start(Tag::Emphasis) => {
                            *pos += 1;
                            let inner = inline_until(events, pos, TagEnd::Emphasis);
                            t.push_str(&format!("{{@i {inner}}}"));
                        }
                        Event::Start(Tag::Link { dest_url, .. }) => {
                            *pos += 1;
                            let inner = inline_until(events, pos, TagEnd::Link);
                            t.push_str(&format!("{{@link {inner}|{dest_url}}}"));
                        }
                        Event::Text(s) => {
                            t.push_str(s);
                            *pos += 1;
                        }
                        Event::Code(c) => {
                            t.push_str(&format!("{{@code {c}}}"));
                            *pos += 1;
                        }
                        Event::SoftBreak | Event::HardBreak => {
                            t.push(' ');
                            *pos += 1;
                        }
                        _ => *pos += 1,
                    }
                }
                text.push_str(&t);
                first_inline = false;
                continue;
            }
            _ => {
                *pos += 1;
            }
        }
    }
    let text = text.trim().to_string();
    if let Some((n, id)) = heading {
        let mut entries = Vec::new();
        if !text.is_empty() {
            entries.push(Value::String(text));
        }
        entries.extend(blocks);
        let mut o = Map::new();
        o.insert("type".into(), "entries".into());
        o.insert("name".into(), n.into());
        if let Some(id) = id {
            o.insert("id".into(), id.into());
        }
        o.insert("entries".into(), Value::Array(entries));
        return Value::Object(o);
    }
    match name {
        Some(n) => {
            let mut o = Map::new();
            o.insert("type".into(), "item".into());
            o.insert("name".into(), n.into());
            if blocks.is_empty() {
                o.insert("entry".into(), text.into());
            } else {
                let mut entries = vec![Value::String(text)];
                entries.extend(blocks);
                o.insert("entries".into(), Value::Array(entries));
            }
            Value::Object(o)
        }
        None if blocks.is_empty() => Value::String(text),
        None => {
            let mut entries = Vec::new();
            if !text.is_empty() {
                entries.push(Value::String(text));
            }
            entries.extend(blocks);
            json!({"type": "entries", "entries": entries})
        }
    }
}

fn table_entry(
    events: &[Event],
    pos: &mut usize,
    _aligns: &[Alignment],
    caption: Option<String>,
) -> Value {
    let mut labels: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut in_head = false;
    while *pos < events.len() {
        let ev = &events[*pos];
        *pos += 1;
        match ev {
            Event::End(TagEnd::Table) => break,
            Event::Start(Tag::TableHead) => in_head = true,
            Event::End(TagEnd::TableHead) => in_head = false,
            Event::Start(Tag::TableRow) => rows.push(Vec::new()),
            Event::Start(Tag::TableCell) => {
                let text = inline_until(events, pos, TagEnd::TableCell);
                let text = text.trim().to_string();
                if in_head {
                    labels.push(text);
                } else if let Some(r) = rows.last_mut() {
                    r.push(text);
                }
            }
            _ => {}
        }
    }
    let mut o = Map::new();
    o.insert("type".into(), "table".into());
    if let Some(c) = caption {
        o.insert("caption".into(), c.into());
    }
    if labels.iter().any(|l| !l.is_empty()) {
        o.insert("colLabels".into(), json!(labels));
    }
    o.insert("rows".into(), json!(rows));
    Value::Object(o)
}

/// A paragraph that is only a stat block tag: `{@creature Name|Source}`.
fn parse_statblock_line(text: &str) -> Option<Value> {
    let t = text.trim();
    if !t.starts_with("{@") || closing(t, 0) != Some(t.len() - 1) {
        return None;
    }
    let inner = &t[2..t.len() - 1];
    let (tag, args) = inner.split_once(' ')?;
    if !STATBLOCK_TAGS.contains(&tag) || args.contains("{@") {
        return None;
    }
    let mut parts = args.split('|');
    let name = parts.next()?.trim();
    let source = parts.next().map(str::trim).filter(|s| !s.is_empty());
    if parts.next().is_some() || name.is_empty() {
        return None;
    }
    let mut o = Map::new();
    o.insert("type".into(), "statblock".into());
    if tag == FLUFF_TAG {
        o.insert("tag".into(), "creature".into());
        o.insert("prop".into(), "monsterFluff".into());
    } else {
        o.insert("tag".into(), tag.into());
    }
    o.insert("name".into(), name.into());
    if let Some(s) = source {
        o.insert("source".into(), s.into());
    }
    Some(Value::Object(o))
}

// =============================================================================
// An adventure as a campaign (MIMIR-T-0739)
// =============================================================================

/// What a part of an adventure (a chapter, an appendix) becomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartRole {
    /// A module, with a document per top-level section.
    Module,
    /// One campaign document.
    Document,
    /// Left out.
    Skip,
}

/// A document of the plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanDocument {
    pub title: String,
    pub markdown: String,
    /// The `{@area}` targets in it (its own section's id, and its kept
    /// heading ids): a link from another document goes here.
    pub anchors: Vec<String>,
}

/// A square-grid map image of a part: it becomes a Mimir map.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanMap {
    pub title: String,
    /// The image under the 5etools images.
    pub path: String,
    /// Pixels a square, from the image's grid.
    pub grid_px: f64,
}

/// A part of the adventure, and what it becomes.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanPart {
    /// Its index in the adventure's data.
    pub index: usize,
    pub name: String,
    /// The role it takes unless the DM chooses another.
    pub role: PartRole,
    /// As a module: one document per top-level section, a keyed area
    /// joined to the section before it.
    pub documents: Vec<PlanDocument>,
    /// As one document (a campaign document, or a module the DM wants in
    /// one piece).
    pub whole: PlanDocument,
    pub maps: Vec<PlanMap>,
    /// The creatures it names (stat blocks and tags): (name, source).
    pub creatures: Vec<(String, String)>,
}

/// A keyed area: "1. Cave Mouth", "K1. Front Courtyard", "A1: Path",
/// "12a. Vault".
pub fn is_keyed(name: &str) -> bool {
    let b = name.as_bytes();
    let mut i = 0;
    while i < b.len() && i < 3 && b[i].is_ascii_uppercase() {
        i += 1;
    }
    let digits = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == digits {
        return false;
    }
    if i < b.len() && b[i].is_ascii_alphabetic() {
        i += 1;
    }
    i + 1 < b.len() && (b[i] == b'.' || b[i] == b':') && b[i + 1] == b' '
}

/// The plan of an adventure: its data (`adventureData[0].data`) and its
/// contents (from `adventures.json`). A numbered chapter or part is a
/// module; the rest (introduction, appendices, credits) a campaign
/// document.
pub fn plan(data: &[Value], contents: &[Value]) -> Vec<PlanPart> {
    let opts = ToMarkdown {
        keep_ids: area_targets(&Value::Array(data.to_vec())),
    };
    data.iter()
        .enumerate()
        .map(|(index, part)| {
            let name = part
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Untitled")
                .to_string();
            let numbered = contents
                .get(index)
                .and_then(|c| c.pointer("/ordinal/type"))
                .and_then(Value::as_str)
                .is_some_and(|t| matches!(t, "chapter" | "part" | "episode" | "level"));
            let entries = part
                .get("entries")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            PlanPart {
                index,
                role: if numbered {
                    PartRole::Module
                } else {
                    PartRole::Document
                },
                documents: section_documents(&name, &entries, &opts),
                whole: PlanDocument {
                    title: name.clone(),
                    markdown: to_markdown(&entries, 2, &opts),
                    anchors: anchors_in(part, &opts.keep_ids),
                },
                maps: square_maps(part, &name),
                creatures: refs(part)
                    .into_iter()
                    .filter(|r| r.kind == "creature")
                    .map(|r| (r.name, r.source))
                    .collect(),
                name,
            }
        })
        .collect()
}

/// One document per top-level named section; a keyed area and unnamed
/// blocks join the document before them; blocks before the first section
/// make a document named after the part.
fn section_documents(part: &str, entries: &[Value], opts: &ToMarkdown) -> Vec<PlanDocument> {
    let mut docs: Vec<(String, Vec<Value>, Vec<String>)> = Vec::new();
    for e in entries {
        let named = e
            .as_object()
            .filter(|o| {
                matches!(
                    o.get("type").and_then(Value::as_str),
                    Some("section" | "entries")
                )
            })
            .and_then(name_of);
        match named {
            Some(n) if !is_keyed(n) || docs.is_empty() => {
                // A new document: its title is the section's name, its body
                // the section's entries; the section's id is an anchor.
                let body = e
                    .get("entries")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let mut anchors = anchors_in(&Value::Array(body.clone()), &opts.keep_ids);
                if let Some(id) = e
                    .get("id")
                    .and_then(Value::as_str)
                    .filter(|id| opts.keep_ids.contains(*id))
                {
                    anchors.insert(0, id.to_string());
                }
                docs.push((n.to_string(), body, anchors));
            }
            _ => match docs.last_mut() {
                Some((_, items, anchors)) => {
                    anchors.extend(anchors_in(e, &opts.keep_ids));
                    items.push(e.clone());
                }
                None => docs.push((
                    part.to_string(),
                    vec![e.clone()],
                    anchors_in(e, &opts.keep_ids),
                )),
            },
        }
    }
    docs.into_iter()
        .map(|(title, items, anchors)| PlanDocument {
            title,
            markdown: to_markdown(&items, 2, opts),
            anchors,
        })
        .collect()
}

/// The kept ids in a value, in order.
fn anchors_in(v: &Value, keep: &HashSet<String>) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(v: &Value, keep: &HashSet<String>, out: &mut Vec<String>) {
        match v {
            Value::Object(o) => {
                if let Some(id) = o.get("id").and_then(Value::as_str) {
                    if keep.contains(id) && !out.iter().any(|x| x == id) {
                        out.push(id.to_string());
                    }
                }
                o.values().for_each(|x| walk(x, keep, out));
            }
            Value::Array(l) => l.iter().for_each(|x| walk(x, keep, out)),
            _ => {}
        }
    }
    walk(v, keep, &mut out);
    out
}

/// The square-grid map images of a part (the DM's version: `map`, not
/// `mapPlayer`), each once.
fn square_maps(part: &Value, part_name: &str) -> Vec<PlanMap> {
    let mut out: Vec<PlanMap> = Vec::new();
    fn walk(v: &Value, part_name: &str, out: &mut Vec<PlanMap>) {
        match v {
            Value::Object(o) => {
                if o.get("type").and_then(Value::as_str) == Some("image")
                    && o.get("imageType").and_then(Value::as_str) == Some("map")
                    && v.pointer("/grid/type").and_then(Value::as_str) == Some("square")
                {
                    let path = v.pointer("/href/path").and_then(Value::as_str);
                    let size = v.pointer("/grid/size").and_then(Value::as_f64);
                    if let (Some(path), Some(size)) = (path, size) {
                        if !out.iter().any(|m| m.path == path) {
                            out.push(PlanMap {
                                title: o
                                    .get("title")
                                    .and_then(Value::as_str)
                                    .unwrap_or(part_name)
                                    .to_string(),
                                path: path.to_string(),
                                grid_px: size,
                            });
                        }
                    }
                }
                o.values().for_each(|x| walk(x, part_name, out));
            }
            Value::Array(l) => l.iter().for_each(|x| walk(x, part_name, out)),
            _ => {}
        }
    }
    walk(part, part_name, &mut out);
    out
}

// =============================================================================
// References
// =============================================================================

/// A catalog entry that a text names.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ref {
    /// The catalog kind: "creature", "item", "spell", "condition", …
    pub kind: String,
    pub name: String,
    /// The source, with the tag's default when the tag gives none.
    pub source: String,
}

/// The default source of a tag that has no catalog link in the reader but
/// names an entry a self-contained adventure needs.
fn rule_kind(tag: &str) -> Option<(&'static str, &'static str)> {
    Some(match tag {
        "status" => ("status", "PHB"),
        "sense" => ("sense", "PHB"),
        "skill" => ("skill", "PHB"),
        "deck" => ("deck", "DMG"),
        _ => return None,
    })
}

/// The catalog entries that entries name: their tags and their stat blocks.
pub fn refs(entries: &Value) -> BTreeSet<Ref> {
    let mut out = BTreeSet::new();
    collect_refs(entries, &mut out);
    out
}

fn collect_refs(v: &Value, out: &mut BTreeSet<Ref>) {
    match v {
        Value::String(s) => text_refs(s, out),
        Value::Array(l) => l.iter().for_each(|x| collect_refs(x, out)),
        Value::Object(o) => {
            if o.get("type").and_then(Value::as_str) == Some("statblock") {
                if let Some(name) = o.get("name").and_then(Value::as_str) {
                    let tag = o.get("tag").and_then(Value::as_str).unwrap_or("creature");
                    let (kind, default) = link_kind(tag)
                        .or_else(|| rule_kind(tag))
                        .unwrap_or(("creature", "MM"));
                    out.insert(Ref {
                        kind: kind.into(),
                        name: name.into(),
                        source: o
                            .get("source")
                            .and_then(Value::as_str)
                            .unwrap_or(default)
                            .into(),
                    });
                }
            }
            for (k, x) in o {
                if k != "href" && k != "id" {
                    collect_refs(x, out);
                }
            }
        }
        _ => {}
    }
}

/// The catalog entries a text (5etools text or Mimir markdown) names.
pub fn text_refs(text: &str, out: &mut BTreeSet<Ref>) {
    let mut rest = text;
    while let Some(pos) = rest.find("{@") {
        let Some(end) = closing(rest, pos) else {
            return;
        };
        let inner = &rest[pos + 2..end];
        if let Some((tag, args)) = inner.split_once(' ') {
            let tag = if tag == FLUFF_TAG { "creature" } else { tag };
            if let Some((kind, default)) = link_kind(tag).or_else(|| rule_kind(tag)) {
                let mut parts = args.split('|');
                let name = parts.next().unwrap_or("").trim();
                let source = parts
                    .next()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .unwrap_or(default);
                if !name.is_empty() && !name.contains("{@") {
                    out.insert(Ref {
                        kind: kind.into(),
                        name: name.into(),
                        source: source.into(),
                    });
                }
            }
            // Tags nest: look inside.
            text_refs(args, out);
        }
        rest = &rest[end + 1..];
    }
}

/// A text with the source of each catalog tag that `to` maps changed:
/// `to(&Ref)` gives the new source (an export copies the entry under its
/// own source). A tag with no source gets one. Tags nest.
pub fn rewrite_sources(text: &str, to: &dyn Fn(&Ref) -> Option<String>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find("{@") {
        out.push_str(&rest[..pos]);
        let Some(end) = closing(rest, pos) else {
            out.push_str(&rest[pos..]);
            return out;
        };
        let inner = &rest[pos + 2..end];
        match inner.split_once(' ') {
            Some((tag, args)) => {
                let kind = link_kind(if tag == FLUFF_TAG { "creature" } else { tag })
                    .or_else(|| rule_kind(tag));
                let args = rewrite_sources(args, to);
                let rewritten = kind.and_then(|(kind, default)| {
                    let mut parts: Vec<&str> = args.split('|').collect();
                    let name = parts.first()?.trim();
                    if name.is_empty() || name.contains("{@") {
                        return None;
                    }
                    let source = parts
                        .get(1)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .unwrap_or(default);
                    let new = to(&Ref {
                        kind: kind.into(),
                        name: name.into(),
                        source: source.into(),
                    })?;
                    if parts.len() < 2 {
                        parts.push("");
                    }
                    let joined = std::iter::once(parts[0])
                        .chain(std::iter::once(new.as_str()))
                        .chain(parts[2..].iter().copied())
                        .collect::<Vec<_>>()
                        .join("|");
                    Some(format!("{{@{tag} {joined}}}"))
                });
                out.push_str(&rewritten.unwrap_or_else(|| format!("{{@{tag} {args}}}")));
            }
            None => out.push_str(&rest[pos..=end]),
        }
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

/// [`rewrite_sources`] over every text of entries, and the source of each
/// stat block it maps.
pub fn rewrite_entry_sources(v: &mut Value, to: &dyn Fn(&Ref) -> Option<String>) {
    match v {
        Value::String(s) => {
            if s.contains("{@") {
                *s = rewrite_sources(s, to);
            }
        }
        Value::Array(l) => l.iter_mut().for_each(|x| rewrite_entry_sources(x, to)),
        Value::Object(o) => {
            if o.get("type").and_then(Value::as_str) == Some("statblock") {
                let tag = o.get("tag").and_then(Value::as_str).unwrap_or("creature");
                if let (Some((kind, default)), Some(name)) = (
                    link_kind(tag).or_else(|| rule_kind(tag)),
                    o.get("name").and_then(Value::as_str),
                ) {
                    let r = Ref {
                        kind: kind.into(),
                        name: name.into(),
                        source: o
                            .get("source")
                            .and_then(Value::as_str)
                            .unwrap_or(default)
                            .into(),
                    };
                    if let Some(new) = to(&r) {
                        o.insert("source".into(), new.into());
                    }
                }
            }
            for (k, x) in o.iter_mut() {
                if k != "href" && k != "id" {
                    rewrite_entry_sources(x, to);
                }
            }
        }
        _ => {}
    }
}

/// The `id`s that `{@area text|id|…}` tags name, in an adventure's data.
pub fn area_targets(data: &Value) -> HashSet<String> {
    let mut out = HashSet::new();
    collect_areas(data, &mut out);
    out
}

fn collect_areas(v: &Value, out: &mut HashSet<String>) {
    match v {
        Value::String(s) => {
            let mut rest = s.as_str();
            while let Some(pos) = rest.find("{@area ") {
                let Some(end) = closing(rest, pos) else {
                    return;
                };
                if let Some(id) = rest[pos + 7..end].split('|').nth(1) {
                    out.insert(id.trim().to_string());
                }
                rest = &rest[end + 1..];
            }
        }
        Value::Array(l) => l.iter().for_each(|x| collect_areas(x, out)),
        Value::Object(o) => o.values().for_each(|x| collect_areas(x, out)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn md(entries: Value) -> String {
        to_markdown(entries.as_array().unwrap(), 1, &ToMarkdown::default())
    }

    #[test]
    fn headings_paragraphs_and_tags() {
        let e = json!([{"type": "section", "name": "Goblin Arrows", "entries": [
            "Meet {@creature Klarg|LMoP} in the cave.",
            {"type": "entries", "name": "Cragmaw Hideout", "entries": ["A cave."]}
        ]}]);
        let m = md(e.clone());
        assert_eq!(
            m,
            "# Goblin Arrows\n\nMeet {@creature Klarg|LMoP} in the cave.\n\n## Cragmaw Hideout\n\nA cave.\n"
        );
        assert_eq!(Value::Array(from_markdown(&m, 1)), e);
    }

    #[test]
    fn a_heading_that_ends_with_a_tag() {
        let e =
            json!([{"type": "section", "name": "Meet {@creature Klarg|LMoP}", "entries": ["x"]}]);
        assert_eq!(Value::Array(from_markdown(&md(e.clone()), 1)), e);
    }

    #[test]
    fn markdown_specials_are_escaped_and_come_back() {
        let s = "1. Not a list * nor _this_ [x] <b> & `code` ~ # # \\ end";
        let e = json!([s, "- dash", "> gt", "#hash"]);
        let m = md(e.clone());
        assert!(m.starts_with("1\\. Not a list \\* nor \\_this\\_"), "{m}");
        assert_eq!(Value::Array(from_markdown(&m, 1)), e);
    }

    #[test]
    fn read_aloud_sidebar_and_quote() {
        let e = json!([
            {"type": "insetReadaloud", "entries": ["The cave is dark.", "Water drips."]},
            {"type": "inset", "name": "Goblin Tactics", "entries": ["They flee at half HP."]},
            {"type": "quote", "entries": ["Beware the mists."], "by": "Madam Eva"}
        ]);
        let m = md(e.clone());
        assert!(
            m.contains("> [!read-aloud]\n>\n> The cave is dark.\n>\n> Water drips."),
            "{m}"
        );
        assert!(
            m.contains("> [!sidebar] Goblin Tactics\n>\n> They flee at half HP."),
            "{m}"
        );
        assert!(
            m.contains("> Beware the mists.\n> — Madam Eva") || m.contains("> — Madam Eva"),
            "{m}"
        );
        assert_eq!(Value::Array(from_markdown(&m, 1)), e);
    }

    #[test]
    fn lists_and_items() {
        let e = json!([
            {"type": "list", "items": ["One", "Two {@spell light}"]},
            {"type": "list", "style": "list-decimal", "items": ["First", "Second"]},
            {"type": "list", "items": [
                {"type": "item", "name": "Finding Artifacts", "entry": "The characters gain a level."},
                {"type": "item", "name": "Str.", "entry": "18 (+4)"}
            ]}
        ]);
        let m = md(e.clone());
        assert!(
            m.contains("- **Finding Artifacts** The characters gain a level."),
            "{m}"
        );
        assert!(m.contains("1. First\n2. Second"), "{m}");
        assert_eq!(Value::Array(from_markdown(&m, 1)), e);
    }

    #[test]
    fn adjacent_lists_stay_apart() {
        let e = json!([
            {"type": "list", "items": ["a"]},
            {"type": "list", "items": ["b"]},
            {"type": "list", "style": "list-decimal", "items": ["c"]},
            {"type": "list", "style": "list-decimal", "items": ["d"]}
        ]);
        assert_eq!(Value::Array(from_markdown(&md(e.clone()), 1)), e);
    }

    #[test]
    fn nested_list_comes_back() {
        let e = json!([{"type": "list", "items": [
            "Top",
            {"type": "list", "items": ["Inner a", "Inner b"]}
        ]}]);
        let m = md(e);
        let back = from_markdown(&m, 1);
        let items = back[0]["items"].as_array().unwrap();
        // The nested list joins the item before it.
        assert_eq!(items[0]["entries"][0], "Top");
        assert_eq!(
            items[0]["entries"][1]["items"],
            json!(["Inner a", "Inner b"])
        );
    }

    #[test]
    fn tables_with_caption_and_rolls() {
        let e = json!([{"type": "table", "caption": "Random Encounters",
        "colLabels": ["{@dice d6}", "Encounter"],
        "colStyles": ["col-2", "col-10"],
        "rows": [
            [{"type": "cell", "roll": {"exact": 1}}, "{@dice 1d4} {@creature goblin|MM|goblins}"],
            [{"type": "cell", "roll": {"min": 2, "max": 4}}, "Nothing"],
            [{"type": "cell", "roll": {"exact": 5}, "entry": "{@area 5|0be|x}"}, "A | pipe"]
        ]}]);
        let m = md(e);
        assert!(
            m.starts_with(
                "Table: Random Encounters\n\n| {@dice d6} | Encounter |\n| --- | --- |\n| 1 |"
            ),
            "{m}"
        );
        assert!(m.contains("| 2–4 | Nothing |"), "{m}");
        assert!(m.contains("A \\| pipe"), "{m}");
        let back = from_markdown(&m, 1);
        assert_eq!(
            back[0],
            json!({"type": "table", "caption": "Random Encounters",
                "colLabels": ["{@dice d6}", "Encounter"],
                "rows": [["1", "{@dice 1d4} {@creature goblin|MM|goblins}"], ["2–4", "Nothing"],
                         ["{@area 5|0be|x}", "A | pipe"]]})
        );
    }

    #[test]
    fn images_and_galleries() {
        let e = json!([
            {"type": "image", "href": {"type": "internal", "path": "adventure/LMoP/map.webp"}, "title": "Cragmaw Hideout", "imageType": "map"},
            {"type": "gallery", "images": [
                {"type": "image", "href": {"type": "external", "url": "https://example.com/a b.png"}, "title": "A"}
            ]}
        ]);
        let m = md(e);
        assert!(
            m.contains("![Cragmaw Hideout](<5etools:adventure/LMoP/map.webp>)"),
            "{m}"
        );
        let back = from_markdown(&m, 1);
        assert_eq!(
            back[0],
            json!({"type": "image", "href": {"type": "internal", "path": "adventure/LMoP/map.webp"}, "title": "Cragmaw Hideout"})
        );
        assert_eq!(back[1]["href"]["url"], "https://example.com/a b.png");
    }

    #[test]
    fn stat_blocks_are_tag_lines() {
        let e = json!([
            {"type": "statblock", "tag": "creature", "name": "Klarg", "source": "LMoP", "style": "narrow"},
            {"type": "statblock", "tag": "item", "name": "Trinket", "source": "CoS"},
            {"type": "statblock", "tag": "creature", "prop": "monsterFluff", "name": "Fume Drake", "source": "DoSI", "data": {}}
        ]);
        let m = md(e);
        assert_eq!(
            m,
            "{@creature Klarg|LMoP}\n\n{@item Trinket|CoS}\n\n{@creatureFluff Fume Drake|DoSI}\n"
        );
        assert_eq!(
            Value::Array(from_markdown(&m, 1)),
            json!([
                {"type": "statblock", "tag": "creature", "name": "Klarg", "source": "LMoP"},
                {"type": "statblock", "tag": "item", "name": "Trinket", "source": "CoS"},
                {"type": "statblock", "tag": "creature", "prop": "monsterFluff", "name": "Fume Drake", "source": "DoSI"}
            ])
        );
        // A text that is only a tag stays text.
        let t = json!(["{@creature Mad Mary|CoS}"]);
        assert_eq!(md(t.clone()), "\\{@creature Mad Mary|CoS}\n");
        assert_eq!(Value::Array(from_markdown(&md(t.clone()), 1)), t);
    }

    #[test]
    fn area_ids_are_kept_as_anchors() {
        let data = json!([{"type": "section", "name": "Cave", "id": "0be", "entries": [
            "See {@area the cave mouth|0be|x}.",
            {"type": "entries", "name": "Unlinked", "id": "0bf", "entries": ["x"]}
        ]}]);
        let opts = ToMarkdown {
            keep_ids: area_targets(&data),
        };
        let m = to_markdown(data.as_array().unwrap(), 1, &opts);
        assert!(m.starts_with("# Cave {#0be}\n"), "{m}");
        assert!(m.contains("## Unlinked\n"), "{m}");
        let back = from_markdown(&m, 1);
        assert_eq!(back[0]["id"], "0be");
        assert!(back[0]["entries"][1].get("id").is_none());
    }

    #[test]
    fn wrappers_unwrap() {
        let e = json!([{"type": "wrapper", "wrappeds": [
            {"type": "insetReadaloud", "entries": ["One of Swords"]}, "The treasure."
        ]}]);
        let back = from_markdown(&md(e), 1);
        assert_eq!(back[0]["type"], "insetReadaloud");
        assert_eq!(back[1], "The treasure.");
    }

    #[test]
    fn mimir_markdown_becomes_entries() {
        let m = "## Arrival\n\nThe party **arrives** at *dusk*. See [the map](https://example.com).\n\n> [!read-aloud]\n> Rain falls.\n\n### Inn\n\n- **Rumors** Goblins on the road.\n";
        let e = from_markdown(m, 1);
        assert_eq!(
            Value::Array(e),
            json!([{"type": "entries", "name": "Arrival", "entries": [
                "The party {@b arrives} at {@i dusk}. See {@link the map|https://example.com}.",
                {"type": "insetReadaloud", "entries": ["Rain falls."]},
                {"type": "entries", "name": "Inn", "entries": [
                    {"type": "list", "items": [{"type": "item", "name": "Rumors", "entry": "Goblins on the road."}]}
                ]}
            ]}])
        );
    }

    #[test]
    fn keyed_areas() {
        for k in [
            "1. Cave Mouth",
            "K1. Front Courtyard",
            "A1: Path",
            "12a. Vault",
            "E10. Hall",
        ] {
            assert!(is_keyed(k), "{k}");
        }
        for n in [
            "Cragmaw Hideout",
            "Level 1: Rotten Halls",
            "1st Floor",
            "Chapter 2. Road",
            "2",
        ] {
            assert!(!is_keyed(n), "{n}");
        }
    }

    #[test]
    fn the_fixture_adventure_as_a_plan() {
        let file: Value = serde_json::from_str(include_str!(
            "../../mimir-core/src/seed/assets/fixture-adventure.json"
        ))
        .unwrap();
        let data = file["adventureData"][0]["data"].as_array().unwrap();
        let contents = file["adventure"][0]["contents"].as_array().unwrap();
        let p = plan(data, contents);
        let roles: Vec<PartRole> = p.iter().map(|x| x.role).collect();
        assert_eq!(
            roles,
            vec![
                PartRole::Document,
                PartRole::Module,
                PartRole::Module,
                PartRole::Document
            ]
        );
        // The tower: "Tower Map", then "1. Gatehouse"… joined to it.
        let tower = &p[2];
        let titles: Vec<&str> = tower.documents.iter().map(|d| d.title.as_str()).collect();
        assert_eq!(titles, vec!["Tower Map"]);
        let md = &tower.documents[0].markdown;
        assert!(md.contains("## 1. Gatehouse {#mfa-g1}"), "{md}");
        assert!(md.contains("## 3. Lantern Room"), "{md}");
        assert!(tower.documents[0].anchors.contains(&"mfa-g1".to_string()));
        // Its square-grid map, and the creatures it names.
        assert_eq!(
            tower.maps,
            vec![PlanMap {
                title: "The Lantern Tower".into(),
                path: "adventure/MFA/tower-map.png".into(),
                grid_px: 32.0
            }]
        );
        assert!(tower
            .creatures
            .contains(&("Lantern Wisp".to_string(), "MFA".to_string())));
        assert!(tower
            .creatures
            .contains(&("Skeleton".to_string(), "MM".to_string())));
        // The causeway: a document per section.
        let causeway: Vec<&str> = p[1].documents.iter().map(|d| d.title.as_str()).collect();
        assert_eq!(
            causeway,
            vec!["Arrival", "Ambush at the Reeds", "Causeway Encounters"]
        );
        // The introduction as one document, its sidebar kept.
        assert!(p[0]
            .whole
            .markdown
            .contains("> [!sidebar] Running This Adventure"));
    }

    #[test]
    fn sources_rewritten() {
        let to = |r: &Ref| (r.kind == "creature" && r.source == "MM").then(|| "HBX".to_string());
        assert_eq!(
            rewrite_sources(
                "{@creature goblin} and {@creature Klarg|LMoP}, {@b {@creature Owlbear|MM|an owlbear}}; {@spell light}.",
                &to
            ),
            "{@creature goblin|HBX} and {@creature Klarg|LMoP}, {@b {@creature Owlbear|HBX|an owlbear}}; {@spell light}."
        );
        let mut v = json!([{"type": "statblock", "tag": "creature", "name": "Goblin", "source": "MM"},
            {"type": "entries", "name": "x", "entries": ["{@creature Goblin|MM}"]}]);
        rewrite_entry_sources(&mut v, &to);
        assert_eq!(v[0]["source"], "HBX");
        assert_eq!(v[1]["entries"][0], "{@creature Goblin|HBX}");
    }

    #[test]
    fn refs_of_tags_and_stat_blocks() {
        let data = json!([
            "Meet {@creature Klarg|LMoP} and two {@creature goblin|MM|goblins}; cast {@spell light}.",
            "{@condition poisoned}, {@status surprised}, {@item Potion of Healing|DMG}, {@b {@creature bugbear}}.",
            "{@dice 1d6} {@area room|0be|x} {@quickref Cover||3}",
            {"type": "statblock", "tag": "item", "name": "Trinket", "source": "CoS"},
            {"type": "statblock", "prop": "monsterFluff", "name": "Fume Drake", "source": "DoSI"}
        ]);
        let r: Vec<String> = refs(&data)
            .into_iter()
            .map(|r| format!("{}:{}|{}", r.kind, r.name, r.source))
            .collect();
        assert_eq!(
            r,
            vec![
                "condition:poisoned|PHB",
                "creature:Fume Drake|DoSI",
                "creature:Klarg|LMoP",
                "creature:bugbear|MM",
                "creature:goblin|MM",
                "item:Potion of Healing|DMG",
                "item:Trinket|CoS",
                "spell:light|PHB",
                "status:surprised|PHB",
            ]
        );
    }
}

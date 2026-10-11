//! Markdown to Typst conversion for campaign documents.
//!
//! This module handles parsing campaign markdown documents (with YAML frontmatter)
//! and converting them to Typst markup for PDF rendering.

use gray_matter::{engine::YAML, Matter, ParsedEntity};
use std::ops::Range;

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd};
use serde_json::Value;
use serde_yaml::Value as YamlValue;

use crate::error::{PrintError, Result};
use crate::sections::monster_cards::plain_tags;

/// Typst for a stat block line (`{@creature Name|Source}`): the tag, the
/// name and the source. `None`: the line prints as a box with the name.
pub type StatblockResolver<'a> = &'a dyn Fn(&str, &str, Option<&str>) -> Option<String>;

/// The bytes of an image a document names (its markdown URL), or none.
pub type ImageResolver<'a> = &'a dyn Fn(&str) -> Option<Vec<u8>>;

/// An image as Typst prints it: PNG, at most 2400 pixels wide (5etools
/// maps are WebP, and large). `None` when it does not decode.
pub fn print_image(bytes: &[u8]) -> Option<Vec<u8>> {
    let img = image::load_from_memory(bytes).ok()?;
    let img = if img.width() > 2400 {
        img.resize(2400, u32::MAX, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

/// The tags a stat block line may hold (as the web's converter).
const STATBLOCK_TAGS: &[&str] = &[
    "creature",
    "creatureFluff",
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

/// A stat block line: (tag, name, source).
pub fn statblock_line(text: &str) -> Option<(&str, &str, Option<&str>)> {
    let inner = text.trim().strip_prefix("{@")?.strip_suffix('}')?;
    if inner.contains(['{', '}']) {
        return None;
    }
    let (tag, args) = inner.split_once(' ')?;
    if !STATBLOCK_TAGS.contains(&tag) {
        return None;
    }
    let mut parts = args.split('|');
    let name = parts.next()?.trim();
    let source = parts.next().map(str::trim).filter(|s| !s.is_empty());
    (parts.next().is_none() && !name.is_empty()).then_some((tag, name, source))
}

fn typst_lit(s: &str) -> String {
    format!("#\"{}\"", escape_for_typst_string(s))
}

/// The opening of a callout box (MIMIR-T-0737), from its marker line.
fn callout_open(line: &str) -> Option<String> {
    let line = line.trim_start();
    if line.starts_with("[!read-aloud]") {
        return Some(
            "#block(width: 100%, inset: 8pt, fill: luma(245), stroke: (left: 2pt + rgb(\"#4a7fb0\")))[#set text(style: \"italic\")\n".into(),
        );
    }
    let rest = line.strip_prefix("[!sidebar]")?;
    let title = rest.trim();
    let head = if title.is_empty() {
        String::new()
    } else {
        format!("*{}*\n\n", typst_lit(&plain_tags(title)))
    };
    Some(format!(
        "#block(width: 100%, inset: 8pt, fill: luma(240), stroke: (left: 2pt + luma(150)))[{head}"
    ))
}

/// A parsed campaign document with frontmatter and converted content.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    /// Parsed YAML frontmatter as JSON
    pub frontmatter: Value,
    /// Markdown content converted to Typst markup
    pub typst_content: String,
    /// The images it prints (virtual file name, PNG bytes).
    pub images: Vec<(String, Vec<u8>)>,
}

/// Parse a campaign document from markdown with YAML frontmatter.
///
/// # Arguments
/// * `markdown` - The raw markdown content including frontmatter
///
/// # Returns
/// A `ParsedDocument` with the frontmatter as JSON and content as Typst markup
pub fn parse_campaign_document(markdown: &str) -> Result<ParsedDocument> {
    parse_campaign_document_full(markdown, &|_, _, _| None, &|_| None)
}

/// As [`parse_campaign_document`], with stat block lines from `resolve`
/// (MIMIR-T-0737).
pub fn parse_campaign_document_with(
    markdown: &str,
    resolve: StatblockResolver,
) -> Result<ParsedDocument> {
    parse_campaign_document_full(markdown, resolve, &|_| None)
}

/// As [`parse_campaign_document_with`], with images (MIMIR-T-0742).
pub fn parse_campaign_document_full(
    markdown: &str,
    resolve: StatblockResolver,
    image: ImageResolver,
) -> Result<ParsedDocument> {
    // Parse frontmatter using gray_matter
    let matter = Matter::<YAML>::new();
    let parsed: ParsedEntity<YamlValue> = matter
        .parse(markdown)
        .map_err(|e| PrintError::InvalidData(format!("Failed to parse frontmatter: {}", e)))?;

    // Extract frontmatter as JSON
    let frontmatter = match parsed.data {
        Some(data) => {
            // gray_matter returns YAML Value, convert to serde_json::Value
            serde_json::to_value(&data).map_err(|e| {
                PrintError::InvalidData(format!("Failed to convert frontmatter: {}", e))
            })?
        }
        None => Value::Object(serde_json::Map::new()),
    };

    // Convert markdown content to Typst
    let (typst_content, images) = markdown_to_typst_full(&parsed.content, resolve, image);

    Ok(ParsedDocument {
        frontmatter,
        typst_content,
        images,
    })
}

/// Convert markdown text to Typst markup.
///
/// Handles:
/// - Headers (`#` -> `=`)
/// - Bold (`**text**` -> `*text*`)
/// - Italic (`*text*` -> `_text_`)
/// - Lists (preserved)
/// - Links (`[text](url)` -> `#link("url")[text]`)
/// - Code blocks (converted to raw blocks)
/// - Tables (converted to Typst table syntax)
pub fn markdown_to_typst(markdown: &str) -> String {
    markdown_to_typst_with(markdown, &|_, _, _| None)
}

/// As [`markdown_to_typst`], with the adventure forms (MIMIR-T-0737):
/// `> [!read-aloud]` and `> [!sidebar] Title` as boxes, a stat block line
/// as `resolve` gives it (else a box with the name), and `{@…}` tags as
/// the words they show.
pub fn markdown_to_typst_with(markdown: &str, resolve: StatblockResolver) -> String {
    markdown_to_typst_full(markdown, resolve, &|_| None).0
}

/// As [`markdown_to_typst_with`], with images: an image `image` resolves
/// prints as a figure at page width, its bytes returned to register as
/// virtual files (`/_virtual/<name>`); another prints as its title in a
/// box.
pub fn markdown_to_typst_full(
    markdown: &str,
    resolve: StatblockResolver,
    image: ImageResolver,
) -> (String, Vec<(String, Vec<u8>)>) {
    let mut images: Vec<(String, Vec<u8>)> = Vec::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    // Adjacent text as one event, with its source range (a marker or a tag
    // is split at brackets by the parser).
    let mut events: Vec<(Event, Range<usize>)> = Vec::new();
    for (e, r) in Parser::new_ext(markdown, options).into_offset_iter() {
        if let (Event::Text(t), Some((Event::Text(prev), pr))) = (&e, events.last_mut()) {
            *prev = CowStr::from(format!("{prev}{t}"));
            pr.end = r.end;
            continue;
        }
        events.push((e, r));
    }
    let mut output = String::new();
    let mut list_depth: usize = 0;
    let mut in_table = false;
    let mut table_row: Vec<String> = Vec::new();
    let mut table_header = false;
    let mut current_cell = String::new();
    let mut link_text = String::new();
    let mut link_url = String::new();
    let mut in_link = false;

    // For each open block quote: a callout box (true) or a quote.
    let mut quotes: Vec<bool> = Vec::new();
    let mut i = 0;
    while i < events.len() {
        let (event, range) = events[i].clone();
        // A block quote whose first line is a callout marker.
        if let Event::Start(Tag::BlockQuote(_)) = &event {
            if let (
                Some((Event::Start(Tag::Paragraph), _)),
                Some((Event::Text(t), _)),
                Some((next, _)),
            ) = (events.get(i + 1), events.get(i + 2), events.get(i + 3))
            {
                if let Some(open) = callout_open(t) {
                    output.push_str(&open);
                    quotes.push(true);
                    i += match next {
                        Event::End(TagEnd::Paragraph) | Event::SoftBreak | Event::HardBreak => 4,
                        _ => 3,
                    };
                    continue;
                }
            }
        }
        // A paragraph that is only a stat block tag (not `\{@…}`).
        if let (
            Event::Start(Tag::Paragraph),
            Some((Event::Text(t), _)),
            Some((Event::End(TagEnd::Paragraph), _)),
        ) = (&event, events.get(i + 1), events.get(i + 2))
        {
            let escaped = markdown
                .get(range.clone())
                .is_some_and(|raw| raw.trim_start().starts_with("\\{"));
            if let (Some((tag, name, source)), false) = (statblock_line(t), escaped) {
                let block = resolve(tag, name, source).unwrap_or_else(|| {
                    format!(
                        "#block(width: 100%, inset: 6pt, stroke: 0.5pt + luma(150))[*{}*]\n\n",
                        typst_lit(name)
                    )
                });
                output.push_str(&block);
                i += 3;
                continue;
            }
        }
        i += 1;
        match event {
            // Headers
            Event::Start(Tag::Heading { level, .. }) => {
                let equals = "=".repeat(level as usize);
                output.push_str(&equals);
                output.push(' ');
            }
            Event::End(TagEnd::Heading(_)) => {
                output.push('\n');
            }

            // Paragraphs
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                if !in_table {
                    output.push_str("\n\n");
                }
            }

            // Bold
            Event::Start(Tag::Strong) => {
                if in_link {
                    link_text.push('*');
                } else if in_table {
                    current_cell.push('*');
                } else {
                    output.push('*');
                }
            }
            Event::End(TagEnd::Strong) => {
                if in_link {
                    link_text.push('*');
                } else if in_table {
                    current_cell.push('*');
                } else {
                    output.push('*');
                }
            }

            // Italic (emphasis)
            Event::Start(Tag::Emphasis) => {
                if in_link {
                    link_text.push('_');
                } else if in_table {
                    current_cell.push('_');
                } else {
                    output.push('_');
                }
            }
            Event::End(TagEnd::Emphasis) => {
                if in_link {
                    link_text.push('_');
                } else if in_table {
                    current_cell.push('_');
                } else {
                    output.push('_');
                }
            }

            // Strikethrough
            Event::Start(Tag::Strikethrough) => {
                if in_table {
                    current_cell.push_str("#strike[");
                } else {
                    output.push_str("#strike[");
                }
            }
            Event::End(TagEnd::Strikethrough) => {
                if in_table {
                    current_cell.push(']');
                } else {
                    output.push(']');
                }
            }

            // Links
            Event::Start(Tag::Link { dest_url, .. }) => {
                in_link = true;
                link_url = dest_url.to_string();
                link_text.clear();
            }
            Event::End(TagEnd::Link) => {
                let link_markup = format!("#link(\"{}\")[{}]", link_url, link_text);
                if in_table {
                    current_cell.push_str(&link_markup);
                } else {
                    output.push_str(&link_markup);
                }
                in_link = false;
                link_text.clear();
                link_url.clear();
            }

            // Lists
            Event::Start(Tag::List(ordered)) => {
                list_depth += 1;
                if ordered.is_some() {
                    // Ordered list - Typst uses + for numbered lists
                    // We'll handle numbering in Item
                }
            }
            Event::End(TagEnd::List(_)) => {
                list_depth -= 1;
                if list_depth == 0 {
                    output.push('\n');
                }
            }
            Event::Start(Tag::Item) => {
                let indent = "  ".repeat(list_depth.saturating_sub(1));
                output.push_str(&indent);
                output.push_str("- ");
            }
            Event::End(TagEnd::Item) => {
                output.push('\n');
            }

            // Code blocks
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        if lang.is_empty() {
                            String::new()
                        } else {
                            lang.to_string()
                        }
                    }
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
                if lang.is_empty() {
                    output.push_str("```\n");
                } else {
                    output.push_str(&format!("```{}\n", lang));
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                output.push_str("```\n\n");
            }

            // Inline code
            Event::Code(code) => {
                let code_markup = format!("`{}`", code);
                if in_link {
                    link_text.push_str(&code_markup);
                } else if in_table {
                    current_cell.push_str(&code_markup);
                } else {
                    output.push_str(&code_markup);
                }
            }

            // Tables
            Event::Start(Tag::Table(_)) => {
                in_table = true;
                // We'll output the table header later once we know the column count
            }
            Event::End(TagEnd::Table) => {
                output.push_str(")\n\n");
                in_table = false;
            }
            Event::Start(Tag::TableHead) => {
                table_header = true;
                table_row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                // Now we know the column count, output the table start with proper columns
                let column_count = table_row.len();
                let cols = vec!["auto"; column_count].join(", ");
                output.push_str(&format!("#table(\n  columns: ({}),\n", cols));

                // Output header row wrapped in a single table.header() call
                // Typst requires all header cells to be in one table.header()
                if !table_row.is_empty() {
                    let header_cells: Vec<String> = table_row
                        .iter()
                        .map(|cell| format!("[*{}*]", cell))
                        .collect();
                    output.push_str(&format!("  table.header({}),\n", header_cells.join(", ")));
                }
                table_header = false;
                table_row.clear();
            }
            Event::Start(Tag::TableRow) => {
                table_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                if !table_header {
                    for cell in &table_row {
                        output.push_str(&format!("  [{}],\n", cell));
                    }
                }
                table_row.clear();
            }
            Event::Start(Tag::TableCell) => {
                current_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                table_row.push(current_cell.clone());
                current_cell.clear();
            }

            // Block quotes
            Event::Start(Tag::BlockQuote(_)) => {
                quotes.push(false);
                output.push_str("#quote[\n");
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                quotes.pop();
                output.push_str("]\n\n");
            }

            // Horizontal rule
            Event::Rule => {
                output.push_str("#line(length: 100%)\n\n");
            }

            // Soft/hard breaks
            Event::SoftBreak => {
                if in_link {
                    link_text.push(' ');
                } else if in_table {
                    current_cell.push(' ');
                } else {
                    output.push(' ');
                }
            }
            Event::HardBreak => {
                if in_table {
                    current_cell.push_str(" \\ ");
                } else {
                    output.push_str(" \\\n");
                }
            }

            // Text content
            Event::Text(text) => {
                // Wrap text in Typst string literal to safely handle special characters
                // This avoids issues with */, #, $, @ etc. in user content
                let escaped = escape_for_typst_string(&plain_tags(&text));
                let safe_text = format!("#\"{}\"", escaped);
                if in_link {
                    link_text.push_str(&safe_text);
                } else if in_table {
                    current_cell.push_str(&safe_text);
                } else {
                    output.push_str(&safe_text);
                }
            }

            // HTML (pass through as comment)
            // Escape */ sequences to prevent breaking Typst block comments
            Event::Html(html) => {
                let escaped_html = html.trim().replace("*/", "*\\/");
                output.push_str(&format!("/* HTML: {} */", escaped_html));
            }
            Event::InlineHtml(html) => {
                let escaped_html = html.trim().replace("*/", "*\\/");
                output.push_str(&format!("/* {} */", escaped_html));
            }

            // Images: the title is the caption.
            Event::Start(Tag::Image { dest_url, .. }) => {
                let mut alt = String::new();
                while i < events.len() {
                    let (e, _) = &events[i];
                    i += 1;
                    match e {
                        Event::End(TagEnd::Image) => break,
                        Event::Text(t) => alt.push_str(t),
                        _ => {}
                    }
                }
                let alt = plain_tags(&alt);
                let printed = image(&dest_url).and_then(|b| print_image(&b)).map(|png| {
                    use std::hash::{Hash, Hasher};
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    dest_url.hash(&mut h);
                    let name = format!("md-{:016x}.png", h.finish());
                    if !images.iter().any(|(n, _)| *n == name) {
                        images.push((name.clone(), png));
                    }
                    name
                });
                let markup = match printed {
                    Some(name) if alt.is_empty() => {
                        format!("\n#figure(image(\"/_virtual/{name}\", width: 100%))\n\n")
                    }
                    Some(name) => format!(
                        "\n#figure(image(\"/_virtual/{name}\", width: 100%), caption: [{}])\n\n",
                        typst_lit(&alt)
                    ),
                    None => format!(
                        "#block(inset: 4pt, stroke: 0.5pt + luma(180))[#emph[{}]]",
                        typst_lit(&format!(
                            "Image: {}",
                            if alt.is_empty() {
                                dest_url.as_ref()
                            } else {
                                alt.as_str()
                            }
                        ))
                    ),
                };
                if in_table {
                    current_cell.push_str(&typst_lit(&alt));
                } else if in_link {
                    link_text.push_str(&typst_lit(&alt));
                } else {
                    output.push_str(&markup);
                }
            }
            Event::End(TagEnd::Image) => {}

            // Footnotes (simplified)
            Event::FootnoteReference(name) => {
                output.push_str(&format!("#super[{}]", name));
            }
            Event::Start(Tag::FootnoteDefinition(_)) => {}
            Event::End(TagEnd::FootnoteDefinition) => {}

            // Task list items
            Event::TaskListMarker(checked) => {
                if checked {
                    output.push_str("[x] ");
                } else {
                    output.push_str("[ ] ");
                }
            }

            // Metadata block (ignore)
            Event::Start(Tag::MetadataBlock(_)) | Event::End(TagEnd::MetadataBlock(_)) => {}

            // Definition list (simplified)
            Event::Start(Tag::DefinitionList)
            | Event::End(TagEnd::DefinitionList)
            | Event::Start(Tag::DefinitionListTitle)
            | Event::End(TagEnd::DefinitionListTitle)
            | Event::Start(Tag::DefinitionListDefinition)
            | Event::End(TagEnd::DefinitionListDefinition) => {}

            // HTML-like events we don't handle
            Event::Start(Tag::HtmlBlock) | Event::End(TagEnd::HtmlBlock) => {}

            // Math (convert to Typst math mode)
            Event::InlineMath(math) => {
                let math_markup = format!("${}$", math);
                if in_link {
                    link_text.push_str(&math_markup);
                } else if in_table {
                    current_cell.push_str(&math_markup);
                } else {
                    output.push_str(&math_markup);
                }
            }
            Event::DisplayMath(math) => {
                output.push_str(&format!("$ {} $\n", math));
            }
        }
    }

    // Clean up extra newlines
    let cleaned = output
        .lines()
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    (cleaned + "\n", images)
}

/// Escape text for use inside a Typst string literal.
/// Only backslashes and quotes need escaping inside strings.
fn escape_for_typst_string(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_document_with_frontmatter() {
        let md = r#"---
title: Test Document
type: session_outline
---

# Hello World

This is a test."#;

        let result = parse_campaign_document(md).unwrap();
        assert_eq!(result.frontmatter["title"], "Test Document");
        assert_eq!(result.frontmatter["type"], "session_outline");
        // Text is wrapped in string literals: = #"Hello World"
        assert!(result.typst_content.contains("= #\"Hello World\""));
    }

    #[test]
    fn test_parse_document_without_frontmatter() {
        let md = "# Just Content\n\nNo frontmatter here.";
        let result = parse_campaign_document(md).unwrap();
        assert!(result.frontmatter.is_object());
        assert!(result.typst_content.contains("= #\"Just Content\""));
    }

    #[test]
    fn test_markdown_headers() {
        let md = "# H1\n## H2\n### H3";
        let typst = markdown_to_typst(md);
        // Text is wrapped in string literals
        assert!(typst.contains("= #\"H1\""));
        assert!(typst.contains("== #\"H2\""));
        assert!(typst.contains("=== #\"H3\""));
    }

    #[test]
    fn test_markdown_bold_italic() {
        let md = "This is **bold** and *italic* text.";
        let typst = markdown_to_typst(md);
        // Bold uses * markers, italic uses _, text is in string literals
        assert!(typst.contains("*#\"bold\"*"));
        assert!(typst.contains("_#\"italic\"_"));
    }

    #[test]
    fn test_markdown_links() {
        let md = "Check out [this link](https://example.com).";
        let typst = markdown_to_typst(md);
        // Link text is wrapped in string literal
        assert!(typst.contains("#link(\"https://example.com\")[#\"this link\""));
    }

    #[test]
    fn test_markdown_lists() {
        let md = "- Item 1\n- Item 2\n- Item 3";
        let typst = markdown_to_typst(md);
        // List items have text in string literals
        assert!(typst.contains("- #\"Item 1\""));
        assert!(typst.contains("- #\"Item 2\""));
        assert!(typst.contains("- #\"Item 3\""));
    }

    #[test]
    fn test_markdown_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let typst = markdown_to_typst(md);
        assert!(typst.contains("```rust"));
        assert!(typst.contains("fn main() {}"));
    }

    #[test]
    fn test_markdown_inline_code() {
        let md = "Use the `println!` macro.";
        let typst = markdown_to_typst(md);
        assert!(typst.contains("`println!`"));
    }

    #[test]
    fn test_special_chars_in_string_literals() {
        // Special characters should be wrapped in string literals
        let md = "Use #hashtag and $money.";
        let typst = markdown_to_typst(md);
        // Text should be wrapped as #"..." string literals
        assert!(
            typst.contains("#\"Use #hashtag and $money.\""),
            "Expected string literal wrapping but got: {}",
            typst
        );
    }

    #[test]
    fn test_block_comment_chars_safe_in_strings() {
        // */ sequence should be safe inside string literals
        let md = "path/to/file";
        let typst = markdown_to_typst(md);
        // The / is safely inside a string literal, not escaped
        assert!(
            typst.contains("#\"path/to/file\""),
            "Expected string literal but got: {}",
            typst
        );
    }

    #[test]
    fn test_quotes_escaped_in_strings() {
        let md = r#"He said "hello" to me."#;
        let typst = markdown_to_typst(md);
        // Quotes inside strings should be escaped
        assert!(
            typst.contains(r#"\"hello\""#),
            "Expected escaped quotes but got: {}",
            typst
        );
    }

    #[test]
    fn test_markdown_blockquote() {
        let md = "> This is a quote.";
        let typst = markdown_to_typst(md);
        assert!(typst.contains("#quote["));
    }

    #[test]
    fn test_markdown_horizontal_rule() {
        let md = "Before\n\n---\n\nAfter";
        let typst = markdown_to_typst(md);
        assert!(typst.contains("#line(length: 100%)"));
    }

    #[test]
    fn test_markdown_table() {
        let md = "| Name | Level |\n| --- | --- |\n| Fighter | 5 |\n| Wizard | 3 |";
        let typst = markdown_to_typst(md);
        // Table structure
        assert!(typst.contains("#table("));
        assert!(typst.contains("columns:"));
        // Header cells are bold
        assert!(typst.contains("table.header("));
        assert!(typst.contains("[*#\"Name\"*]"));
        assert!(typst.contains("[*#\"Level\"*]"));
        // Body cells
        assert!(typst.contains("[#\"Fighter\"]"));
        assert!(typst.contains("[#\"5\"]"));
    }

    #[test]
    fn test_markdown_table_with_formatting() {
        let md = "| Name | Note |\n| --- | --- |\n| **Bold** | *Italic* |";
        let typst = markdown_to_typst(md);
        assert!(typst.contains("*#\"Bold\"*"));
        assert!(typst.contains("_#\"Italic\"_"));
    }

    #[test]
    fn test_markdown_strikethrough() {
        let md = "This is ~~deleted~~ text.";
        let typst = markdown_to_typst(md);
        assert!(typst.contains("#strike["));
    }

    #[test]
    fn test_markdown_nested_list() {
        let md = "- Item 1\n  - Sub item\n- Item 2";
        let typst = markdown_to_typst(md);
        assert!(typst.contains("- #\"Item 1\""));
        assert!(typst.contains("  - #\"Sub item\""));
        assert!(typst.contains("- #\"Item 2\""));
    }

    #[test]
    fn test_markdown_html_comment() {
        let md = "<div>Some HTML</div>";
        let typst = markdown_to_typst(md);
        // HTML should be wrapped in a comment
        assert!(typst.contains("/*"));
    }

    #[test]
    fn test_markdown_image() {
        // An image no resolver finds prints as its title in a box (it used
        // to print `#image("image.png"` with the title after it: not Typst).
        let md = "![alt text](image.png)";
        let typst = markdown_to_typst(md);
        assert!(typst.contains("#emph[#\"Image: alt text\"]"), "{typst}");
        assert!(!typst.contains("#image("), "{typst}");
        // A resolved one is a figure on a virtual file, with its caption.
        let png = {
            let img = image::RgbImage::new(4, 4);
            let mut out = std::io::Cursor::new(Vec::new());
            img.write_to(&mut out, image::ImageFormat::Png).unwrap();
            out.into_inner()
        };
        let (typst, images) = markdown_to_typst_full(md, &|_, _, _| None, &|u| {
            (u == "image.png").then(|| png.clone())
        });
        assert_eq!(images.len(), 1);
        assert!(
            typst.contains(&format!(
                "#figure(image(\"/_virtual/{}\", width: 100%), caption: [#\"alt text\"])",
                images[0].0
            )),
            "{typst}"
        );
    }

    #[test]
    fn test_empty_markdown() {
        let typst = markdown_to_typst("");
        assert!(typst.trim().is_empty() || typst == "\n");
    }

    #[test]
    fn test_backslash_escaped_in_string_literal() {
        let md = r"path\to\file";
        let typst = markdown_to_typst(md);
        // Backslashes should be escaped inside string literal
        assert!(typst.contains(r#"\\to\\file"#));
    }
}

#[cfg(test)]
mod adventure_forms {
    use super::*;
    use crate::builder::DocumentBuilder;
    use crate::sections::MarkdownSection;

    const FORMS: &str = "## Arrival\n\n> [!read-aloud]\n>\n> Fog lies thick.\n\n> [!sidebar] Goblin Tactics\n>\n> They flee.\n\n> A plain quote.\n\n{@creature Goblin|MM}\n\n{@creature Nobody|XX}\n\n\\{@creature Rat|MM}\n\nMeet {@creature Lich|MM|the keeper} at {@area the gate|g1|x}.\n";

    #[test]
    fn boxes_stat_blocks_and_tags() {
        let resolve = |tag: &str, name: &str, _: Option<&str>| {
            (tag == "creature" && name == "Goblin").then(|| "/* GOBLIN STAT BLOCK */\n".to_string())
        };
        let t = markdown_to_typst_with(FORMS, &resolve);
        assert!(
            t.contains("#block(width: 100%, inset: 8pt, fill: luma(245)"),
            "{t}"
        );
        assert!(t.contains("*#\"Goblin Tactics\"*"), "{t}");
        assert!(t.contains("#quote["), "{t}");
        assert!(!t.contains("[!"), "{t}");
        assert!(t.contains("/* GOBLIN STAT BLOCK */"), "{t}");
        // A name the resolver does not know: a box with the name.
        assert!(t.contains("[*#\"Nobody\"*]"), "{t}");
        // The escaped line is text; tags print as their words.
        assert!(t.contains("#\"Rat\""), "{t}");
        assert!(t.contains("#\"Meet the keeper at the gate.\""), "{t}");
    }

    #[test]
    fn a_document_with_the_forms_compiles_to_pdf() {
        let goblin = serde_json::json!({"name": "Goblin", "size": ["S"], "type": "humanoid", "ac": [15],
            "hp": {"average": 7, "formula": "2d6"}, "speed": {"walk": 30},
            "str": 8, "dex": 14, "con": 10, "int": 10, "wis": 8, "cha": 8, "cr": "1/4",
            "action": [{"name": "Scimitar", "entries": ["{@hit 4} to hit."]}]});
        let resolve = move |tag: &str, _: &str, _: Option<&str>| {
            (tag == "creature")
                .then(|| crate::sections::monster_cards::monster_statblock_typst(&goblin))
        };
        let section = MarkdownSection::from_content_with(FORMS, Some("Causeway"), &resolve);
        let pdf = DocumentBuilder::new("Forms")
            .append(section)
            .to_pdf()
            .expect("compiles");
        assert!(pdf.starts_with(b"%PDF"));
    }
}

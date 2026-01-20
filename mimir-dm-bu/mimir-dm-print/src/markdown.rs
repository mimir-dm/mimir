//! Markdown to Typst conversion for campaign documents.
//!
//! This module handles parsing campaign markdown documents (with YAML frontmatter)
//! and converting them to Typst markup for PDF rendering.

use gray_matter::{engine::YAML, Matter, ParsedEntity};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde_json::Value;
use serde_yaml::Value as YamlValue;

use crate::error::{PrintError, Result};

/// A parsed campaign document with frontmatter and converted content.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    /// Parsed YAML frontmatter as JSON
    pub frontmatter: Value,
    /// Markdown content converted to Typst markup
    pub typst_content: String,
}

/// Parse a campaign document from markdown with YAML frontmatter.
///
/// # Arguments
/// * `markdown` - The raw markdown content including frontmatter
///
/// # Returns
/// A `ParsedDocument` with the frontmatter as JSON and content as Typst markup
pub fn parse_campaign_document(markdown: &str) -> Result<ParsedDocument> {
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
    let typst_content = markdown_to_typst(&parsed.content);

    Ok(ParsedDocument {
        frontmatter,
        typst_content,
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
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(markdown, options);
    let mut output = String::new();
    let mut list_depth: usize = 0;
    let mut in_table = false;
    let mut table_row: Vec<String> = Vec::new();
    let mut table_header = false;
    let mut current_cell = String::new();
    let mut link_text = String::new();
    let mut link_url = String::new();
    let mut in_link = false;

    for event in parser {
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
                output.push_str("#quote[\n");
            }
            Event::End(TagEnd::BlockQuote(_)) => {
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
                let escaped = escape_for_typst_string(&text);
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

            // Images
            Event::Start(Tag::Image { dest_url, .. }) => {
                output.push_str(&format!("#image(\"{}\"", dest_url));
            }
            Event::End(TagEnd::Image) => {
                output.push(')');
            }

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

    cleaned + "\n"
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
}

//! Markdown document section

use std::path::Path;

use crate::builder::{RenderContext, Renderable};
use crate::error::{PrintError, Result};
use crate::markdown::{markdown_to_typst, parse_campaign_document};

/// A markdown document section with optional YAML frontmatter
pub struct MarkdownSection {
    /// Document title (from frontmatter or explicit)
    title: Option<String>,
    /// Document type (from frontmatter, e.g., "session_outline", "npc_profile")
    doc_type: Option<String>,
    /// Typst content converted from markdown
    typst_content: String,
}

impl MarkdownSection {
    /// Create from raw markdown string (with optional YAML frontmatter)
    pub fn from_markdown(markdown: &str) -> Result<Self> {
        let parsed = parse_campaign_document(markdown)?;

        let title = parsed
            .frontmatter
            .get("title")
            .and_then(|v| v.as_str())
            .map(String::from);

        let doc_type = parsed
            .frontmatter
            .get("type")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(Self {
            title,
            doc_type,
            typst_content: parsed.typst_content,
        })
    }

    /// Create from a markdown file path
    pub fn from_file(path: &Path) -> Result<Self> {
        let markdown = std::fs::read_to_string(path).map_err(|e| {
            PrintError::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to read markdown file {:?}: {}", path, e),
            ))
        })?;
        Self::from_markdown(&markdown)
    }

    /// Create from raw markdown content without frontmatter
    pub fn from_content(content: &str, title: Option<&str>) -> Self {
        Self {
            title: title.map(String::from),
            doc_type: None,
            typst_content: markdown_to_typst(content),
        }
    }

    /// Set an explicit title (overrides frontmatter)
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Get the document type (from frontmatter)
    pub fn doc_type(&self) -> Option<&str> {
        self.doc_type.as_deref()
    }
}

impl Renderable for MarkdownSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        Ok(self.typst_content.clone())
    }

    fn toc_title(&self) -> Option<String> {
        self.title.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_markdown_with_frontmatter() {
        let md = r#"---
title: Test Document
type: session_outline
---

# Hello World

This is a test.
"#;
        let section = MarkdownSection::from_markdown(md).unwrap();
        assert_eq!(section.title, Some("Test Document".to_string()));
        assert_eq!(section.doc_type, Some("session_outline".to_string()));
        assert!(section.typst_content.contains("Hello World"));
    }

    #[test]
    fn test_from_content() {
        let section = MarkdownSection::from_content("# Test\n\nSome content.", Some("My Title"));
        assert_eq!(section.title, Some("My Title".to_string()));
        assert!(section.typst_content.contains("Test"));
    }

    #[test]
    fn test_toc_title() {
        let section = MarkdownSection::from_content("Content", Some("Chapter 1"));
        assert_eq!(section.toc_title(), Some("Chapter 1".to_string()));
    }
}

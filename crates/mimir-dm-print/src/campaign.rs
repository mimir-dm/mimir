//! Campaign document rendering
//!
//! This module provides campaign-specific PDF rendering functionality,
//! including single documents, combined documents, and documents with
//! module monsters and NPCs.

use std::path::PathBuf;

use tracing::{debug, error, info, instrument};

use crate::error::Result;
use crate::markdown::{parse_campaign_document, ParsedDocument};
use crate::service::PrintService;

impl PrintService {
    /// Render a campaign document (markdown with YAML frontmatter) to PDF
    ///
    /// # Arguments
    /// * `file_path` - Path to the markdown document file
    /// * `campaign_name` - Name of the campaign (optional, used in header)
    ///
    /// # Returns
    /// PDF file contents as bytes
    #[instrument(skip(self), fields(file = %file_path.display()))]
    pub fn render_campaign_document(
        &self,
        file_path: &PathBuf,
        campaign_name: Option<&str>,
    ) -> Result<Vec<u8>> {
        info!("Rendering campaign document to PDF");

        // Read the markdown file
        let markdown = std::fs::read_to_string(file_path)?;

        // Parse the document
        let parsed = parse_campaign_document(&markdown)?;

        // Build the data structure for the template
        let data = self.build_campaign_document_data(&parsed, campaign_name)?;

        // Render using the campaign document template
        self.render_to_pdf("campaign/document.typ", data)
    }

    /// Render multiple campaign documents as a single combined PDF
    ///
    /// # Arguments
    /// * `documents` - List of document file paths
    /// * `campaign_name` - Name of the campaign
    ///
    /// # Returns
    /// PDF file contents as bytes
    #[instrument(skip(self, documents), fields(count = documents.len()))]
    pub fn render_campaign_combined(
        &self,
        documents: &[PathBuf],
        campaign_name: &str,
    ) -> Result<Vec<u8>> {
        info!("Rendering {} campaign documents to combined PDF", documents.len());

        // Parse all documents (title/type come from YAML frontmatter)
        let mut parsed_docs = Vec::new();
        for file_path in documents {
            debug!("Reading document: {:?}", file_path);
            let markdown = std::fs::read_to_string(file_path)?;
            let parsed = parse_campaign_document(&markdown)?;
            parsed_docs.push(parsed);
        }

        // Build the combined data structure
        let data = self.build_campaign_combined_data(&parsed_docs, campaign_name)?;

        // Render using the combined campaign template
        self.render_to_pdf("campaign/combined.typ", data)
    }

    /// Render multiple campaign documents with module monsters as a single combined PDF
    ///
    /// # Arguments
    /// * `documents` - List of document file paths
    /// * `campaign_name` - Name of the campaign
    /// * `modules` - JSON array of module data with monsters
    ///
    /// # Returns
    /// PDF file contents as bytes
    #[instrument(skip(self, documents, modules), fields(count = documents.len()))]
    pub fn render_campaign_combined_with_monsters(
        &self,
        documents: &[PathBuf],
        campaign_name: &str,
        modules: serde_json::Value,
    ) -> Result<Vec<u8>> {
        info!(
            "Rendering {} campaign documents with modules to combined PDF",
            documents.len()
        );

        // Parse all documents (title/type come from YAML frontmatter)
        let mut parsed_docs = Vec::new();
        for file_path in documents {
            debug!("Reading document: {:?}", file_path);
            let markdown = std::fs::read_to_string(file_path)?;
            let parsed = parse_campaign_document(&markdown)?;
            parsed_docs.push(parsed);
        }

        // Build the combined data structure with modules
        let data = self.build_campaign_combined_data_with_monsters(&parsed_docs, campaign_name, modules)?;

        // Render using the combined campaign template
        self.render_to_pdf("campaign/combined.typ", data)
    }

    /// Render multiple campaign documents with module monsters and NPCs as a single combined PDF
    ///
    /// # Arguments
    /// * `documents` - List of document file paths
    /// * `campaign_name` - Name of the campaign
    /// * `modules` - JSON array of module data with monsters
    /// * `npcs` - JSON array of NPC character data
    ///
    /// # Returns
    /// PDF file contents as bytes
    #[instrument(skip(self, documents, modules, npcs), fields(count = documents.len()))]
    pub fn render_campaign_combined_with_monsters_and_npcs(
        &self,
        documents: &[PathBuf],
        campaign_name: &str,
        modules: serde_json::Value,
        npcs: serde_json::Value,
    ) -> Result<Vec<u8>> {
        info!(
            "Rendering {} campaign documents with modules and {} NPCs to combined PDF",
            documents.len(),
            npcs.as_array().map(|a| a.len()).unwrap_or(0)
        );

        // Parse all documents (title/type come from YAML frontmatter)
        let mut parsed_docs = Vec::new();
        for file_path in documents {
            debug!("Reading document: {:?}", file_path);
            let markdown = std::fs::read_to_string(file_path)?;
            let parsed = parse_campaign_document(&markdown)?;
            parsed_docs.push(parsed);
        }

        // Build the combined data structure with modules and NPCs
        let data = self.build_campaign_combined_data_with_monsters_and_npcs(
            &parsed_docs,
            campaign_name,
            modules,
            npcs,
        )?;

        // Render using the combined campaign template
        self.render_to_pdf("campaign/combined.typ", data)
    }

    /// Render multiple campaign documents with module monsters, NPCs, and maps as a single combined PDF
    ///
    /// # Arguments
    /// * `documents` - List of document file paths
    /// * `campaign_name` - Name of the campaign
    /// * `modules` - JSON array of module data with monsters
    /// * `npcs` - JSON array of NPC character data
    /// * `maps` - Map data with tokens for each map
    /// * `campaign_base_path` - Base path for resolving map image files
    ///
    /// # Returns
    /// PDF file contents as bytes
    #[instrument(skip(self, documents, modules, npcs, maps), fields(count = documents.len()))]
    pub fn render_campaign_combined_with_all(
        &self,
        documents: &[PathBuf],
        campaign_name: &str,
        modules: serde_json::Value,
        npcs: serde_json::Value,
        maps: Vec<(crate::map_renderer::RenderMap, Vec<crate::map_renderer::RenderToken>)>,
        campaign_base_path: &PathBuf,
    ) -> Result<Vec<u8>> {
        info!(
            "Rendering {} campaign documents with modules, {} NPCs, and {} maps to combined PDF",
            documents.len(),
            npcs.as_array().map(|a| a.len()).unwrap_or(0),
            maps.len()
        );

        // Parse all documents (title/type come from YAML frontmatter)
        let mut parsed_docs = Vec::new();
        for file_path in documents {
            debug!("Reading document: {:?}", file_path);
            let markdown = std::fs::read_to_string(file_path)?;
            let parsed = parse_campaign_document(&markdown)?;
            parsed_docs.push(parsed);
        }

        // Render all maps
        let mut rendered_maps = Vec::new();
        for (map, tokens) in &maps {
            debug!("Rendering map: {}", map.name);
            let rendered = crate::map_renderer::render_map(map, tokens, campaign_base_path)?;
            rendered_maps.push(rendered);
        }

        // Build the combined data structure with modules, NPCs, and maps
        let data = self.build_campaign_combined_data_with_all(
            &parsed_docs,
            campaign_name,
            modules,
            npcs,
            &rendered_maps,
        )?;

        // Render using the combined campaign template
        self.render_to_pdf("campaign/combined.typ", data)
    }

    /// Build the data structure for a single campaign document template
    fn build_campaign_document_data(
        &self,
        parsed: &ParsedDocument,
        campaign_name: Option<&str>,
    ) -> Result<serde_json::Value> {
        // Extract title from frontmatter or use default
        let title = parsed
            .frontmatter
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Document");

        // Extract document type from frontmatter or use default
        let document_type = parsed
            .frontmatter
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("document");

        let mut data = serde_json::json!({
            "title": title,
            "document_type": document_type,
            "content": parsed.typst_content,
        });

        if let Some(name) = campaign_name {
            data["campaign_name"] = serde_json::Value::String(name.to_string());
        }

        Ok(data)
    }

    /// Build the data structure for the combined campaign template
    fn build_campaign_combined_data(
        &self,
        documents: &[ParsedDocument],
        campaign_name: &str,
    ) -> Result<serde_json::Value> {
        let docs: Vec<serde_json::Value> = documents
            .iter()
            .enumerate()
            .map(|(idx, parsed)| {
                // Extract title and type from YAML frontmatter
                let title = parsed
                    .frontmatter
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled Document");

                let document_type = parsed
                    .frontmatter
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("document");

                // Sanitize content to remove any dangerous Typst commands
                let safe_content = sanitize_typst_content(&parsed.typst_content);

                // Debug: log first 500 chars of first document's content
                if idx == 0 {
                    debug!(
                        "First document '{}' typst content (first 500 chars): {}",
                        title,
                        &safe_content.chars().take(500).collect::<String>()
                    );
                }

                // Debug: check if content contains any suspicious patterns
                if safe_content.contains("#set") || safe_content.contains("#page") {
                    tracing::warn!(
                        "Document '{}' still contains #set or #page after sanitization!",
                        title
                    );
                }

                serde_json::json!({
                    "title": title,
                    "document_type": document_type,
                    "content": safe_content,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "campaign_name": campaign_name,
            "documents": docs,
        }))
    }

    /// Build the data structure for the combined campaign template with module monsters
    fn build_campaign_combined_data_with_monsters(
        &self,
        documents: &[ParsedDocument],
        campaign_name: &str,
        modules: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.build_campaign_combined_data_with_monsters_and_npcs(
            documents,
            campaign_name,
            modules,
            serde_json::Value::Array(vec![]),
        )
    }

    /// Build the data structure for the combined campaign template with module monsters and NPCs
    fn build_campaign_combined_data_with_monsters_and_npcs(
        &self,
        documents: &[ParsedDocument],
        campaign_name: &str,
        modules: serde_json::Value,
        npcs: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let docs: Vec<serde_json::Value> = documents
            .iter()
            .enumerate()
            .map(|(idx, parsed)| {
                // Extract title and type from YAML frontmatter
                let title = parsed
                    .frontmatter
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled Document");

                let document_type = parsed
                    .frontmatter
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("document");

                // Sanitize content to remove any dangerous Typst commands
                let safe_content = sanitize_typst_content(&parsed.typst_content);

                // Debug: log first 500 chars of first document's content
                if idx == 0 {
                    debug!(
                        "First document '{}' typst content (first 500 chars): {}",
                        title,
                        &safe_content.chars().take(500).collect::<String>()
                    );
                }

                serde_json::json!({
                    "title": title,
                    "document_type": document_type,
                    "content": safe_content,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "campaign_name": campaign_name,
            "documents": docs,
            "modules": modules,
            "npcs": npcs,
        }))
    }

    /// Build the data structure for the combined campaign template with all data including maps
    ///
    /// Maps are saved as temporary files and paths are passed to the template.
    fn build_campaign_combined_data_with_all(
        &self,
        documents: &[ParsedDocument],
        campaign_name: &str,
        modules: serde_json::Value,
        npcs: serde_json::Value,
        rendered_maps: &[crate::map_renderer::RenderedMap],
    ) -> Result<serde_json::Value> {
        let docs: Vec<serde_json::Value> = documents
            .iter()
            .enumerate()
            .map(|(idx, parsed)| {
                // Extract title and type from YAML frontmatter
                let title = parsed
                    .frontmatter
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled Document");

                let document_type = parsed
                    .frontmatter
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("document");

                // Sanitize content to remove any dangerous Typst commands
                let safe_content = sanitize_typst_content(&parsed.typst_content);

                // Debug: log first 500 chars of first document's content
                if idx == 0 {
                    debug!(
                        "First document '{}' typst content (first 500 chars): {}",
                        title,
                        &safe_content.chars().take(500).collect::<String>()
                    );
                }

                serde_json::json!({
                    "title": title,
                    "document_type": document_type,
                    "content": safe_content,
                })
            })
            .collect();

        // Save rendered maps as temp files and collect paths
        let temp_dir = std::env::temp_dir().join("mimir-maps");
        std::fs::create_dir_all(&temp_dir)?;

        let maps_json: Vec<serde_json::Value> = rendered_maps
            .iter()
            .enumerate()
            .map(|(idx, rendered)| {
                // Generate unique filenames
                let sanitized_name = rendered.name.replace(' ', "_").replace('/', "_");
                let grid_path = temp_dir.join(format!("{}_{}_grid.png", idx, sanitized_name));
                let tokens_path = temp_dir.join(format!("{}_{}_tokens.png", idx, sanitized_name));

                // Write grid image
                if let Err(e) = std::fs::write(&grid_path, &rendered.with_grid) {
                    error!("Failed to write map grid image: {}", e);
                }

                let mut map_data = serde_json::json!({
                    "name": rendered.name,
                    "grid_path": grid_path.to_string_lossy(),
                    "has_tokens": rendered.with_tokens.is_some(),
                });

                // Write tokens image if exists
                if let Some(ref tokens_bytes) = rendered.with_tokens {
                    if let Err(e) = std::fs::write(&tokens_path, tokens_bytes) {
                        error!("Failed to write map tokens image: {}", e);
                    }
                    map_data["tokens_path"] = serde_json::Value::String(tokens_path.to_string_lossy().to_string());
                }

                map_data
            })
            .collect();

        Ok(serde_json::json!({
            "campaign_name": campaign_name,
            "documents": docs,
            "modules": modules,
            "npcs": npcs,
            "maps": maps_json,
        }))
    }
}

/// Sanitize Typst content to remove dangerous commands that could affect page layout
///
/// This removes commands like `#set page(...)` that could cause conflicts when
/// the content is eval'd within a template.
fn sanitize_typst_content(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Check for #set page( or #page( patterns
        if chars[i] == '#' {
            // Look ahead for "set page(" or "page("
            let remaining: String = chars[i..].iter().collect();

            if remaining.starts_with("#set page(") || remaining.starts_with("#set  page(") {
                // Skip "#set page" and find the opening paren
                let paren_start = remaining.find('(').unwrap();
                i += paren_start;
                // Skip the balanced parentheses
                i += skip_balanced(&chars[i..], '(', ')');
                continue;
            } else if remaining.starts_with("#page(") {
                // Skip "#page" and find the opening paren
                let paren_start = remaining.find('(').unwrap();
                i += paren_start;
                // Skip the balanced parentheses
                i += skip_balanced(&chars[i..], '(', ')');
                // Also skip following bracket if present (for #page(...)[...])
                if i < len && chars[i] == '[' {
                    i += skip_balanced(&chars[i..], '[', ']');
                }
                continue;
            } else if remaining.starts_with("#pagebreak(") {
                // Allow #pagebreak() - it's safe and commonly used
                // Just output as-is
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Skip balanced brackets/parens, returning the number of characters consumed
fn skip_balanced(chars: &[char], open: char, close: char) -> usize {
    if chars.is_empty() || chars[0] != open {
        return 0;
    }

    let mut depth = 0;
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == open {
            depth += 1;
        } else if chars[i] == close {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
        i += 1;
    }

    // Unbalanced - return whole remaining content
    chars.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_typst_content() {
        // Test removing #set page commands
        let content_with_set_page = r#"= Hello World

#set page(header: [My Header])

Some content here.

#set page(footer: [My Footer], margin: 1in)

More content."#;
        let sanitized = sanitize_typst_content(content_with_set_page);
        assert!(!sanitized.contains("#set page"), "Should remove #set page");
        assert!(sanitized.contains("= Hello World"));
        assert!(sanitized.contains("Some content here."));
        assert!(sanitized.contains("More content."));

        // Test content without page commands passes through unchanged
        let clean_content = "= Normal Content\n\nJust text here.";
        let sanitized_clean = sanitize_typst_content(clean_content);
        assert_eq!(sanitized_clean, clean_content);

        // Test removing #page function calls
        let content_with_page_fn = r#"= Title

#page(header: [Test])[
  Some content in a page block
]

After the page."#;
        let sanitized_fn = sanitize_typst_content(content_with_page_fn);
        assert!(!sanitized_fn.contains("#page("), "Should remove #page function");

        // Test nested brackets/parens are handled correctly
        let content_with_nested = r#"= Title

#set page(header: [text(fill: red)[Hello]], footer: [page #context counter(page).display()])

After nested."#;
        let sanitized_nested = sanitize_typst_content(content_with_nested);
        assert!(!sanitized_nested.contains("#set page"), "Should handle nested brackets");
        assert!(sanitized_nested.contains("After nested."));

        // Test #pagebreak is preserved
        let content_with_pagebreak = "= Title\n\n#pagebreak()\n\n= Next Section";
        let sanitized_pagebreak = sanitize_typst_content(content_with_pagebreak);
        assert!(sanitized_pagebreak.contains("#pagebreak()"), "Should preserve #pagebreak");

        // Test other safe # commands are preserved
        let content_with_safe = "= Title\n\n#table(columns: 2)[A][B]\n\n#link(\"url\")[text]";
        let sanitized_safe = sanitize_typst_content(content_with_safe);
        assert!(sanitized_safe.contains("#table"), "Should preserve #table");
        assert!(sanitized_safe.contains("#link"), "Should preserve #link");
    }
}

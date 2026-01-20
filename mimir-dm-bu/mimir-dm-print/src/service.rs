//! PrintService - Core service for PDF generation

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use typst::diag::{SourceDiagnostic, Severity};

use crate::error::{PrintError, Result};
use crate::world::MimirTypstWorld;

/// Information about an available template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Template identifier (relative path without .typ extension)
    pub id: String,
    /// Display name
    pub name: String,
    /// Template category (e.g., "character", "spell", "monster")
    pub category: String,
    /// Description of what this template produces
    pub description: Option<String>,
}

/// Service for generating PDFs from Typst templates
pub struct PrintService {
    /// Root directory containing templates
    templates_root: PathBuf,
}

impl PrintService {
    /// Create a new PrintService
    ///
    /// # Arguments
    /// * `templates_root` - Root directory containing Typst templates
    pub fn new(templates_root: PathBuf) -> Self {
        Self { templates_root }
    }

    /// Get the templates root directory
    pub fn templates_root(&self) -> &PathBuf {
        &self.templates_root
    }

    /// Render a template to PDF bytes
    ///
    /// # Arguments
    /// * `template_path` - Path to template relative to templates root (e.g., "character/sheet.typ")
    /// * `data` - JSON data to inject into the template
    ///
    /// # Returns
    /// PDF file contents as bytes
    #[instrument(skip(self, data), fields(template = %template_path))]
    pub fn render_to_pdf(
        &self,
        template_path: &str,
        data: serde_json::Value,
    ) -> Result<Vec<u8>> {
        info!("Rendering template to PDF");

        // Create world with template and data
        let world = MimirTypstWorld::from_template(
            self.templates_root.clone(),
            template_path,
            data,
        )?;

        // Compile the document
        debug!("Compiling Typst document");
        let warned = typst::compile(&world);

        // Log any warnings
        for warning in &warned.warnings {
            tracing::warn!("Typst warning: {}", warning.message);
        }

        match warned.output {
            Ok(document) => {
                // Generate PDF
                debug!("Generating PDF from compiled document");
                let pdf_result = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default());

                match pdf_result {
                    Ok(pdf_bytes) => {
                        info!("PDF generated successfully ({} bytes)", pdf_bytes.len());
                        Ok(pdf_bytes)
                    }
                    Err(errors) => {
                        let error_msg = format_diagnostics(&errors);
                        Err(PrintError::PdfError(error_msg))
                    }
                }
            }
            Err(errors) => {
                let error_msg = format_diagnostics(&errors);
                Err(PrintError::CompilationError(error_msg))
            }
        }
    }

    /// Save PDF bytes to a file
    #[instrument(skip(self, pdf_bytes))]
    pub fn save_pdf(&self, path: &PathBuf, pdf_bytes: &[u8]) -> Result<()> {
        info!("Saving PDF to {:?}", path);
        std::fs::write(path, pdf_bytes)?;
        Ok(())
    }

    /// List all available templates
    #[instrument(skip(self))]
    pub fn list_templates(&self) -> Result<Vec<TemplateInfo>> {
        let mut templates = Vec::new();

        if !self.templates_root.exists() {
            return Ok(templates);
        }

        for entry in walkdir::WalkDir::new(&self.templates_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip directories and non-.typ files
            if !path.is_file() || path.extension().is_none_or(|e| e != "typ") {
                continue;
            }

            // Skip files in _shared directory (these are imports, not templates)
            if path.components().any(|c| c.as_os_str() == "_shared") {
                continue;
            }

            // Get relative path without extension
            if let Ok(rel_path) = path.strip_prefix(&self.templates_root) {
                let id = rel_path
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/");

                // Determine category from first directory component
                let category = rel_path
                    .components()
                    .next()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .unwrap_or_else(|| "general".to_string());

                // Use filename as display name
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| id.clone());

                templates.push(TemplateInfo {
                    id,
                    name: titlecase(&name),
                    category,
                    description: None,
                });
            }
        }

        templates.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(templates)
    }

    /// Check if a template exists
    pub fn template_exists(&self, template_path: &str) -> bool {
        self.templates_root.join(template_path).exists()
    }
}

/// Format Typst diagnostics into a readable error message
fn format_diagnostics(diagnostics: &[SourceDiagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diag| {
            let severity = match diag.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };
            format!("{}: {}", severity, diag.message)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert a string to title case
fn titlecase(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' {
            result.push(' ');
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_template(dir: &TempDir) -> PathBuf {
        let template = r#"
#set page(width: 8.5in, height: 11in, margin: 1in)
#set text(font: "DejaVu Sans", size: 12pt)

= Hello World

This is a test document.

#if data != none [
  Data received: #data
]
"#;
        let template_path = dir.path().join("test/hello.typ");
        fs::create_dir_all(template_path.parent().unwrap()).unwrap();
        fs::write(&template_path, template).unwrap();
        template_path
    }

    #[test]
    fn test_service_creation() {
        let temp = TempDir::new().unwrap();
        let service = PrintService::new(temp.path().to_path_buf());
        assert_eq!(service.templates_root(), temp.path());
    }

    #[test]
    fn test_list_templates_empty() {
        let temp = TempDir::new().unwrap();
        let service = PrintService::new(temp.path().to_path_buf());
        let templates = service.list_templates().unwrap();
        assert!(templates.is_empty());
    }

    #[test]
    fn test_list_templates() {
        let temp = TempDir::new().unwrap();
        setup_test_template(&temp);

        let service = PrintService::new(temp.path().to_path_buf());
        let templates = service.list_templates().unwrap();

        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].id, "test/hello");
        assert_eq!(templates[0].category, "test");
    }

    #[test]
    fn test_template_exists() {
        let temp = TempDir::new().unwrap();
        setup_test_template(&temp);

        let service = PrintService::new(temp.path().to_path_buf());
        assert!(service.template_exists("test/hello.typ"));
        assert!(!service.template_exists("nonexistent.typ"));
    }

    #[test]
    fn test_titlecase() {
        assert_eq!(titlecase("hello_world"), "Hello World");
        assert_eq!(titlecase("character-sheet"), "Character Sheet");
        assert_eq!(titlecase("test"), "Test");
    }

    #[test]
    fn test_render_to_pdf() {
        let temp = TempDir::new().unwrap();

        // Create a simple template that doesn't require specific fonts
        let template = r#"
#set page(width: 8.5in, height: 11in, margin: 1in)

= Hello World

This is a test document.

#if "name" in data [
  Name: #data.name
]
"#;
        let template_path = temp.path().join("test/simple.typ");
        fs::create_dir_all(template_path.parent().unwrap()).unwrap();
        fs::write(&template_path, template).unwrap();

        let service = PrintService::new(temp.path().to_path_buf());
        let data = serde_json::json!({
            "name": "Test User"
        });

        let result = service.render_to_pdf("test/simple.typ", data);
        assert!(result.is_ok(), "PDF generation failed: {:?}", result.err());

        let pdf_bytes = result.unwrap();
        // PDF files start with %PDF
        assert!(pdf_bytes.len() > 100, "PDF seems too small");
        assert_eq!(&pdf_bytes[0..4], b"%PDF", "Output is not a valid PDF");
    }

    #[test]
    fn test_render_campaign_document() {
        use crate::campaign::build_single_document_pdf;

        let temp = TempDir::new().unwrap();
        let templates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        // Create a test markdown document
        let markdown = r#"---
title: Session 1 - The Beginning
type: session_outline
---

# The Adventure Begins

Our heroes gather at the **Yawning Portal** tavern in Waterdeep.

## Key NPCs

- *Durnan* - the barkeep
- *Volothamp Geddarm* - famous explorer

## Objectives

1. Meet with Volo
2. Accept the quest
3. Head to the warehouse district
"#;
        let doc_path = temp.path().join("session_1.md");
        fs::write(&doc_path, markdown).unwrap();

        // Test single document render using new API
        let result = build_single_document_pdf(
            &doc_path,
            Some("Waterdeep Dragon Heist"),
            templates_root,
        );
        assert!(result.is_ok(), "Campaign document render failed: {:?}", result.err());

        let pdf_bytes = result.unwrap();
        assert!(pdf_bytes.len() > 1000, "Campaign document PDF seems too small");
        assert_eq!(&pdf_bytes[0..4], b"%PDF", "Output is not a valid PDF");
    }

    #[test]
    fn test_render_campaign_combined() {
        use crate::campaign::{build_campaign_pdf, CampaignExportData, ExportOptions};

        let temp = TempDir::new().unwrap();
        let templates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        // Create multiple test documents
        let doc1 = r#"---
title: Session 1 - The Beginning
type: session_outline
---

# The Adventure Begins

Our heroes meet at the tavern.
"#;

        let doc2 = r#"---
title: Durnan
type: npc_profile
---

# Durnan

The gruff but fair barkeep of the Yawning Portal.

## Personality

- Taciturn
- Protective of his establishment
"#;

        let doc3 = r#"---
title: The Yawning Portal
type: location_guide
---

# The Yawning Portal

A famous tavern in Waterdeep built over the entrance to Undermountain.
"#;

        let doc1_path = temp.path().join("session_1.md");
        let doc2_path = temp.path().join("npc_durnan.md");
        let doc3_path = temp.path().join("location_yawning_portal.md");

        fs::write(&doc1_path, doc1).unwrap();
        fs::write(&doc2_path, doc2).unwrap();
        fs::write(&doc3_path, doc3).unwrap();

        // Test combined render using new API
        let documents = vec![doc1_path, doc2_path, doc3_path];
        let data = CampaignExportData {
            name: "Waterdeep Dragon Heist".to_string(),
            documents,
            monsters: None,
            traps: None,
            npcs: None,
            campaign_maps: vec![],
            module_maps: vec![],
            base_path: temp.path().to_path_buf(),
            templates_root,
        };

        let options = ExportOptions {
            include_toc: true,
            include_monsters: false,
            include_traps: false,
            include_npcs: false,
            include_campaign_map_previews: false,
            include_campaign_tiled_maps: false,
            include_module_map_previews: false,
            include_module_tiled_maps: false,
            include_token_cutouts: false,
            preview_grid: false,
            preview_los_walls: false,
            preview_positions: false,
            play_grid: false,
            play_los_walls: false,
        };

        let result = build_campaign_pdf(data, options);
        assert!(result.is_ok(), "Combined campaign render failed: {:?}", result.err());

        let pdf_bytes = result.unwrap();
        assert!(pdf_bytes.len() > 2000, "Combined PDF seems too small for 3 documents");
        assert_eq!(&pdf_bytes[0..4], b"%PDF", "Output is not a valid PDF");
    }
}

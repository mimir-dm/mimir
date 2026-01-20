//! Template loader that validates and imports templates with frontmatter

use crate::dal::campaign::template_documents::TemplateRepository;
use crate::models::campaign::template_documents::NewTemplateDocument;
use crate::models::campaign::template_frontmatter::TemplateFrontmatter;
use diesel::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateLoadError {
    #[error("Failed to read template file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse frontmatter: {0}")]
    FrontmatterParse(#[from] serde_yaml::Error),

    #[error("Template has no frontmatter")]
    NoFrontmatter,

    #[error("Failed to validate template rendering: {0}")]
    TemplateValidation(String),

    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Template loader for importing and validating templates
pub struct TemplateLoader {
    tera: Tera,
}

impl Default for TemplateLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateLoader {
    /// Create a new template loader
    pub fn new() -> Self {
        // Create a Tera instance without auto-reload for validation
        let tera = match Tera::new("dummy/*.html") {
            Ok(mut t) => {
                t.autoescape_on(vec![]);
                t
            }
            Err(_) => {
                // If pattern fails, create empty Tera
                let mut t = Tera::default();
                t.autoescape_on(vec![]);
                t
            }
        };

        Self { tera }
    }

    /// Load a single template file
    pub fn load_template_file(
        &mut self,
        file_path: &Path,
    ) -> Result<(TemplateFrontmatter, String), TemplateLoadError> {
        // Read the file
        let content = fs::read_to_string(file_path)?;

        // Parse frontmatter
        let frontmatter = TemplateFrontmatter::parse_from_markdown(&content)
            .ok_or(TemplateLoadError::NoFrontmatter)?;

        // Extract template content (after frontmatter)
        let template_content = TemplateFrontmatter::extract_content(&content);

        // Validate the template renders
        self.validate_template(&frontmatter.id, &template_content, &frontmatter)?;

        Ok((frontmatter, template_content))
    }

    /// Validate that a template renders with its default values
    fn validate_template(
        &mut self,
        template_id: &str,
        content: &str,
        frontmatter: &TemplateFrontmatter,
    ) -> Result<(), TemplateLoadError> {
        // Add template to Tera
        self.tera
            .add_raw_template(template_id, content)
            .map_err(|e| TemplateLoadError::TemplateValidation(e.to_string()))?;

        // Create context with default values
        let mut context = Context::new();
        for var in &frontmatter.variables {
            context.insert(&var.name, &var.default);
        }

        // Try to render the template
        self.tera
            .render(template_id, &context)
            .map_err(|e| TemplateLoadError::TemplateValidation(e.to_string()))?;

        Ok(())
    }

    /// Load and import a template into the database
    pub fn import_template(
        &mut self,
        conn: &mut SqliteConnection,
        file_path: &Path,
    ) -> Result<String, TemplateLoadError> {
        let (frontmatter, content) = self.load_template_file(file_path)?;

        // Create the template document
        let new_template = NewTemplateDocument {
            document_id: frontmatter.id.clone(),
            version_number: None, // Will auto-increment
            document_content: content,
            content_hash: None, // Will be computed by repository
            document_type: Some(frontmatter.template_type.clone()),
            document_level: Some(frontmatter.level.clone()),
            purpose: Some(frontmatter.purpose.clone()),
            variables_schema: Some(frontmatter.variables_schema()?),
            default_values: Some(serde_json::to_string(&frontmatter.defaults_map())?),
            is_active: Some(true),
            metadata: Some(frontmatter.to_json()?),
        };

        // Insert into database
        let created = TemplateRepository::create(conn, new_template)?;

        Ok(created.document_id)
    }

    /// Load all templates from a directory
    pub fn load_directory(
        &mut self,
        conn: &mut SqliteConnection,
        dir_path: &Path,
    ) -> Result<LoadSummary, TemplateLoadError> {
        let mut summary = LoadSummary::default();

        // Read all .md files in directory
        let entries = fs::read_dir(dir_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip non-markdown files
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            // Skip README
            if path.file_name().and_then(|s| s.to_str()) == Some("README.md") {
                continue;
            }

            summary.total += 1;

            // Try to load the template
            match self.import_template(conn, &path) {
                Ok(template_id) => {
                    summary.loaded += 1;
                    summary.loaded_ids.push(template_id);
                }
                Err(e) => {
                    summary.errors += 1;
                    summary.error_details.push((path.clone(), e.to_string()));

                    // Check if it's because template already exists
                    if let TemplateLoadError::Database(diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        _,
                    )) = e
                    {
                        summary.skipped += 1;
                        summary.errors -= 1; // Don't count duplicates as errors
                    }
                }
            }
        }

        Ok(summary)
    }
}

/// Summary of template loading operation
#[derive(Debug, Default)]
pub struct LoadSummary {
    pub total: usize,
    pub loaded: usize,
    pub skipped: usize,
    pub errors: usize,
    pub loaded_ids: Vec<String>,
    pub error_details: Vec<(PathBuf, String)>,
}

impl LoadSummary {
    /// Check if all templates loaded successfully
    pub fn all_loaded(&self) -> bool {
        self.errors == 0 && self.loaded + self.skipped == self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_validate_template() {
        let mut loader = TemplateLoader::new();

        let frontmatter = TemplateFrontmatter {
            id: "test-template".to_string(),
            title: "Test Template".to_string(),
            template_type: "test".to_string(),
            level: "test".to_string(),
            purpose: "Testing".to_string(),
            variables: vec![
                crate::models::campaign::template_frontmatter::TemplateVariable {
                    name: "name".to_string(),
                    var_type: "string".to_string(),
                    description: "A name".to_string(),
                    default: serde_json::json!("World"),
                    required: true,
                },
            ],
            author: "Test".to_string(),
        };

        let content = "Hello {{ name }}!";

        // Should validate successfully
        loader
            .validate_template("test", content, &frontmatter)
            .unwrap();
    }

    #[test]
    fn test_load_template_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let content = r#"---
id: test-template
title: Test Template
type: test
level: campaign
purpose: Testing template loading
author: Test Author
variables:
  - name: campaign_name
    type: string
    description: Campaign name
    default: "Test Campaign"
    required: true
---

# {{ campaign_name }}

This is a test template."#;

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let mut loader = TemplateLoader::new();
        let (frontmatter, template_content) = loader.load_template_file(&file_path).unwrap();

        assert_eq!(frontmatter.id, "test-template");
        assert_eq!(frontmatter.title, "Test Template");
        assert!(template_content.contains("# {{ campaign_name }}"));
    }
}

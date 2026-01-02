//! DocumentBuilder - Composable PDF document assembly
//!
//! Provides a trait-based architecture for assembling PDF documents from
//! various section types. Each section implements `Renderable` to produce
//! Typst markup, and `DocumentBuilder` combines them into a single PDF.
//!
//! # Example
//!
//! ```ignore
//! use mimir_dm_print::{DocumentBuilder, MarkdownSection};
//!
//! let pdf = DocumentBuilder::new("My Campaign")
//!     .with_toc(true)
//!     .append(MarkdownSection::from_file(&doc_path)?)
//!     .append(MarkdownSection::from_file(&session_notes)?)
//!     .to_pdf()?;
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::error::{PrintError, Result};

/// Registry for virtual files that will be available to Typst
///
/// Files registered here are accessible via `/_virtual/filename` paths in Typst.
#[derive(Debug, Clone, Default)]
pub struct VirtualFileRegistry {
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl VirtualFileRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a file and return the virtual path to use in Typst
    pub fn register(&self, name: &str, data: Vec<u8>) -> String {
        let virtual_path = format!("/_virtual/{}", name);
        if let Ok(mut files) = self.files.write() {
            files.insert(virtual_path.clone(), data);
        }
        virtual_path
    }

    /// Get all registered files
    pub fn into_files(self) -> HashMap<String, Vec<u8>> {
        Arc::try_unwrap(self.files)
            .map(|lock| lock.into_inner().unwrap_or_default())
            .unwrap_or_else(|arc| arc.read().map(|g| g.clone()).unwrap_or_default())
    }
}

/// Context passed to Renderable implementations during rendering
///
/// Contains shared resources and configuration needed for rendering.
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// Temporary directory for intermediate files (e.g., converted images)
    pub temp_dir: PathBuf,
    /// Base path for resolving relative image/file references
    pub base_path: Option<PathBuf>,
    /// Virtual file registry for in-memory images
    pub virtual_files: VirtualFileRegistry,
}

impl RenderContext {
    /// Create a new render context with a temporary directory
    pub fn new(temp_dir: PathBuf) -> Self {
        Self {
            temp_dir,
            base_path: None,
            virtual_files: VirtualFileRegistry::new(),
        }
    }

    /// Set the base path for resolving relative references
    pub fn with_base_path(mut self, path: PathBuf) -> Self {
        self.base_path = Some(path);
        self
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self {
            temp_dir: std::env::temp_dir(),
            base_path: None,
            virtual_files: VirtualFileRegistry::new(),
        }
    }
}

/// Trait for anything that can be rendered to Typst markup
///
/// Implementors produce Typst markup strings that can be assembled
/// into a complete document by the `DocumentBuilder`.
pub trait Renderable: Send + Sync {
    /// Convert this section to Typst markup
    ///
    /// # Arguments
    /// * `ctx` - Render context with shared resources
    ///
    /// # Returns
    /// Typst markup string for this section
    fn to_typst(&self, ctx: &RenderContext) -> Result<String>;

    /// Title for table of contents entry
    ///
    /// Return `Some(title)` to include this section in the TOC,
    /// or `None` to exclude it.
    fn toc_title(&self) -> Option<String> {
        None
    }

    /// Whether this section should start on a new page
    ///
    /// Default is true for most sections.
    fn page_break_before(&self) -> bool {
        true
    }

    /// Custom page margin for this section (in inches)
    ///
    /// Return `Some(margin)` to override the document default,
    /// or `None` to use the document's configured margin.
    fn page_margin(&self) -> Option<f32> {
        None
    }
}

/// Configuration for document assembly
#[derive(Debug, Clone)]
pub struct DocumentConfig {
    /// Include table of contents
    pub include_toc: bool,
    /// Include page numbers
    pub page_numbers: bool,
    /// Page margin in inches
    pub margin: f32,
    /// Base font size in points
    pub font_size: f32,
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self {
            include_toc: false,
            page_numbers: true,
            margin: 0.5, // Reduced for more drawing area
            font_size: 10.0,
        }
    }
}

/// Builder for assembling PDF documents from sections
///
/// Collects `Renderable` sections and assembles them into a single
/// PDF document with optional table of contents and page numbers.
pub struct DocumentBuilder {
    title: String,
    sections: Vec<Box<dyn Renderable>>,
    config: DocumentConfig,
    context: RenderContext,
    /// Path to templates directory (for shared components)
    templates_root: PathBuf,
}

impl DocumentBuilder {
    /// Create a new document builder with the given title
    pub fn new(title: impl Into<String>) -> Self {
        // Default templates root - can be overridden with with_templates_root
        let templates_root = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("templates"))
            .unwrap_or_else(|| PathBuf::from("templates"));

        Self {
            title: title.into(),
            sections: Vec::new(),
            config: DocumentConfig::default(),
            context: RenderContext::default(),
            templates_root,
        }
    }

    /// Set the templates root directory (for shared components)
    pub fn with_templates_root(mut self, path: PathBuf) -> Self {
        self.templates_root = path;
        self
    }

    /// Enable or disable table of contents generation
    pub fn with_toc(mut self, include: bool) -> Self {
        self.config.include_toc = include;
        self
    }

    /// Enable or disable page numbers
    pub fn with_page_numbers(mut self, include: bool) -> Self {
        self.config.page_numbers = include;
        self
    }

    /// Set page margins (in inches)
    pub fn with_margin(mut self, margin: f32) -> Self {
        self.config.margin = margin;
        self
    }

    /// Set the base font size (in points)
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.config.font_size = size;
        self
    }

    /// Set the render context (temp directory, base path, etc.)
    pub fn with_context(mut self, context: RenderContext) -> Self {
        self.context = context;
        self
    }

    /// Set the temporary directory for intermediate files
    pub fn with_temp_dir(mut self, temp_dir: PathBuf) -> Self {
        self.context.temp_dir = temp_dir;
        self
    }

    /// Set the base path for resolving relative references
    pub fn with_base_path(mut self, base_path: PathBuf) -> Self {
        self.context.base_path = Some(base_path);
        self
    }

    /// Append a section to the document
    ///
    /// Sections are rendered in the order they are appended.
    pub fn append<R: Renderable + 'static>(mut self, section: R) -> Self {
        self.sections.push(Box::new(section));
        self
    }

    /// Append a section to the document (mutable reference version)
    pub fn push<R: Renderable + 'static>(&mut self, section: R) -> &mut Self {
        self.sections.push(Box::new(section));
        self
    }

    /// Check if the document has any sections
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// Get the number of sections in the document
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    /// Build the complete Typst document
    fn build_typst(&self) -> Result<String> {
        let mut output = String::new();

        // Document preamble with settings
        output.push_str(&self.build_preamble());

        // Title page
        output.push_str(&self.build_title_page());

        // Table of contents (if enabled)
        if self.config.include_toc {
            output.push_str(&self.build_toc());
        }

        // Track current margin to avoid redundant set commands
        let mut current_margin = self.config.margin;

        // Render each section
        for (i, section) in self.sections.iter().enumerate() {
            // Page break before section (except first if no TOC)
            if section.page_break_before() && (i > 0 || self.config.include_toc) {
                output.push_str("\n#pagebreak()\n\n");
            }

            // Check if section wants a different margin
            let section_margin = section.page_margin().unwrap_or(self.config.margin);
            if (section_margin - current_margin).abs() > 0.001 {
                output.push_str(&format!(
                    "#set page(margin: {}in)\n",
                    section_margin
                ));
                current_margin = section_margin;
            }

            // Add TOC anchor if section has a title
            if let Some(title) = section.toc_title() {
                // Create a heading that will appear in the outline
                output.push_str(&format!(
                    "\n#heading(level: 1, outlined: true)[{}]\n\n",
                    escape_typst_string(&title)
                ));
            }

            // Render section content
            let typst_content = section.to_typst(&self.context)?;
            output.push_str(&typst_content);
            output.push('\n');
        }

        Ok(output)
    }

    /// Build the document preamble (page setup, fonts, shared imports)
    fn build_preamble(&self) -> String {
        let mut preamble = String::new();

        // Import shared styles and components
        preamble.push_str("#import \"/_shared/styles.typ\": *\n");
        preamble.push_str("#import \"/_shared/components.typ\": *\n");
        preamble.push_str("#import \"/_shared/icons.typ\": *\n\n");

        // Page setup using shared styles
        preamble.push_str(&format!(
            "#set page(width: 8.5in, height: 11in, margin: {}in",
            self.config.margin
        ));

        // Page numbers in footer
        if self.config.page_numbers {
            preamble.push_str(", footer: context [#h(1fr) #counter(page).display() #h(1fr)]");
        }

        preamble.push_str(")\n");

        // Text settings using shared fonts
        preamble.push_str("#set text(font: font-body, size: sizes.base, fill: colors.text)\n");

        // Heading styles
        preamble.push_str("#set heading(numbering: none)\n");
        preamble.push_str("#show heading.where(level: 1): it => {\n");
        preamble.push_str("  set text(size: sizes.xl, weight: \"bold\", font: font-heading)\n");
        preamble.push_str("  v(spacing.md)\n");
        preamble.push_str("  it\n");
        preamble.push_str("  v(spacing.sm)\n");
        preamble.push_str("}\n");
        preamble.push_str("#show heading.where(level: 2): it => {\n");
        preamble.push_str("  set text(size: sizes.lg, weight: \"bold\", font: font-heading)\n");
        preamble.push_str("  v(spacing.md)\n");
        preamble.push_str("  it\n");
        preamble.push_str("  v(spacing.xs)\n");
        preamble.push_str("}\n");
        preamble.push_str("#show heading.where(level: 3): it => {\n");
        preamble.push_str("  set text(size: sizes.md, weight: \"bold\")\n");
        preamble.push_str("  v(spacing.sm)\n");
        preamble.push_str("  it\n");
        preamble.push_str("  v(spacing.xs)\n");
        preamble.push_str("}\n\n");

        preamble
    }

    /// Build the title page
    fn build_title_page(&self) -> String {
        let mut title_page = String::new();

        title_page.push_str("#align(center + horizon)[\n");
        title_page.push_str(&format!(
            "  #title-text[{}]\n",
            escape_typst_string(&self.title)
        ));
        title_page.push_str("]\n");

        title_page
    }

    /// Build the table of contents
    fn build_toc(&self) -> String {
        let mut toc = String::new();

        toc.push_str("#pagebreak()\n");
        toc.push_str("#heading(level: 1, outlined: false)[Table of Contents]\n");
        toc.push_str("#v(1em)\n");
        toc.push_str("#outline(title: none, indent: 1em)\n");

        toc
    }

    /// Render the document to PDF bytes
    ///
    /// # Returns
    /// PDF file contents as bytes
    pub fn to_pdf(self) -> Result<Vec<u8>> {
        if self.sections.is_empty() {
            return Err(PrintError::InvalidData(
                "Cannot render empty document".to_string(),
            ));
        }

        // Build the complete Typst document in memory
        let typst_content = self.build_typst()?;

        // Always write debug Typst content to temp file for inspection
        let debug_path = std::env::temp_dir().join("mimir_debug.typ");
        if let Err(e) = std::fs::write(&debug_path, &typst_content) {
            tracing::warn!("Failed to write debug Typst file: {}", e);
        } else {
            tracing::debug!("Debug Typst written to: {}", debug_path.display());
        }

        // Extract virtual files registered during rendering
        let virtual_files = self.context.virtual_files.into_files();
        tracing::debug!("Registered {} virtual files", virtual_files.len());

        // Create world with in-memory content and virtual files
        use crate::world::MimirTypstWorld;
        use typst::diag::Severity;

        let world = MimirTypstWorld::from_content_with_files(
            typst_content.clone(),
            self.templates_root.clone(),
            virtual_files,
        );

        let warned = typst::compile(&world);

        for warning in &warned.warnings {
            tracing::warn!("Typst warning: {}", warning.message);
        }

        match warned.output {
            Ok(document) => {
                let pdf_result = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default());
                match pdf_result {
                    Ok(pdf_bytes) => Ok(pdf_bytes),
                    Err(errors) => {
                        let error_msg = errors
                            .iter()
                            .map(|d| format!("{}: {}",
                                match d.severity { Severity::Error => "error", Severity::Warning => "warning" },
                                d.message
                            ))
                            .collect::<Vec<_>>()
                            .join("\n");
                        Err(PrintError::PdfError(error_msg))
                    }
                }
            }
            Err(errors) => {
                // Write debug file on error
                let debug_path = std::env::temp_dir().join("mimir_debug_error.typ");
                if let Err(e) = std::fs::write(&debug_path, &typst_content) {
                    tracing::warn!("Failed to write debug Typst file: {}", e);
                } else {
                    tracing::error!("Typst compilation failed. Debug file: {}", debug_path.display());
                }

                let error_msg = errors
                    .iter()
                    .map(|d| format!("{}: {}",
                        match d.severity { Severity::Error => "error", Severity::Warning => "warning" },
                        d.message
                    ))
                    .collect::<Vec<_>>()
                    .join("\n");
                Err(PrintError::CompilationError(error_msg))
            }
        }
    }
}

/// Escape special characters for Typst strings
fn escape_typst_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple test section that renders static content
    struct TestSection {
        content: String,
        title: Option<String>,
    }

    impl TestSection {
        fn new(content: &str) -> Self {
            Self {
                content: content.to_string(),
                title: None,
            }
        }

        fn with_title(mut self, title: &str) -> Self {
            self.title = Some(title.to_string());
            self
        }
    }

    impl Renderable for TestSection {
        fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
            Ok(self.content.clone())
        }

        fn toc_title(&self) -> Option<String> {
            self.title.clone()
        }
    }

    #[test]
    fn test_document_builder_empty() {
        let builder = DocumentBuilder::new("Test");
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn test_document_builder_append() {
        let builder = DocumentBuilder::new("Test")
            .append(TestSection::new("Content 1"))
            .append(TestSection::new("Content 2"));

        assert!(!builder.is_empty());
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn test_document_builder_config() {
        let builder = DocumentBuilder::new("Test")
            .with_toc(true)
            .with_page_numbers(false)
            .with_margin(1.0)
            .with_font_size(12.0);

        assert!(builder.config.include_toc);
        assert!(!builder.config.page_numbers);
        assert_eq!(builder.config.margin, 1.0);
        assert_eq!(builder.config.font_size, 12.0);
    }

    #[test]
    fn test_build_typst_simple() {
        let builder = DocumentBuilder::new("Test Document")
            .append(TestSection::new("Hello, world!"));

        let typst = builder.build_typst().unwrap();

        assert!(typst.contains("Test Document"));
        assert!(typst.contains("Hello, world!"));
        assert!(typst.contains("#set page"));
    }

    #[test]
    fn test_build_typst_with_toc() {
        let builder = DocumentBuilder::new("Test Document")
            .with_toc(true)
            .append(TestSection::new("Content").with_title("Chapter 1"));

        let typst = builder.build_typst().unwrap();

        assert!(typst.contains("#outline"));
        assert!(typst.contains("Chapter 1"));
    }

    #[test]
    fn test_escape_typst_string() {
        assert_eq!(escape_typst_string("hello"), "hello");
        assert_eq!(escape_typst_string("hello [world]"), "hello \\[world\\]");
        assert_eq!(escape_typst_string("#hashtag"), "\\#hashtag");
        assert_eq!(escape_typst_string("$money$"), "\\$money\\$");
    }
}

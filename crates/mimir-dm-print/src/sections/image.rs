//! Image section for PDF export
//!
//! Renders a standalone image document to a full page.

use std::fs;
use std::path::PathBuf;

use crate::builder::{RenderContext, Renderable};
use crate::error::{PrintError, Result};

/// Image section - renders an image fit to a single page
pub struct ImageSection {
    /// Title for the image (shown as heading)
    title: String,
    /// Source of the image data
    source: ImageSource,
}

enum ImageSource {
    /// Load from file path
    FromFile { path: PathBuf },
    /// Use pre-loaded bytes
    FromBytes { bytes: Vec<u8>, extension: String },
}

impl ImageSection {
    /// Create an image section from a file path
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| {
                // Convert kebab-case or snake_case to Title Case
                s.replace('-', " ")
                    .replace('_', " ")
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            Some(c) => c.to_uppercase().chain(chars).collect(),
                            None => String::new(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_else(|| "Image".to_string());

        Ok(Self {
            title,
            source: ImageSource::FromFile { path: path.clone() },
        })
    }

    /// Create an image section from bytes
    pub fn from_bytes(title: String, bytes: Vec<u8>, extension: &str) -> Self {
        Self {
            title,
            source: ImageSource::FromBytes {
                bytes,
                extension: extension.to_string(),
            },
        }
    }

    /// Set the title
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
}

impl Renderable for ImageSection {
    fn to_typst(&self, ctx: &RenderContext) -> Result<String> {
        let (image_bytes, extension) = match &self.source {
            ImageSource::FromFile { path } => {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png")
                    .to_lowercase();

                let bytes = fs::read(path).map_err(|e| {
                    PrintError::InvalidData(format!("Failed to read image: {}", e))
                })?;

                (bytes, ext)
            }
            ImageSource::FromBytes { bytes, extension } => {
                (bytes.clone(), extension.clone())
            }
        };

        // Generate a unique filename for the virtual file
        let image_filename = format!("image_{}.{}", sanitize_filename(&self.title), extension);

        // Register image in virtual file system
        let virtual_path = ctx.virtual_files.register(&image_filename, image_bytes);
        tracing::debug!("Registered image as: {}", virtual_path);

        // Render with heading and centered image
        let typst = format!(
            r#"#block(breakable: false)[
  #heading(level: 1, outlined: true)[{}]
  #v(1em)
  #align(center)[
    #image("{}", width: 100%, height: auto, fit: "contain")
  ]
]
"#,
            escape_typst_string(&self.title),
            virtual_path
        );
        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        // Return None - we render our own heading in to_typst()
        None
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

/// Sanitize a string for use as a filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Check if a file extension indicates an image type
pub fn is_image_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_extension() {
        assert!(is_image_extension("png"));
        assert!(is_image_extension("PNG"));
        assert!(is_image_extension("jpg"));
        assert!(is_image_extension("jpeg"));
        assert!(is_image_extension("gif"));
        assert!(is_image_extension("webp"));
        assert!(is_image_extension("svg"));
        assert!(!is_image_extension("md"));
        assert!(!is_image_extension("txt"));
        assert!(!is_image_extension("pdf"));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello world"), "hello_world");
        assert_eq!(sanitize_filename("test-image"), "test-image");
        assert_eq!(sanitize_filename("my_file"), "my_file");
        assert_eq!(sanitize_filename("image.png"), "image_png");
    }
}

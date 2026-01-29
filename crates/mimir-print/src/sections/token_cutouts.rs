//! Token cutout sheet section for physical play
//!
//! Renders tokens as printable paper standees (fold-in-half standees)
//! for use on physical battle maps.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Counter for unique token image filenames
static TOKEN_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Token data for cutout rendering
#[derive(Debug, Clone)]
pub struct CutoutToken {
    /// Display name for the token
    pub name: String,
    /// Size category (tiny, small, medium, large, huge, gargantuan)
    pub size: String,
    /// Token type for color fallback (monster, npc, pc)
    pub token_type: String,
    /// Image bytes (pre-loaded)
    pub image_bytes: Option<Vec<u8>>,
    /// Number of copies to print
    pub quantity: u32,
}

impl CutoutToken {
    /// Create a new cutout token
    pub fn new(name: String, size: String, token_type: String) -> Self {
        Self {
            name,
            size,
            token_type,
            image_bytes: None,
            quantity: 1,
        }
    }

    /// Set the image bytes
    pub fn with_image(mut self, bytes: Vec<u8>) -> Self {
        self.image_bytes = Some(bytes);
        self
    }

    /// Set the quantity
    pub fn with_quantity(mut self, qty: u32) -> Self {
        self.quantity = qty;
        self
    }

    /// Get the size multiplier for layout calculations
    fn size_multiplier(&self) -> f64 {
        match self.size.to_lowercase().as_str() {
            "tiny" | "t" => 0.5,
            "small" | "s" => 1.0,
            "medium" | "m" => 1.0,
            "large" | "l" => 2.0,
            "huge" | "h" => 3.0,
            "gargantuan" | "g" => 4.0,
            _ => 1.0,
        }
    }
}

/// Token cutout sheet - renders tokens as printable standees
pub struct TokenCutoutSection {
    tokens: Vec<CutoutToken>,
    /// Base cell size in inches (1.0 = 1 inch = 1 grid square)
    cell_size: f64,
    /// Whether to show cut lines
    show_cut_lines: bool,
}

impl TokenCutoutSection {
    /// Create a new token cutout section
    pub fn new(tokens: Vec<CutoutToken>) -> Self {
        Self {
            tokens,
            cell_size: 1.0,
            show_cut_lines: true,
        }
    }

    /// Set the base cell size in inches
    pub fn with_cell_size(mut self, size: f64) -> Self {
        self.cell_size = size;
        self
    }

    /// Set whether to show cut lines
    pub fn with_cut_lines(mut self, show: bool) -> Self {
        self.show_cut_lines = show;
        self
    }

    /// Get color for token type (for fallback circles)
    fn token_type_color(token_type: &str) -> &'static str {
        match token_type.to_lowercase().as_str() {
            "monster" => "#dc3545",
            "pc" => "#28a745",
            "npc" => "#007bff",
            "trap" => "#ffc107",
            _ => "#6c757d",
        }
    }

    /// Render a single token standee as a tent (back panel on top, image on bottom)
    fn render_token(
        &self,
        token: &CutoutToken,
        ctx: &RenderContext,
        size_inches: f64,
    ) -> Result<String> {
        let size_pt = size_inches * 72.0;

        // Generate unique filename if we have image bytes
        let image_content = if let Some(ref bytes) = token.image_bytes {
            let counter = TOKEN_COUNTER.fetch_add(1, Ordering::SeqCst);
            let safe_name = sanitize_filename(&token.name);
            let filename = format!("token_{}_{}.png", safe_name, counter);

            // Convert image to PNG (Typst doesn't support webp)
            let png_bytes = match image::load_from_memory(bytes) {
                Ok(img) => {
                    let mut png_data = Vec::new();
                    if img.write_to(
                        &mut std::io::Cursor::new(&mut png_data),
                        image::ImageFormat::Png,
                    ).is_ok() {
                        png_data
                    } else {
                        bytes.clone()
                    }
                }
                Err(_) => bytes.clone(),
            };

            // Register in virtual file system
            let virtual_path = ctx.virtual_files.register(&filename, png_bytes.clone());
            tracing::debug!(
                "Token image registered: {} ({} bytes)",
                virtual_path,
                png_bytes.len()
            );

            Some(format!(
                "#image(\"{}\", width: {}pt, height: {}pt)",
                virtual_path, size_pt, size_pt
            ))
        } else {
            None
        };

        // Fallback to colored circle with initial if no image
        let content = image_content.unwrap_or_else(|| {
            let color = Self::token_type_color(&token.token_type);
            let initial = token.name.chars().next().unwrap_or('?');
            format!(
                "#circle(radius: {}pt, fill: rgb(\"{}\"), stroke: 1pt)[#align(center + horizon)[#text(fill: white, weight: \"bold\")[{}]]]",
                size_pt / 2.0, color, initial
            )
        });

        // Tent-style standee:
        // |_____|  <- back panel (blank with name)
        // |image|  <- front panel (token image)
        // Fold at the line between them to create standing tent
        let cut_style = if self.show_cut_lines {
            "stroke: (dash: \"dashed\", paint: luma(180))"
        } else {
            "stroke: luma(220)"
        };

        let name = truncate_name(&token.name, 14);

        // Create tent: back panel on top, fold line, image on bottom
        Ok(format!(
            r#"box(width: {}pt, {})[
  #align(center)[
    #box(width: {}pt, height: {}pt, stroke: luma(200))[
      #align(center + horizon)[#text(size: 7pt)[{}]]
    ]
    #v(-1pt)
    #line(length: {}pt, stroke: 1.5pt + black)
    #v(-1pt)
    #box(stroke: luma(200))[{}]
  ]
]"#,
            size_pt + 4.0,
            cut_style,
            size_pt,
            size_pt,
            name,
            size_pt,
            content
        ))
    }

    /// Render a grid of same-sized tokens
    fn render_token_grid(
        &self,
        tokens: &[&CutoutToken],
        ctx: &RenderContext,
        size_mult: f64,
    ) -> Result<String> {
        let cell_size = self.cell_size * size_mult;
        // Calculate how many fit in 7" width
        let cols = (7.0 / cell_size).floor() as usize;
        let cols = cols.max(1);

        let mut typst = String::new();

        typst.push_str(&format!(
            "#grid(columns: {}, gutter: 4pt,\n",
            cols
        ));

        for token in tokens {
            // Render each copy of the token
            for _ in 0..token.quantity {
                let token_typst = self.render_token(token, ctx, cell_size)?;
                typst.push_str(&format!("  {},\n", token_typst));
            }
        }

        typst.push_str(")\n#v(0.5em)\n");
        Ok(typst)
    }
}

impl Renderable for TokenCutoutSection {
    fn to_typst(&self, ctx: &RenderContext) -> Result<String> {
        if self.tokens.is_empty() {
            return Ok("// No tokens to render\n".to_string());
        }

        let mut typst = String::new();

        // Header
        typst.push_str("#pagebreak(weak: true)\n");
        typst.push_str("#align(center)[#text(size: 14pt, weight: \"bold\")[Token Cutouts]]\n");
        typst.push_str("#v(0.3em)\n");
        typst.push_str("#text(size: 9pt)[Cut along dashed lines. Fold at solid line to create standing tokens.]\n");
        typst.push_str("#v(0.5em)\n\n");

        // Group tokens by size for efficient layout
        let mut tiny_tokens: Vec<&CutoutToken> = Vec::new();
        let mut small_medium_tokens: Vec<&CutoutToken> = Vec::new();
        let mut large_tokens: Vec<&CutoutToken> = Vec::new();
        let mut huge_tokens: Vec<&CutoutToken> = Vec::new();
        let mut gargantuan_tokens: Vec<&CutoutToken> = Vec::new();

        for token in &self.tokens {
            let mult = token.size_multiplier();
            if mult <= 0.5 {
                tiny_tokens.push(token);
            } else if mult <= 1.0 {
                small_medium_tokens.push(token);
            } else if mult <= 2.0 {
                large_tokens.push(token);
            } else if mult <= 3.0 {
                huge_tokens.push(token);
            } else {
                gargantuan_tokens.push(token);
            }
        }

        // Render each group (smallest first for efficient packing)
        if !tiny_tokens.is_empty() {
            typst.push_str(&self.render_token_grid(&tiny_tokens, ctx, 0.5)?);
        }
        if !small_medium_tokens.is_empty() {
            typst.push_str(&self.render_token_grid(&small_medium_tokens, ctx, 1.0)?);
        }
        if !large_tokens.is_empty() {
            typst.push_str(&self.render_token_grid(&large_tokens, ctx, 2.0)?);
        }
        if !huge_tokens.is_empty() {
            typst.push_str(&self.render_token_grid(&huge_tokens, ctx, 3.0)?);
        }
        if !gargantuan_tokens.is_empty() {
            typst.push_str(&self.render_token_grid(&gargantuan_tokens, ctx, 4.0)?);
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        Some("Token Cutouts".to_string())
    }
}

/// Sanitize a string for use as filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Truncate name to max length
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len - 3])
    }
}

/// Detect image format from magic bytes
#[allow(dead_code)]
fn detect_image_format(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 3 {
        // JPEG magic: FF D8 FF
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return "jpg";
        }
    }
    if bytes.len() >= 8 {
        // PNG magic: 89 50 4E 47 0D 0A 1A 0A
        if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return "png";
        }
    }
    if bytes.len() >= 12 {
        // WebP magic: RIFF....WEBP
        if bytes.starts_with(&[0x52, 0x49, 0x46, 0x46]) && &bytes[8..12] == b"WEBP" {
            return "webp";
        }
    }
    "png" // Default to PNG
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_multiplier() {
        let token = CutoutToken::new("Test".into(), "medium".into(), "monster".into());
        assert_eq!(token.size_multiplier(), 1.0);

        let large = CutoutToken::new("Test".into(), "large".into(), "monster".into());
        assert_eq!(large.size_multiplier(), 2.0);

        let tiny = CutoutToken::new("Test".into(), "tiny".into(), "monster".into());
        assert_eq!(tiny.size_multiplier(), 0.5);
    }

    #[test]
    fn test_truncate_name() {
        assert_eq!(truncate_name("Goblin", 12), "Goblin");
        assert_eq!(truncate_name("Ancient Red Dragon", 12), "Ancient R...");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Goblin Boss"), "Goblin_Boss");
        assert_eq!(sanitize_filename("Test (Variant)"), "Test__Variant_");
    }

    #[test]
    fn test_detect_image_format() {
        // PNG magic bytes
        let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_image_format(&png), "png");

        // JPEG magic bytes
        let jpg = [0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(detect_image_format(&jpg), "jpg");

        // WebP magic bytes
        let webp = [0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50];
        assert_eq!(detect_image_format(&webp), "webp");
    }
}

//! Token cutout sheet section for physical play

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;
use crate::map_renderer::RenderToken;

/// Counter for unique token image filenames
static TOKEN_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Token cutout sheet - renders tokens as printable standees
pub struct TokenCutoutSheet {
    tokens: Vec<RenderToken>,
    /// Base path for resolving token images
    base_path: PathBuf,
    /// Size of each token cell in inches (default: 1.0 for medium creatures)
    cell_size: f64,
}

impl TokenCutoutSheet {
    /// Create a new token cutout sheet
    pub fn new(tokens: Vec<RenderToken>, base_path: PathBuf) -> Self {
        Self {
            tokens,
            base_path,
            cell_size: 1.0,
        }
    }

    /// Set the base cell size in inches
    pub fn with_cell_size(mut self, size: f64) -> Self {
        self.cell_size = size;
        self
    }

    /// Get the size multiplier for a token
    fn size_multiplier(size: &str) -> f64 {
        match size.to_lowercase().as_str() {
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

impl Renderable for TokenCutoutSheet {
    fn to_typst(&self, ctx: &RenderContext) -> Result<String> {
        if self.tokens.is_empty() {
            return Ok("// No tokens to render\n".to_string());
        }

        let mut typst = String::new();
        typst.push_str("#align(center)[#text(size: 14pt, weight: \"bold\")[Token Cutouts]]\n");
        typst.push_str("#v(0.5em)\n");
        typst.push_str("#text(size: 9pt)[Cut along dashed lines. Fold at solid line for standees.]\n");
        typst.push_str("#v(1em)\n\n");

        // Group tokens by size for efficient layout
        let mut small_tokens: Vec<&RenderToken> = Vec::new();
        let mut medium_tokens: Vec<&RenderToken> = Vec::new();
        let mut large_tokens: Vec<&RenderToken> = Vec::new();

        for token in &self.tokens {
            let mult = Self::size_multiplier(&token.size);
            if mult <= 0.5 {
                small_tokens.push(token);
            } else if mult <= 1.0 {
                medium_tokens.push(token);
            } else {
                large_tokens.push(token);
            }
        }

        // Render each group
        if !medium_tokens.is_empty() {
            typst.push_str(&self.render_token_grid(&medium_tokens, ctx, 1.0)?);
        }
        if !small_tokens.is_empty() {
            typst.push_str(&self.render_token_grid(&small_tokens, ctx, 0.5)?);
        }
        if !large_tokens.is_empty() {
            for token in large_tokens {
                typst.push_str(&self.render_large_token(token, ctx)?);
            }
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        Some("Token Cutouts".to_string())
    }
}

impl TokenCutoutSheet {
    /// Render a grid of same-sized tokens
    fn render_token_grid(
        &self,
        tokens: &[&RenderToken],
        ctx: &RenderContext,
        size_mult: f64,
    ) -> Result<String> {
        let cell_size = self.cell_size * size_mult;
        let cols = (7.0 / cell_size).floor() as usize; // Fit in 7" width

        let mut typst = String::new();
        typst.push_str(&format!(
            "#grid(columns: {}, gutter: 4pt,\n",
            cols
        ));

        for token in tokens {
            let token_typst = self.render_single_token(token, ctx, cell_size)?;
            typst.push_str(&format!("  {},\n", token_typst));
        }

        typst.push_str(")\n#v(0.5em)\n");
        Ok(typst)
    }

    /// Render a single token cell
    fn render_single_token(
        &self,
        token: &RenderToken,
        ctx: &RenderContext,
        size_inches: f64,
    ) -> Result<String> {
        let size_pt = size_inches * 72.0;

        // Try to load token image and register in virtual files
        let image_content = if let Some(ref img_path) = token.image_path {
            let full_path = if img_path.starts_with('/') {
                PathBuf::from(img_path)
            } else {
                self.base_path.join(img_path)
            };

            if full_path.exists() {
                // Read the image file and register in virtual file system
                if let Ok(bytes) = std::fs::read(&full_path) {
                    // Generate unique filename
                    let counter = TOKEN_COUNTER.fetch_add(1, Ordering::SeqCst);
                    let safe_name = sanitize_filename(&token.name);
                    let filename = format!("token_{}_{}.png", safe_name, counter);

                    // Register in virtual file system
                    let virtual_path = ctx.virtual_files.register(&filename, bytes.clone());
                    tracing::debug!(
                        "Token image registered: {} ({} bytes)",
                        virtual_path,
                        bytes.len()
                    );
                    Some(format!(
                        "#image(\"{}\", width: {}pt, height: {}pt)",
                        virtual_path,
                        size_pt,
                        size_pt
                    ))
                } else {
                    None
                }
            } else {
                tracing::debug!("Token image not found: {}", full_path.display());
                None
            }
        } else {
            None
        };

        // Fallback to colored circle with initial
        let content = image_content.unwrap_or_else(|| {
            let color = token_type_color(&token.token_type);
            let initial = token.name.chars().next().unwrap_or('?');
            format!(
                "#circle(radius: {}pt, fill: rgb(\"{}\"), stroke: 1pt)[#align(center + horizon)[#text(fill: white, weight: \"bold\")[{}]]]",
                size_pt / 2.0, color, initial
            )
        });

        // Token with name below and fold line
        Ok(format!(
            r#"box(width: {}pt)[
  #align(center)[
    #box(stroke: (dash: "dashed"))[{}]
    #v(-1pt)
    #line(length: {}pt, stroke: 1pt)
    #v(2pt)
    #text(size: 7pt)[{}]
  ]
]"#,
            size_pt + 4.0,
            content,
            size_pt,
            truncate_name(&token.name, 12)
        ))
    }

    /// Render a large token (2x2 or bigger)
    fn render_large_token(&self, token: &RenderToken, ctx: &RenderContext) -> Result<String> {
        let mult = Self::size_multiplier(&token.size);
        let size_inches = self.cell_size * mult;
        let token_content = self.render_single_token(token, ctx, size_inches)?;

        Ok(format!(
            "#align(center)[{}]\n#v(0.5em)\n",
            token_content
        ))
    }
}

/// Get color for token type
fn token_type_color(token_type: &str) -> &'static str {
    match token_type.to_lowercase().as_str() {
        "monster" => "#dc3545",
        "pc" => "#28a745",
        "npc" => "#007bff",
        "trap" => "#ffc107",
        _ => "#6c757d",
    }
}

/// Sanitize a string for use as filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_multiplier() {
        assert_eq!(TokenCutoutSheet::size_multiplier("medium"), 1.0);
        assert_eq!(TokenCutoutSheet::size_multiplier("Large"), 2.0);
        assert_eq!(TokenCutoutSheet::size_multiplier("tiny"), 0.5);
    }

    #[test]
    fn test_truncate_name() {
        assert_eq!(truncate_name("Goblin", 12), "Goblin");
        assert_eq!(truncate_name("Ancient Red Dragon", 12), "Ancient R...");
    }
}

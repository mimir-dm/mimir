//! Map rendering for PDF export
//!
//! Renders map images with grid overlays and optional tokens for print output.

use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_hollow_rect_mut, draw_line_segment_mut};
use imageproc::rect::Rect;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{PrintError, Result};

/// Token data for rendering on maps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderToken {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub size: String,
    pub color: Option<String>,
    pub token_type: String,
}

impl RenderToken {
    /// Get grid squares for this token size
    fn grid_squares(&self) -> f32 {
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

    /// Get default color based on token type
    fn default_color(&self) -> Rgba<u8> {
        match self.token_type.to_lowercase().as_str() {
            "monster" => Rgba([220, 53, 69, 255]),   // Red
            "pc" => Rgba([40, 167, 69, 255]),        // Green
            "npc" => Rgba([0, 123, 255, 255]),       // Blue
            "trap" => Rgba([255, 193, 7, 255]),      // Yellow
            "marker" => Rgba([108, 117, 125, 255]), // Gray
            _ => Rgba([128, 128, 128, 255]),
        }
    }

    /// Parse hex color or use default
    fn get_color(&self) -> Rgba<u8> {
        if let Some(ref hex) = self.color {
            parse_hex_color(hex).unwrap_or_else(|| self.default_color())
        } else {
            self.default_color()
        }
    }
}

/// Map data for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMap {
    pub name: String,
    pub image_path: String,
    pub width_px: i32,
    pub height_px: i32,
    pub grid_type: String,
    pub grid_size_px: Option<i32>,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
}

impl RenderMap {
    /// Check if this map has a grid configured
    pub fn has_grid(&self) -> bool {
        self.grid_size_px.is_some() && self.grid_type != "none"
    }
}

/// Rendered map output
#[derive(Debug)]
pub struct RenderedMap {
    /// Map name
    pub name: String,
    /// PNG bytes of map with grid only
    pub with_grid: Vec<u8>,
    /// PNG bytes of map with grid and tokens (if tokens exist)
    pub with_tokens: Option<Vec<u8>>,
}

/// Parse a hex color string to Rgba
fn parse_hex_color(hex: &str) -> Option<Rgba<u8>> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Rgba([r, g, b, 255]))
    } else {
        None
    }
}

/// Render a map with grid overlay
pub fn render_map_with_grid(
    map: &RenderMap,
    base_path: &Path,
) -> Result<RgbaImage> {
    // Load the base image
    let image_path = base_path.join(&map.image_path);
    let img = image::open(&image_path).map_err(|e| {
        PrintError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to load map image {:?}: {}", image_path, e),
        ))
    })?;

    let mut img = img.to_rgba8();

    // Draw grid if configured
    if map.has_grid() {
        draw_grid(&mut img, map);
    }

    Ok(img)
}

/// Draw grid lines on an image
fn draw_grid(img: &mut RgbaImage, map: &RenderMap) {
    let grid_size = map.grid_size_px.unwrap_or(50) as f32;
    let offset_x = map.grid_offset_x as f32;
    let offset_y = map.grid_offset_y as f32;
    let width = img.width() as f32;
    let height = img.height() as f32;

    // Grid line color - semi-transparent dark
    let grid_color = Rgba([0, 0, 0, 100]);

    // Draw vertical lines
    let mut x = offset_x;
    while x < width {
        if x >= 0.0 {
            draw_line_segment_mut(
                img,
                (x, 0.0),
                (x, height),
                grid_color,
            );
        }
        x += grid_size;
    }

    // Draw horizontal lines
    let mut y = offset_y;
    while y < height {
        if y >= 0.0 {
            draw_line_segment_mut(
                img,
                (0.0, y),
                (width, y),
                grid_color,
            );
        }
        y += grid_size;
    }
}

/// Draw tokens on an image
fn draw_tokens(img: &mut RgbaImage, tokens: &[RenderToken], grid_size_px: i32) {
    let grid_size = grid_size_px as f32;
    // Token visual scale factor (85% of grid cell like frontend)
    let token_scale = 0.85;

    for token in tokens {
        let token_grid_squares = token.grid_squares();
        let token_size_px = (token_grid_squares * grid_size * token_scale) as i32;
        let radius = token_size_px / 2;

        // Token center position
        let center_x = token.x as i32;
        let center_y = token.y as i32;

        let color = token.get_color();

        // Draw filled circle for token
        draw_filled_circle_mut(img, (center_x, center_y), radius, color);

        // Draw border (slightly darker)
        let border_color = Rgba([
            (color[0] as i32 - 30).max(0) as u8,
            (color[1] as i32 - 30).max(0) as u8,
            (color[2] as i32 - 30).max(0) as u8,
            255,
        ]);

        // Draw border as a slightly larger circle outline
        // imageproc doesn't have draw_hollow_circle, so we approximate with a rect
        let rect = Rect::at(center_x - radius, center_y - radius)
            .of_size(token_size_px as u32, token_size_px as u32);
        draw_hollow_rect_mut(img, rect, border_color);

        // TODO: Add text labels using rusttype
        // For now, tokens are just colored circles
    }
}

/// Render a map with optional tokens
///
/// Returns two images:
/// 1. Map with grid overlay
/// 2. Map with grid and tokens (if tokens exist)
pub fn render_map(
    map: &RenderMap,
    tokens: &[RenderToken],
    base_path: &Path,
) -> Result<RenderedMap> {
    // Render map with grid
    let img_with_grid = render_map_with_grid(map, base_path)?;

    // Encode to PNG
    let mut grid_bytes: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut grid_bytes);
    img_with_grid
        .write_with_encoder(encoder)
        .map_err(|e| PrintError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to encode map image: {}", e),
        )))?;

    // If tokens exist, render version with tokens
    let with_tokens = if !tokens.is_empty() && map.grid_size_px.is_some() {
        let mut img_with_tokens = img_with_grid.clone();
        draw_tokens(&mut img_with_tokens, tokens, map.grid_size_px.unwrap_or(50));

        let mut token_bytes: Vec<u8> = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut token_bytes);
        img_with_tokens
            .write_with_encoder(encoder)
            .map_err(|e| PrintError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to encode map image with tokens: {}", e),
            )))?;

        Some(token_bytes)
    } else {
        None
    };

    Ok(RenderedMap {
        name: map.name.clone(),
        with_grid: grid_bytes,
        with_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ff0000"), Some(Rgba([255, 0, 0, 255])));
        assert_eq!(parse_hex_color("00ff00"), Some(Rgba([0, 255, 0, 255])));
        assert_eq!(parse_hex_color("#0000ff"), Some(Rgba([0, 0, 255, 255])));
        assert_eq!(parse_hex_color("invalid"), None);
    }

    #[test]
    fn test_token_grid_squares() {
        let token = RenderToken {
            name: "Test".to_string(),
            x: 0.0,
            y: 0.0,
            size: "medium".to_string(),
            color: None,
            token_type: "monster".to_string(),
        };
        assert_eq!(token.grid_squares(), 1.0);

        let large = RenderToken {
            size: "large".to_string(),
            ..token.clone()
        };
        assert_eq!(large.grid_squares(), 2.0);
    }
}

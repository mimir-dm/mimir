//! Map sections for PDF export
//!
//! Provides two construction approaches:
//! 1. From map data (RenderMap) - loads and renders the map image
//! 2. From pre-rendered bytes - uses already-processed image data

use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD, Engine};

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;
use crate::map_renderer::{
    load_image_from_file, render_map_for_print, MapPrintOptions, RenderMap, RenderToken,
};

/// Map preview section - renders a map fit to a single page
pub struct MapPreview {
    name: String,
    source: MapPreviewSource,
}

enum MapPreviewSource {
    /// Render from map data
    FromMap {
        map: RenderMap,
        tokens: Vec<RenderToken>,
        options: MapPrintOptions,
        base_path: PathBuf,
    },
    /// Use pre-rendered image bytes
    PreRendered { image_bytes: Vec<u8> },
}

impl MapPreview {
    /// Create a new map preview section from map data
    pub fn new(map: RenderMap, tokens: Vec<RenderToken>, base_path: PathBuf) -> Self {
        let name = map.name.clone();
        Self {
            name,
            source: MapPreviewSource::FromMap {
                map,
                tokens,
                options: MapPrintOptions {
                    show_grid: true,
                    ..Default::default()
                },
                base_path,
            },
        }
    }

    /// Create from pre-rendered image bytes
    pub fn from_rendered(name: String, image_bytes: Vec<u8>) -> Self {
        Self {
            name,
            source: MapPreviewSource::PreRendered { image_bytes },
        }
    }

    /// Set rendering options (only applies to FromMap source)
    pub fn with_options(mut self, options: MapPrintOptions) -> Self {
        if let MapPreviewSource::FromMap {
            options: ref mut opts,
            ..
        } = self.source
        {
            *opts = options;
        }
        self
    }
}

impl Renderable for MapPreview {
    fn to_typst(&self, ctx: &RenderContext) -> Result<String> {
        let image_filename = format!("map_preview_{}.png", sanitize_filename(&self.name));

        let image_bytes = match &self.source {
            MapPreviewSource::FromMap {
                map,
                tokens,
                options,
                base_path,
            } => {
                // Load the map image and convert to base64
                let file_path = base_path.join(&map.image_path);
                tracing::debug!("Loading map image from: {}", file_path.display());
                let raw_bytes = load_image_from_file(&file_path)?;
                let image_base64 = STANDARD.encode(&raw_bytes);

                // Use unified render function
                let rendered = render_map_for_print(map, tokens, base_path, &image_base64, options)?;
                tracing::debug!("Rendered map preview: {} bytes", rendered.image_bytes.len());
                rendered.image_bytes
            }
            MapPreviewSource::PreRendered { image_bytes } => {
                tracing::debug!("Using pre-rendered map: {} bytes", image_bytes.len());
                image_bytes.clone()
            }
        };

        // Register image in virtual file system
        let virtual_path = ctx.virtual_files.register(&image_filename, image_bytes);
        tracing::debug!("Registered map preview as: {}", virtual_path);

        // Use heading(outlined: true) so it appears in TOC, wrapped in block to keep together
        let typst = format!(
            r#"#block(breakable: false)[
  #heading(level: 1, outlined: true)[Map: {}]
  #align(center)[
    #image("{}", width: 100%, height: 100%, fit: "contain")
  ]
]
"#,
            escape_typst_string(&self.name), virtual_path
        );
        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        // Return None - we render our own heading in to_typst() to keep it with the image
        None
    }
}

/// Pre-sliced tile data for tiled map printing
#[derive(Debug, Clone)]
pub struct TileData {
    /// PNG image bytes for this tile
    pub image_bytes: Vec<u8>,
    /// Tile label (e.g., "A1", "B2")
    pub label: String,
    /// Row index (0-based)
    pub row: u32,
    /// Column index (0-based)
    pub col: u32,
    /// Tile width in pixels
    pub width_px: u32,
    /// Tile height in pixels
    pub height_px: u32,
}

/// Tiled map section - renders a map at true scale across multiple pages
pub struct TiledMapSection {
    name: String,
    source: TiledMapSource,
    /// Page margin in inches (smaller = more drawing area)
    margin: f32,
}

enum TiledMapSource {
    /// Render from map data (does its own tiling)
    FromMap {
        map: RenderMap,
        tokens: Vec<RenderToken>,
        options: MapPrintOptions,
        base_path: PathBuf,
    },
    /// Use pre-rendered image bytes with grid info
    PreRendered {
        image_bytes: Vec<u8>,
        grid_size_px: i32,
    },
    /// Use pre-sliced tiles
    PreSliced {
        tiles: Vec<TileData>,
        tiles_x: u32,
        tiles_y: u32,
    },
}

/// Default margin for tiled maps (small for maximum drawing area)
const TILED_MAP_DEFAULT_MARGIN: f32 = 0.25;

impl TiledMapSection {
    /// Create a new tiled map section from map data
    pub fn new(map: RenderMap, tokens: Vec<RenderToken>, base_path: PathBuf) -> Self {
        let name = map.name.clone();
        Self {
            name,
            source: TiledMapSource::FromMap {
                map,
                tokens,
                options: MapPrintOptions {
                    show_grid: true,
                    show_positions: false, // No positions on play tiles
                    ..Default::default()
                },
                base_path,
            },
            margin: TILED_MAP_DEFAULT_MARGIN,
        }
    }

    /// Create from pre-sliced tile data
    pub fn from_tiles(name: String, tiles: Vec<TileData>, tiles_x: u32, tiles_y: u32) -> Self {
        Self {
            name,
            source: TiledMapSource::PreSliced {
                tiles,
                tiles_x,
                tiles_y,
            },
            margin: TILED_MAP_DEFAULT_MARGIN,
        }
    }

    /// Create from pre-rendered image bytes with grid information
    ///
    /// Used when the image is already loaded (e.g., from UVTT base64).
    /// The actual image dimensions are read from the image bytes.
    pub fn from_rendered(name: String, image_bytes: Vec<u8>, grid_size_px: i32) -> Self {
        Self {
            name,
            source: TiledMapSource::PreRendered {
                image_bytes,
                grid_size_px,
            },
            margin: TILED_MAP_DEFAULT_MARGIN,
        }
    }

    /// Set custom margin (in inches)
    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    /// Set rendering options (only applies to FromMap source)
    pub fn with_options(mut self, options: MapPrintOptions) -> Self {
        if let TiledMapSource::FromMap {
            options: ref mut opts,
            ..
        } = self.source
        {
            *opts = options;
        }
        self
    }

    /// Render assembly guide page
    fn render_assembly_guide(&self, cols: u32, rows: u32) -> String {
        let mut typst = String::new();

        typst.push_str(&format!(
            r#"#align(center)[
  #text(size: 14pt, weight: "bold")[{} - Assembly Guide]
]
#v(1em)
#align(center)[
  This map tiles across {} pages ({} columns × {} rows).
  Print all pages and align using the tile labels.
]
#v(1em)
#align(center)[
  #grid(
    columns: {},
    gutter: 2pt,
"#,
            self.name,
            cols * rows,
            cols,
            rows,
            cols
        ));

        // Draw assembly diagram with labels
        for r in 0..rows {
            for c in 0..cols {
                let row_label = (b'A' + r as u8) as char;
                let col_label = c + 1;
                typst.push_str(&format!(
                    "    rect(width: 30pt, height: 30pt, stroke: 1pt)[#align(center + horizon)[{}{}]],\n",
                    row_label, col_label
                ));
            }
        }
        typst.push_str("  )\n]\n");

        typst
    }
}

impl Renderable for TiledMapSection {
    fn to_typst(&self, ctx: &RenderContext) -> Result<String> {
        match &self.source {
            TiledMapSource::FromMap { map, tokens: _, options, base_path } => {
                // Load the map image and convert to base64
                let file_path = base_path.join(&map.image_path);
                let image_bytes = load_image_from_file(&file_path)?;
                let image_base64 = STANDARD.encode(&image_bytes);

                // Use unified render function (no tokens for play tiles - use physical tokens)
                let rendered = render_map_for_print(map, &[], base_path, &image_base64, options)?;

                let image_filename = format!("map_tiled_{}.png", sanitize_filename(&self.name));
                let (width_px, height_px) = (rendered.width_px, rendered.height_px);

                // Register image in virtual file system
                let virtual_path = ctx.virtual_files.register(&image_filename, rendered.image_bytes);
                tracing::debug!("Registered tiled map as: {}", virtual_path);

                // Calculate dimensions for true scale
                let grid_px = map.grid_size_px.unwrap_or(70) as f64;
                let width_inches = width_px as f64 / grid_px;
                let height_inches = height_px as f64 / grid_px;

                // Page dimensions (letter size minus margins, minus space for label)
                let margin = self.margin as f64;
                let page_width = 8.5 - (2.0 * margin);
                let page_height = 11.0 - (2.0 * margin) - 0.5; // Leave 0.5in for tile label

                let cols = (width_inches / page_width).ceil() as u32;
                let rows = (height_inches / page_height).ceil() as u32;

                let mut typst = self.render_assembly_guide(cols, rows);

                // Generate a page for each tile
                for r in 0..rows {
                    for c in 0..cols {
                        let row_label = (b'A' + r as u8) as char;
                        let col_label = c + 1;
                        let x_offset = c as f64 * page_width;
                        let y_offset = r as f64 * page_height;

                        // Use block(breakable: false) to keep label and image together
                        typst.push_str("\n#pagebreak()\n");
                        typst.push_str(&format!(
                            r#"#block(breakable: false)[
  #align(center)[
    #text(size: 10pt)[Tile {}{} | Row {}, Column {}]
  ]
  #v(0.25em)
  #box(clip: true, width: {}in, height: {}in)[
    #move(dx: -{}in, dy: -{}in)[
      #image("{}", width: {}in)
    ]
  ]
]
"#,
                            row_label,
                            col_label,
                            r + 1,
                            c + 1,
                            page_width,
                            page_height,
                            x_offset,
                            y_offset,
                            virtual_path,
                            width_inches
                        ));
                    }
                }

                Ok(typst)
            }

            TiledMapSource::PreRendered {
                image_bytes,
                grid_size_px,
            } => {
                use image::GenericImageView;

                // Load the image to get its actual dimensions
                let img = image::load_from_memory(image_bytes).map_err(|e| {
                    crate::error::PrintError::IoError(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to load pre-rendered image: {}", e),
                    ))
                })?;
                let actual_width = img.width();
                let actual_height = img.height();

                // Calculate dimensions for true scale using actual image dimensions
                let grid_px = *grid_size_px as f64;
                let width_inches = actual_width as f64 / grid_px;
                let height_inches = actual_height as f64 / grid_px;

                // Page dimensions (letter size minus margins, minus space for label)
                let margin = self.margin as f64;
                let page_width = 8.5 - (2.0 * margin);
                let page_height = 11.0 - (2.0 * margin) - 0.5; // Leave 0.5in for tile label

                // Tile size in pixels (each tile fills one page at 1 grid = 1 inch)
                let tile_width_px = (page_width * grid_px) as u32;
                let tile_height_px = (page_height * grid_px) as u32;

                let cols = (actual_width as f64 / tile_width_px as f64).ceil() as u32;
                let rows = (actual_height as f64 / tile_height_px as f64).ceil() as u32;

                tracing::info!(
                    "TiledMapSection '{}': image={}x{}px, grid={}px, size={}x{}in, tile={}x{}px, tiles={}cols x {}rows = {} pages",
                    self.name, actual_width, actual_height, grid_px,
                    width_inches, height_inches, tile_width_px, tile_height_px,
                    cols, rows, cols * rows
                );

                let mut typst = self.render_assembly_guide(cols, rows);

                // Pre-slice the image into tiles and register each one
                for r in 0..rows {
                    for c in 0..cols {
                        let row_label = (b'A' + r as u8) as char;
                        let col_label = c + 1;
                        let tile_label = format!("{}{}", row_label, col_label);

                        // Calculate crop region (in pixels)
                        let x_start = c * tile_width_px;
                        let y_start = r * tile_height_px;
                        let crop_width = (tile_width_px).min(actual_width.saturating_sub(x_start));
                        let crop_height = (tile_height_px).min(actual_height.saturating_sub(y_start));

                        // Skip empty tiles
                        if crop_width == 0 || crop_height == 0 {
                            continue;
                        }

                        // Crop the tile from the image
                        let tile_img = img.crop_imm(x_start, y_start, crop_width, crop_height);

                        // Encode tile as PNG
                        let mut tile_bytes = Vec::new();
                        tile_img
                            .write_to(
                                &mut std::io::Cursor::new(&mut tile_bytes),
                                image::ImageFormat::Png,
                            )
                            .map_err(|e| {
                                crate::error::PrintError::IoError(std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    format!("Failed to encode tile {}: {}", tile_label, e),
                                ))
                            })?;

                        // Register tile image
                        let tile_filename = format!(
                            "tile_{}_{}.png",
                            sanitize_filename(&self.name),
                            tile_label
                        );
                        let virtual_path = ctx.virtual_files.register(&tile_filename, tile_bytes);

                        // Calculate tile size in inches for display
                        let tile_width_in = crop_width as f64 / grid_px;
                        let tile_height_in = crop_height as f64 / grid_px;

                        // Generate the tile page
                        typst.push_str("\n#pagebreak()\n");
                        typst.push_str(&format!(
                            r#"#block(breakable: false)[
  #align(center)[
    #text(size: 10pt)[Tile {} | Row {}, Column {}]
  ]
  #v(0.25em)
  #align(center)[
    #image("{}", width: {}in, height: {}in)
  ]
]
"#,
                            tile_label,
                            r + 1,
                            c + 1,
                            virtual_path,
                            tile_width_in,
                            tile_height_in
                        ));
                    }
                }

                Ok(typst)
            }

            TiledMapSource::PreSliced {
                tiles,
                tiles_x,
                tiles_y,
            } => {
                let mut typst = self.render_assembly_guide(*tiles_x, *tiles_y);

                // Each tile is already cropped - just render at 1:1 scale (1 grid = 1 inch)
                for tile in tiles {
                    // Register tile image in virtual file system
                    let tile_filename =
                        format!("tile_{}_{}.png", sanitize_filename(&self.name), tile.label);
                    let virtual_path = ctx.virtual_files.register(&tile_filename, tile.image_bytes.clone());

                    // Use block(breakable: false) to keep label and image together
                    typst.push_str("\n#pagebreak()\n");
                    typst.push_str(&format!(
                        r#"#block(breakable: false)[
  #align(center)[
    #text(size: 10pt)[Tile {} | Row {}, Column {}]
  ]
  #v(0.25em)
  #align(center)[
    #image("{}", width: auto, height: auto)
  ]
]
"#,
                        tile.label,
                        tile.row + 1,
                        tile.col + 1,
                        virtual_path
                    ));
                }

                Ok(typst)
            }
        }
    }

    fn toc_title(&self) -> Option<String> {
        Some(format!("Tiled Map: {}", self.name))
    }

    fn page_break_before(&self) -> bool {
        true
    }

    fn page_margin(&self) -> Option<f32> {
        Some(self.margin)
    }
}

/// Sanitize a filename for use in paths
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
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
    use crate::builder::RenderContext;

    fn test_map() -> RenderMap {
        RenderMap {
            name: "Test Map".to_string(),
            image_path: "test.png".to_string(),
            width_px: 1400,
            height_px: 1400,
            grid_type: "square".to_string(),
            grid_size_px: Some(70),
            grid_offset_x: 0,
            grid_offset_y: 0,
        }
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Test Map"), "Test_Map");
        assert_eq!(sanitize_filename("map-1_final"), "map-1_final");
        assert_eq!(sanitize_filename("map/with:bad*chars"), "map_with_bad_chars");
    }

    #[test]
    fn test_map_preview_toc_title() {
        let map = test_map();
        let preview = MapPreview::new(map, vec![], PathBuf::new());
        // toc_title returns None because MapPreview renders its own heading
        // in to_typst() to keep the heading and image together on one page
        assert_eq!(preview.toc_title(), None);
    }

    /// Create a minimal PNG image for testing
    fn create_test_tile_image() -> Vec<u8> {
        use image::{Rgba, RgbaImage};
        let mut img = RgbaImage::new(100, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([128, 128, 128, 255]);
        }
        let mut bytes = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("Failed to encode test image");
        bytes
    }

    #[test]
    fn test_tiled_map_one_page_per_tile() {
        // Create a 2x2 grid of tiles (4 tiles total)
        let tiles = vec![
            TileData {
                image_bytes: create_test_tile_image(),
                label: "A1".to_string(),
                row: 0,
                col: 0,
                width_px: 100,
                height_px: 100,
            },
            TileData {
                image_bytes: create_test_tile_image(),
                label: "A2".to_string(),
                row: 0,
                col: 1,
                width_px: 100,
                height_px: 100,
            },
            TileData {
                image_bytes: create_test_tile_image(),
                label: "B1".to_string(),
                row: 1,
                col: 0,
                width_px: 100,
                height_px: 100,
            },
            TileData {
                image_bytes: create_test_tile_image(),
                label: "B2".to_string(),
                row: 1,
                col: 1,
                width_px: 100,
                height_px: 100,
            },
        ];

        let section = TiledMapSection::from_tiles("Test Map".to_string(), tiles, 2, 2);

        let temp_dir = std::env::temp_dir().join("mimir-test-tiled");
        std::fs::create_dir_all(&temp_dir).ok();
        let ctx = RenderContext::new(temp_dir);

        let typst = section.to_typst(&ctx).expect("Failed to render tiled map");

        // Count pagebreaks: 1 for assembly guide + 4 for tiles = 5 total
        // But assembly guide doesn't have a pagebreak before it, so tiles add 4 pagebreaks
        let pagebreak_count = typst.matches("#pagebreak()").count();
        assert_eq!(
            pagebreak_count, 4,
            "Expected 4 pagebreaks (one per tile), got {}. Each tile should be on its own page.",
            pagebreak_count
        );

        // Verify each tile label appears exactly once
        assert!(typst.contains("Tile A1"), "Missing tile A1");
        assert!(typst.contains("Tile A2"), "Missing tile A2");
        assert!(typst.contains("Tile B1"), "Missing tile B1");
        assert!(typst.contains("Tile B2"), "Missing tile B2");
    }
}

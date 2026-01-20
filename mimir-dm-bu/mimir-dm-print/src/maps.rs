//! Map PDF generation
//!
//! Core functions for generating map PDFs, separated from Tauri command handling.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::builder::DocumentBuilder;
use crate::error::{PrintError, Result};
use crate::map_renderer::{render_map_for_print, MapPrintOptions, RenderMap, RenderToken, RenderedMapForPrint};
use crate::sections::{MapPreview, TileData, TiledMapSection, TokenCutoutSheet};

/// Options for map PDF generation
///
/// Both preview and play sections can be included in a single PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPdfOptions {
    // Preview section options
    /// Include preview page (fit to single page)
    #[serde(default = "default_true")]
    pub include_preview: bool,
    /// Show grid overlay on preview
    #[serde(default)]
    pub preview_grid: bool,
    /// Show LOS wall segments on preview
    #[serde(default)]
    pub preview_los_walls: bool,
    /// Show position markers on preview
    #[serde(default)]
    pub preview_positions: bool,

    // Play section options
    /// Include play tiles (1"=5ft scale)
    #[serde(default)]
    pub include_play: bool,
    /// Show grid overlay on tiles
    #[serde(default)]
    pub play_grid: bool,
    /// Show LOS wall segments on tiles
    #[serde(default)]
    pub play_los_walls: bool,
    /// Include token cutout sheets
    #[serde(default)]
    pub play_cutouts: bool,

    // Shared data (set by backend)
    /// LOS wall segments as polylines (grid coordinates)
    #[serde(default)]
    pub los_walls: Vec<Vec<(f64, f64)>>,
    /// Pixels per grid cell (from UVTT resolution)
    #[serde(default)]
    pub pixels_per_grid: u32,
}

fn default_true() -> bool {
    true
}

impl Default for MapPdfOptions {
    fn default() -> Self {
        Self {
            include_preview: true, // Match serde default
            preview_grid: false,
            preview_los_walls: false,
            preview_positions: false,
            include_play: false,
            play_grid: false,
            play_los_walls: false,
            play_cutouts: false,
            los_walls: Vec::new(),
            pixels_per_grid: 0,
        }
    }
}

/// Generate a map PDF from map data
///
/// This is the core function for map PDF generation, separated from
/// Tauri command handling to enable testing and reuse.
///
/// Both preview and play sections can be included in a single PDF.
///
/// # Arguments
/// * `map` - Map metadata
/// * `tokens` - Tokens to render on preview
/// * `image_base64` - Base64-encoded map image (from UVTT)
/// * `options` - PDF generation options
/// * `_token_images_base` - Base path for resolving token image paths (unused currently)
/// * `templates_root` - Path to Typst templates
pub fn generate_map_pdf(
    map: &RenderMap,
    tokens: &[RenderToken],
    image_base64: &str,
    options: &MapPdfOptions,
    _token_images_base: PathBuf,
    templates_root: PathBuf,
) -> Result<Vec<u8>> {
    let mut builder = DocumentBuilder::new(&map.name).with_templates_root(templates_root);

    // Add preview section if requested
    if options.include_preview {
        let preview_options = MapPrintOptions {
            show_grid: options.preview_grid,
            show_los_walls: options.preview_los_walls,
            show_positions: false, // Positions shown on separate page
            los_walls: options.los_walls.clone(),
            pixels_per_grid: options.pixels_per_grid,
        };

        let rendered = render_map_for_print(
            map,
            tokens,
            &PathBuf::new(),
            image_base64,
            &preview_options,
        )?;

        builder = builder.append(MapPreview::from_rendered(
            map.name.clone(),
            rendered.image_bytes,
        ));
    }

    // Add starting positions map if requested (separate page)
    if options.preview_positions && !tokens.is_empty() {
        let positions_options = MapPrintOptions {
            show_grid: options.preview_grid,
            show_los_walls: options.preview_los_walls,
            show_positions: true,
            los_walls: options.los_walls.clone(),
            pixels_per_grid: options.pixels_per_grid,
        };

        let rendered = render_map_for_print(
            map,
            tokens,
            &PathBuf::new(),
            image_base64,
            &positions_options,
        )?;

        builder = builder.append(MapPreview::from_rendered(
            format!("{} - Starting Positions", map.name),
            rendered.image_bytes,
        ));
    }

    // Add play tiles section if requested
    if options.include_play {
        let play_options = MapPrintOptions {
            show_grid: options.play_grid,
            show_los_walls: options.play_los_walls,
            show_positions: false, // No position markers on play tiles
            los_walls: options.los_walls.clone(),
            pixels_per_grid: options.pixels_per_grid,
        };

        // Render without tokens (physical tokens used during play)
        let rendered = render_map_for_print(
            map,
            &[],
            &PathBuf::new(),
            image_base64,
            &play_options,
        )?;

        // Slice into tiles
        let (tiles, tiles_x, tiles_y) = slice_map_into_tiles(&rendered, options.pixels_per_grid)?;

        builder = builder.append(TiledMapSection::from_tiles(
            map.name.clone(),
            tiles,
            tiles_x,
            tiles_y,
        ));

        // Add token cutouts if requested (and we have tokens)
        if options.play_cutouts && !tokens.is_empty() {
            builder = builder.append(TokenCutoutSheet::new(
                tokens.to_vec(),
                _token_images_base,
            ));
        }
    }

    builder.to_pdf()
}

/// Slice a rendered map into tiles for play mode
///
/// Tiles are sized for letter paper at 1" = 5ft (1 grid square = 1 inch).
pub fn slice_map_into_tiles(
    rendered: &RenderedMapForPrint,
    pixels_per_grid: u32,
) -> Result<(Vec<TileData>, u32, u32)> {
    let ppg = pixels_per_grid as f64;

    // Calculate grid dimensions
    let grid_width = (rendered.width_px as f64 / ppg).ceil() as u32;
    let grid_height = (rendered.height_px as f64 / ppg).ceil() as u32;

    // Printable area in grid squares (with margin for labels)
    // Letter landscape: ~10" x 7.5" printable, use 9x6 for safety
    let tile_grid_width: u32 = 9;
    let tile_grid_height: u32 = 6;

    // Calculate number of tiles needed
    let tiles_x = ((grid_width as f64) / (tile_grid_width as f64)).ceil() as u32;
    let tiles_y = ((grid_height as f64) / (tile_grid_height as f64)).ceil() as u32;

    // Tile size in pixels
    let tile_px_width = tile_grid_width * pixels_per_grid;
    let tile_px_height = tile_grid_height * pixels_per_grid;

    // Load the rendered image
    let img = image::load_from_memory(&rendered.image_bytes).map_err(|e| {
        PrintError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to load rendered image: {}", e),
        ))
    })?;

    let mut tiles = Vec::new();

    for ty in 0..tiles_y {
        for tx in 0..tiles_x {
            let x = tx * tile_px_width;
            let y = ty * tile_px_height;

            // Calculate actual tile dimensions (may be smaller at edges)
            let w = std::cmp::min(tile_px_width, rendered.width_px.saturating_sub(x));
            let h = std::cmp::min(tile_px_height, rendered.height_px.saturating_sub(y));

            if w == 0 || h == 0 {
                continue;
            }

            // Crop the tile
            let tile_img = img.crop_imm(x, y, w, h);

            // Encode to PNG
            let mut tile_bytes: Vec<u8> = Vec::new();
            tile_img
                .write_to(
                    &mut std::io::Cursor::new(&mut tile_bytes),
                    image::ImageFormat::Png,
                )
                .map_err(|e| {
                    PrintError::IoError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to encode tile: {}", e),
                    ))
                })?;

            // Generate tile label (A1, A2, B1, B2, etc.)
            let row_label = (b'A' + ty as u8) as char;
            let col_label = tx + 1;
            let label = format!("{}{}", row_label, col_label);

            tiles.push(TileData {
                image_bytes: tile_bytes,
                label,
                row: ty,
                col: tx,
                width_px: w,
                height_px: h,
            });
        }
    }

    Ok((tiles, tiles_x, tiles_y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};
    use std::path::PathBuf;

    /// Create a minimal valid PNG image for testing
    /// Returns base64-encoded PNG bytes
    fn create_test_image_base64(width: u32, height: u32) -> String {
        use image::{Rgba, RgbaImage};

        let mut img = RgbaImage::new(width, height);

        // Fill with a simple gradient pattern
        for y in 0..height {
            for x in 0..width {
                let r = ((x * 255) / width) as u8;
                let g = ((y * 255) / height) as u8;
                let b = 128;
                img.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }

        // Encode to PNG
        let mut png_bytes: Vec<u8> = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .expect("Failed to encode test image");

        STANDARD.encode(&png_bytes)
    }

    /// Create a sample RenderMap for testing
    fn sample_map() -> RenderMap {
        RenderMap {
            name: "Test Dungeon".to_string(),
            image_path: "test.png".to_string(),
            width_px: 540,  // 10 grid squares at 54px
            height_px: 324, // 6 grid squares at 54px
            grid_type: "square".to_string(),
            grid_size_px: Some(54),
            grid_offset_x: 0,
            grid_offset_y: 0,
        }
    }

    /// Create sample tokens for testing
    fn sample_tokens() -> Vec<RenderToken> {
        vec![
            RenderToken {
                name: "Goblin 1".to_string(),
                x: 81.0,  // 1.5 grid squares in
                y: 81.0,
                size: "small".to_string(),
                color: Some("#ff0000".to_string()),
                token_type: "monster".to_string(),
                image_path: None,
            },
            RenderToken {
                name: "Hero".to_string(),
                x: 270.0, // 5 grid squares in
                y: 162.0, // 3 grid squares in
                size: "medium".to_string(),
                color: Some("#00ff00".to_string()),
                token_type: "pc".to_string(),
                image_path: None,
            },
            RenderToken {
                name: "Ogre".to_string(),
                x: 432.0, // 8 grid squares in
                y: 216.0, // 4 grid squares in
                size: "large".to_string(),
                color: None, // Uses default monster color
                token_type: "monster".to_string(),
                image_path: None,
            },
        ]
    }

    fn templates_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
    }

    #[test]
    fn test_map_pdf_options_default() {
        let opts = MapPdfOptions::default();
        assert!(opts.include_preview);
        assert!(!opts.include_play);
        assert!(!opts.preview_grid);
    }

    #[test]
    fn test_generate_map_pdf_preview_only() {
        let map = sample_map();
        let tokens = sample_tokens();
        let image_base64 = create_test_image_base64(540, 324);
        let options = MapPdfOptions {
            include_preview: true,
            include_play: false,
            pixels_per_grid: 54,
            ..Default::default()
        };

        let result = generate_map_pdf(
            &map,
            &tokens,
            &image_base64,
            &options,
            PathBuf::new(),
            templates_path(),
        );

        assert!(result.is_ok(), "PDF generation failed: {:?}", result.err());

        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
        assert_eq!(
            &pdf_bytes[0..4],
            b"%PDF",
            "Output should be a valid PDF (starts with %PDF)"
        );
    }

    #[test]
    fn test_generate_map_pdf_play_only() {
        let map = sample_map();
        let tokens = sample_tokens();
        let image_base64 = create_test_image_base64(540, 324);
        let options = MapPdfOptions {
            include_preview: false,
            include_play: true,
            pixels_per_grid: 54,
            ..Default::default()
        };

        let result = generate_map_pdf(
            &map,
            &tokens,
            &image_base64,
            &options,
            PathBuf::new(),
            templates_path(),
        );

        assert!(result.is_ok(), "PDF generation failed: {:?}", result.err());

        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
        assert_eq!(
            &pdf_bytes[0..4],
            b"%PDF",
            "Output should be a valid PDF (starts with %PDF)"
        );
    }

    #[test]
    fn test_generate_map_pdf_both_sections() {
        let map = sample_map();
        let tokens = sample_tokens();
        let image_base64 = create_test_image_base64(540, 324);
        let options = MapPdfOptions {
            include_preview: true,
            include_play: true,
            pixels_per_grid: 54,
            ..Default::default()
        };

        let result = generate_map_pdf(
            &map,
            &tokens,
            &image_base64,
            &options,
            PathBuf::new(),
            templates_path(),
        );

        assert!(result.is_ok(), "PDF generation with both sections failed: {:?}", result.err());
    }

    #[test]
    fn test_generate_map_pdf_with_grid_overlay() {
        let map = sample_map();
        let image_base64 = create_test_image_base64(540, 324);
        let options = MapPdfOptions {
            include_preview: true,
            preview_grid: true,
            pixels_per_grid: 54,
            ..Default::default()
        };

        let result = generate_map_pdf(
            &map,
            &[],
            &image_base64,
            &options,
            PathBuf::new(),
            templates_path(),
        );

        assert!(result.is_ok(), "PDF generation with grid failed: {:?}", result.err());
    }

    #[test]
    fn test_generate_map_pdf_with_los_walls() {
        let map = sample_map();
        let image_base64 = create_test_image_base64(540, 324);

        // Simple L-shaped wall
        let los_walls = vec![vec![
            (2.0, 2.0),
            (2.0, 4.0),
            (5.0, 4.0),
        ]];

        let options = MapPdfOptions {
            include_preview: true,
            preview_los_walls: true,
            los_walls,
            pixels_per_grid: 54,
            ..Default::default()
        };

        let result = generate_map_pdf(
            &map,
            &[],
            &image_base64,
            &options,
            PathBuf::new(),
            templates_path(),
        );

        assert!(result.is_ok(), "PDF generation with LOS walls failed: {:?}", result.err());
    }

    #[test]
    fn test_generate_map_pdf_empty_tokens() {
        let map = sample_map();
        let image_base64 = create_test_image_base64(540, 324);
        let options = MapPdfOptions {
            include_preview: true,
            pixels_per_grid: 54,
            ..Default::default()
        };

        let result = generate_map_pdf(
            &map,
            &[],
            &image_base64,
            &options,
            PathBuf::new(),
            templates_path(),
        );

        assert!(result.is_ok(), "PDF generation with no tokens failed: {:?}", result.err());
    }

    #[test]
    fn test_slice_map_into_tiles_small_map() {
        // Small map that fits in one tile
        let image_base64 = create_test_image_base64(486, 324); // 9x6 grid at 54px = exactly one tile

        // Create a rendered map result
        let rendered = crate::map_renderer::RenderedMapForPrint {
            image_bytes: STANDARD.decode(&image_base64).unwrap(),
            width_px: 486,
            height_px: 324,
        };

        let result = slice_map_into_tiles(&rendered, 54);
        assert!(result.is_ok());

        let (tiles, tiles_x, tiles_y) = result.unwrap();
        assert_eq!(tiles_x, 1, "Small map should be 1 tile wide");
        assert_eq!(tiles_y, 1, "Small map should be 1 tile tall");
        assert_eq!(tiles.len(), 1, "Should have exactly 1 tile");
        assert_eq!(tiles[0].label, "A1", "First tile should be labeled A1");
    }

    #[test]
    fn test_slice_map_into_tiles_large_map() {
        // Larger map: 20x12 grid at 54px = 1080x648 pixels
        // Should need 3x2 tiles (9x6 per tile)
        let image_base64 = create_test_image_base64(1080, 648);

        let rendered = crate::map_renderer::RenderedMapForPrint {
            image_bytes: STANDARD.decode(&image_base64).unwrap(),
            width_px: 1080,
            height_px: 648,
        };

        let result = slice_map_into_tiles(&rendered, 54);
        assert!(result.is_ok());

        let (tiles, tiles_x, tiles_y) = result.unwrap();
        assert_eq!(tiles_x, 3, "20-grid wide map should be 3 tiles wide");
        assert_eq!(tiles_y, 2, "12-grid tall map should be 2 tiles tall");
        assert_eq!(tiles.len(), 6, "Should have 6 tiles total");

        // Check tile labels
        let labels: Vec<&str> = tiles.iter().map(|t| t.label.as_str()).collect();
        assert!(labels.contains(&"A1"));
        assert!(labels.contains(&"A2"));
        assert!(labels.contains(&"A3"));
        assert!(labels.contains(&"B1"));
        assert!(labels.contains(&"B2"));
        assert!(labels.contains(&"B3"));
    }

    #[test]
    fn test_slice_map_into_tiles_labels() {
        // 27x18 grid = 3x3 tiles
        let image_base64 = create_test_image_base64(1458, 972);

        let rendered = crate::map_renderer::RenderedMapForPrint {
            image_bytes: STANDARD.decode(&image_base64).unwrap(),
            width_px: 1458,
            height_px: 972,
        };

        let result = slice_map_into_tiles(&rendered, 54);
        assert!(result.is_ok());

        let (tiles, _, _) = result.unwrap();

        // Verify labeling pattern: A1, A2, A3, B1, B2, B3, C1, C2, C3
        let expected_labels = ["A1", "A2", "A3", "B1", "B2", "B3", "C1", "C2", "C3"];
        for (tile, expected) in tiles.iter().zip(expected_labels.iter()) {
            assert_eq!(tile.label, *expected, "Tile label mismatch");
        }
    }
}

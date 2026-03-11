//! Declarative Dungeondraft map generation.
//!
//! `mimir-mapgen` generates `.dungeondraft_map` files from YAML configuration,
//! using procedural generation algorithms (noise, Poisson Disc sampling,
//! Marching Squares, Bezier curves) to create natural-looking outdoor maps.
//!
//! This crate is standalone — it has no dependency on `mimir-core` or any
//! database layer.

pub mod assets;
pub mod biomes;
pub mod contour;
pub mod curves;
pub mod distribution;
pub mod elevation;
pub mod format;
pub mod noise_gen;
pub mod objects;
pub mod paths;
pub mod pipeline;
pub mod terrain;
pub mod water;

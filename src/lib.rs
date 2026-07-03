//! hypso — procedural generator for 4K topographic-style wallpapers.
//!
//! The library holds all logic and is what the tests cover; `main.rs` is thin
//! Slint GUI glue. The single source of truth is the SVG produced by
//! [`svg::emit`], rasterized identically for the live preview and the export.

pub mod config;
pub mod contour;
pub mod export;
pub mod format;
pub mod geom;
pub mod grain;
pub mod noise;
pub mod preset;
pub mod raster;
pub mod render;
pub mod smooth;
pub mod stroke;
pub mod svg;
pub mod text_zone;
pub mod util;

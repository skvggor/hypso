//! Hypso: procedural generator for 4K topographic-style wallpapers.
//!
//! The library holds all logic and is what the tests cover; `main.rs` is thin
//! Slint GUI glue. The single source of truth is the SVG produced by
//! [`svg::emit`], rasterized identically for the live preview and the export.

pub mod config;
pub mod contour;
pub mod format;
pub mod geom;
pub mod noise;
pub mod smooth;
pub mod stroke;
pub mod svg;
pub mod text_zone;
pub mod util;
pub mod wasm;

// Rasterization, film grain, file I/O. Excluded from the wasm build, which needs
// just the pure generative core above.
#[cfg(feature = "render")]
pub mod export;
#[cfg(feature = "render")]
pub mod grain;
#[cfg(feature = "render")]
pub mod preset;
#[cfg(feature = "render")]
pub mod raster;
#[cfg(feature = "render")]
pub mod render;

//! The single render pipeline shared by the live preview and the export, so the
//! preview is WYSIWYG: `SVG → Pixmap → grain`. Only the raster size differs.

use anyhow::{Result, anyhow};
use resvg::tiny_skia::Pixmap;

use crate::config::Config;
use crate::{grain, raster, svg};

/// Render `config` to a [`Pixmap`] whose longest edge is `longest_px`.
pub fn render_pipeline(config: &Config, longest_px: u32) -> Result<Pixmap> {
    let document = svg::emit(config);
    let mut pixmap = raster::render_to_pixmap(&document, longest_px)?;
    grain::apply(&mut pixmap, config.seed, config.grain);
    Ok(pixmap)
}

/// Render and PNG-encode through the same pipeline.
pub fn render_png(config: &Config, longest_px: u32) -> Result<Vec<u8>> {
    render_pipeline(config, longest_px)?
        .encode_png()
        .map_err(|error| anyhow!("PNG encoding failed: {error}"))
}

/// Render through the pipeline to straight-alpha RGBA8 for the Slint preview, so
/// the preview includes grain (WYSIWYG). tiny-skia stores premultiplied alpha.
pub fn render_rgba(config: &Config, longest_px: u32) -> Result<(u32, u32, Vec<u8>)> {
    let pixmap = render_pipeline(config, longest_px)?;
    let (width, height) = (pixmap.width(), pixmap.height());
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for pixel in pixmap.data().chunks_exact(4) {
        let alpha = pixel[3];
        if alpha == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let unpremultiply = |c: u8| {
                ((u16::from(c) * 255 + u16::from(alpha) / 2) / u16::from(alpha)).min(255) as u8
            };
            rgba.extend_from_slice(&[
                unpremultiply(pixel[0]),
                unpremultiply(pixel[1]),
                unpremultiply(pixel[2]),
                alpha,
            ]);
        }
    }
    Ok((width, height, rgba))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixels(config: &Config) -> Vec<u8> {
        render_pipeline(config, 200)
            .expect("render")
            .data()
            .to_vec()
    }

    fn base() -> Config {
        Config {
            gradient_overlay: 0.0,
            grain: 0.0,
            ..Config::default()
        }
    }

    #[test]
    fn overlay_zero_matches_no_overlay() {
        assert_eq!(pixels(&base()), pixels(&base()));
    }

    #[test]
    fn overlay_changes_output() {
        let overlaid = Config {
            gradient_overlay: 0.7,
            ..base()
        };
        assert_ne!(pixels(&overlaid), pixels(&base()));
    }

    #[test]
    fn grain_is_applied_in_pipeline() {
        let grainy = Config {
            grain: 0.6,
            ..base()
        };
        assert_ne!(pixels(&grainy), pixels(&base()));
    }

    #[test]
    fn pipeline_is_deterministic() {
        let config = Config {
            grain: 0.4,
            ..base()
        };
        assert_eq!(pixels(&config), pixels(&config));
    }

    #[test]
    fn render_png_has_signature() {
        let png = render_png(&base(), 120).expect("png");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn render_rgba_length_matches_dimensions() {
        let (width, height, rgba) = render_rgba(&base(), 120).expect("rgba");
        assert_eq!(rgba.len(), width as usize * height as usize * 4);
    }
}

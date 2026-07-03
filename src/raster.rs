//! SVG → raster, via resvg. The bundled Montserrat faces are registered in a
//! shared `fontdb`, so the same pipeline backs the live preview and the export.

use std::sync::{Arc, OnceLock};

use anyhow::{Result, anyhow};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{self, fontdb};

const FONT_REGULAR: &[u8] = include_bytes!("../assets/fonts/Montserrat-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../assets/fonts/Montserrat-Bold.ttf");
const FONT_BLACK: &[u8] = include_bytes!("../assets/fonts/Montserrat-Black.ttf");

fn shared_fontdb() -> Arc<fontdb::Database> {
    static DB: OnceLock<Arc<fontdb::Database>> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_font_data(FONT_REGULAR.to_vec());
        db.load_font_data(FONT_BOLD.to_vec());
        db.load_font_data(FONT_BLACK.to_vec());
        db.set_sans_serif_family("Montserrat");
        Arc::new(db)
    })
    .clone()
}

fn parse(svg: &str) -> Result<usvg::Tree> {
    let options = usvg::Options {
        fontdb: shared_fontdb(),
        ..Default::default()
    };
    usvg::Tree::from_str(svg, &options).map_err(|error| anyhow!("invalid wallpaper SVG: {error}"))
}

/// Rasterize `svg` so its longest edge is `longest_px`, keeping its aspect ratio.
pub fn render_to_pixmap(svg: &str, longest_px: u32) -> Result<Pixmap> {
    let tree = parse(svg)?;
    let size = tree.size();
    let scale = longest_px as f32 / size.width().max(size.height());
    let width = (size.width() * scale).round() as u32;
    let height = (size.height() * scale).round() as u32;
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| anyhow!("invalid pixmap size {width}×{height}"))?;
    resvg::render(
        &tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

/// Rasterize and PNG-encode.
pub fn png_bytes(svg: &str, longest_px: u32) -> Result<Vec<u8>> {
    render_to_pixmap(svg, longest_px)?
        .encode_png()
        .map_err(|error| anyhow!("PNG encoding failed: {error}"))
}

/// Straight-alpha RGBA8 for the Slint preview (tiny-skia stores premultiplied).
pub fn render_to_rgba(svg: &str, longest_px: u32) -> Result<(u32, u32, Vec<u8>)> {
    let pixmap = render_to_pixmap(svg, longest_px)?;
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
    use crate::config::Config;
    use crate::format::Format;
    use crate::svg::emit;

    #[test]
    fn rasterizes_default_to_requested_longest_edge() {
        let svg = emit(&Config::default());
        let pixmap = render_to_pixmap(&svg, 384).expect("rasterize");
        // Default format is 16:9, so 384 longest ⇒ 384×216.
        assert_eq!((pixmap.width(), pixmap.height()), (384, 216));
    }

    #[test]
    fn keeps_portrait_aspect() {
        let config = Config {
            format: Format::Mobile9x16,
            ..Config::default()
        };
        let pixmap = render_to_pixmap(&emit(&config), 320).expect("rasterize");
        assert!(pixmap.height() > pixmap.width());
    }

    #[test]
    fn png_bytes_have_png_signature() {
        let png = png_bytes(&emit(&Config::default()), 128).expect("encode png");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn rgba_length_matches_dimensions() {
        let (width, height, rgba) = render_to_rgba(&emit(&Config::default()), 96).expect("rgba");
        assert_eq!(rgba.len(), width as usize * height as usize * 4);
        // Opaque background ⇒ every pixel fully opaque.
        assert!(rgba.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }
}

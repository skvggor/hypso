//! Generates the landing page's static imagery with Hypso's own engine, the way
//! `gallery.rs` and `logo.rs` do: a 1200×630 Open Graph card (contour field +
//! AAA-contrast title) and a 16:9 hero poster used as the reduced-motion /
//! no-WASM fallback. Deterministic — the same seeds always produce the same
//! files.
//!
//! Run with: `cargo run --example site`

use std::fs;
use std::path::Path;

use anyhow::Result;

use hypso::config::{Config, Gradient};
use hypso::format::Format;
use hypso::noise::{self, NoiseParams};
use hypso::render::render_png;
use hypso::text_zone::TextZone;
use hypso::{contour, smooth};

// Dark cartographic / instrument palette, shared with the live hero and CSS.
const BACKGROUND: &str = "#0b0e12";
const LINE_START: &str = "#2b4b57";
const LINE_END: &str = "#7fd4e6";
const TITLE: &str = "#f2efe9";
const TAGLINE: &str = "#a9bcc2";

const OG_WIDTH: f32 = 1200.0;
const OG_HEIGHT: f32 = 630.0;
const OG_SEED: u64 = 731;
const FIELD_WIDTH: usize = 220;
const FIELD_HEIGHT: usize = 116; // ≈ 1200:630 so contours fill the frame.
const LEVELS: u32 = 20; // fewer, more open bands

fn field() -> noise::Field {
    noise::field(
        OG_SEED,
        FIELD_WIDTH,
        FIELD_HEIGHT,
        NoiseParams {
            octaves: 5,
            frequency: 3.4,
            persistence: 0.5,
        },
    )
}

/// Map a field-grid point to full-bleed OG canvas coordinates.
fn to_canvas(point: (f32, f32)) -> (f32, f32) {
    (
        point.0 / (FIELD_WIDTH - 1) as f32 * OG_WIDTH,
        point.1 / (FIELD_HEIGHT - 1) as f32 * OG_HEIGHT,
    )
}

fn polyline_path(points: &[(f32, f32)]) -> String {
    let mut path = String::new();
    for (index, point) in points.iter().enumerate() {
        let (x, y) = to_canvas(*point);
        let command = if index == 0 { 'M' } else { 'L' };
        path.push_str(&format!("{command}{x:.1} {y:.1} "));
    }
    path.trim_end().to_string()
}

fn og_svg() -> String {
    let field = field();
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
<defs>
<linearGradient id="ink" x1="0" y1="0" x2="1" y2="1">
<stop offset="0" stop-color="{LINE_START}"/>
<stop offset="1" stop-color="{LINE_END}"/>
</linearGradient>
</defs>
<rect width="1200" height="630" fill="{BACKGROUND}"/>
<g fill="none" stroke="url(#ink)" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" opacity="0.85">
"##
    );

    for level in contour::level_values(LEVELS) {
        for line in contour::march(&field, level) {
            let smoothed = smooth::fit(&line, 3);
            if smoothed.len() < 6 {
                continue;
            }
            svg.push_str(&format!(r#"<path d="{}"/>"#, polyline_path(&smoothed)));
            svg.push('\n');
        }
    }

    svg.push_str("</g>\n");

    // Carve the wordmark like the app's text zones: a background-colored margin
    // pushes the contours away (feather), then the solid glyphs sit on top —
    // legible at any thumbnail size, faithful to the real render.
    svg.push_str(&format!(
        r#"<text x="60" y="432" font-family="Montserrat" font-weight="900" font-size="250" letter-spacing="-8" fill="{BACKGROUND}" stroke="{BACKGROUND}" stroke-width="24" stroke-linejoin="round">Hypso</text>"#
    ));
    svg.push('\n');
    svg.push_str(&format!(
        r#"<text x="60" y="432" font-family="Montserrat" font-weight="900" font-size="250" letter-spacing="-8" fill="{TITLE}">Hypso</text>"#
    ));
    svg.push('\n');
    let tagline = "Procedural topographic wallpapers, reproducible by seed.";
    svg.push_str(&format!(
        r#"<text x="66" y="486" font-family="Montserrat" font-weight="400" font-size="28" fill="{BACKGROUND}" stroke="{BACKGROUND}" stroke-width="9" stroke-linejoin="round">{tagline}</text>"#
    ));
    svg.push('\n');
    svg.push_str(&format!(
        r#"<text x="66" y="486" font-family="Montserrat" font-weight="400" font-size="28" fill="{TAGLINE}">{tagline}</text>"#
    ));
    svg.push('\n');

    // Registration marks, echoing the live sheet.
    for (x, y, dx, dy) in [
        (34.0, 34.0, 1.0, 1.0),
        (1166.0, 34.0, -1.0, 1.0),
        (34.0, 596.0, 1.0, -1.0),
        (1166.0, 596.0, -1.0, -1.0),
    ] {
        svg.push_str(&format!(
            r#"<path d="M{x} {} L{x} {y} L{} {y}" fill="none" stroke="{TAGLINE}" stroke-width="1.4" opacity="0.5"/>"#,
            y + dy * 14.0,
            x + dx * 14.0,
        ));
        svg.push('\n');
    }

    svg.push_str("</svg>\n");
    svg
}

/// A dark cartographic poster through the real render pipeline, with the wordmark
/// carved into the field via the app's own text-zone feature — the same effect
/// the live hero reproduces. Used for the reduced-motion / no-WASM fallback.
fn poster_config() -> Config {
    Config {
        seed: 4021,
        levels: 20,
        frequency: 3.4,
        format: Format::Desktop16x9,
        background: BACKGROUND.into(),
        line_gradient: Some(Gradient {
            start: LINE_START.into(),
            end: LINE_END.into(),
        }),
        base_stroke: 1.7,
        index_interval: 4,
        gradient_overlay: 0.22,
        grain: 0.18,
        text_zones: vec![TextZone {
            x: 0.06,
            y: 0.64,
            width: 0.58,
            height: 0.2,
            text: "Hypso".into(),
        }],
        text_color: TITLE.into(),
        feather: 10.0,
        ..Config::default()
    }
}

/// Rasterize the app's own logo SVG into the favicon / PWA icon sizes the page
/// needs, so no external image tool is required.
fn write_icons(root: &Path) -> Result<()> {
    let icons = root.join("web/assets/icons");
    fs::create_dir_all(&icons)?;
    let logo = fs::read_to_string(root.join("assets/icons/icon.svg"))?;
    for size in [32u32, 180, 192, 512] {
        let bytes = hypso::raster::png_bytes(&logo, size)?;
        let path = icons.join(format!("icon-{size}.png"));
        fs::write(&path, bytes)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let img = root.join("web/assets/img");
    fs::create_dir_all(&img)?;

    let og = hypso::raster::png_bytes(&og_svg(), 1200)?;
    fs::write(img.join("og.png"), &og)?;
    println!("wrote {}", img.join("og.png").display());

    let poster = render_png(&poster_config(), 1600)?;
    fs::write(img.join("poster.png"), &poster)?;
    println!("wrote {}", img.join("poster.png").display());

    write_icons(root)?;
    Ok(())
}

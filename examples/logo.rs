//! Generates the Hypso logo with the app's own engine: a radial "peak" field
//! perturbed by fBm noise, contoured into concentric rings: a stylized
//! topographic summit. Writes `assets/icons/icon.svg` plus PNG renditions.
//!
//! Run with: `cargo run --example logo`, then rebuild the Windows icon with
//! `magick assets/icons/icon-256.png -define icon:auto-resize=16,24,32,48,64,128,256 assets/icons/icon.ico`.

use std::fs;
use std::path::Path;

use hypso::noise::{self, NoiseParams};
use hypso::{contour, raster, smooth};

const FIELD_SIZE: usize = 240;
const CANVAS: f32 = 512.0;
const MARGIN: f32 = 34.0;
const LEVELS: u32 = 3;
const SEED: u64 = 7;

const BACKGROUND: &str = "#0f1216";
const LINE: &str = "#6fb6c8";
const PEAK: &str = "#f2efe9";

fn peak_field() -> noise::Field {
    let perturbation = noise::field(
        SEED,
        FIELD_SIZE,
        FIELD_SIZE,
        NoiseParams {
            octaves: 4,
            frequency: 2.3,
            persistence: 0.5,
        },
    );
    let (center_x, center_y) = (0.52_f32, 0.49_f32);
    let mut values = Vec::with_capacity(FIELD_SIZE * FIELD_SIZE);
    for y in 0..FIELD_SIZE {
        for x in 0..FIELD_SIZE {
            let nx = x as f32 / (FIELD_SIZE - 1) as f32;
            let ny = y as f32 / (FIELD_SIZE - 1) as f32;
            let distance = ((nx - center_x).powi(2) + (ny - center_y).powi(2)).sqrt();
            let radial = 1.0 - distance * 2.1;
            let wobble = (perturbation.at(x, y) - 0.5) * 0.22;
            values.push((radial + wobble).clamp(0.0, 1.0));
        }
    }
    noise::Field {
        width: FIELD_SIZE,
        height: FIELD_SIZE,
        values,
    }
}

fn to_canvas(point: (f32, f32)) -> (f32, f32) {
    let scale = (CANVAS - 2.0 * MARGIN) / (FIELD_SIZE - 1) as f32;
    (MARGIN + point.0 * scale, MARGIN + point.1 * scale)
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

fn emit_logo_svg() -> String {
    let field = peak_field();
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512">
<rect width="512" height="512" fill="{BACKGROUND}"/>
"##
    );

    for level in contour::level_values(LEVELS) {
        let width = 20.0;
        for polyline in contour::march(&field, level) {
            if polyline.len() < 8 {
                continue;
            }
            let smoothed = smooth::fit(&polyline, 3);
            svg.push_str(&format!(
                r#"<path d="{}" fill="none" stroke="{LINE}" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round"/>
"#,
                polyline_path(&smoothed),
            ));
        }
    }

    let peak_index = field
        .values
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(index, _)| index)
        .unwrap_or(0);
    let (peak_x, peak_y) = to_canvas((
        (peak_index % FIELD_SIZE) as f32,
        (peak_index / FIELD_SIZE) as f32,
    ));
    svg.push_str(&format!(
        r#"<path d="M{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} Z" fill="{PEAK}"/>
"#,
        peak_x,
        peak_y - 26.0,
        peak_x + 24.0,
        peak_y + 16.0,
        peak_x - 24.0,
        peak_y + 16.0,
    ));
    svg.push_str("</svg>\n");
    svg
}

fn main() -> anyhow::Result<()> {
    let icons_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icons");
    let svg = emit_logo_svg();
    fs::write(icons_dir.join("icon.svg"), &svg)?;
    for size in [128u32, 256, 512, 1024] {
        fs::write(
            icons_dir.join(format!("icon-{size}.png")),
            raster::png_bytes(&svg, size)?,
        )?;
    }
    println!("Logo written to {}", icons_dir.display());
    Ok(())
}

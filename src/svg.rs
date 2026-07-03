//! Assembles the whole wallpaper into a single SVG string — the source of truth
//! rasterized identically for preview and export.
//!
//! Pipeline: fractal field → iso-contours → Chaikin smoothing → text-zone
//! exclusion → Catmull-Rom cubic Bézier paths, over a flat background, with an
//! optional gradient overlay and Montserrat text.

use crate::config::Config;
use crate::geom::{Point, Polyline};
use crate::{contour, noise, smooth, stroke, text_zone};

/// Cells along the field's longest edge. Enough detail for crisp contours while
/// keeping the live preview responsive.
const FIELD_DETAIL: usize = 200;

/// Field grid dimensions matching the format's aspect ratio.
fn field_dims(width: u32, height: u32) -> (usize, usize) {
    let detail = FIELD_DETAIL as f32;
    if width >= height {
        (
            FIELD_DETAIL,
            (detail * height as f32 / width as f32).round() as usize,
        )
    } else {
        (
            (detail * width as f32 / height as f32).round() as usize,
            FIELD_DETAIL,
        )
    }
}

/// Catmull-Rom spline through `points`, emitted as cubic Bézier (`C`) commands.
fn bezier_path(points: &[Point]) -> String {
    if points.len() < 2 {
        return String::new();
    }
    let n = points.len();
    let mut data = format!("M {:.2} {:.2}", points[0].0, points[0].1);
    for i in 0..n - 1 {
        let p0 = points[i.saturating_sub(1)];
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = points[(i + 2).min(n - 1)];
        let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
        let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
        data.push_str(&format!(
            " C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
            c1.0, c1.1, c2.0, c2.1, p2.0, p2.1
        ));
    }
    data
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Build the single SVG string for `config`.
pub fn emit(config: &Config) -> String {
    let (canvas_w, canvas_h) = config.format.dimensions();
    let (field_w, field_h) = field_dims(canvas_w, canvas_h);
    let scale_x = canvas_w as f32 / field_w as f32;
    let scale_y = canvas_h as f32 / field_h as f32;

    let field = noise::field(
        config.seed,
        field_w,
        field_h,
        noise::NoiseParams {
            octaves: config.octaves,
            frequency: config.frequency,
            persistence: 0.5,
        },
    );

    // Text zones are stored normalized in [0, 1]; map them to canvas pixels.
    let zones: Vec<text_zone::TextZone> = config
        .text_zones
        .iter()
        .map(|zone| text_zone::TextZone {
            x: zone.x * canvas_w as f32,
            y: zone.y * canvas_h as f32,
            width: zone.width * canvas_w as f32,
            height: zone.height * canvas_h as f32,
            text: zone.text.clone(),
        })
        .collect();
    let feather_px = config.feather * canvas_w.min(canvas_h) as f32 / FIELD_DETAIL as f32;

    let line_paint = if config.line_gradient.is_some() {
        "url(#lineGradient)".to_string()
    } else {
        config.line_color.clone()
    };

    let mut paths = String::new();
    let levels = contour::level_values(config.levels);
    for (level_index, level) in levels.iter().enumerate() {
        let width = stroke::width_for(
            level_index as u32,
            config.base_stroke,
            config.index_interval,
        );
        for polyline in contour::march(&field, *level) {
            let smoothed = smooth::fit(&polyline, config.smoothing);
            let canvas: Polyline = smoothed
                .iter()
                .map(|&(x, y)| (x * scale_x, y * scale_y))
                .collect();
            for run in text_zone::reserve(&[canvas], &zones, feather_px) {
                let data = bezier_path(&run.points);
                if data.is_empty() {
                    continue;
                }
                paths.push_str(&format!(
                    r#"<path d="{data}" fill="none" stroke="{line_paint}" stroke-width="{width:.2}" stroke-opacity="{:.3}" stroke-linecap="round" stroke-linejoin="round"/>"#,
                    run.opacity
                ));
            }
        }
    }

    let mut defs = String::new();
    if let Some(gradient) = &config.line_gradient {
        defs.push_str(&format!(
            r#"<linearGradient id="lineGradient" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="{}"/><stop offset="1" stop-color="{}"/></linearGradient>"#,
            gradient.start, gradient.end
        ));
    }
    if config.gradient_overlay > 0.0 {
        defs.push_str(
            r##"<linearGradient id="overlay" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#000000" stop-opacity="0"/><stop offset="1" stop-color="#000000" stop-opacity="1"/></linearGradient>"##,
        );
    }

    let overlay = if config.gradient_overlay > 0.0 {
        format!(
            r#"<rect x="0" y="0" width="{canvas_w}" height="{canvas_h}" fill="url(#overlay)" opacity="{:.3}"/>"#,
            config.gradient_overlay
        )
    } else {
        String::new()
    };

    let mut text = String::new();
    for zone in &zones {
        let font_size = (zone.height * 0.32).max(12.0);
        text.push_str(&format!(
            r#"<text x="{:.1}" y="{:.1}" font-family="Montserrat" font-weight="700" font-size="{font_size:.1}" fill="{}">{}</text>"#,
            zone.x,
            zone.y + font_size,
            config.text_color,
            escape_xml(&zone.text)
        ));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{canvas_w}" height="{canvas_h}" viewBox="0 0 {canvas_w} {canvas_h}"><defs>{defs}</defs><rect x="0" y="0" width="{canvas_w}" height="{canvas_h}" fill="{}"/>{paths}{overlay}{text}</svg>"#,
        config.background
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Gradient;

    #[test]
    fn has_format_viewbox() {
        let svg = emit(&Config::default());
        assert!(
            svg.contains(r#"viewBox="0 0 3840 2160""#),
            "viewBox missing"
        );
    }

    #[test]
    fn has_full_canvas_background_in_chosen_color() {
        let config = Config {
            background: "#102030".into(),
            ..Config::default()
        };
        let svg = emit(&config);
        assert!(svg.contains("<rect"));
        assert!(svg.contains(r#"width="3840""#) && svg.contains(r#"height="2160""#));
        assert!(
            svg.contains(r##"fill="#102030""##),
            "background color missing"
        );
    }

    #[test]
    fn has_at_least_one_bezier_contour() {
        let svg = emit(&Config::default());
        assert!(svg.contains("<path"), "no contour paths");
        assert!(svg.contains(" C "), "contours must be cubic Bézier");
    }

    #[test]
    fn single_color_applies_to_every_stroke() {
        let config = Config {
            line_color: "#abcdef".into(),
            line_gradient: None,
            ..Config::default()
        };
        let svg = emit(&config);
        assert!(
            !svg.contains("url(#lineGradient)"),
            "should not define a gradient"
        );
        let paths = svg.matches("<path").count();
        let colored = svg.matches(r##"stroke="#abcdef""##).count();
        assert!(
            paths > 0 && colored == paths,
            "{colored} colored vs {paths} paths"
        );
    }

    #[test]
    fn text_uses_its_own_color_not_the_line_color() {
        let config = Config {
            line_color: "#abcdef".into(),
            text_color: "#402010".into(),
            text_zones: vec![crate::text_zone::TextZone {
                x: 0.1,
                y: 0.1,
                width: 0.4,
                height: 0.2,
                text: "Summit".into(),
            }],
            ..Config::default()
        };
        let svg = emit(&config);
        assert!(
            svg.contains(r##"<text"##) && svg.contains(r##"fill="#402010""##),
            "text must be filled with text_color"
        );
        assert_eq!(
            svg.matches(r##"fill="#abcdef""##).count(),
            0,
            "line color must not drive the text fill"
        );
    }

    #[test]
    fn gradient_is_defined_once_and_used_by_all_strokes() {
        let config = Config {
            line_gradient: Some(Gradient {
                start: "#111111".into(),
                end: "#999999".into(),
            }),
            ..Config::default()
        };
        let svg = emit(&config);
        assert_eq!(svg.matches(r#"id="lineGradient""#).count(), 1);
        let paths = svg.matches("<path").count();
        let gradient_strokes = svg.matches(r#"stroke="url(#lineGradient)""#).count();
        assert!(paths > 0 && gradient_strokes == paths);
    }
}

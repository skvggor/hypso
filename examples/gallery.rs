//! Regenerates the committed visual samples in `docs/samples/`. These back the
//! by-eye quality check that TDD deliberately does not cover, and the README
//! gallery.
//!
//! Run with: `cargo run --example gallery`

use std::path::Path;

use anyhow::Result;

use hypso::config::{Config, Gradient};
use hypso::format::Format;
use hypso::render::render_png;
use hypso::text_zone::TextZone;

/// Sample size (longest edge), smaller than 4K to keep committed files light.
const SAMPLE_PIXELS: u32 = 1280;

fn samples() -> Vec<(&'static str, Config)> {
    vec![
        (
            "midnight",
            Config {
                seed: 731,
                levels: 30,
                frequency: 3.4,
                background: "#0f1216".into(),
                line_gradient: Some(Gradient {
                    start: "#2b4b57".into(),
                    end: "#7fd4e6".into(),
                }),
                base_stroke: 1.6,
                grain: 0.14,
                ..Config::default()
            },
        ),
        (
            "paper",
            Config {
                seed: 12,
                levels: 26,
                frequency: 3.0,
                background: "#f2efe9".into(),
                line_color: "#1b1b1b".into(),
                grain: 0.10,
                ..Config::default()
            },
        ),
        (
            "ocean",
            Config {
                seed: 4021,
                levels: 34,
                frequency: 3.8,
                background: "#0a1f2e".into(),
                line_gradient: Some(Gradient {
                    start: "#1d5c7a".into(),
                    end: "#9be3f0".into(),
                }),
                base_stroke: 1.4,
                gradient_overlay: 0.22,
                ..Config::default()
            },
        ),
        (
            "ember",
            Config {
                seed: 88,
                levels: 28,
                frequency: 3.2,
                background: "#150a0d".into(),
                line_gradient: Some(Gradient {
                    start: "#ff8a00".into(),
                    end: "#e52e71".into(),
                }),
                base_stroke: 1.7,
                grain: 0.18,
                ..Config::default()
            },
        ),
        (
            "sage",
            Config {
                seed: 517,
                levels: 22,
                frequency: 2.6,
                background: "#edf0e6".into(),
                line_color: "#3c5548".into(),
                base_stroke: 1.8,
                index_interval: 4,
                ..Config::default()
            },
        ),
        (
            "summit",
            Config {
                seed: 2077,
                levels: 30,
                frequency: 3.4,
                background: "#101418".into(),
                line_color: "#8fa8b8".into(),
                text_zones: vec![TextZone {
                    x: 0.07,
                    y: 0.72,
                    width: 0.42,
                    height: 0.18,
                    text: "SUMMIT".into(),
                }],
                text_color: "#f2efe9".into(),
                grain: 0.15,
                ..Config::default()
            },
        ),
        (
            "ocean-mobile",
            Config {
                seed: 4021,
                levels: 34,
                frequency: 3.8,
                format: Format::Mobile9x16,
                background: "#0a1f2e".into(),
                line_gradient: Some(Gradient {
                    start: "#1d5c7a".into(),
                    end: "#9be3f0".into(),
                }),
                base_stroke: 1.4,
                gradient_overlay: 0.22,
                ..Config::default()
            },
        ),
    ]
}

fn main() -> Result<()> {
    let dir = Path::new("docs/samples");
    std::fs::create_dir_all(dir)?;
    for (name, config) in samples() {
        let bytes = render_png(&config, SAMPLE_PIXELS)?;
        let path = dir.join(format!("{name}.png"));
        std::fs::write(&path, bytes)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

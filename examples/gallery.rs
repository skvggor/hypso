//! Regenerates the committed visual samples in `docs/samples/`. These back the
//! by-eye quality check that TDD deliberately does not cover.
//!
//! Run with: `cargo run --example gallery`

use std::path::Path;

use anyhow::Result;

use hypso::config::{Config, Gradient};
use hypso::render::render_png;
use hypso::text_zone::TextZone;

/// Sample size (longest edge) — smaller than 4K to keep committed files light.
const SAMPLE_PIXELS: u32 = 1280;

fn samples() -> Vec<(&'static str, Config)> {
    vec![
        ("classic", Config::default()),
        (
            "ocean",
            Config {
                seed: 42,
                background: "#0e2230".into(),
                line_color: "#7fd4e6".into(),
                ..Config::default()
            },
        ),
        (
            "grain-overlay",
            Config {
                seed: 7,
                gradient_overlay: 0.45,
                grain: 0.5,
                ..Config::default()
            },
        ),
        (
            "gradient-lines",
            Config {
                seed: 99,
                line_gradient: Some(Gradient {
                    start: "#ff8a00".into(),
                    end: "#e52e71".into(),
                }),
                ..Config::default()
            },
        ),
        (
            "with-text",
            Config {
                seed: 13,
                text_zones: vec![TextZone {
                    x: 0.08,
                    y: 0.10,
                    width: 0.46,
                    height: 0.20,
                    text: "hypso".into(),
                }],
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

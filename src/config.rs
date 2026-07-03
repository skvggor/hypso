//! The full wallpaper configuration.
//!
//! Parameters are split into two castes so the live preview stays responsive:
//! **structural** params change the noise field and require regeneration (and are
//! debounced in the GUI); **style** params only re-render and are cheap.

use serde::{Deserialize, Serialize};

use crate::format::Format;
use crate::text_zone::TextZone;

/// A CSS color string (e.g. `"#1b1b1b"`), consumed directly by the SVG.
pub type Color = String;

/// A two-stop linear gradient for the contour lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    pub start: Color,
    pub end: Color,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    // ---- structural: regenerate the noise field ----
    pub seed: u64,
    pub octaves: u32,
    pub frequency: f32,
    pub levels: u32,
    pub format: Format,

    // ---- style: cheap re-render only ----
    pub background: Color,
    pub line_color: Color,
    /// When set, lines use this gradient instead of `line_color`.
    pub line_gradient: Option<Gradient>,
    pub base_stroke: f32,
    /// Every `index_interval`-th contour is rendered thicker (index contours).
    pub index_interval: u32,
    pub smoothing: u32,
    /// Gradient overlay intensity in `[0, 1]`; `0` disables it.
    pub gradient_overlay: f32,
    /// Film-grain intensity in `[0, 1]`; `0` disables it.
    pub grain: f32,

    /// Reserved text zones; the pattern flows around them.
    pub text_zones: Vec<TextZone>,
    /// Fill color of the zone text. Defaults for presets saved before it existed.
    #[serde(default = "default_text_color")]
    pub text_color: Color,
    /// Feather margin around text zones, in field coordinates.
    pub feather: f32,
}

fn default_text_color() -> Color {
    "#1b1b1b".into()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            seed: 1,
            octaves: 5,
            frequency: 3.0,
            levels: 24,
            format: Format::Desktop16x9,
            background: "#f2efe9".into(),
            line_color: "#1b1b1b".into(),
            line_gradient: None,
            base_stroke: 1.5,
            index_interval: 5,
            smoothing: 3,
            gradient_overlay: 0.0,
            grain: 0.0,
            text_zones: Vec::new(),
            text_color: default_text_color(),
            feather: 6.0,
        }
    }
}

// `Format` must round-trip through serde for presets; the derives below give it
// a stable string representation.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_desktop_with_disabled_effects() {
        let config = Config::default();
        assert_eq!(config.format, Format::Desktop16x9);
        assert_eq!(config.gradient_overlay, 0.0);
        assert_eq!(config.grain, 0.0);
        assert!(config.line_gradient.is_none());
    }

    #[test]
    fn presets_without_text_color_still_load() {
        let serialized = toml::to_string(&Config::default()).expect("serialize");
        let legacy: String = serialized
            .lines()
            .filter(|line| !line.starts_with("text_color"))
            .collect::<Vec<_>>()
            .join("\n");
        let config: Config = toml::from_str(&legacy).expect("legacy preset must load");
        assert_eq!(config.text_color, Config::default().text_color);
    }
}

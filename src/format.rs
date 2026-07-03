//! Output formats and their fixed pixel dimensions.

use serde::{Deserialize, Serialize};

/// A wallpaper output format. Each maps to fixed pixel dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    /// 16:9 desktop at UHD (3840×2160).
    Desktop16x9,
    /// 9:16 mobile portrait (2160×3840).
    Mobile9x16,
    /// 9:19.5 tall mobile portrait (1644×3556).
    Mobile9x19_5,
}

impl Format {
    /// Every format, in UI display order.
    pub const ALL: [Format; 3] = [
        Format::Desktop16x9,
        Format::Mobile9x16,
        Format::Mobile9x19_5,
    ];

    /// Pixel dimensions as `(width, height)`.
    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Format::Desktop16x9 => (3840, 2160),
            Format::Mobile9x16 => (2160, 3840),
            Format::Mobile9x19_5 => (1644, 3556),
        }
    }

    /// Output width in pixels.
    pub fn width(self) -> u32 {
        self.dimensions().0
    }

    /// Output height in pixels.
    pub fn height(self) -> u32 {
        self.dimensions().1
    }

    /// Longest edge in pixels — the size passed to the rasterizer.
    pub fn longest_edge(self) -> u32 {
        let (width, height) = self.dimensions();
        width.max(height)
    }

    /// Aspect ratio as width / height.
    pub fn aspect_ratio(self) -> f32 {
        let (width, height) = self.dimensions();
        width as f32 / height as f32
    }

    /// Human-readable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            Format::Desktop16x9 => "Desktop 16:9 (3840×2160)",
            Format::Mobile9x16 => "Mobile 9:16 (2160×3840)",
            Format::Mobile9x19_5 => "Mobile 9:19.5 (1644×3556)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_is_uhd() {
        assert_eq!(Format::Desktop16x9.dimensions(), (3840, 2160));
    }

    #[test]
    fn mobile_portrait_is_taller_than_wide() {
        for format in [Format::Mobile9x16, Format::Mobile9x19_5] {
            let (width, height) = format.dimensions();
            assert!(height > width, "{format:?} should be portrait");
        }
    }

    #[test]
    fn mobile_dimensions_are_exact() {
        assert_eq!(Format::Mobile9x16.dimensions(), (2160, 3840));
        assert_eq!(Format::Mobile9x19_5.dimensions(), (1644, 3556));
    }

    #[test]
    fn helpers_agree_with_dimensions() {
        for format in Format::ALL {
            let (width, height) = format.dimensions();
            assert_eq!(format.width(), width);
            assert_eq!(format.height(), height);
            assert_eq!(format.longest_edge(), width.max(height));
            assert!((format.aspect_ratio() - width as f32 / height as f32).abs() < 1e-6);
            assert!(!format.label().is_empty());
        }
    }
}

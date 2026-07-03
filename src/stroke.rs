//! Stroke width by contour level (index contours).
//!
//! Topographic maps draw every Nth contour thicker. Width is the only thing
//! that varies with level — color is uniform and handled by the SVG layer.

/// How much thicker an index contour is than a regular one.
const INDEX_MULTIPLIER: f32 = 2.2;

/// Whether `level_index` is an index contour for the given interval.
pub fn is_index(level_index: u32, index_interval: u32) -> bool {
    index_interval > 0 && level_index.is_multiple_of(index_interval)
}

/// Stroke width for a contour at `level_index`, scaling `base` up on index
/// contours.
pub fn width_for(level_index: u32, base: f32, index_interval: u32) -> f32 {
    if is_index(level_index, index_interval) {
        base * INDEX_MULTIPLIER
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: f32 = 1.5;

    #[test]
    fn index_levels_are_thicker() {
        let thick = width_for(5, BASE, 5);
        let thin = width_for(1, BASE, 5);
        assert!(thick > thin, "index contour {thick} should exceed {thin}");
    }

    #[test]
    fn non_index_levels_share_base_width() {
        assert_eq!(width_for(1, BASE, 5), BASE);
        assert_eq!(width_for(2, BASE, 5), width_for(3, BASE, 5));
    }
}

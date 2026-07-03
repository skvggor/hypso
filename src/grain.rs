//! Seeded film-grain post-processing over a rasterized image.
//!
//! Grain is sampled **per output pixel** so it sits uniformly over the whole
//! composition — background, contour lines, and text alike. (Coarser virtual
//! cells left thin lines untouched.) It is deterministic: identical
//! `(image, seed, intensity)` produce identical output.

use resvg::tiny_skia::Pixmap;

/// Maximum per-channel delta at full intensity.
const MAX_DELTA: f32 = 26.0;

/// Signed hash noise in `[-1, 1)` for a pixel.
fn hash_noise(gx: i64, gy: i64, seed: u64) -> f32 {
    let mut h = seed
        ^ (gx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (gy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    let unit = (h >> 40) as f32 / (1u64 << 24) as f32;
    unit * 2.0 - 1.0
}

/// Signed grain value in `[-1, 1)`: two independent samples averaged, giving a
/// triangular distribution — soft film grain instead of salt-and-pepper noise.
fn grain_at(gx: i64, gy: i64, seed: u64) -> f32 {
    (hash_noise(gx, gy, seed) + hash_noise(gx, gy, seed ^ 0x6A09_E667_F3BC_C909)) * 0.5
}

/// Apply grain to `pixmap` in place. `intensity <= 0` is a no-op.
pub fn apply(pixmap: &mut Pixmap, seed: u64, intensity: f32) {
    if intensity <= 0.0 {
        return;
    }
    let (width, height) = (pixmap.width(), pixmap.height());
    let amplitude = intensity.clamp(0.0, 1.0) * MAX_DELTA;
    let data = pixmap.data_mut();
    for y in 0..height {
        for x in 0..width {
            let delta = grain_at(x as i64, y as i64, seed) * amplitude;
            let base = ((y * width + x) * 4) as usize;
            for channel in 0..3 {
                let value = data[base + channel] as f32 + delta;
                data[base + channel] = value.clamp(0.0, 255.0) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use resvg::tiny_skia::Color;

    fn gray_pixmap() -> Pixmap {
        let mut pixmap = Pixmap::new(64, 48).expect("pixmap");
        pixmap.fill(Color::from_rgba8(128, 128, 128, 255));
        pixmap
    }

    #[test]
    fn zero_intensity_leaves_pixels_unchanged() {
        let mut pixmap = gray_pixmap();
        let original = pixmap.data().to_vec();
        apply(&mut pixmap, 7, 0.0);
        assert_eq!(pixmap.data(), original.as_slice());
    }

    #[test]
    fn same_seed_reproduces_grain() {
        let mut a = gray_pixmap();
        let mut b = gray_pixmap();
        apply(&mut a, 99, 0.5);
        apply(&mut b, 99, 0.5);
        assert_eq!(a.data(), b.data());
    }

    #[test]
    fn non_zero_intensity_alters_image() {
        let mut pixmap = gray_pixmap();
        let original = pixmap.data().to_vec();
        apply(&mut pixmap, 5, 0.6);
        assert_ne!(pixmap.data(), original.as_slice());
    }

    #[test]
    fn grain_reaches_dark_regions_not_only_bright_background() {
        // Line-colored pixels, not bright background.
        let mut pixmap = Pixmap::new(64, 48).expect("pixmap");
        pixmap.fill(Color::from_rgba8(27, 27, 27, 255));
        let original = pixmap.data().to_vec();
        apply(&mut pixmap, 9, 0.7);
        assert_ne!(pixmap.data(), original.as_slice());
    }

    #[test]
    fn grain_is_soft() {
        // Full intensity must stay subtle: bounded peaks, gentle average — not
        // salt-and-pepper.
        let mut pixmap = gray_pixmap();
        apply(&mut pixmap, 11, 1.0);
        let deltas: Vec<f32> = pixmap
            .data()
            .iter()
            .step_by(4)
            .map(|&value| (value as f32 - 128.0).abs())
            .collect();
        let max = deltas.iter().fold(0.0_f32, |a, &b| a.max(b));
        let mean = deltas.iter().sum::<f32>() / deltas.len() as f32;
        assert!(max <= MAX_DELTA, "grain peaks too strong: {max}");
        assert!(mean <= 9.0, "grain too harsh on average: {mean}");
    }

    #[test]
    fn grain_is_fine_per_pixel() {
        // Coarse virtual cells made adjacent pixels share a value at high
        // resolutions, leaving thin lines clean. Per-pixel grain must not.
        let mut pixmap = Pixmap::new(2000, 1).expect("pixmap");
        pixmap.fill(Color::from_rgba8(128, 128, 128, 255));
        apply(&mut pixmap, 5, 0.6);
        let luminance: Vec<u8> = pixmap.data().iter().step_by(4).copied().collect();
        let adjacent_equal = luminance.windows(2).filter(|w| w[0] == w[1]).count();
        assert!(
            adjacent_equal < luminance.len() / 4,
            "grain too coarse: {adjacent_equal} adjacent-equal pixels"
        );
    }
}

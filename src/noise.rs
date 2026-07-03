//! Deterministic multi-octave fractal (fBm) noise field.
//!
//! The field is fully determined by `seed` and [`NoiseParams`], so the same
//! inputs always produce the same values, the property TDD leans on and that
//! makes presets reproducible.

/// A row-major scalar field; every value lies in `[0.0, 1.0]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl Field {
    /// Sample at integer grid coordinates.
    pub fn at(&self, x: usize, y: usize) -> f32 {
        self.values[y * self.width + x]
    }
}

/// Structural parameters controlling the fractal field.
#[derive(Debug, Clone, Copy)]
pub struct NoiseParams {
    pub octaves: u32,
    pub frequency: f32,
    pub persistence: f32,
}

impl Default for NoiseParams {
    fn default() -> Self {
        NoiseParams {
            octaves: 5,
            frequency: 3.0,
            persistence: 0.5,
        }
    }
}

/// Hash a lattice corner to a value in `[0, 1)` (splitmix64 finalizer).
fn hash01(ix: i64, iy: i64, seed: u64) -> f32 {
    let mut h = seed
        ^ (ix as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (iy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    // Top 24 bits → [0, 1).
    (h >> 40) as f32 / (1u64 << 24) as f32
}

/// Smoothstep fade so interpolation has zero first-derivative at the lattice.
fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Bilinearly interpolated value noise at `(x, y)`, in `[0, 1]`.
fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (ix, iy) = (x0 as i64, y0 as i64);
    let (u, v) = (fade(x - x0), fade(y - y0));

    let v00 = hash01(ix, iy, seed);
    let v10 = hash01(ix + 1, iy, seed);
    let v01 = hash01(ix, iy + 1, seed);
    let v11 = hash01(ix + 1, iy + 1, seed);

    let top = v00 + u * (v10 - v00);
    let bottom = v01 + u * (v11 - v01);
    top + v * (bottom - top)
}

/// Generate the fractal (fBm) field. Octaves are summed at doubling frequency
/// and decaying amplitude, then normalized by total amplitude so every value
/// stays in `[0, 1]`.
pub fn field(seed: u64, width: usize, height: usize, params: NoiseParams) -> Field {
    let octaves = params.octaves.max(1);
    let max_amplitude: f32 = (0..octaves)
        .scan(1.0_f32, |amplitude, _| {
            let current = *amplitude;
            *amplitude *= params.persistence;
            Some(current)
        })
        .sum();

    let mut values = Vec::with_capacity(width * height);
    for y in 0..height {
        let v = y as f32 / height as f32;
        for x in 0..width {
            let u = x as f32 / width as f32;
            let mut sum = 0.0;
            let mut amplitude = 1.0;
            let mut frequency = params.frequency;
            for octave in 0..octaves {
                let octave_seed = seed.wrapping_add(octave as u64).wrapping_mul(0x9E37_79B9);
                sum += amplitude * value_noise(u * frequency, v * frequency, octave_seed);
                amplitude *= params.persistence;
                frequency *= 2.0;
            }
            values.push((sum / max_amplitude).clamp(0.0, 1.0));
        }
    }
    Field {
        width,
        height,
        values,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: usize = 32;
    const H: usize = 24;

    #[test]
    fn same_seed_reproduces_field() {
        let params = NoiseParams::default();
        let a = field(7, W, H, params);
        let b = field(7, W, H, params);
        assert_eq!(a.values, b.values);
    }

    #[test]
    fn different_seed_changes_field() {
        let params = NoiseParams::default();
        let a = field(1, W, H, params);
        let b = field(2, W, H, params);
        assert_ne!(a.values, b.values);
    }

    #[test]
    fn values_are_normalized() {
        let field = field(42, W, H, NoiseParams::default());
        assert!(
            field.values.iter().all(|&v| (0.0..=1.0).contains(&v)),
            "all samples must lie in [0, 1]"
        );
    }
}

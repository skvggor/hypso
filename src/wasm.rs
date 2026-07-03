//! WebAssembly surface for the landing-page hero.
//!
//! Re-exposes the native generative core (`noise` → `contour` → `smooth`)
//! unchanged to JavaScript as a flat, self-describing `Float32Array` of
//! normalized contour polylines. The pure [`contour_buffer`] builder is compiled
//! on every target, so its determinism and parity with the core are unit-tested
//! natively; only the thin `#[wasm_bindgen]` entry point is wasm-only.

use crate::geom::Polyline;
use crate::noise::{self, NoiseParams};
use crate::{contour, smooth};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Smoothed contours shorter than this are dropped as visual noise.
const MIN_POINTS: usize = 3;

/// Everything the hero needs to reproduce one topographic frame.
#[derive(Debug, Clone, Copy)]
pub struct HeroParams {
    pub seed: u64,
    pub width: usize,
    pub height: usize,
    pub octaves: u32,
    pub frequency: f32,
    pub persistence: f32,
    pub levels: u32,
    pub smoothing: u32,
}

/// Build normalized contour polylines for `params` and encode them into a single
/// flat buffer:
///
/// `[ polyline_count, (level_index, point_count, x0, y0, x1, y1, …), … ]`
///
/// `level_index` is the contour's elevation band (`0..levels`), so the browser
/// can vary stroke weight the way the app's index contours do. Every coordinate
/// lies in `[0, 1]` (x divided by `width - 1`, y by `height - 1`), so the browser
/// can draw straight onto any canvas size. The encoding is self-describing:
/// JavaScript walks it linearly with no schema.
pub fn contour_buffer(params: &HeroParams) -> Vec<f32> {
    if params.width < 2 || params.height < 2 {
        return vec![0.0];
    }

    let field = noise::field(
        params.seed,
        params.width,
        params.height,
        NoiseParams {
            octaves: params.octaves,
            frequency: params.frequency,
            persistence: params.persistence,
        },
    );

    encode_contours(&field, params.levels, params.smoothing)
}

/// Same as [`contour_buffer`], but first raises the field by `relief * mask[i]`
/// (clamped to `[0, 1]`) before contouring, so the map's own contour lines bend
/// around and cling to the stamped shape — the landing page uses this to make the
/// terrain hug the wordmark. `mask` is row-major, `width × height`, in `[0, 1]`.
pub fn relief_contour_buffer(params: &HeroParams, mask: &[f32], relief: f32) -> Vec<f32> {
    if params.width < 2 || params.height < 2 {
        return vec![0.0];
    }

    let mut field = noise::field(
        params.seed,
        params.width,
        params.height,
        NoiseParams {
            octaves: params.octaves,
            frequency: params.frequency,
            persistence: params.persistence,
        },
    );
    for (index, value) in field.values.iter_mut().enumerate() {
        if let Some(&bump) = mask.get(index) {
            *value = (*value + relief * bump).clamp(0.0, 1.0);
        }
    }

    encode_contours(&field, params.levels, params.smoothing)
}

/// Extract, smooth, and pack a field's contours into the self-describing buffer
/// `[ count, (level_index, point_count, x0, y0, …), … ]`, coordinates normalized
/// to `[0, 1]`.
fn encode_contours(field: &noise::Field, levels: u32, smoothing: u32) -> Vec<f32> {
    let inverse_width = 1.0 / (field.width - 1).max(1) as f32;
    let inverse_height = 1.0 / (field.height - 1).max(1) as f32;

    let mut records: Vec<(u32, Polyline)> = Vec::new();
    for (level_index, level) in contour::level_values(levels).into_iter().enumerate() {
        for line in contour::march(field, level) {
            let smoothed = smooth::fit(&line, smoothing);
            if smoothed.len() >= MIN_POINTS {
                records.push((level_index as u32, smoothed));
            }
        }
    }

    let capacity = 1 + records
        .iter()
        .map(|(_, line)| 2 + line.len() * 2)
        .sum::<usize>();
    let mut buffer = Vec::with_capacity(capacity);
    buffer.push(records.len() as f32);
    for (level_index, polyline) in &records {
        buffer.push(*level_index as f32);
        buffer.push(polyline.len() as f32);
        for &(x, y) in polyline {
            buffer.push((x * inverse_width).clamp(0.0, 1.0));
            buffer.push((y * inverse_height).clamp(0.0, 1.0));
        }
    }
    buffer
}

/// JavaScript entry point. Takes the seed as an `f64` (a plain JS number) and
/// returns a `Float32Array`; otherwise identical to [`contour_buffer`]. The wide
/// signature is deliberate — flat scalars marshal across the wasm boundary far
/// more cheaply than a constructed object.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn generate(
    seed: f64,
    width: usize,
    height: usize,
    octaves: u32,
    frequency: f32,
    persistence: f32,
    levels: u32,
    smoothing: u32,
) -> Vec<f32> {
    contour_buffer(&HeroParams {
        seed: seed as u64,
        width,
        height,
        octaves,
        frequency,
        persistence,
        levels,
        smoothing,
    })
}

/// Build the raw elevation field for `params`, encoded as
/// `[ width, height, v0, v1, … ]` with every value in `[0, 1]`. The landing page
/// samples this so the wordmark's stroke weight can follow the terrain and to
/// find where a contour meets a letter. `levels`/`smoothing` are ignored.
pub fn field_buffer(params: &HeroParams) -> Vec<f32> {
    let field = noise::field(
        params.seed,
        params.width,
        params.height,
        NoiseParams {
            octaves: params.octaves,
            frequency: params.frequency,
            persistence: params.persistence,
        },
    );
    let mut buffer = Vec::with_capacity(2 + field.values.len());
    buffer.push(params.width as f32);
    buffer.push(params.height as f32);
    buffer.extend_from_slice(&field.values);
    buffer
}

/// JavaScript entry point for [`field_buffer`]; seed as an `f64`, returns a
/// `Float32Array`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn field(
    seed: f64,
    width: usize,
    height: usize,
    octaves: u32,
    frequency: f32,
    persistence: f32,
) -> Vec<f32> {
    field_buffer(&HeroParams {
        seed: seed as u64,
        width,
        height,
        octaves,
        frequency,
        persistence,
        levels: 0,
        smoothing: 0,
    })
}

/// JavaScript entry point for [`relief_contour_buffer`]; seed as an `f64`, the
/// letter mask as a `Float32Array`, returns a `Float32Array`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn generate_relief(
    seed: f64,
    width: usize,
    height: usize,
    octaves: u32,
    frequency: f32,
    persistence: f32,
    levels: u32,
    smoothing: u32,
    mask: &[f32],
    relief: f32,
) -> Vec<f32> {
    relief_contour_buffer(
        &HeroParams {
            seed: seed as u64,
            width,
            height,
            octaves,
            frequency,
            persistence,
            levels,
            smoothing,
        },
        mask,
        relief,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(seed: u64) -> HeroParams {
        HeroParams {
            seed,
            width: 48,
            height: 32,
            octaves: 5,
            frequency: 3.0,
            persistence: 0.5,
            levels: 12,
            smoothing: 3,
        }
    }

    fn sample() -> Vec<f32> {
        contour_buffer(&params(42))
    }

    /// Walk the flat buffer back into `(level, coordinates)` polylines.
    fn decode(buffer: &[f32]) -> Vec<(u32, Vec<(f32, f32)>)> {
        let mut polylines = Vec::new();
        let mut cursor = 0;
        let polyline_count = buffer[cursor] as usize;
        cursor += 1;
        for _ in 0..polyline_count {
            let level = buffer[cursor] as u32;
            let point_count = buffer[cursor + 1] as usize;
            cursor += 2;
            let mut points = Vec::with_capacity(point_count);
            for _ in 0..point_count {
                points.push((buffer[cursor], buffer[cursor + 1]));
                cursor += 2;
            }
            polylines.push((level, points));
        }
        polylines
    }

    #[test]
    fn same_seed_reproduces_buffer() {
        assert_eq!(sample(), sample());
    }

    #[test]
    fn different_seed_changes_buffer() {
        assert_ne!(contour_buffer(&params(1)), contour_buffer(&params(2)));
    }

    #[test]
    fn coordinates_are_normalized() {
        for (_, polyline) in decode(&sample()) {
            for (x, y) in polyline {
                assert!((0.0..=1.0).contains(&x), "x out of range: {x}");
                assert!((0.0..=1.0).contains(&y), "y out of range: {y}");
            }
        }
    }

    #[test]
    fn header_matches_polyline_count() {
        let buffer = sample();
        assert_eq!(buffer[0] as usize, decode(&buffer).len());
    }

    #[test]
    fn level_indices_stay_in_range() {
        let reference = params(42);
        for (level, _) in decode(&contour_buffer(&reference)) {
            assert!(level < reference.levels, "level {level} out of range");
        }
    }

    #[test]
    fn parity_with_native_core() {
        let reference = params(42);
        let field = noise::field(
            reference.seed,
            reference.width,
            reference.height,
            NoiseParams {
                octaves: reference.octaves,
                frequency: reference.frequency,
                persistence: reference.persistence,
            },
        );
        let inverse_width = 1.0 / (reference.width - 1) as f32;
        let inverse_height = 1.0 / (reference.height - 1) as f32;

        let mut expected: Vec<(u32, Vec<(f32, f32)>)> = Vec::new();
        for (level_index, level) in contour::level_values(reference.levels)
            .into_iter()
            .enumerate()
        {
            for line in contour::march(&field, level) {
                let smoothed = smooth::fit(&line, reference.smoothing);
                if smoothed.len() >= MIN_POINTS {
                    expected.push((
                        level_index as u32,
                        smoothed
                            .iter()
                            .map(|&(x, y)| {
                                (
                                    (x * inverse_width).clamp(0.0, 1.0),
                                    (y * inverse_height).clamp(0.0, 1.0),
                                )
                            })
                            .collect(),
                    ));
                }
            }
        }

        assert_eq!(decode(&sample()), expected);
    }

    #[test]
    fn field_buffer_carries_dimensions_and_normalized_values() {
        let reference = params(42);
        let buffer = field_buffer(&reference);
        assert_eq!(buffer[0] as usize, reference.width);
        assert_eq!(buffer[1] as usize, reference.height);
        assert_eq!(buffer.len(), 2 + reference.width * reference.height);
        assert!(buffer[2..].iter().all(|&v| (0.0..=1.0).contains(&v)));
    }

    #[test]
    fn field_buffer_is_deterministic() {
        assert_eq!(field_buffer(&params(7)), field_buffer(&params(7)));
    }

    #[test]
    fn relief_mask_changes_the_contours() {
        let reference = params(42);
        let flat = vec![0.0_f32; reference.width * reference.height];
        let mut bump = flat.clone();
        for value in bump.iter_mut().take(reference.width * reference.height / 2) {
            *value = 1.0;
        }
        let plain = relief_contour_buffer(&reference, &flat, 0.6);
        let raised = relief_contour_buffer(&reference, &bump, 0.6);
        assert_eq!(plain, contour_buffer(&reference), "a zero mask is a no-op");
        assert_ne!(plain, raised, "a relief mask must reshape the contours");
    }

    #[test]
    fn relief_contour_buffer_is_deterministic() {
        let reference = params(7);
        let mask = vec![0.4_f32; reference.width * reference.height];
        assert_eq!(
            relief_contour_buffer(&reference, &mask, 0.5),
            relief_contour_buffer(&reference, &mask, 0.5)
        );
    }

    #[test]
    fn degenerate_size_yields_empty_header() {
        let buffer = contour_buffer(&HeroParams {
            width: 1,
            height: 1,
            ..params(1)
        });
        assert_eq!(buffer, vec![0.0]);
    }
}

//! Iso-contour extraction from a scalar field via marching squares.
//!
//! Each contour is returned as a [`Polyline`] in field coordinates (the `x`
//! range is `0..width`, `y` is `0..height`). The SVG layer maps these to canvas
//! pixels.

use crate::geom::{Point, Polyline};
use crate::noise::Field;

/// Evenly spaced contour levels in the open interval `(0, 1)`.
pub fn level_values(count: u32) -> Vec<f32> {
    (1..=count)
        .map(|index| index as f32 / (count + 1) as f32)
        .collect()
}

/// Fraction along an edge where it crosses `level`, given the two endpoint
/// values. Falls back to the midpoint on a degenerate (flat) edge.
fn interp(v0: f32, v1: f32, level: f32) -> f32 {
    let denominator = v1 - v0;
    if denominator.abs() < f32::EPSILON {
        0.5
    } else {
        ((level - v0) / denominator).clamp(0.0, 1.0)
    }
}

/// Extract iso-contour line segments from `field` at a single `level`, one
/// two-point [`Polyline`] per crossing. [`march`] stitches these into connected
/// curves.
fn cell_segments(field: &Field, level: f32) -> Vec<Polyline> {
    let mut segments = Vec::new();
    if field.width < 2 || field.height < 2 {
        return segments;
    }

    for gy in 0..field.height - 1 {
        for gx in 0..field.width - 1 {
            let a = field.at(gx, gy);
            let b = field.at(gx + 1, gy);
            let c = field.at(gx + 1, gy + 1);
            let d = field.at(gx, gy + 1);

            let case = (((a >= level) as u8) << 3)
                | (((b >= level) as u8) << 2)
                | (((c >= level) as u8) << 1)
                | ((d >= level) as u8);

            let (gxf, gyf) = (gx as f32, gy as f32);
            let top: Point = (gxf + interp(a, b, level), gyf);
            let right: Point = (gxf + 1.0, gyf + interp(b, c, level));
            let bottom: Point = (gxf + interp(d, c, level), gyf + 1.0);
            let left: Point = (gxf, gyf + interp(a, d, level));

            match case {
                1 | 14 => segments.push(vec![left, bottom]),
                2 | 13 => segments.push(vec![bottom, right]),
                3 | 12 => segments.push(vec![left, right]),
                4 | 11 => segments.push(vec![top, right]),
                6 | 9 => segments.push(vec![top, bottom]),
                7 | 8 => segments.push(vec![top, left]),
                5 => {
                    segments.push(vec![top, left]);
                    segments.push(vec![bottom, right]);
                }
                10 => {
                    segments.push(vec![top, right]);
                    segments.push(vec![bottom, left]);
                }
                _ => {} // 0 and 15: no crossing.
            }
        }
    }
    segments
}

/// Quantize a point so endpoints shared between adjacent cells match exactly.
fn endpoint_key(point: Point) -> (i64, i64) {
    (
        (point.0 * 1024.0).round() as i64,
        (point.1 * 1024.0).round() as i64,
    )
}

/// Stitch two-point segments that share endpoints into connected polylines, so
/// each contour is one continuous line that smooths cleanly.
fn stitch(segments: Vec<Polyline>) -> Vec<Polyline> {
    use std::collections::{HashMap, VecDeque};

    let mut adjacency: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (index, segment) in segments.iter().enumerate() {
        adjacency
            .entry(endpoint_key(segment[0]))
            .or_default()
            .push(index);
        adjacency
            .entry(endpoint_key(segment[segment.len() - 1]))
            .or_default()
            .push(index);
    }

    let other_end = |segment: &Polyline, key: (i64, i64)| -> Point {
        if endpoint_key(segment[0]) == key {
            segment[segment.len() - 1]
        } else {
            segment[0]
        }
    };

    let mut used = vec![false; segments.len()];
    let mut lines = Vec::new();
    for start in 0..segments.len() {
        if used[start] {
            continue;
        }
        used[start] = true;
        let mut chain: VecDeque<Point> = segments[start].iter().copied().collect();

        // Grow from each end, consuming any unused segment that touches it.
        for forward in [true, false] {
            loop {
                let tip = endpoint_key(if forward {
                    *chain.back().unwrap()
                } else {
                    *chain.front().unwrap()
                });
                let next = adjacency
                    .get(&tip)
                    .and_then(|candidates| candidates.iter().copied().find(|&j| !used[j]));
                let Some(index) = next else { break };
                used[index] = true;
                let point = other_end(&segments[index], tip);
                if forward {
                    chain.push_back(point);
                } else {
                    chain.push_front(point);
                }
            }
        }
        lines.push(chain.into_iter().collect());
    }
    lines
}

/// Extract stitched iso-contour polylines from `field` at a single `level`.
pub fn march(field: &Field, level: f32) -> Vec<Polyline> {
    stitch(cell_segments(field, level))
}

/// Extract contours at `count` evenly spaced levels.
pub fn contours(field: &Field, count: u32) -> Vec<Polyline> {
    level_values(count)
        .into_iter()
        .flat_map(|level| march(field, level))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A single-cell field with only the top-left corner above the level.
    fn single_corner_field() -> Field {
        Field {
            width: 2,
            height: 2,
            // (0,0)=1.0 is above 0.5; the other three corners are below.
            values: vec![1.0, 0.0, 0.0, 0.0],
        }
    }

    fn approx_contains(points: &[Point], target: Point) -> bool {
        points
            .iter()
            .any(|&(x, y)| (x - target.0).abs() < 1e-4 && (y - target.1).abs() < 1e-4)
    }

    #[test]
    fn known_field_yields_expected_crossing() {
        let lines = march(&single_corner_field(), 0.5);
        assert_eq!(lines.len(), 1, "one corner above ⇒ one segment");
        let segment = &lines[0];
        assert_eq!(segment.len(), 2);
        // Crossings are at the midpoints of the top and left edges.
        assert!(approx_contains(segment, (0.5, 0.0)));
        assert!(approx_contains(segment, (0.0, 0.5)));
    }

    #[test]
    fn more_levels_do_not_decrease_polyline_count() {
        let field = crate::noise::field(3, 48, 48, crate::noise::NoiseParams::default());
        let few = contours(&field, 6).len();
        let many = contours(&field, 18).len();
        assert!(many >= few, "more levels: {many} >= {few}");
    }

    #[test]
    fn out_of_range_level_yields_no_contour() {
        // Field values are in [0,1]; a level of 2.0 crosses nothing.
        let lines = march(&single_corner_field(), 2.0);
        assert!(lines.is_empty());
    }
}

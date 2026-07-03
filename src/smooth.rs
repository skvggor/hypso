//! Polyline smoothing via Chaikin corner-cutting.
//!
//! Marching squares emits faceted polylines; Chaikin subdivision rounds the
//! corners so the SVG layer can emit staircase-free curves. Every generated
//! point is a convex combination of the input, so the result stays inside the
//! input's convex hull (and bounding box).

use crate::geom::Polyline;

/// Smooth `polyline` with `iterations` of Chaikin corner-cutting. The two
/// endpoints are preserved; zero iterations returns the input unchanged.
pub fn fit(polyline: &Polyline, iterations: u32) -> Polyline {
    let mut current = polyline.clone();
    for _ in 0..iterations {
        if current.len() < 2 {
            break;
        }
        let mut next = Vec::with_capacity(current.len() * 2);
        next.push(current[0]);
        for window in current.windows(2) {
            let (p, q) = (window[0], window[1]);
            // Cut each corner at 1/4 and 3/4 along the segment.
            next.push((0.75 * p.0 + 0.25 * q.0, 0.75 * p.1 + 0.25 * q.1));
            next.push((0.25 * p.0 + 0.75 * q.0, 0.25 * p.1 + 0.75 * q.1));
        }
        next.push(current[current.len() - 1]);
        current = next;
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Polyline {
        vec![(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]
    }

    #[test]
    fn smoothing_increases_vertex_density() {
        let input = square();
        let smoothed = fit(&input, 2);
        assert!(
            smoothed.len() > input.len(),
            "expected denser polyline: {} > {}",
            smoothed.len(),
            input.len()
        );
    }

    #[test]
    fn stays_within_bounding_box() {
        let input = square();
        let smoothed = fit(&input, 3);
        let within = smoothed
            .iter()
            .all(|&(x, y)| (0.0..=4.0).contains(&x) && (0.0..=4.0).contains(&y));
        assert!(within, "corner-cutting must stay inside the convex hull");
    }

    #[test]
    fn zero_iterations_is_pass_through() {
        let input = square();
        assert_eq!(fit(&input, 0), input);
    }
}

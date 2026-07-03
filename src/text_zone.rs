//! Reserved text zones and feathered pattern exclusion.
//!
//! Zones and contour points share a common coordinate space (the field is
//! generated at the canvas aspect ratio, so grid coordinates map directly to
//! the canvas). Contours inside a zone are dropped; contours that come within
//! `feather` of a zone are dimmed so lines fade out near the text.

use serde::{Deserialize, Serialize};

use crate::geom::{Point, Polyline};

/// A rectangular region reserved for text, in field coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextZone {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
}

impl TextZone {
    /// Whether `point` lies inside the rectangle.
    pub fn contains(&self, point: Point) -> bool {
        point.0 >= self.x
            && point.0 <= self.x + self.width
            && point.1 >= self.y
            && point.1 <= self.y + self.height
    }

    /// Euclidean distance from an outside `point` to this rectangle (0 if inside).
    fn distance(&self, point: Point) -> f32 {
        let dx = (self.x - point.0)
            .max(point.0 - (self.x + self.width))
            .max(0.0);
        let dy = (self.y - point.1)
            .max(point.1 - (self.y + self.height))
            .max(0.0);
        (dx * dx + dy * dy).sqrt()
    }
}

/// A contiguous run of contour points sharing one opacity after feathering.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub points: Polyline,
    pub opacity: f32,
}

/// Opacity for a point outside all zones: `1.0` away from text, fading toward
/// `0.0` as it approaches a zone edge within `feather`.
fn point_opacity(point: Point, zones: &[TextZone], feather: f32) -> f32 {
    if feather <= 0.0 {
        return 1.0;
    }
    zones
        .iter()
        .map(|zone| (zone.distance(point) / feather).clamp(0.0, 1.0))
        .fold(1.0_f32, f32::min)
}

/// Exclude the pattern from `zones` with a feathered margin, returning drawable
/// runs (each with at least two points).
///
pub fn reserve(polylines: &[Polyline], zones: &[TextZone], feather: f32) -> Vec<Run> {
    let mut runs = Vec::new();
    for polyline in polylines {
        let mut points: Polyline = Vec::new();
        let mut opacity = 1.0_f32;
        let flush = |points: &mut Polyline, opacity: &mut f32, runs: &mut Vec<Run>| {
            if points.len() >= 2 {
                runs.push(Run {
                    points: std::mem::take(points),
                    opacity: *opacity,
                });
            } else {
                points.clear();
            }
            *opacity = 1.0;
        };

        for &point in polyline {
            if zones.iter().any(|zone| zone.contains(point)) {
                flush(&mut points, &mut opacity, &mut runs);
                continue;
            }
            opacity = opacity.min(point_opacity(point, zones, feather));
            points.push(point);
        }
        flush(&mut points, &mut opacity, &mut runs);
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zone() -> TextZone {
        TextZone {
            x: 10.0,
            y: 10.0,
            width: 10.0,
            height: 10.0,
            text: "hi".into(),
        }
    }

    #[test]
    fn zone_stores_requested_bounds() {
        let zone = zone();
        assert!(zone.contains((15.0, 15.0)));
        assert!(!zone.contains((5.0, 5.0)));
        assert_eq!(
            (zone.x, zone.y, zone.width, zone.height),
            (10.0, 10.0, 10.0, 10.0)
        );
    }

    #[test]
    fn vertex_inside_zone_is_excluded() {
        let poly = vec![(0.0, 15.0), (15.0, 15.0), (30.0, 15.0)];
        let runs = reserve(&[poly], &[zone()], 5.0);
        let inside_kept = runs
            .iter()
            .flat_map(|run| run.points.iter())
            .any(|&(x, y)| x == 15.0 && y == 15.0);
        assert!(!inside_kept, "point inside the zone must be dropped");
    }

    #[test]
    fn contour_fully_outside_is_kept() {
        let poly = vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)];
        let runs = reserve(std::slice::from_ref(&poly), &[zone()], 5.0);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].points, poly);
        assert_eq!(runs[0].opacity, 1.0);
    }

    #[test]
    fn contour_within_feather_is_dimmed() {
        // Points 2–4 units to the right of the zone edge, within feather = 5.
        let poly = vec![(22.0, 15.0), (23.0, 16.0), (24.0, 17.0)];
        let runs = reserve(&[poly], &[zone()], 5.0);
        assert!(
            runs.iter().any(|run| run.opacity < 1.0),
            "lines near a zone should be dimmed"
        );
    }
}

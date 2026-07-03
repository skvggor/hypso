//! Shared 2D geometry types used across the generative core.

/// A point in field/grid coordinates.
pub type Point = (f32, f32);

/// An ordered sequence of points forming an open or closed line.
pub type Polyline = Vec<Point>;

# pattern-generation

## Purpose

Generate the topographic pattern: a deterministic, seeded fractal noise field, iso-contour extraction, curve smoothing to Bézier paths, and level-based stroke widths.

## Requirements

### Requirement: Deterministic fractal noise field

The system SHALL generate a 2D scalar field from multi-octave fractal (fBm)
noise that is fully determined by a seed and its structural parameters (octaves,
frequency), so the same inputs always produce the same field.

#### Scenario: Same seed reproduces the field

- **WHEN** the field is generated twice with the same seed and parameters
- **THEN** both fields are identical value-for-value

#### Scenario: Different seed changes the field

- **WHEN** the field is generated with two different seeds and otherwise equal parameters
- **THEN** the two fields differ in at least one sample

#### Scenario: Values are normalized

- **WHEN** a field is generated
- **THEN** every sample value lies within the inclusive range [0.0, 1.0]

### Requirement: Iso-contour extraction

The system SHALL extract iso-contours from the scalar field at evenly spaced
levels using marching squares, returning each contour as an ordered polyline.

#### Scenario: Known field yields expected contour

- **WHEN** marching squares runs over a small field with a known single threshold crossing
- **THEN** it returns one polyline whose vertices match the expected crossing points

#### Scenario: Level count controls density

- **WHEN** the requested number of levels increases with the field unchanged
- **THEN** the total number of extracted contour polylines does not decrease

#### Scenario: Empty crossings yield no contour

- **WHEN** a level falls entirely outside the field's value range
- **THEN** no polyline is produced for that level

### Requirement: Curve smoothing to Bézier paths

The system SHALL smooth each contour polyline into curved paths emitted as cubic
Bézier segments, so rendered lines show no straight-segment staircasing. The
smoothing strength SHALL be controllable.

#### Scenario: Smoothing increases vertex density

- **WHEN** a polyline is smoothed with one or more iterations
- **THEN** the result has more vertices than the input and follows its overall shape

#### Scenario: Smoothing introduces no self-intersection on a convex input

- **WHEN** a non-self-intersecting convex polyline is smoothed
- **THEN** the smoothed path remains free of self-intersections

#### Scenario: Zero smoothing is a pass-through

- **WHEN** smoothing runs with zero iterations
- **THEN** the output vertices equal the input vertices

### Requirement: Stroke width by contour level

The system SHALL assign stroke width based on the contour level (index
contours), never on color. Selected levels SHALL render thicker than the rest.

#### Scenario: Index contours are thicker

- **WHEN** stroke width is computed for a level that is a multiple of the index interval
- **THEN** the returned width is greater than the width for a non-index level

#### Scenario: Color is independent of level

- **WHEN** stroke widths are computed across all levels
- **THEN** the line color value is unchanged across levels

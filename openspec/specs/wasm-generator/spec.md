# wasm-generator

## Purpose

Expose the existing generative core (seeded field, contour polylines, relief) to JavaScript through an additive WebAssembly binding, preserving determinism and parity with the native core while leaving that core unmodified.

## Requirements

### Requirement: Browser-callable generative surface

The project SHALL expose a WebAssembly binding that lets JavaScript generate a
topographic contour set from a seed and basic parameters (field resolution,
level count, noise frequency). The binding SHALL return contour polylines as
normalized coordinates suitable for drawing on a canvas or SVG.

#### Scenario: Generate contours from a seed

- **WHEN** JavaScript calls the WASM entry point with a seed and parameters
- **THEN** it receives a set of contour polylines whose point coordinates are
  normalized to `[0, 1]`

#### Scenario: Parameter bounds are respected

- **WHEN** JavaScript requests a specific level count and field size
- **THEN** the returned data reflects those parameters without panicking

### Requirement: Elevation field and relief contours

The binding SHALL also expose the raw elevation field for a seed, and a relief
variant of the contour surface that raises the field by a caller-supplied mask
(`relief * mask[i]`, clamped) before contouring. This lets the page stamp the
wordmark into the terrain so the map's own contours bend around and cling to the
letters. Both SHALL reuse the same generative core and stay deterministic.

#### Scenario: Field exposed for sampling

- **WHEN** JavaScript requests the elevation field for a seed
- **THEN** it receives the field dimensions and every value normalized to `[0, 1]`

#### Scenario: Relief mask reshapes the contours

- **WHEN** JavaScript supplies a non-zero mask to the relief contour surface
- **THEN** the returned contours differ from the unmasked contours (they wrap the
  raised region), and a zero mask reproduces the plain contour output

### Requirement: Determinism preserved across the boundary

The WASM surface SHALL preserve the core's determinism: the same seed and
parameters SHALL always produce byte-identical contour output, matching the
native library's result for the same inputs.

#### Scenario: Same seed yields same output

- **WHEN** the WASM entry point is called twice with identical seed and
  parameters
- **THEN** both calls return identical polyline data

#### Scenario: Parity with native core

- **WHEN** the same seed and parameters are run through the native `contour`
  pipeline and through the WASM surface
- **THEN** the resulting polylines are equivalent

### Requirement: Core remains unmodified

The WASM binding SHALL reuse the existing `noise`, `contour`, `smooth`, and
`geom` functions without altering their signatures or behavior, and SHALL NOT
touch `svg::emit` or the desktop render pipeline. Adding the binding SHALL NOT
change existing native library tests or coverage.

#### Scenario: Native build unaffected

- **WHEN** the crate is built and tested for the native desktop target
- **THEN** existing library tests pass unchanged and no new native dependency is
  pulled into the desktop binary

#### Scenario: Binding is additive only

- **WHEN** the WASM module is built
- **THEN** it links against the existing pure generative functions and defines no
  fork of their logic

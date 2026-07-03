## ADDED Requirements

### Requirement: Gradient overlay

The system SHALL support an optional gradient overlay drawn over the composition,
with a user-controlled intensity. At zero intensity the overlay SHALL have no
visible effect.

#### Scenario: Zero intensity is a no-op

- **WHEN** the gradient overlay intensity is zero
- **THEN** the rendered output is identical to rendering without the overlay

#### Scenario: Higher intensity strengthens the overlay

- **WHEN** the gradient overlay intensity increases
- **THEN** the overlay's contribution to the output increases

### Requirement: Film grain

The system SHALL apply film grain as a deterministic, seeded raster
post-processing step over the rasterized image, with a user-controlled
intensity. Grain SHALL be sampled per output pixel so it covers the whole
composition uniformly — background, contour lines, and text alike — rather than
only large flat areas.

#### Scenario: Zero intensity leaves pixels unchanged

- **WHEN** grain is applied with zero intensity
- **THEN** the output pixels equal the input pixels

#### Scenario: Same seed reproduces grain

- **WHEN** grain is applied twice with the same seed and intensity to the same image
- **THEN** both outputs are identical

#### Scenario: Non-zero intensity alters the image

- **WHEN** grain is applied with non-zero intensity
- **THEN** the output differs from the input in at least one pixel

#### Scenario: Grain covers dark regions, not only the background

- **WHEN** grain is applied with non-zero intensity over dark, line-colored pixels
- **THEN** those pixels change too, so grain reads across the whole image

### Requirement: Effects applied identically to preview and export

The system SHALL run the same effects pipeline for the live preview and the
export, differing only in raster size, so the preview is WYSIWYG.

#### Scenario: Preview matches export pipeline

- **WHEN** a preview and an export are produced from the same configuration
- **THEN** both pass through the same gradient-overlay and grain steps

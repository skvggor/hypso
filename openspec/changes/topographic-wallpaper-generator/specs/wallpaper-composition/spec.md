## ADDED Requirements

### Requirement: Output formats and aspect ratios

The system SHALL support a fixed set of output formats: 16:9 desktop at UHD
(3840×2160), and mobile portrait at 9:16 (2160×3840) and 9:19.5 (1644×3556).
Each format SHALL define its pixel dimensions and aspect ratio.

#### Scenario: Desktop format dimensions

- **WHEN** the 16:9 desktop format is requested
- **THEN** its dimensions are 3840×2160

#### Scenario: Mobile portrait is taller than wide

- **WHEN** a mobile portrait format is requested
- **THEN** its height is greater than its width

### Requirement: Background fill

The system SHALL fill the entire canvas with a user-selected background color
behind the pattern.

#### Scenario: Background covers the canvas

- **WHEN** a configuration with a chosen background color is rendered
- **THEN** the assembled SVG contains a background rectangle spanning the full canvas in that color

### Requirement: Line color or gradient

The system SHALL render contour lines in either a single user-selected color or
a user-defined gradient applied uniformly across all lines. The line appearance
SHALL NOT vary by contour level.

#### Scenario: Single color applied to lines

- **WHEN** a single line color is configured
- **THEN** every contour stroke uses that color

#### Scenario: Gradient applied uniformly

- **WHEN** a line gradient is configured
- **THEN** the SVG defines one gradient and all contour strokes reference it

### Requirement: SVG as single source of truth

The system SHALL assemble background, contours, and reserved-text geometry into a
single SVG string that is the sole source for both the live preview and the
export; the two SHALL differ only in rasterization size.

#### Scenario: Preview and export share the SVG

- **WHEN** a preview and an export are produced from the same configuration
- **THEN** both rasterize the same SVG string and differ only in output dimensions

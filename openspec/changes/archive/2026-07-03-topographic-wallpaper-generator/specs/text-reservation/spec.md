## ADDED Requirements

### Requirement: User-defined text zones

The system SHALL let the user place one or more rectangular text zones on the
canvas, each positioned and sized by the user.

#### Scenario: Zone is positioned where requested

- **WHEN** the user adds a text zone at a given position and size
- **THEN** the configuration stores a rectangle with those bounds

### Requirement: Pattern exclusion around text zones

The system SHALL exclude the contour pattern from within each text zone, with a
feathered margin so lines fade out near the zone rather than being cut at a hard
edge. No contour SHALL be rendered inside the reserved rectangle.

#### Scenario: Contours inside a zone are removed

- **WHEN** a contour vertex falls inside a reserved text zone
- **THEN** that vertex is excluded from the rendered pattern

#### Scenario: Contours outside zones are kept

- **WHEN** a contour lies entirely outside all reserved zones
- **THEN** it is rendered unchanged

#### Scenario: Feathered margin near the edge

- **WHEN** a contour passes within the feather margin of a zone edge
- **THEN** its opacity is reduced toward the edge instead of cut abruptly

### Requirement: Montserrat text rendering

The system SHALL render user text inside text zones using the embedded
Montserrat font, requiring no system-installed fonts at runtime.

#### Scenario: Text renders with embedded font

- **WHEN** a text zone contains user text and the canvas is rendered
- **THEN** the text appears in Montserrat using the embedded font data

### Requirement: Dedicated text color

The system SHALL fill zone text with its own user-selected color, independent of
the contour line color or gradient.

#### Scenario: Text color is independent of line color

- **WHEN** a configuration sets distinct line and text colors
- **THEN** the rendered text uses the text color and no text is filled with the line color

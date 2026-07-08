## ADDED Requirements

### Requirement: Generative island silhouette

The hero runtime SHALL derive the island's silhouette from the engine's own
output: a closed contour polyline selected from the generated field (meeting
minimum area and aspect thresholds so the content safe zone fits), lightly
smoothed, re-plotted on every seed. When no suitable closed contour exists for
a seed, the runtime SHALL fall back to a smoothed default blob derived
deterministically from the same seed, so the island always renders and is
reproducible per seed.

#### Scenario: Silhouette changes with the seed

- **WHEN** a new generation is plotted
- **THEN** the island's silhouette is re-derived from that generation's field
  and differs across seeds

#### Scenario: Fallback silhouette

- **WHEN** a seed's field contains no closed contour meeting the thresholds
- **THEN** a deterministic fallback blob for that seed is used and the island
  renders normally

### Requirement: Island fused with the terrain

The hero SHALL stamp the island silhouette into the elevation field's relief
mask (combined with the wordmark before blurring) so the map's contour lines
bend around and hug the island from behind, the same way they ring the
wordmark.

#### Scenario: Contours hug the island

- **WHEN** the hero plots a generation with the island relief active
- **THEN** contour lines wrap the island's silhouette (denser near its edge)
  and the island surface reads clearly in front of them

### Requirement: Ink contrast floor

The generation palette's light stop SHALL have a lightness floor — the ink
shared by the contour gradient and island surface stays light enough that dark
text (`#0b0e12`) on the ink surface meets at least WCAG AA contrast for any
hue the palette can produce.

#### Scenario: Any generated ink passes AA

- **WHEN** any seed generates a palette
- **THEN** the contrast ratio between `#0b0e12` and the palette's light stop is
  at least 4.5:1

### Requirement: Ink transition policy

When the map re-inks, surfaces tinted by the ink SHALL transition according to
the trigger: on the automatic seed drift, the ink SHALL cross-fade slowly
(on the order of seconds); on an explicit user action (click, tap, or keyboard
shortcut), the ink SHALL switch immediately with the replot.

#### Scenario: Slow cross-fade on drift

- **WHEN** the automatic drift plots a new generation
- **THEN** the island and islet surfaces cross-fade to the new ink over a slow
  tween rather than snapping

#### Scenario: Immediate switch on user action

- **WHEN** the visitor explicitly requests a new map
- **THEN** the tinted surfaces adopt the new ink immediately with the replot

### Requirement: Cursor follower

On devices with a fine pointer and motion allowed, the hero SHALL render a
purely decorative cartographic cursor follower (a registration cross or ring
in the current ink) that trails the native cursor with the same inertia as the
parallax lerp. The follower SHALL NOT hide or replace the native cursor, SHALL
NOT react to hover targets, SHALL ignore pointer events, SHALL hide when the
pointer leaves the window, SHALL NOT render on coarse-pointer devices, and
SHALL be removed under `prefers-reduced-motion: reduce`.

#### Scenario: Follower trails with inertia

- **WHEN** a visitor moves a fine pointer across the page
- **THEN** the follower eases toward the pointer's position with visible lag
- **AND** the native cursor remains visible and unmodified

#### Scenario: Follower absent when inapplicable

- **WHEN** the device has only a coarse pointer, or reduced motion is set, or
  the pointer leaves the window
- **THEN** no follower is rendered

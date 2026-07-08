# live-hero

## Purpose

Drive the browser-side hero runtime that consumes `wasm-generator` output and animates it with GSAP — stroke-draw, wordmark relief, seed drift, and pointer parallax — with reduced-motion and load-failure safeguards.

## Requirements

### Requirement: Living topographic hero

The hero SHALL render contour polylines produced by `wasm-generator` and animate
their appearance with GSAP as a progressive stroke-draw, so the field visibly
builds itself on load rather than appearing as a static image.

#### Scenario: Hero draws on load

- **WHEN** the page finishes loading with motion allowed and the runtime
  available
- **THEN** the contour lines animate into view via a stroke-draw sequence
- **AND** the final frame matches the deterministic output for the active seed

### Requirement: Wordmark fused with the terrain

The hero SHALL stamp the wordmark into the elevation field (via the relief
surface of `wasm-generator`) so the map's own contour lines bend around and cling
to the letters instead of a font sitting on top. A legible wordmark SHALL read in
front, occluding the lines within the letter shapes, while the hugging contours
render behind it.

#### Scenario: Contours hug the wordmark

- **WHEN** the hero plots a generation with the wordmark relief active
- **THEN** contour lines wrap the letters (denser near their edges) and the
  wordmark reads clearly in front of them

### Requirement: Seed drift and continuity

The hero SHALL evolve over time by drifting between seeds (or morphing the field)
so the artwork continuously changes, demonstrating the generator's range without
any user input. Transitions SHALL be smooth and free of visible popping.

#### Scenario: Field evolves over time

- **WHEN** the hero has been running for several seconds
- **THEN** the topographic field has visibly changed from its initial state
- **AND** the transition between states is animated, not an instant cut

### Requirement: Parallax and depth response

The hero SHALL provide lively, sensitive parallax/depth motion driven by pointer
movement on devices with a pointer AND by **device orientation (gyroscope)** on
touch devices, so the sheet reacts to tilting a phone. On touch platforms that
gate the motion sensor behind a permission (e.g. iOS), the page SHALL request it
via a user gesture and continue to work if it is denied. It SHALL degrade
gracefully to a coherent, still-animated composition when neither input is
available.

#### Scenario: Pointer parallax

- **WHEN** a visitor moves the pointer over the hero
- **THEN** layers of the artwork shift to convey depth, responsively

#### Scenario: Device-orientation parallax on mobile

- **WHEN** a visitor tilts a touch device whose motion sensor is available (or
  granted)
- **THEN** the hero's layers shift with the tilt, conveying depth without a
  pointer

#### Scenario: No motion input available

- **WHEN** neither pointer nor granted device orientation is available
- **THEN** the hero remains coherent and animated without requiring either input

### Requirement: Motion and failure safeguards

The hero runtime SHALL stop all animation and yield to the static poster when
`prefers-reduced-motion: reduce` is set, when the WASM module fails to load, or
when GSAP fails to load. It SHALL avoid layout shift when swapping between live
and poster states.

#### Scenario: Reduced motion halts animation

- **WHEN** `prefers-reduced-motion: reduce` is active
- **THEN** the hero runtime does not start any continuous animation and the
  poster is shown

#### Scenario: Graceful degradation

- **WHEN** the WASM or GSAP asset is unavailable at runtime
- **THEN** the hero swaps to the static poster with no layout shift and no
  uncaught errors

### Requirement: Deliberate wordmark reveal

On load and on each new generation, the wordmark's transparency SHALL fade out
over a deliberate multi-second window — the text ramps from transparent to fully
opaque over roughly a few seconds rather than snapping opaque — so the letters
visibly settle into the sheet.

#### Scenario: Text lingers translucent before settling

- **WHEN** the hero plots a generation with motion allowed
- **THEN** the wordmark starts transparent and reaches full opacity only after a
  few seconds, not instantly

#### Scenario: Reduced motion skips the fade

- **WHEN** `prefers-reduced-motion: reduce` is active (static poster fallback)
- **THEN** no timed fade runs and the wordmark is shown fully opaque

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

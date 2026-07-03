## ADDED Requirements

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

The hero SHALL provide subtle parallax/depth motion tied to pointer movement (or
scroll), degrading gracefully to a static composition when pointer input is
unavailable.

#### Scenario: Pointer parallax

- **WHEN** a visitor moves the pointer over the hero
- **THEN** layers of the artwork shift subtly to convey depth

#### Scenario: No pointer available

- **WHEN** no pointer input is available (e.g. touch-only or keyboard-only)
- **THEN** the hero remains coherent and animated without requiring pointer input

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

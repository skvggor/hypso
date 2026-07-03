## MODIFIED Requirements

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

## ADDED Requirements

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

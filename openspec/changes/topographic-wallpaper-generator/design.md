## Context

`hypso` is a greenfield native desktop app that generates 4K
topographic-style wallpapers procedurally. It deliberately mirrors the sibling
project `acag` (Rust + Slint + resvg/tiny-skia, embedded Montserrat, software
renderer, AppImage + `.exe` release), reusing that proven stack so the only new
work is the generative core and its controls.

The visual target is the contour-map look: fractal-noise iso-lines flowing across
a flat background, lines of varying thickness, with room reserved for typography.
The user requires strict TDD (red → green → refactor) over the whole core.

## Goals / Non-Goals

**Goals:**

- Reproduce `acag`'s single-source-of-truth pipeline: one SVG string drives both
  the live preview and the export, differing only in raster size.
- Procedural, fully deterministic pattern (seeded) so output is reproducible and
  presets reload exactly.
- Smooth, staircase-free contour lines via Bézier curve fitting.
- Pattern that flows around user-placed text zones with a feathered margin.
- 4K PNG export for 16:9 desktop and mobile portrait formats.
- A pure-function core with ≥ 80% library line coverage, driven by TDD.

**Non-Goals:**

- No AI/diffusion generation.
- No SVG or non-PNG export.
- No applying the wallpaper to the desktop (no Hyprland/Windows integration).
- No color-by-altitude (only stroke width varies by level).
- No network, backend, or telemetry.

## Decisions

### Pattern is vector geometry, emitted into the shared SVG

The topographic pattern is iso-contours of an fBm noise field. Contours are
vector polylines, so they are emitted as `<path>` strokes into the same SVG that
carries background and text, preserving `acag`'s "preview == export" invariant
for all geometry. *Alternative considered:* rasterizing the pattern directly
(per-pixel): rejected because it would fork preview/export rendering and lose
resolution independence.

### Smoothing happens in geometry, not rasterization

Staircasing on curves comes from coarse marching-squares polylines, not from the
rasterizer (tiny-skia already anti-aliases strokes). So each polyline is smoothed
(Chaikin corner-cutting or Catmull-Rom → Bézier) and emitted as cubic Bézier
(`C`) commands, not straight `L` segments. Smoothing strength is a style
parameter. *Alternative considered:* simply increasing field resolution:
rejected because it multiplies path count without removing facets.

### Grain is a seeded raster post-process; the invariant becomes "same pipeline"

Film grain is inherently per-pixel, so it cannot live honestly in the SVG. The
invariant is therefore restated: preview and export run the **same pipeline**
(`SVG → Pixmap → effects`) at different sizes, not the same SVG. Grain is seeded
(deterministic) and sampled **per output pixel**, so it sits uniformly over the
whole composition (background, lines, and text). Each pixel averages two
independent hash samples, giving a triangular distribution with a bounded
amplitude: soft film texture rather than salt-and-pepper noise.
*Alternative considered:* coarser virtual grain cells scaled to resolution:
rejected because at 4K a cell exceeded a thin contour's width, leaving the lines
clean while only the background looked grainy. *Alternative considered:*
`<feTurbulence>` in SVG: rejected as slow at 4K in resvg, and grain in SVG units
stretches with scale.

### Text zones use feathered exclusion, not hard clip or field warp

Contours are excluded inside reserved rectangles with a feathered margin (opacity
fades near the edge). *Alternatives considered:* hard rectangular clip: rejected
as visually jarring against organic lines; noise-field domain warp so lines
repel: deferred as a future refinement (higher complexity for marginal gain).

### Parameters split into structural vs style for a responsive preview

Structural params (seed, octaves, frequency, number of levels) regenerate the
noise field and are debounced; style params (colors, stroke width, gradient,
grain intensity, smoothing iterations, text position) only re-render and stay
instant. This keeps the live preview smooth while dragging sliders.

### Preview rasterizes at its displayed physical size

The preview is rendered at the exact physical size it occupies on screen
(display area times the window scale factor, within performance bounds) instead
of a fixed raster size, so preview pixels map one-to-one to screen pixels and
the image stays crisp at any window size. Resizes trigger a debounced re-render.
*Alternative considered:* a fixed preview raster scaled by the GUI: rejected
because non-integer scaling blurs thin contour lines.

### Editor widgets are custom, flat and accessible

The standard Slint widgets carry rounded corners that cannot be styled away, so
the editor uses a small custom widget set (button, slider, text field, checkbox,
combo box, collapsible section, color field, info tooltip) with square corners,
short animated state transitions, keyboard operation via `FocusScope` (Tab
order, Enter and Space activation, arrow-key adjustment) and accessibility
metadata (role, label, value, state) on every control. Color fields combine
labeled RGB sliders, a live swatch and a hex or `r, g, b` text entry kept in
two-way sync. The export path is shown in a full-window overlay whose link
reveals the file in the platform file manager (Explorer selection on Windows,
containing folder elsewhere).

### Module layout mirrors `acag`: thin GUI, fat tested library

```
src/
  noise.rs       fBm field (seeded)                         [TDD]
  contour.rs     marching squares: field+level -> polylines [TDD]
  smooth.rs      polyline -> smoothed Bézier vertices       [TDD]
  format.rs      output formats / dimensions                [TDD]
  stroke.rs      level -> stroke width (index contours)     [TDD]
  text_zone.rs   reserved zones + feathered exclusion       [TDD]
  svg.rs         config -> single SVG string                [TDD]
  raster.rs      SVG -> Pixmap (resvg, embedded Montserrat)  reused from acag
  grain.rs       seeded grain over Pixmap                    [TDD]
  export.rs      Pixmap -> PNG, output dir, no overwrite    [TDD]
  preset.rs      TOML save/load round-trip                  [TDD]
  config.rs      structural + style parameters
  lib.rs         library root (all logic; what tests cover)
  main.rs        thin Slint GUI glue                         not TDD
ui/app.slint     editor + live preview
examples/gallery.rs   committed visual samples (quality check by eye)
```

### TDD pact

```
✓ red → green → refactor on every pure-function core module
✓ determinism (seed) is a first-class test for noise, grain, export, presets
✓ library line coverage ≥ 80% (acag's gate, enforced in CI)
✗ no full SVG/PNG snapshot strings as regression tests (brittle); assert
  structural invariants instead (path count, viewBox, seeded equality, masking
  correctness on tiny known fields)
✗ visual quality is validated by examples/gallery.rs + human eye, outside TDD
```

## Risks / Trade-offs

- **Marching-squares + smoothing produce many path segments at 4K** → preview
  could lag. Mitigation: preview at reduced size; split structural vs style
  params so most edits skip field regeneration; debounce structural changes.
- **Grain scaling between preview size and 4K may not match perfectly** →
  WYSIWYG drift. Mitigation: scale grain granularity by output resolution and
  validate the preview-vs-export match in the gallery.
- **Feathered exclusion math is the most error-prone geometry** → lines could
  bleed into text or cut too hard. Mitigation: cover it with TDD on small known
  fields and explicit edge/margin scenarios before wiring the GUI.
- **resvg gradient/filter coverage** → an unsupported feature could surface late.
  Mitigation: keep effects minimal (linear gradient overlay + raster grain),
  both within resvg/tiny-skia's well-supported set.
- **TDD over-reach into visual code** → wasted, brittle tests. Mitigation: the
  TDD pact above explicitly bounds scope to mechanics, not aesthetics.

## Open Questions

- Stroke-width mapping detail: fixed index interval (every Nth contour thick) vs
  continuous width by level: decide during `stroke.rs` implementation; both
  satisfy the spec.
- Whether mobile 9:19.5 ships in the first cut or is added once 16:9 and 9:16 are
  validated.

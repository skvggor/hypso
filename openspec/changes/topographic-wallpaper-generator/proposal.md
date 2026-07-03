## Why

There is no quick way to produce clean, original 4K wallpapers in the topographic
contour-map style — the look of fractal-noise iso-lines flowing across a flat
background, with room reserved for typography. Stock images are watermarked and
not customizable, and AI generators are non-deterministic and heavy. A small
native app that generates this style procedurally gives reproducible, fully
controllable output (color, density, filters, text placement) as a single
exportable PNG.

## What Changes

- New native desktop app (`hypso`), Rust + Slint, single binary for
  Linux and Windows, mirroring the proven `acag` stack (resvg/tiny-skia
  rasterization, embedded Montserrat, software renderer, AppImage + `.exe`
  release pipeline).
- Procedural **topographic pattern**: multi-octave fractal (fBm) noise field →
  iso-contours via marching squares → curve-smoothed Bézier strokes. Fully
  deterministic from a seed. Stroke thickness varies by contour level (index
  contours), never color.
- **Composition controls**: selectable background color, line color or gradient,
  output in 16:9 desktop (UHD 3840×2160) and mobile portrait (9:16, 9:19.5).
- **Text reservation zones**: user places rectangular regions for Montserrat
  text; the pattern flows around them (feathered exclusion) so lines never cross
  into the text area.
- **Effects**: gradient overlay and film grain, each with an intensity control.
- **PNG-only export** at 4K, plus a live WYSIWYG preview and TOML presets.
- Strict **TDD** (red → green → refactor) on the whole pure-function core, with
  determinism as a first-class tested property and library coverage ≥ 80%.

## Capabilities

### New Capabilities

- `pattern-generation`: deterministic fractal-noise field, iso-contour extraction
  (marching squares), curve smoothing to Bézier paths, and level-based stroke
  width.
- `wallpaper-composition`: output formats/aspect ratios, background fill, line
  color/gradient, and SVG assembly as the single source of truth for preview and
  export.
- `text-reservation`: user-defined text zones with Montserrat rendering and
  feathered pattern exclusion so contours do not enter reserved regions.
- `image-effects`: gradient overlay and film-grain raster post-processing, each
  with intensity, applied identically to preview and export.
- `png-export`: 4K PNG rasterization and file output to a predictable directory,
  deterministic for a given configuration.
- `interactive-editor`: Slint live preview that runs the same pipeline as export,
  plus save/load of named presets as TOML.

### Modified Capabilities

<!-- None — greenfield project, no existing specs. -->

## Impact

- New repository scaffolding: `Cargo.toml` (edition 2024), `build.rs` (Slint
  compile + Windows icon), `src/` library + thin `main.rs`, `ui/app.slint`,
  embedded Montserrat assets, `examples/gallery.rs` for visual samples.
- Dependencies (latest stable): `slint`, `resvg`/`usvg`/`tiny-skia`,
  `ttf-parser`, `fastrand` (seeded noise/grain), `serde` + `toml` (presets),
  `dirs`, `anyhow`; `winresource` on Windows.
- CI/release reused from `acag`: fmt + clippy `-D warnings` + tests + coverage
  ≥ 80% gate; `v*` tag builds AppImage + tarball + standalone `.exe`.
- No backend, network, or desktop-integration surface; output is PNG files only.

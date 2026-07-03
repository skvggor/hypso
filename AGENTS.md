# AGENTS.md

Guidance for AI agents working in this repository.

## What this is

`hypso` is a native desktop app that generates 4K **topographic-style
wallpapers**: fractal contour-map line art over a flat background, with reserved
zones for typography. Rust + [Slint](https://slint.dev) for the GUI,
[resvg](https://github.com/linebender/resvg) / tiny-skia for rasterization.
Single binary named `hypso`. Sibling project of `acag`.

## Core invariant

`svg::emit(&Config) -> String` is the single source of truth, and
`render::render_pipeline(&Config, longest_px)` is the one pipeline both the live
preview and the export run: `SVG → Pixmap → grain`. Only the raster size differs,
so the preview is WYSIWYG. Geometry (background, contours, text) lives in the SVG;
grain is the one honest raster post-step. Never fork rendering between preview and
export.

## Layout

```
src/
  config.rs    Config (structural vs style params) + presets serde
  format.rs    output formats / pixel dimensions (16:9, 9:16, 9:19.5)
  noise.rs     deterministic fBm field (seeded)
  contour.rs   marching squares: field+level → stitched polylines
  smooth.rs    Chaikin corner-cutting
  stroke.rs    level → stroke width (index contours)
  text_zone.rs reserved zones + feathered exclusion
  svg.rs       Config → single SVG (Catmull-Rom cubic Bézier paths)
  raster.rs    SVG → Pixmap/PNG (resvg + embedded Montserrat)
  grain.rs     seeded film grain over a Pixmap
  render.rs    the shared pipeline (svg → raster → grain) + RGBA for preview
  export.rs    PNG export, output dir, no overwrite
  preset.rs    TOML presets save/load/list
  util.rs      slug helper
  main.rs      thin Slint GUI glue (not unit-tested)
ui/app.slint   editor + live preview
examples/gallery.rs   committed visual samples (by-eye quality check)
```

The library holds all logic and is what tests cover; `main.rs` is thin glue.
Montserrat (Regular/Bold/Black) is embedded via `include_bytes!`.

## TDD pact

Built strictly test-first (red → green → refactor). Determinism (same seed ⇒ same
output) is a first-class tested property across `noise`, `grain`, `export`, and
`preset`. Library line coverage ≥ 80% (currently ~95%). Do **not** snapshot whole
SVG/PNG strings as regression tests; assert structural invariants. Visual quality
is validated by `examples/gallery.rs` + human eye, outside TDD.

## Commands

```sh
cargo run --release            # run the app
cargo test --lib               # fast library tests (no Slint)
cargo fmt --all --check        # formatting gate
cargo clippy --all-targets -- -D warnings   # lint gate
cargo llvm-cov --lib --fail-under-lines 80  # coverage gate
cargo run --example gallery    # regenerate docs/samples
```

## Platform & release

- Software renderer by default (`SLINT_BACKEND=winit-software`) so it runs without
  OpenGL. Edition 2024: `std::env::set_var` is `unsafe`; only call before threads.
- Output dirs override via `HYPSO_OUTPUT_DIR`; presets via `HYPSO_PRESETS_DIR`.
- `.github/workflows/release.yml` on `v*` tags builds a Linux AppImage + tarball
  and a standalone Windows `.exe`. The AppImage `.desktop` `Categories` must use
  only freedesktop-registered values.
```

## Cargo features

The crate splits into three feature levels so the same core serves the desktop
app and the browser:

- **(no features)** — the pure generative core only (`noise`, `contour`,
  `smooth`, `geom`, `svg`, `stroke`, `config`, `format`, `text_zone`, `util`,
  `wasm`). Only `serde` is linked. This is what the WebAssembly build compiles.
- **`render`** — adds rasterization + file I/O (`resvg`, `anyhow`, `dirs`,
  `toml`): `raster`, `render`, `grain`, `export`, `preset`. Enough for the asset
  examples; never pulls in Slint or X11.
- **`gui`** (default) — the full desktop app on top of `render` (`slint`, `open`,
  `fastrand`, `slint-build`). The `hypso` binary requires it.

`src/wasm.rs` re-exposes the pure core to JS via `wasm-bindgen` (compiled only
for `wasm32`); the pure `contour_buffer` builder is tested natively for
determinism and parity with `contour`. Do **not** widen the wasm surface to pull
in `render`.

## Landing page (`web/` → `dist/`)

`https://skvggor.github.io/hypso/` is a static site whose hero runs the real
engine compiled to WASM. Sources in `web/`; `examples/site.rs` generates the OG
image, poster, and icons with Hypso itself; `examples/build_site.rs` minifies
`index.html` and copies the rest into `dist/`. `.github/workflows/deploy-site.yml`
builds and deploys to Pages, scoped to site sources + the pure core (a GUI-only
change does not redeploy). Generated assets (`web/wasm/`, `web/assets/img/`,
`web/assets/icons/`, `dist/`) are gitignored and rebuilt in CI.

## 1. Project scaffolding (not TDD)

- [x] 1.1 Initialize `Cargo.toml` (edition 2024, `[[bin]]` name `hypso`), release profile (lto, codegen-units=1, strip) mirroring `acag`
- [x] 1.2 Add latest-stable dependencies: `slint` (renderer-software), `resvg`/`usvg`/`tiny-skia`, `ttf-parser`, `fastrand`, `serde`+`toml`, `dirs`, `anyhow`; `winresource` on Windows
- [x] 1.3 Add `build.rs` (compile `ui/app.slint`, embed Windows icon) and a minimal `ui/app.slint`
- [x] 1.4 Embed Montserrat font assets via `include_bytes!`; create `src/lib.rs` + thin `src/main.rs`
- [x] 1.5 Port CI (`fmt`, `clippy -D warnings`, `cargo test`, `llvm-cov --lib --fail-under-lines 80`) and `release.yml` (AppImage + tarball + `.exe`) from `acag`
- [x] 1.6 Define `config.rs` with structural vs style parameter split (no logic yet)

## 2. format.rs: output formats (red → green → refactor)

- [x] 2.1 RED: write failing tests: 16:9 ⇒ 3840×2160; mobile portrait height > width; 9:16 and 9:19.5 dimensions
- [x] 2.2 GREEN: implement `Format` enum + `dimensions()` to pass
- [x] 2.3 REFACTOR: tidy naming/derive traits; tests stay green

## 3. noise.rs: deterministic fBm field (red → green → refactor)

- [x] 3.1 RED: failing tests: same seed ⇒ identical field; different seed ⇒ differs; all samples in [0,1]
- [x] 3.2 GREEN: implement seeded multi-octave fBm `field(seed, params)` to pass
- [x] 3.3 REFACTOR: extract octave/frequency helpers; keep determinism tests green

## 4. contour.rs: marching squares (red → green → refactor)

- [x] 4.1 RED: failing tests: known small field + single threshold ⇒ expected polyline vertices; more levels ⇒ polyline count does not decrease; out-of-range level ⇒ no polyline
- [x] 4.2 GREEN: implement `march(field, level)` returning ordered polylines to pass
- [x] 4.3 REFACTOR: simplify case table / dedupe segment stitching; tests green

## 5. smooth.rs: Bézier curve smoothing (red → green → refactor)

- [x] 5.1 RED: failing tests: smoothing increases vertex density and follows shape; convex input stays self-intersection-free; zero iterations is pass-through
- [x] 5.2 GREEN: implement `fit(polyline, iterations)` (Chaikin / Catmull-Rom → Bézier) to pass
- [x] 5.3 REFACTOR: clarify the subdivision math; tests green

## 6. stroke.rs: width by contour level (red → green → refactor)

- [x] 6.1 RED: failing tests: index-level width > non-index width; line color unchanged across levels
- [x] 6.2 GREEN: implement `width_for(level, index_interval)` to pass
- [x] 6.3 REFACTOR: extract the index-contour rule; tests green

## 7. text_zone.rs: reserved zones + feathered exclusion (red → green → refactor)

- [x] 7.1 RED: failing tests: zone stores requested bounds; vertex inside zone excluded; contour fully outside kept; vertex within feather margin gets reduced opacity
- [x] 7.2 GREEN: implement zone model + `reserve(paths, zones)` feathered exclusion to pass
- [x] 7.3 REFACTOR: isolate the margin/opacity math; tests green

## 8. svg.rs: single-source SVG assembly (red → green → refactor)

- [x] 8.1 RED: failing tests: SVG has correct viewBox/dimensions; full-canvas background rect in chosen color; ≥1 contour `<path>`; single color ⇒ all strokes that color; gradient ⇒ one `<gradient>` referenced by all strokes (assert structure, never full-string snapshot)
- [x] 8.2 GREEN: implement `emit(config)` assembling background + Bézier contours + gradient/overlay + text to pass
- [x] 8.3 REFACTOR: split builders per layer; tests green

## 9. raster.rs: SVG → Pixmap (reuse acag, light tests)

- [x] 9.1 Port `acag`'s resvg pipeline + shared `fontdb` (Montserrat) and `render_to_pixmap(svg, longest_px)`
- [x] 9.2 Test: a minimal valid SVG rasterizes to a Pixmap of the expected dimensions

## 10. grain.rs: seeded raster post-process (red → green → refactor)

- [x] 10.1 RED: failing tests: intensity 0 ⇒ pixels unchanged; same seed+intensity ⇒ identical output; non-zero intensity ⇒ differs in ≥1 pixel
- [x] 10.2 GREEN: implement `apply(pixmap, seed, intensity)` with resolution-scaled granularity to pass
- [x] 10.3 REFACTOR: extract noise sampling; tests green

## 11. Effects pipeline integration (red → green → refactor)

- [x] 11.1 RED: failing test: gradient overlay intensity 0 ⇒ output equals no-overlay render
- [x] 11.2 GREEN: wire `SVG → Pixmap → gradient overlay → grain` as one pipeline used by both preview and export
- [x] 11.3 REFACTOR: single `render_pipeline(config, size)` entry; tests green

## 12. export.rs: PNG output (red → green → refactor)

- [x] 12.1 RED: failing tests: export 16:9 ⇒ PNG 3840×2160; same config exported twice ⇒ pixel-identical; existing filename ⇒ non-colliding name; file lands in output dir (override via env var)
- [x] 12.2 GREEN: implement `export_png(config)` + `output_dir()` to pass
- [x] 12.3 REFACTOR: share slug/no-overwrite helpers; tests green

## 13. preset.rs: TOML presets (red → green → refactor)

- [x] 13.1 RED: failing tests: config save→load round-trips equal; reloaded preset re-exports pixel-identical PNG; saved preset file exists in presets dir (override via env var)
- [x] 13.2 GREEN: implement serde TOML `save`/`load` + presets dir to pass
- [x] 13.3 REFACTOR: tidy serialization; tests green

## 14. Slint GUI (thin glue, not TDD)

- [x] 14.1 Build `ui/app.slint`: flat square-cornered widget set, collapsible sections (Pattern / Colors / Lines / Effects / Text zone / Presets), color pickers with labeled RGB sliders, live swatch, and hex/RGB text entry
- [x] 14.2 Wire structural-vs-style updates: style edits re-render only; structural edits debounce + regenerate field
- [x] 14.3 Live preview rendered pixel-exact at the displayed size via the same `render_pipeline`; background export with progress state

## 15. Samples, coverage gate, release

- [x] 15.1 Add `examples/gallery.rs` generating committed visual samples for by-eye quality review
- [x] 15.2 Run full gate: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo llvm-cov --lib --fail-under-lines 80`
- [x] 15.3 Tag-driven release build: AppImage + tarball (Linux) and standalone `.exe` (Windows)

## 16. Editor polish and branding (post-apply iterations)

- [x] 16.1 Soften grain: per-pixel sampling, triangular distribution, bounded amplitude (TDD)
- [x] 16.2 Color pickers: labeled RGB sliders, live swatch, hex / `r, g, b` text entry in two-way real-time sync
- [x] 16.3 Flat square-cornered widget set (button, slider, edit, checkbox, combo) with animated microinteractions
- [x] 16.4 Collapsible sections; gradient toggle replaces the single line color; percent display for intensities and zone bounds
- [x] 16.5 Dedicated text color with serde default so older presets still load (TDD)
- [x] 16.6 Pixel-exact preview rendered at displayed physical size, re-rendered on resize
- [x] 16.7 Full-window export overlay: progress state, error state, content-sized card, clickable path that reveals the file (Explorer selection on Windows, containing folder elsewhere)
- [x] 16.8 InfoTip component with seed explanation; keyboard navigation (Tab, Enter/Space, arrows) and accessibility metadata on every control
- [x] 16.9 Text fields commit on focus loss as well as Enter
- [x] 16.10 Hypso logo generated by the engine (`examples/logo.rs`): icon SVG/PNGs/ico, window and header branding
- [x] 16.11 README gallery presets (`examples/gallery.rs` regenerates `docs/samples/`)

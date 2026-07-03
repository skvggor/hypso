## Why

Hypso is a finished, high-quality generator but has no public face. It needs a
landing page that drives visitors to the GitHub repository. Rather than a
conventional feature-list page, the landing should *be* the product: the same
Rust engine that renders 4K wallpapers, compiled to WebAssembly, generating a
living topographic field in the visitor's browser. This is the single most
faithful way to express the project's purpose — procedural, seed-reproducible
contour art — and it reuses the codebase instead of describing it.

> Open decisions carried in from analysis (confirm/adjust): live hero runs the
> **real Rust core compiled to WASM** (not a JS reimplementation), aesthetic is
> **dark cartographic / instrument**. Both are the recommended defaults and are
> reflected below.

## What Changes

- Add a static landing page under `web/`, minified into `dist/` and served by
  **GitHub Pages** — mirroring the sibling `terminal-webcam` project's layout
  (`.nojekyll`, `robots.txt`, `sitemap.xml`, `site.webmanifest`, OG image).
- Compile the existing generative core (`noise` + `contour` + `smooth` + `geom`)
  to **WebAssembly** via a new, additive `wasm-bindgen` surface. The core stays
  untouched; `svg::emit` and the desktop pipeline are not modified.
- The hero is a **live generative canvas**: contour polylines produced by the
  real engine, drawn and animated with **GSAP** (stroke-draw, seed drift,
  parallax). No feature list, no screenshots pretending to be a demo — one CTA
  to GitHub.
- Add a new `examples/site.rs` that uses the app's own render pipeline to
  generate the **1200×630 Open Graph image** and any static poster/fallback
  visuals, exactly as `gallery.rs` / `logo.rs` already generate committed assets.
- Vendor **GSAP locally** in `web/js/` (no CDN) so the page is fully
  self-contained and CSP-friendly.
- Excellent **SEO**: canonical, Open Graph, Twitter card, JSON-LD
  `SoftwareApplication`, sitemap, robots.
- Perfect **accessibility**: AAA contrast, visible focus, semantic landmarks,
  `aria-label` on the canvas, and `prefers-reduced-motion` that disables WASM
  animation and shows the static poster instead.
- Add a **GitHub Actions** workflow to build and deploy the site (Rust + WASM
  asset build → minify `web/` → `dist/` → Pages), scoped to relevant path
  changes, mirroring the reference project's deploy workflow.

## Capabilities

### New Capabilities
- `landing-page`: The public static page — content, structure, SEO metadata,
  accessibility contract, self-contained asset delivery, and the reduced-motion
  fallback.
- `wasm-generator`: The WebAssembly binding surface exposing the existing
  generative core (seeded field → contour polylines) to JavaScript, and its
  determinism / no-core-mutation guarantees.
- `live-hero`: The browser-side runtime that consumes `wasm-generator` output
  and animates it with GSAP (stroke-draw, seed drift, parallax), including the
  reduced-motion and load-failure behaviors.
- `site-build-deploy`: Reproducible build of static assets (Rust-generated OG
  image + WASM module) and minified `dist/` output, plus the GitHub Pages
  deployment workflow.

### Modified Capabilities
<!-- None. The existing desktop specs (pattern-generation, wallpaper-composition,
     image-effects, png-export, text-reservation, interactive-editor) are
     unchanged; the WASM surface only re-exposes existing pure functions. -->

## Impact

- **New code**: `web/` (HTML/CSS/JS + vendored GSAP), `dist/` (build output),
  `examples/site.rs`, a WASM binding module (e.g. `src/wasm.rs` behind a feature
  or a dedicated `wasm` crate target), `.github/workflows/deploy-site.yml`.
- **Build tooling**: adds a `wasm32-unknown-unknown` target + `wasm-bindgen`
  (and `wasm-pack` or `wasm-bindgen-cli`) to the CI site job only. Desktop build,
  tests, and coverage gates are unaffected.
- **Dependencies**: `wasm-bindgen` added as an optional/target-gated dependency;
  GSAP and opentype.js vendored as static assets (the latter reads the bundled
  Montserrat to stamp the wordmark into the terrain); no npm dependency in the
  Rust crate.
- **Untouched**: the generative core, `svg::emit` single-source-of-truth
  invariant, the Slint GUI, and all existing library tests.
- **External surface**: publishes `https://skvggor.github.io/hypso/` (repo is
  `github.com/skvggor/hypso`).

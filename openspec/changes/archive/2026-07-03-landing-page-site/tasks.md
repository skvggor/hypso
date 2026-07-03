## 1. Confirm scope decisions

- [x] 1.1 Confirm Pages URL / repo slug (`hypso` vs `wallpapers.gen`) and set the canonical base path used everywhere
- [x] 1.2 Confirm carried-over defaults: real Rust→WASM hero and dark cartographic aesthetic
- [x] 1.3 Decide `docs/` committed vs CI-built, and WASM build tool (`wasm-pack` vs `cargo` + `wasm-bindgen-cli`)
- [x] 1.4 Decide seed interactivity (autonomous drift only vs optional "new seed" affordance)

## 2. WASM generative surface (wasm-generator)

- [x] 2.1 Add a target-gated `wasm-bindgen` dependency so the desktop binary never links it
- [x] 2.2 Add `src/wasm.rs` (wasm-target only) exposing `generate(seed, field_size, levels, frequency)` reusing `noise`/`contour`/`smooth`/`geom` unchanged
- [x] 2.3 Return normalized `[0,1]` polylines as a flat `Float32Array` + offsets buffer
- [x] 2.4 Add a wasm build profile (small `opt-level`, strip) without altering the release profile
- [x] 2.5 Add a native parity test: same seed/params through `contour` vs the binding produce equivalent polylines
- [x] 2.6 Verify native `cargo test --lib`, `clippy`, and coverage gate still pass unchanged

## 3. Rust-generated static assets (examples/site.rs)

- [x] 3.1 Add `examples/site.rs` generating the 1200×630 OG image via `render::render_png` with an AAA-contrast title text zone
- [x] 3.2 Generate the static hero poster(s) used for reduced-motion / fallback in the dark cartographic palette
- [x] 3.3 Confirm reproducibility (same inputs → identical files) and document the `cargo run --example site` command

## 4. Page scaffold and content (landing-page)

- [x] 4.1 Create `web/` skeleton mirroring `terminal-webcam` (`index.html`, `css/`, `js/`, `assets/`, `.nojekyll`, `robots.txt`, `sitemap.xml`, `site.webmanifest`)
- [x] 4.2 Write semantic HTML (`header`/`main`/`footer`), one-line purpose statement, and a single GitHub CTA — no feature list
- [x] 4.3 Author dark cartographic CSS (AAA contrast, visible focus, mono type, HUD texture), mobile-first
- [x] 4.4 Add full SEO head: title, description, canonical, Open Graph (+image width/height/alt), Twitter card, JSON-LD `SoftwareApplication`
- [x] 4.5 Vendor GSAP locally into `web/js/` (free core only) and reference it same-origin
- [x] 4.6 Copy favicons/manifest icons and wire `site.webmanifest` `scope`/`start_url` to the confirmed base path

## 5. Live hero runtime (live-hero)

- [x] 5.1 Load the WASM module and draw returned polylines onto canvas/SVG
- [x] 5.2 GSAP stroke-draw sequence on load; final frame matches deterministic seed output
- [x] 5.3 Seed drift / field morph over time with smooth (non-popping) transitions
- [x] 5.4 Pointer parallax with graceful no-pointer fallback
- [x] 5.5 Reduced-motion, no-WASM, and no-GSAP paths swap to the static poster with no layout shift and no uncaught errors
- [x] 5.6 Provide `aria-label` + offscreen description for the hero; ensure content/CTA usable without it

## 6. Build and deploy (site-build-deploy)

- [x] 6.1 Add/extend the build script to minify `web/` → `docs/` and copy WASM + posters verbatim
- [x] 6.2 Add `.github/workflows/deploy-site.yml`: install wasm toolchain, build WASM, run `examples/site.rs`, minify, deploy `docs/` to Pages
- [x] 6.3 Scope workflow triggers to `web/**`, WASM source, `examples/site.rs`, and the workflow file; add `workflow_dispatch`
- [x] 6.4 Verify an unrelated (GUI-only) change does not trigger the site deploy

## 7. Verification

- [x] 7.1 Verify AAA contrast on all text and that the OG/poster share preview renders correctly
- [x] 7.2 Keyboard-only pass: every interactive element has visible focus and correct order; CTA activatable
- [x] 7.3 Accessibility/SEO audit: pa11y (axe-core + HTML CodeSniffer, WCAG2AA) reports no issues; JSON-LD valid, Open Graph complete, self-contained confirmed
- [x] 7.4 Load with external hosts blocked to confirm fully self-contained delivery
- [x] 7.5 Confirm reduced-motion and runtime-failure fallbacks on a real browser
- [x] 7.6 Update README to link the live site

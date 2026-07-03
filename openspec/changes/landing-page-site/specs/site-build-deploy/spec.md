## ADDED Requirements

### Requirement: Rust-generated static assets

The Open Graph image (1200×630) and any static poster/fallback visuals SHALL be
produced by the app's own render pipeline through a committed example
(`examples/site.rs`), analogous to the existing `gallery.rs` and `logo.rs`. The
generation SHALL be reproducible and runnable with a single Cargo command.

#### Scenario: Assets regenerate deterministically

- **WHEN** the site asset example is run
- **THEN** it writes the 1200×630 OG image and poster(s) using the Hypso engine
- **AND** re-running with the same inputs produces identical files

#### Scenario: OG image meets contrast target

- **WHEN** the generated OG image is inspected
- **THEN** any overlaid title text meets AAA contrast against its background

### Requirement: Reproducible WASM build

The build SHALL compile the `wasm-generator` surface to a `wasm32-unknown-unknown`
artifact plus its JS bindings, producing files consumable directly by the static
page with no bundler required at runtime.

#### Scenario: WASM artifact is produced

- **WHEN** the site build runs
- **THEN** a `.wasm` module and its generated JS glue are emitted into the web
  asset tree

### Requirement: Assembled static output

Source assets in `web/` SHALL be assembled into `dist/` for serving: `index.html`
minified, every other asset copied verbatim. The output SHALL include
`.nojekyll`, `robots.txt`, `sitemap.xml`, and `site.webmanifest`.

#### Scenario: Build populates dist

- **WHEN** the site build runs
- **THEN** `dist/` contains the minified `index.html`, the CSS/JS/assets copied
  verbatim, the WASM module, `.nojekyll`, `robots.txt`, `sitemap.xml`, and
  `site.webmanifest`

### Requirement: GitHub Pages deployment

A GitHub Actions workflow SHALL build the assets and deploy `dist/` to GitHub
Pages. It SHALL trigger only on changes to the site sources (`web/**`, the site
example, the WASM surface, and the workflow itself) and support manual dispatch.

#### Scenario: Deploy on site change

- **WHEN** a commit to the default branch changes site sources
- **THEN** the workflow builds the assets and publishes `dist/` to Pages

#### Scenario: Unrelated changes do not deploy

- **WHEN** a commit changes only unrelated source (e.g. desktop GUI code)
- **THEN** the site deploy workflow does not run

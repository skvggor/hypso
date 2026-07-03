## ADDED Requirements

### Requirement: Single-purpose conceptual page

The landing page SHALL present the project's purpose — procedural,
seed-reproducible topographic art — without a feature list, pricing, or
screenshot grid. It SHALL provide exactly one primary call to action pointing to
the GitHub repository.

#### Scenario: Page communicates purpose and links to source

- **WHEN** a visitor loads the page
- **THEN** the page displays the project name, a one-line statement of its
  purpose, and a living topographic hero
- **AND** a single primary link to `https://github.com/skvggor/hypso` is present
  and reachable by keyboard

#### Scenario: No feature enumeration

- **WHEN** the page content is inspected
- **THEN** there is no bulleted feature list, comparison table, or pricing block

### Requirement: SEO metadata

The page SHALL include complete, valid SEO metadata: a unique `<title>` and
`<meta name="description">`, a `<link rel="canonical">`, Open Graph tags
(`og:type`, `og:title`, `og:description`, `og:url`, `og:image` with width/height
and `og:image:alt`), Twitter card tags, and a JSON-LD `SoftwareApplication`
block. A `sitemap.xml` and `robots.txt` SHALL be served.

#### Scenario: Metadata is present and consistent

- **WHEN** the rendered HTML `<head>` is inspected
- **THEN** canonical, Open Graph, Twitter, and JSON-LD entries are present
- **AND** the `og:url` and canonical URL both equal the deployed Pages URL
- **AND** `og:image` references the 1200×630 image with matching
  `og:image:width`/`og:image:height`

#### Scenario: Crawlers are guided

- **WHEN** a crawler requests `/robots.txt`
- **THEN** it allows indexing and references `/sitemap.xml`

### Requirement: Accessibility contract

The page SHALL meet WCAG AAA text-contrast for body copy and headings, expose
visible keyboard focus on all interactive elements, use semantic landmarks
(`header`/`main`/`footer`), and provide a text alternative (`aria-label` or
equivalent) for the generative canvas. Color SHALL never be the sole carrier of
meaning.

#### Scenario: Keyboard-only navigation

- **WHEN** a user navigates with the Tab key only
- **THEN** every interactive element receives a visible focus indicator in order
- **AND** the primary GitHub link is reachable and activatable with Enter

#### Scenario: Assistive technology description

- **WHEN** a screen reader encounters the generative hero
- **THEN** it announces a concise description of the topographic artwork rather
  than reading raw canvas/SVG nodes

#### Scenario: AAA contrast

- **WHEN** foreground text is measured against its background
- **THEN** the contrast ratio is at least 7:1 for normal text and 4.5:1 for
  large text

### Requirement: Reduced-motion fallback

The page SHALL honor `prefers-reduced-motion: reduce` by disabling the animated
WebAssembly hero and presenting a static, equivalent poster image instead. The
page SHALL also remain fully usable if the WebAssembly module or GSAP fails to
load.

#### Scenario: User prefers reduced motion

- **WHEN** the visitor's OS/browser signals `prefers-reduced-motion: reduce`
- **THEN** no continuous animation runs
- **AND** a static topographic poster is shown in place of the live hero

#### Scenario: Runtime unavailable

- **WHEN** the WebAssembly module or GSAP fails to load
- **THEN** the static poster is shown and the page content and CTA remain usable

### Requirement: Self-contained delivery

The page SHALL serve all of its assets from the same origin, including styles,
scripts (GSAP included), fonts, images, and the WASM module. The page SHALL NOT
depend on any third-party CDN or external network request at runtime.

#### Scenario: No external requests

- **WHEN** the page is loaded with external hosts blocked
- **THEN** the page renders, styles apply, and the hero (or its poster fallback)
  displays without failed cross-origin requests

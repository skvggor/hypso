## ADDED Requirements

### Requirement: Island content presentation

The page's content SHALL live inside an **island** — the purpose statement,
seed, contour count, and Generate action in an organic closed shape filled with
the exact ink color of the current generation's contour lines, with all text
on it rendered in dark (`#0b0e12`). The island SHALL NOT use legend/terminal
vocabulary: no "Fig." captions, no dotted metadata rows, no thin-bordered
rectangular box. Hierarchy SHALL be carried by scale: the purpose statement in
bold uppercase display type and the seed as a large numeral, not small
metadata. The island content SHALL be regular DOM (HTML clipped by the shape),
preserving semantics and keyboard order.

#### Scenario: Content reads inside the island

- **WHEN** a visitor loads the page
- **THEN** the purpose statement, seed numeral, contour count, and Generate
  control render inside an organic shape filled with the map's line color
- **AND** all text on the island is dark and meets at least AA contrast
- **AND** no "Fig." caption or dotted metadata row is present

#### Scenario: Text stays within a safe zone

- **WHEN** any seed produces the island silhouette
- **THEN** the content is laid out inside a rectangular safe zone inscribed in
  the shape and never overflows the irregular edge

#### Scenario: Generate action radiates contour rings

- **WHEN** a visitor hovers or presses the Generate control (motion allowed)
- **THEN** concentric contour rings radiate from the control
- **AND** under `prefers-reduced-motion: reduce` no ring animation runs

### Requirement: Mobile-first layout

The page SHALL be designed mobile-first with an equivalent-quality experience
on desktop. On small viewports the map SHALL fill the screen with the island
anchored in the lower half and the downloads islet beside or below it; on
desktop the island SHALL sit in the map's margin region leaving the wordmark
and the majority of the map visible. Interacting with the island or islet
SHALL NOT plot a new seed; only the map surface (and the keyboard shortcut)
reseeds.

#### Scenario: Small viewport layout

- **WHEN** the page renders on a small viewport
- **THEN** the map fills the screen, the island anchors in the lower half, and
  the wordmark remains visible and unobstructed by the collapsed layout

#### Scenario: Island interaction never reseeds

- **WHEN** a visitor taps or clicks inside the island or islet
- **THEN** no new seed is plotted

## MODIFIED Requirements

### Requirement: Binary downloads

The landing page SHALL offer to download the desktop binaries for Linux and
Windows from a **downloads islet**: a smaller organic shape sibling to the main
island, sharing its ink surface and dark text. When the islet is opened it
SHALL resolve each platform link to the matching asset of the project's latest
release via the GitHub API (regardless of the asset's exact filename), falling
back to the releases page if the request fails or no release exists — so the
links never 404 and never depend on a fixed version in the markup. The
downloads SHALL be **revealed by a user action** (activating the islet), SHALL
show a per-OS icon beside each label, and SHALL follow the colors currently
rendered on the map. On small viewports the expanded downloads SHALL present
as a sheet that never covers the wordmark and dismisses via an explicit close
control and Escape. On touch devices it SHALL note that the builds are
desktop-only. It SHALL be keyboard-reachable with visible focus and meet AA
contrast.

#### Scenario: Links resolve to the latest release's assets

- **WHEN** a visitor opens the downloads islet and a release exists
- **THEN** each platform link points at the matching asset of the latest release,
  whatever it is named, and downloads it on activation
- **AND** if resolution fails, the links open the releases page instead of 404ing

#### Scenario: Desktop-only note on mobile

- **WHEN** the islet is opened on a touch device
- **THEN** it states that the binaries are desktop-only (Linux and Windows)

#### Scenario: Islet tinted by the map

- **WHEN** the visitor opens the downloads islet
- **THEN** it appears as an organic panel whose surface matches the current
  generation's ink, updating as the map re-inks

#### Scenario: Mobile sheet behavior

- **WHEN** the islet is expanded on a small viewport
- **THEN** the downloads present as a sheet that does not cover the wordmark
- **AND** the sheet closes via an explicit close control and via Escape

#### Scenario: Component integrates with the design and a11y

- **WHEN** the downloads islet is rendered
- **THEN** it is keyboard-reachable with visible focus and meets AA contrast —
  not a feature/pricing block

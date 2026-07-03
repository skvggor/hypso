## MODIFIED Requirements

### Requirement: Single-purpose conceptual page

The landing page SHALL present the project's purpose — procedural,
seed-reproducible topographic art — without a feature list, pricing, or
screenshot grid. It SHALL provide a primary call to action pointing to the GitHub
repository, and MAY provide per-platform binary download links as secondary
actions.

#### Scenario: Page communicates purpose and links to source

- **WHEN** a visitor loads the page
- **THEN** the page displays the project name, a one-line statement of its
  purpose, and a living topographic hero
- **AND** a primary link to `https://github.com/skvggor/hypso` is present and
  reachable by keyboard

#### Scenario: No feature enumeration

- **WHEN** the page content is inspected
- **THEN** there is no bulleted feature list, comparison table, or pricing block

## ADDED Requirements

### Requirement: Binary downloads

The landing page SHALL offer to download the desktop binaries for Linux and
Windows. The links SHALL point at the project's latest release and resolve to an
actual downloadable asset (not a 404), and SHALL be styled as part of the page's
cartographic design rather than a feature grid. They SHALL be keyboard-reachable
with visible focus and meet AA contrast.

#### Scenario: Download links resolve to release assets

- **WHEN** a visitor activates a platform download link
- **THEN** the browser downloads (or navigates to) the corresponding binary from
  the latest release
- **AND** each supported platform (Linux, Windows) has its own link

#### Scenario: Links integrate with the design

- **WHEN** the download links are rendered
- **THEN** they follow the page's legend/marginalia styling, keyboard focus, and
  AA-contrast rules — not a separate feature/pricing block

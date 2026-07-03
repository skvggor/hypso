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
Windows. When the component is opened it SHALL resolve each platform link to the
matching asset of the project's latest release via the GitHub API (regardless of
the asset's exact filename), falling back to the releases page if the request
fails or no release exists — so the links never 404 and never depend on a fixed
version in the markup. The downloads SHALL live in a distinct component that is
**revealed by a user action** (a toggle), SHALL show a per-OS icon beside each
label, and SHALL follow the colors currently rendered on the map. On touch
devices it SHALL note that the builds are desktop-only. It SHALL be
keyboard-reachable with visible focus and meet AA contrast.

#### Scenario: Links resolve to the latest release's assets

- **WHEN** a visitor opens the download component and a release exists
- **THEN** each platform link points at the matching asset of the latest release,
  whatever it is named, and downloads it on activation
- **AND** if resolution fails, the links open the releases page instead of 404ing

#### Scenario: Desktop-only note on mobile

- **WHEN** the component is opened on a touch device
- **THEN** it states that the binaries are desktop-only (Linux and Windows)

#### Scenario: Revealed component tinted by the map

- **WHEN** the visitor toggles the download component open
- **THEN** it appears as a distinct panel whose accents (border, icons) match the
  current generation's ink, updating as the map re-inks

#### Scenario: Component integrates with the design and a11y

- **WHEN** the download component is rendered
- **THEN** it follows the page's legend styling, is keyboard-reachable with
  visible focus, closes on Escape, and meets AA contrast — not a feature/pricing
  block

## Why

The landing page drives visitors to GitHub to read the source, but people who
just want to *run* Hypso have no direct path — the desktop binaries already exist
(the release workflow ships a Linux AppImage, a Linux tarball, and a Windows
`.exe`) yet the page never links to them. Separately, the live hero's motion is
too timid on phones (pointer parallax barely fires on touch) and the wordmark
fades to full opacity almost instantly; both undercut the "living sheet" feel.

## What Changes

- Add **binary download links** to the landing page — Linux and Windows — that
  point at the latest release and actually download. Style them as part of the
  cartographic legend, not a feature list.
- Publish **stable-named release assets** (version-independent filenames) so the
  page can link to `releases/latest/download/<name>` and the links keep working
  across versions.
- Make the **parallax more alive and sensitive**, especially on mobile: drive it
  from **device orientation (gyroscope)** on touch devices (with the required
  permission handling) in addition to pointer, and increase amplitude/responsiveness.
- Slow the wordmark's **reveal so its transparency takes a few seconds to
  disappear**: the text ramps from transparent to fully opaque over a deliberate
  multi-second window instead of snapping opaque.

## Capabilities

### New Capabilities
<!-- None. This builds on the existing landing-page and live-hero capabilities. -->

### Modified Capabilities
- `landing-page`: the single-CTA rule relaxes to also allow per-platform binary
  download links, and a new requirement covers those downloads.
- `live-hero`: the parallax requirement gains device-orientation input and higher
  sensitivity (mobile-first), and a new requirement covers the deliberate,
  multi-second wordmark reveal.

## Impact

- **Web**: `web/index.html` (download links in the legend), `web/css/style.css`
  (download styling), `web/js/main.js` (device-orientation parallax, tuned
  amplitude/lerp, slower reveal ramp).
- **Release**: `.github/workflows/release.yml` gains stable-named asset copies
  (e.g. `hypso-linux-x86_64.AppImage`, `hypso-linux-x86_64.tar.gz`,
  `hypso-windows-x86_64.zip`) alongside the versioned ones.
- **Accessibility**: iOS requires a user gesture to grant `DeviceOrientation`;
  the page must request it gracefully and never depend on it (pointer and a
  static-but-coherent fallback remain). Download links must be keyboard-reachable
  with visible focus and AA contrast.
- **Self-contained**: download links are user-initiated navigations to GitHub,
  not runtime fetches — the no-CDN/self-contained guarantee is preserved.
- **Untouched**: the Rust generative core, `wasm-generator` surface, and the
  desktop app.

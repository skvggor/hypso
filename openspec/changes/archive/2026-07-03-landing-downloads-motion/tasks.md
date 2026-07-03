## 1. Stable-named release assets

- [x] 1.1 In `.github/workflows/release.yml`, publish version-independent copies alongside the versioned assets: `hypso-linux-x86_64.AppImage`, `hypso-linux-x86_64.tar.gz`, `hypso-windows-x86_64.zip`
- [x] 1.2 Confirm the release step still uploads everything (`files: dist/*`) and the versioned assets are unchanged

## 2. Download links on the page (landing-page)

- [x] 2.1 Add per-platform download links (Linux AppImage, Linux tarball, Windows) to the legend, pointing at `https://github.com/skvggor/hypso/releases/latest/download/<stable-name>`, plus a small "all releases" link
- [x] 2.2 Style them as legend marginalia (hairline/mono, AA contrast, visible focus) — no button grid or feature list
- [x] 2.3 Ensure keyboard reachability and correct `download`/`rel` attributes; verify they degrade to the releases page if an asset is missing

## 3. Lively, mobile-sensitive parallax (live-hero)

- [x] 3.1 Raise parallax amplitude and lerp responsiveness for a livelier feel
- [x] 3.2 Add a `deviceorientation` listener mapping tilt (gamma/beta) to the parallax target, normalized and clamped
- [x] 3.3 Handle iOS `DeviceOrientationEvent.requestPermission()` from a user gesture (reuse tap/replot); keep working if denied/unavailable
- [x] 3.4 Throttle sensor updates and mark frames dirty so battery cost stays bounded

## 4. Deliberate wordmark reveal (live-hero)

- [x] 4.1 Give the wordmark its own opacity ramp (`word.reveal`) decoupled from the contour stroke-draw
- [x] 4.2 Tween it 0 → 1 over ~3–4 s with an ease that lingers translucent before settling; apply as the wordmark alpha
- [x] 4.3 Keep the wordmark fully opaque (no timed fade) in the reduced-motion / poster fallback

## 5. Verification

- [x] 5.1 Desktop: pointer parallax feels lively; wordmark takes a few seconds to reach full opacity
- [x] 5.2 Mobile: tilting shifts the layers (with permission granted); pointer/touch still works; layout intact
- [x] 5.3 Download links resolve to release assets (or the releases page pre-first-release); keyboard focus + AA contrast hold
- [x] 5.4 Accessibility/self-contained checks still pass (pa11y clean; no external runtime requests)
- [x] 5.5 Update README if the download story needs a note

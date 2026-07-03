## Context

The landing page is a self-contained static site whose hero runs the Rust core
in WebAssembly (see the archived `landing-page-site` change). It currently offers
one CTA (GitHub) and parallax driven only by `pointermove`. The desktop binaries
are published by `.github/workflows/release.yml` on `v*` tags, but with
**versioned** filenames (`hypso-<version>-linux-x86_64.tar.gz`,
`hypso-<tag>-x86_64.AppImage`, `hypso-<version>-windows-x86_64.zip`), so no stable
URL exists to link to.

## Goals / Non-Goals

**Goals:**
- Direct, working per-platform download links that survive new releases.
- Parallax that feels alive on desktop and, crucially, reacts to tilt on phones.
- A wordmark whose transparency takes a few seconds to resolve to opaque.
- Keep the page self-contained (no runtime CDN/fetch) and accessible (AA, focus).

**Non-Goals:**
- No feature list, changelog, or version selector on the page.
- No auto-detection that hides the "wrong" platform (show both; keep it simple).
- No new runtime dependency; no change to the Rust core or `wasm-generator`.

## Decisions

### D1 — Stable-named release assets + `releases/latest/download`
Add version-independent copies of each asset in `release.yml`
(`hypso-linux-x86_64.AppImage`, `hypso-linux-x86_64.tar.gz`,
`hypso-windows-x86_64.zip`) alongside the versioned ones, then link the page to
`https://github.com/skvggor/hypso/releases/latest/download/<stable-name>`. GitHub
resolves `latest/download/<name>` to the newest release's asset, so the links keep
working across versions.
- *Alternative — link to the `releases/latest` page*: always valid but drops the
  user on a list; kept as a secondary "all releases" link.
- *Alternative — fetch the latest release via the GitHub API at runtime*: gives
  exact URLs but is an external request that breaks the self-contained rule.
  Rejected.

### D2 — Downloads live in the legend, as marginalia
Render the links inside the existing legend block (or a sibling), styled like the
`source` link / hint — hairline, mono, AA contrast, visible focus. Labelled by
platform (Linux, Windows) with a small format note (AppImage / tarball / zip).
Not a button grid, not a feature list.

### D3 — Parallax: pointer + device orientation, tuned for mobile
Keep the pointer path; add a `deviceorientation` listener mapping tilt
(`gamma`/`beta`) to the same parallax target, normalized and clamped. Raise the
amplitude and the lerp factor so the motion reads as lively. On iOS,
`DeviceOrientationEvent.requestPermission()` must be called from a user gesture —
reuse the existing tap/replot gesture (or a one-time subtle "enable motion"
affordance) to request it; if denied or unavailable, pointer + the coherent
still-animated fallback remain. Throttle sensor updates and mark the frame dirty
so battery cost stays bounded.

### D4 — Deliberate wordmark reveal
Give the wordmark its own opacity ramp, decoupled from the contour stroke-draw:
a dedicated value (e.g. `word.reveal`) tweened 0 → 1 over ~3–4 s with an ease
that lingers translucent before settling, used as the wordmark's alpha. The
contour field keeps its existing draw-on. Under `prefers-reduced-motion` (poster
fallback) no fade runs.

## Risks / Trade-offs

- **iOS motion permission denied / not HTTPS** → pointer parallax and the
  coherent fallback still work; device orientation is a progressive enhancement,
  never required.
- **No release exists yet** → `latest/download/<name>` 404s until the first
  tagged release ships the stable-named assets. Mitigation: also link the
  `releases/latest` page; document that downloads light up after the first
  release built with the updated workflow.
- **Stable-named assets** must not clobber or break the existing versioned
  uploads → add copies, keep both; verify `files: dist/*` still uploads all.
- **Continuous sensor/parallax redraw** → throttle and only redraw on meaningful
  delta, matching the existing dirty-flag loop.
- **Reveal too slow feels broken** → keep it in the ~3–4 s range and ensure the
  contour field is already drawing so the page never looks frozen.

## Migration Plan

Additive and web-only except for the release workflow copies. Rollback = revert
the `web/` changes and the `release.yml` stable-name step. The first deploy after
merge ships the links; the first `v*` release after merge ships the stable-named
assets that make direct downloads resolve.

## Open Questions

1. Request device-orientation permission silently on first interaction, or via a
   small explicit "enable tilt" control? (Leaning: piggyback the first tap.)
2. Exact reveal duration/easing (start ~3.5 s, tune by eye).
3. Include the Linux tarball as a separate link, or AppImage-only for Linux to
   keep the list short? (Leaning: AppImage + a small "tarball" secondary.)

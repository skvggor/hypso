## Why

The landing page's information lives in a single legend box with a terminal
aesthetic: small mono type, 1px rules, metadata rows. Hierarchy is carried only
by font size inside one container, and the UI is visually detached from the
generative map it sits on. The redesign replaces the legend with an **island**:
an organic landmass whose silhouette is produced by the Hypso engine itself,
filled with the exact color of the map's contour lines, holding the page's
content in bold dark display type. The UI stops describing the terrain and
becomes a piece of it.

## What Changes

- **Remove the legend box** and its cartographic-margin vocabulary ("Fig. 1"
  caption, dotted metadata rows, thin borders). No registration-marks-style
  framing for content.
- **Island component**: the description, seed (as a large numeral), contour
  count, and the Generate action live inside an organic closed shape. The
  silhouette is a closed high-level contour from the engine's own field and is
  re-plotted on every seed. A smaller **islet** next to it holds the downloads
  reveal; the source link stays loose on the dark margin.
- **Reciprocal relief**: the island's shape is stamped into the relief mask
  (like the wordmark today) so the map's contour lines hug the island from
  behind.
- **High-contrast color rule**: the island surface uses the exact ink of the
  current generation's contour lines (the palette's light stop, with a raised
  lightness floor so any hue passes AA against near-black text). Text on the
  island is always dark (`#0b0e12`); text on the dark background stays light.
- **Mobile-first**: full-screen map, island anchored in the lower half, islet
  expanding into a downloads sheet without covering the wordmark. Touch on the
  island never reseeds; the map does.
- **Organic microinteractions**: Generate shows concentric contour rings
  radiating on hover/press; ink transitions cross-fade slowly on automatic
  drift and switch immediately on explicit user action; a decorative
  cartographic **cursor follower** trails the native pointer with the same
  inertia as the parallax (fine pointers only, `pointer-events: none`, hidden
  when the pointer leaves, disabled under reduced motion).

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `landing-page`: content presentation moves from a legend box to the island
  component; the contrast rule becomes "engine ink surface + fixed dark text";
  layout requirements become explicitly mobile-first.
- `live-hero`: the relief mask gains the island silhouette alongside the
  wordmark; the palette gains a lightness floor on its light stop; new
  requirements cover the generative island silhouette, ink transition policy,
  and the cursor follower.

## Impact

- **Web**: `web/index.html` (legend markup replaced by island/islet markup),
  `web/css/style.css` (island styling, color rule, cursor follower, mobile
  layout), `web/js/main.js` (silhouette extraction from engine output, relief
  mask composition, ink transitions, follower loop, ring interaction).
- **Engine/WASM**: potentially none — the flat contour buffer already contains
  closed polylines; silhouette selection can happen in JS. If no suitable
  closed contour exists for a seed, fall back to a smoothed default blob.
- **Accessibility**: AA minimum guaranteed by the lightness floor (AAA where
  the hue allows); the island content is regular DOM (not canvas), so semantics
  and keyboard order are preserved; reduced motion falls back to the static
  poster with a static island.
- **Untouched**: SEO metadata, release workflow, desktop app, `wasm-generator`
  public surface.

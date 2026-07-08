## 1. Palette and color rule

- [x] 1.1 Raise the lightness floor of `palette.to` in `randomPalette()` (~68–70%) and verify `#0b0e12` on any generated ink meets ≥ 4.5:1
- [x] 1.2 Keep exposing the ink via `--ink` and add `--ink-text: #0b0e12` as the fixed on-surface text color

## 2. Island silhouette

- [x] 2.1 Implement silhouette selection: pick a closed contour polyline from the generated buffer meeting area/aspect thresholds near the anchor region
- [x] 2.2 Implement light smoothing of the selected polyline and conversion to an SVG path
- [x] 2.3 Implement the deterministic fallback blob (seeded from the same integer) when no suitable ring exists
- [x] 2.4 Compute the inscribed rectangular safe zone and expose it to layout (CSS custom properties or inline style)

## 3. Island and islet markup/styles

- [x] 3.1 Replace the legend markup in `index.html` with the island (purpose statement in bold uppercase display type, seed as large numeral, contour count, Generate control) and the downloads islet — no "Fig." captions or metadata rows
- [x] 3.2 Style the island as an SVG-clipped DOM container filled with `--ink`, dark text, content constrained to the safe zone
- [x] 3.3 Style the islet collapsed state and its expanded states: in-place growth on desktop, bottom sheet on small viewports (never covering the wordmark, explicit close + Escape)
- [x] 3.4 Keep the loose source link and bottom edge annotation on the dark margin with light text
- [x] 3.5 Mobile-first layout: full-screen map, island anchored lower half, islet beside/below; desktop places the island in the margin region

## 4. Relief integration

- [x] 4.1 Rasterize the island silhouette into the relief mask combined with the wordmark before blurring (`buildMask`)
- [x] 4.2 Verify contours hug the island (reuse `tagHuggingLines` behavior) and the island reads in front

## 5. Interactions

- [x] 5.1 Ensure island/islet clicks never reseed; only the map surface and Space do
- [x] 5.2 Ink transition policy: slow cross-fade (~2s tween) of `--ink` on automatic drift, immediate switch on explicit user action
- [x] 5.3 Generate control: concentric contour rings radiate on hover/press; no animation under reduced motion
- [x] 5.4 Cursor follower: decorative cross/ring in `--ink` trailing the pointer via the existing lerp; `pointer-events: none`, hidden on window leave, gated to `pointer: fine`, absent under reduced motion; native cursor untouched
- [x] 5.5 Downloads islet keeps the GitHub API asset resolution, per-OS icons, and the touch-device desktop-only note

## 6. Fallbacks and accessibility

- [x] 6.1 Reduced motion / no WASM / no GSAP: static poster with a static island (no silhouette animation, no follower, no rings)
- [x] 6.2 Keyboard order, visible focus, and Escape handling across island, islet, and sheet
- [x] 6.3 Verify AA contrast on island surfaces across many seeds (spot-check hues at the lightness floor)

## 7. Verification

- [x] 7.1 Manual pass on small viewport (layout, sheet behavior, tap-to-reseed only on map)
- [x] 7.2 Manual pass on desktop (island placement, follower, rings, drift cross-fade vs. immediate switch)
- [x] 7.3 Run the site build and confirm no external requests and no layout shift on fallback swap

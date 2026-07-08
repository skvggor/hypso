## Context

Direction chosen after exploring three candidates (poster band, island,
offset slab): the **island** — UI as a literal piece of the generated terrain.
Croquis: https://claude.ai/code/artifact/d5564c27-ac6f-4949-92a1-cda052fcf9ac
(direction B). Mobile and desktop are both first-class; mobile drives the
layout decisions.

## Goals / Non-Goals

- Goals: hierarchical content, high-contrast surfaces tinted by the live map
  ink, organic/generative microinteractions, quality mobile experience.
- Non-goals: new engine features, new pages/sections, changing SEO or the
  release pipeline. No circular "seal" element (explicitly rejected).

## Decisions

### Island silhouette comes from the engine's own field

Pick a closed contour polyline from the generated buffer (adequate area,
closed ring, near the intended anchor region), smooth it lightly, and use it
as the island's SVG path/clip. Re-plotted on every seed. Fallback when no
suitable ring exists: a smoothed default blob seeded from the same integer, so
the island always renders and stays deterministic per seed.

**Legibility constraint**: content is laid out inside a rectangular safe zone
inscribed in the silhouette; smoothing and a minimum-area threshold guarantee
the safe zone fits. Text never follows the irregular edge.

### Island is DOM, not canvas

The shape is an SVG-clipped container; description, seed numeral, contours
count, and the Generate button are regular HTML inside it. Keeps semantics,
focus order, and the existing no-JS/reduced-motion fallbacks trivial.

### Reciprocal relief

The island silhouette is rasterized into the same relief mask as the wordmark
(`buildMask`), so field contours ring the island exactly as they ring the
letters. The mask combines wordmark + island before blurring.

### Color rule

- `--ink` = the palette's light stop (`palette.to`), which is also the bright
  end of the contour gradient — surface and lines share literal ink.
- Raise the lightness floor of `palette.to` in `randomPalette()` to ~68–70%
  so any hue gives ≥ AA against `#0b0e12` text. Dark text on ink surfaces is a
  fixed rule; light text on the dark background stays as today.
- Ink transition: slow cross-fade (~2s tween) on automatic drift; immediate
  swap on explicit user action (click/tap/space).

### Downloads islet

A smaller sibling shape near the island. Collapsed: an affordance only.
Expanded (mobile): a sheet rising from the islet, map dimmed behind, Escape
and an explicit close control dismiss it; never covers the wordmark.
Expanded (desktop): grows in place next to the island. Keeps the existing
GitHub-API asset resolution and `pointer: coarse` desktop-only note.

### Microinteractions

- **Generate**: concentric contour rings radiate from the control on
  hover/press — the `◎` glyph grows into a small field.
- **Cursor follower**: a registration-cross/ring in `--ink` trailing the
  native cursor using the existing pointer lerp (`frame()`), purely
  decorative: no hover states, `pointer-events: none`, hidden on window
  leave, gated to `pointer: fine`, removed under `prefers-reduced-motion`.
  The native cursor is never hidden.
- All motion collapses to the static poster + static island under reduced
  motion (existing fallback path).

### Mobile layout

Full-screen map; island anchored lower half; islet beside/below it; source
link on the margin. Tapping the island or islet never reseeds — only the map
surface (and Space on desktop) does.

## Risks / Trade-offs

- **Silhouette variability**: extreme seeds may produce awkward rings →
  mitigated by area/aspect thresholds + fallback blob.
- **Panel repaint churn**: full re-tint every 10s drift could feel frantic →
  mitigated by the slow cross-fade policy.
- **Perf**: silhouette extraction and mask composition run per generation;
  both are O(field) like the existing mask work — no new frame-loop cost
  besides the follower transform.

## Open Questions

- Exact anchor position of the island on desktop (right third vs. center-right)
  — decide visually during implementation.
- Whether the islet also gets relief-mask treatment or only the main island.

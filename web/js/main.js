// The map sheet. The real Hypso engine (Rust → WebAssembly) plots the contour
// field — and the wordmark is stamped into that field as elevation, so the map's
// own contour lines bend around and cling to every letter, the way contours ring
// a peak. The word is written by the terrain itself. Every generation re-inks and
// re-plots. Reduced motion / no WASM / no GSAP fall back to the static poster.

const WORD = 'Hypso';
const FIELD = {
  width: 210,
  height: 122,
  octaves: 5,
  frequency: 3.0,
  persistence: 0.5,
  levels: 16,
  smoothing: 4,
};
const INDEX_INTERVAL = 4; // every Nth contour is a thick "index" line
const STROKE_THIN = 1; // px, a normal contour
const STROKE_INDEX = 2.5; // px, a thick index contour
const RELIEF = 0.62; // how high the letters rise into the field
const BG = '#0b0e12';
const PARALLAX = 44; // px of parallax travel — livelier than before
const TILT_RANGE = 22; // degrees of device tilt mapped to full parallax
const WORD_FADE_SECONDS = 3.8; // the wordmark's transparency resolves this slowly
const DRIFT_SECONDS = 10;

const root = document.documentElement;
const canvas = document.getElementById('field');
const poster = document.getElementById('poster');
const reseedButton = document.getElementById('reseed');
const seedField = document.getElementById('seed');
const contoursField = document.getElementById('contours');
const downloadToggle = document.getElementById('download-toggle');
const downloadsPanel = document.getElementById('downloads');
const downloadsClose = document.getElementById('downloads-close');
const islandElement = document.getElementById('island');
const islandPathElement = document.getElementById('island-path');
const isletPathElement = document.getElementById('islet-path');
const generatePathElement = document.getElementById('generate-path');
const downloadsPathElement = document.getElementById('downloads-path');
const followerElement = document.getElementById('follower');

const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
const hasGsap = typeof window.gsap !== 'undefined';

root.classList.add('js');

const state = { lines: [], total: 0, reveal: 0, wordReveal: 0, seed: 0, palette: null };
const pointer = { targetX: 0, targetY: 0, x: 0, y: 0 };
const island = { radii: null, isletRadii: null };
const follower = { targetX: 0, targetY: 0, x: 0, y: 0, active: false, shown: false };
const word = { ready: false, size: 0, x: 0, y: 0, mask: null, outline: null };
const maskCanvas = document.createElement('canvas');
let ctx;
let cssWidth = 0;
let cssHeight = 0;
let dpr = 1;
let gradient = null;
let generate = null;
let generateRelief = null;
let otFont = null;
let dirty = true;
let driftCall = null;

function clamp(value, low, high) {
  return value < low ? low : value > high ? high : value;
}

function randomSeed() {
  return Math.floor(Math.random() * 0xffffffff);
}

function margin() {
  return Math.max(20, Math.min(cssWidth, cssHeight) * 0.05);
}

/** A fresh two-stop ink gradient each generation: a dim base fading to a bright
 *  hue, at a random angle. The light stop has a 70% lightness floor so dark text
 *  on an ink surface meets AA contrast for any hue. */
function randomPalette() {
  const hue = Math.random() * 360;
  const drift = hue + (Math.random() * 60 - 30);
  return {
    from: `hsl(${hue.toFixed(0)} ${(42 + Math.random() * 18).toFixed(0)}% ${(30 + Math.random() * 10).toFixed(0)}%)`,
    to: `hsl(${drift.toFixed(0)} ${(62 + Math.random() * 16).toFixed(0)}% ${(70 + Math.random() * 8).toFixed(0)}%)`,
    angle: Math.random() * Math.PI * 2,
  };
}

/** Decode the flat `[count, (level, len, x,y…), …]` buffer into polylines. */
function toLines(buffer) {
  const lines = [];
  let cursor = 0;
  const count = buffer[cursor++];
  let total = 0;
  for (let n = 0; n < count; n += 1) {
    const level = buffer[cursor++];
    const length = buffer[cursor++];
    const coordinates = buffer.subarray(cursor, cursor + length * 2);
    cursor += length * 2;
    lines.push({ level, length, coordinates });
    total += length;
  }
  return { lines, total };
}

/** Plot the field for `seed`, raised by the wordmark relief when it is ready. */
function linesFor(seed) {
  const buffer =
    word.mask && generateRelief
      ? generateRelief(seed, FIELD.width, FIELD.height, FIELD.octaves, FIELD.frequency, FIELD.persistence, FIELD.levels, FIELD.smoothing, word.mask, RELIEF)
      : generate(seed, FIELD.width, FIELD.height, FIELD.octaves, FIELD.frequency, FIELD.persistence, FIELD.levels, FIELD.smoothing);
  const result = toLines(buffer);
  tagHuggingLines(result.lines);
  return result;
}

/** Mark the contours that ring the raised letters, so they can read brighter than
 *  the surrounding terrain — the word emerging from the map. */
function tagHuggingLines(lines) {
  const mask = word.mask;
  if (!mask) {
    for (const line of lines) line.hug = false;
    return;
  }
  const w = FIELD.width;
  const h = FIELD.height;
  for (const line of lines) {
    const step = Math.max(1, Math.floor(line.length / 5));
    let sum = 0;
    let count = 0;
    for (let k = 0; k < line.length; k += step) {
      const mx = clamp(Math.floor(line.coordinates[k * 2] * w), 0, w - 1);
      const my = clamp(Math.floor(line.coordinates[k * 2 + 1] * h), 0, h - 1);
      sum += mask[my * w + mx];
      count += 1;
    }
    line.hug = count > 0 && sum / count > 0.18;
  }
}

// --- The island: a landmass grown by the engine ------------------------------
// Its silhouette comes from a closed contour of the generated field, morphed
// into a star-shaped blob whose radius never dips below BLOB_MIN_RADIUS — that
// clamp is what guarantees the rectangular content safe zone always fits.

const BLOB_POINTS = 64;
const BLOB_MIN_RADIUS = 0.8;
const BLOB_RX = 48; // viewBox half-extents (0..100 box)
const BLOB_RY = 46;

function mulberry32(seed) {
  let a = seed >>> 0;
  return function next() {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Deterministic blob for a seed: also the fallback island when the field
 *  offers no usable ring. `minRadius` bounds the safe zone; `wobble` sets how
 *  irregular the coastline gets. */
function fallbackRadii(seed, minRadius = BLOB_MIN_RADIUS, wobble = 0.4) {
  const random = mulberry32(seed);
  const harmonics = [];
  for (let h = 2; h <= 5; h += 1) {
    harmonics.push({ h, amplitude: (random() * wobble) / h, phase: random() * Math.PI * 2 });
  }
  const radii = new Array(BLOB_POINTS);
  const base = (1 + minRadius) / 2 + 0.06;
  for (let i = 0; i < BLOB_POINTS; i += 1) {
    const angle = (i / BLOB_POINTS) * Math.PI * 2;
    let radius = base;
    for (const { h, amplitude, phase } of harmonics) {
      radius += amplitude * Math.sin(h * angle + phase);
    }
    radii[i] = clamp(radius, minRadius, 1);
  }
  return radii;
}

/** Radial signature of the best closed contour in the plot, or null. */
function contourRadii(lines) {
  let best = null;
  for (const line of lines) {
    if (line.length < 24) continue;
    const c = line.coordinates;
    const dx = c[0] - c[(line.length - 1) * 2];
    const dy = c[1] - c[(line.length - 1) * 2 + 1];
    if (dx * dx + dy * dy > 0.0004) continue; // open polyline
    let minX = 1;
    let maxX = 0;
    let minY = 1;
    let maxY = 0;
    for (let k = 0; k < line.length; k += 1) {
      const x = c[k * 2];
      const y = c[k * 2 + 1];
      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
      if (y < minY) minY = y;
      if (y > maxY) maxY = y;
    }
    const width = maxX - minX;
    const height = maxY - minY;
    if (width < 0.06 || height < 0.06 || width > 0.5 || height > 0.5) continue;
    const aspect = width / height;
    if (aspect < 0.45 || aspect > 2.2) continue;
    const area = width * height;
    if (!best || area > best.area) {
      best = { line, area, cx: (minX + maxX) / 2, cy: (minY + maxY) / 2, hw: width / 2, hh: height / 2 };
    }
  }
  if (!best) return null;

  const bins = new Array(BLOB_POINTS).fill(0);
  const c = best.line.coordinates;
  for (let k = 0; k < best.line.length; k += 1) {
    const nx = (c[k * 2] - best.cx) / best.hw;
    const ny = (c[k * 2 + 1] - best.cy) / best.hh;
    const angle = Math.atan2(ny, nx);
    const bin = (((Math.round((angle / (Math.PI * 2)) * BLOB_POINTS) % BLOB_POINTS) + BLOB_POINTS) % BLOB_POINTS);
    const radius = Math.hypot(nx, ny);
    if (radius > bins[bin]) bins[bin] = radius;
  }

  for (let pass = 0; pass < 3; pass += 1) {
    for (let i = 0; i < BLOB_POINTS; i += 1) {
      if (bins[i] !== 0) continue;
      const before = bins[(i + BLOB_POINTS - 1) % BLOB_POINTS];
      const after = bins[(i + 1) % BLOB_POINTS];
      if (before || after) bins[i] = (before + after) / ((before ? 1 : 0) + (after ? 1 : 0));
    }
  }
  let max = 0;
  for (const value of bins) if (value > max) max = value;
  if (!max) return null;
  const radii = bins.map((value) => (value || max * 0.85) / max);
  for (let pass = 0; pass < 2; pass += 1) {
    const previous = radii.slice();
    for (let i = 0; i < BLOB_POINTS; i += 1) {
      radii[i] =
        (previous[(i + BLOB_POINTS - 1) % BLOB_POINTS] + previous[i] * 2 + previous[(i + 1) % BLOB_POINTS]) / 4;
    }
  }
  return radii.map((value) => clamp(value, BLOB_MIN_RADIUS, 1));
}

function blobPoint(radii, index, rx, ry) {
  const radius = radii[index % radii.length];
  const angle = ((index % radii.length) / radii.length) * Math.PI * 2;
  return [50 + Math.cos(angle) * radius * rx, 50 + Math.sin(angle) * radius * ry];
}

/** A smooth closed path through the blob's midpoints (0..100 viewBox). */
function radiiToPath(radii, rx = BLOB_RX, ry = BLOB_RY) {
  const count = radii.length;
  const mid = (i) => {
    const a = blobPoint(radii, i, rx, ry);
    const b = blobPoint(radii, i + 1, rx, ry);
    return `${((a[0] + b[0]) / 2).toFixed(2)} ${((a[1] + b[1]) / 2).toFixed(2)}`;
  };
  let d = `M ${mid(count - 1)}`;
  for (let i = 0; i < count; i += 1) {
    const p = blobPoint(radii, i, rx, ry);
    d += ` Q ${p[0].toFixed(2)} ${p[1].toFixed(2)} ${mid(i)}`;
  }
  return `${d} Z`;
}

/** Re-derive the island, islet, and generate-button silhouettes. */
function updateIsland() {
  if (!islandPathElement || !isletPathElement) return;
  const fromField = state.lines.length ? contourRadii(state.lines) : null;
  island.radii = fromField || fallbackRadii(state.seed);
  island.isletRadii = fallbackRadii((state.seed ^ 0x9e3779b9) >>> 0, 0.72, 0.6);
  islandPathElement.setAttribute('d', radiiToPath(island.radii));
  isletPathElement.setAttribute('d', radiiToPath(island.isletRadii, 49, 44));
  if (generatePathElement) {
    generatePathElement.setAttribute('d', radiiToPath(fallbackRadii((state.seed ^ 0x51ed5eed) >>> 0, 0.78, 0.55), 49, 42));
  }
  // The downloads panel holds a dense list, so its blob is deliberately
  // subtle: high radius floor, gentle wobble — organic edge, no deep coves.
  if (downloadsPathElement) {
    downloadsPathElement.setAttribute('d', radiiToPath(fallbackRadii((state.seed ^ 0x7a3d0a11) >>> 0, 0.955, 0.09), 49.5, 49));
  }
}

/** Stamp one blob's on-screen shape into the relief mask. */
function stampShape(mctx, mw, mh, element, radii, rx, ry) {
  if (!radii || !element) return;
  const rect = element.getBoundingClientRect();
  if (!rect.width || !rect.height) return;
  mctx.beginPath();
  for (let i = 0; i <= BLOB_POINTS; i += 1) {
    const [px, py] = blobPoint(radii, i, rx, ry);
    const x = ((rect.left + (px / 100) * rect.width) / cssWidth) * mw;
    const y = ((rect.top + (py / 100) * rect.height) / cssHeight) * mh;
    if (i === 0) mctx.moveTo(x, y);
    else mctx.lineTo(x, y);
  }
  mctx.closePath();
  mctx.fillStyle = '#ffffff';
  mctx.fill();
}

/** The island and islet both rise from the field, so the map's contours ring
 *  them the same way they ring the wordmark. */
function stampIsland(mctx, mw, mh) {
  if (!cssWidth || !cssHeight) return;
  stampShape(mctx, mw, mh, islandElement, island.radii, BLOB_RX, BLOB_RY);
  stampShape(mctx, mw, mh, downloadToggle, island.isletRadii, 49, 44);
}

function updateLegend() {
  seedField.textContent =
    '0x' + (state.seed >>> 0).toString(16).toUpperCase().padStart(8, '0');
  contoursField.textContent = String(state.lines.length);
}

function markDirty() {
  dirty = true;
}

function makeGradient() {
  const palette = state.palette;
  const centerX = cssWidth / 2;
  const centerY = cssHeight / 2;
  const reach = Math.max(cssWidth, cssHeight);
  const dx = Math.cos(palette.angle) * reach;
  const dy = Math.sin(palette.angle) * reach;
  gradient = ctx.createLinearGradient(centerX - dx, centerY - dy, centerX + dx, centerY + dy);
  gradient.addColorStop(0, palette.from);
  gradient.addColorStop(1, palette.to);
}

/** Tint the ink surfaces with this generation's line color. On the automatic
 *  drift the ink cross-fades slowly (registered @property transition); on an
 *  explicit user action it switches immediately with the replot. */
function applyInk(slow = false) {
  if (!state.palette) return;
  root.classList.toggle('ink-drift', Boolean(slow));
  root.style.setProperty('--ink', state.palette.to);
}

function newGeneration(seed, slow = false) {
  state.seed = seed;
  state.palette = randomPalette();
  // First plot: derive the island silhouette from this seed's field, then
  // stamp it into the relief mask and re-plot so the contours hug the island.
  state.lines = linesFor(seed).lines;
  updateIsland();
  buildMask();
  const next = linesFor(seed);
  state.lines = next.lines;
  state.total = next.total;
  if (ctx) makeGradient();
  applyInk(slow);
  updateLegend();
}

/** Re-plot the current seed (e.g. once the relief mask becomes available) without
 *  changing the ink. */
function replotCurrent() {
  if (!generate) return;
  const next = linesFor(state.seed);
  state.lines = next.lines;
  state.total = next.total;
  updateLegend();
  markDirty();
}

function layoutWord() {
  if (!ctx) return;
  let unit;
  if (otFont) {
    unit = otFont.getAdvanceWidth(WORD, 100);
  } else {
    ctx.font = '900 100px sans-serif';
    unit = ctx.measureText(WORD).width || 100;
  }
  const target = cssWidth < 40 * 16 ? cssWidth * 0.82 : Math.min(cssWidth * 0.62, cssHeight * 2);
  word.size = Math.min((target / unit) * 100, cssHeight * 0.46);
  word.x = margin();
  // On small viewports the island anchors in the lower half, so the wordmark
  // sits in the upper third and stays unobstructed.
  word.y = cssWidth < 40 * 16 ? cssHeight * 0.34 : cssHeight - margin() - word.size * 0.28;
}

// --- The relief mask: the wordmark rasterized into the field grid -------------

function boxBlur1D(source, target, width, height, radius, horizontal) {
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      let sum = 0;
      let count = 0;
      for (let k = -radius; k <= radius; k += 1) {
        const xx = horizontal ? x + k : x;
        const yy = horizontal ? y : y + k;
        if (xx < 0 || xx >= width || yy < 0 || yy >= height) continue;
        sum += source[yy * width + xx];
        count += 1;
      }
      target[y * width + x] = sum / count;
    }
  }
}

/** A soft falloff around the letters so the raised region ramps down over several
 *  cells — that gap is what a fan of contours settles into. */
function blurMask(source, width, height, radius, passes) {
  let a = Float32Array.from(source);
  const b = new Float32Array(width * height);
  for (let p = 0; p < passes; p += 1) {
    boxBlur1D(a, b, width, height, radius, true);
    boxBlur1D(b, a, width, height, radius, false);
  }
  return a;
}

function buildMask() {
  word.mask = null;
  if ((!otFont && !island.radii) || !cssWidth || !cssHeight) return;
  const mw = FIELD.width;
  const mh = FIELD.height;
  maskCanvas.width = mw;
  maskCanvas.height = mh;
  const mctx = maskCanvas.getContext('2d');
  mctx.setTransform(1, 0, 0, 1, 0, 0);
  mctx.clearRect(0, 0, mw, mh);
  if (otFont) {
    mctx.save();
    mctx.scale(mw / cssWidth, mh / cssHeight);
    const path = otFont.getPath(WORD, word.x, word.y, word.size);
    path.fill = '#ffffff';
    path.stroke = null;
    path.draw(mctx);
    mctx.restore();
  }
  stampIsland(mctx, mw, mh);

  const pixels = mctx.getImageData(0, 0, mw, mh).data;
  const raw = new Float32Array(mw * mh);
  for (let i = 0; i < mw * mh; i += 1) raw[i] = pixels[i * 4 + 3] / 255;
  word.mask = blurMask(raw, mw, mh, 3, 3);
}

// --- Glyph outline (a faint legibility guide over the contours) ---------------

function sampleCubic(p0, p1, p2, p3, out, steps) {
  for (let i = 1; i <= steps; i += 1) {
    const t = i / steps;
    const mt = 1 - t;
    const a = mt * mt * mt;
    const b = 3 * mt * mt * t;
    const c = 3 * mt * t * t;
    const d = t * t * t;
    out.push({ x: a * p0.x + b * p1.x + c * p2.x + d * p3.x, y: a * p0.y + b * p1.y + c * p2.y + d * p3.y });
  }
}

function sampleQuad(p0, p1, p2, out, steps) {
  for (let i = 1; i <= steps; i += 1) {
    const t = i / steps;
    const mt = 1 - t;
    const a = mt * mt;
    const b = 2 * mt * t;
    const c = t * t;
    out.push({ x: a * p0.x + b * p1.x + c * p2.x, y: a * p0.y + b * p1.y + c * p2.y });
  }
}

function flattenPath(path) {
  const contours = [];
  let current = null;
  let previous = null;
  for (const command of path.commands) {
    if (command.type === 'M') {
      if (current && current.length > 1) contours.push(current);
      current = [{ x: command.x, y: command.y }];
      previous = current[0];
    } else if (command.type === 'L') {
      current.push({ x: command.x, y: command.y });
      previous = { x: command.x, y: command.y };
    } else if (command.type === 'C') {
      sampleCubic(previous, { x: command.x1, y: command.y1 }, { x: command.x2, y: command.y2 }, { x: command.x, y: command.y }, current, 8);
      previous = { x: command.x, y: command.y };
    } else if (command.type === 'Q') {
      sampleQuad(previous, { x: command.x1, y: command.y1 }, { x: command.x, y: command.y }, current, 6);
      previous = { x: command.x, y: command.y };
    } else if (command.type === 'Z') {
      if (current && current.length > 1) contours.push(current);
      current = null;
    }
  }
  if (current && current.length > 1) contours.push(current);
  return contours;
}

function buildWordOutline() {
  word.outline = otFont && ctx ? flattenPath(otFont.getPath(WORD, word.x, word.y, word.size)) : null;
}

function resize() {
  dpr = Math.min(window.devicePixelRatio || 1, 2);
  cssWidth = window.innerWidth;
  cssHeight = window.innerHeight;
  canvas.width = Math.round(cssWidth * dpr);
  canvas.height = Math.round(cssHeight * dpr);
  makeGradient();
  layoutWord();
  buildMask();
  buildWordOutline();
  markDirty();
}

// --- Drawing ------------------------------------------------------------------

function drawLines() {
  ctx.lineJoin = 'round';
  ctx.lineCap = 'round';
  ctx.strokeStyle = gradient;

  const drawn = state.reveal * state.total;
  const count = state.lines.length || 1;
  const maxLevel = Math.max(1, FIELD.levels - 1);
  let seen = 0;

  for (let index = 0; index < state.lines.length; index += 1) {
    if (seen >= drawn) break;
    const line = state.lines[index];
    const allow = Math.min(line.length, Math.max(0, drawn - seen));
    seen += line.length;
    if (allow < 2) continue;

    const isIndex = line.level % INDEX_INTERVAL === 0;
    ctx.lineWidth = (isIndex ? STROKE_INDEX : STROKE_THIN) * (line.hug ? 1.2 : 1);
    let alpha = (isIndex ? 0.9 : 0.5) + 0.4 * (line.level / maxLevel);
    if (line.hug) alpha = Math.min(1, alpha + 0.4); // the word rises out of the map
    ctx.globalAlpha = alpha;

    const depth = 0.45 + 0.55 * (index / count);
    const offsetX = pointer.x * depth;
    const offsetY = pointer.y * depth;

    ctx.beginPath();
    for (let k = 0; k < allow; k += 1) {
      const x = line.coordinates[k * 2] * cssWidth + offsetX;
      const y = line.coordinates[k * 2 + 1] * cssHeight + offsetY;
      if (k === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();
  }
  ctx.globalAlpha = 1;
}

/** The wordmark sits in front: its shape is knocked out of the terrain so the
 *  contours ring it from behind, and a clear outline defines each letter. */
function drawWord() {
  if (!word.outline) return;
  const ink = state.palette ? state.palette.to : '#f2efe9';

  // One path for all glyph contours (even-odd keeps counters like o, p, s hollow).
  ctx.save();
  ctx.lineJoin = 'round';
  ctx.lineCap = 'round';
  ctx.beginPath();
  for (const contour of word.outline) {
    for (let i = 0; i < contour.length; i += 1) {
      const p = contour[i];
      if (i === 0) ctx.moveTo(p.x, p.y);
      else ctx.lineTo(p.x, p.y);
    }
    ctx.closePath();
  }

  // 1) Knock the terrain out of the letters — the word occludes the lines behind.
  //    Driven by wordReveal (its own slow ramp) so the text stays translucent —
  //    the contours showing through — for a few seconds before settling opaque.
  ctx.globalAlpha = state.wordReveal;
  ctx.fillStyle = BG;
  ctx.fill('evenodd');

  // 2) A clear outline in this generation's ink defines the word in front.
  ctx.globalAlpha = 0.92 * state.wordReveal;
  ctx.strokeStyle = ink;
  ctx.lineWidth = STROKE_INDEX;
  ctx.stroke();
  ctx.restore();
}

function draw() {
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, cssWidth, cssHeight);
  drawLines();
  drawWord();
}

function frame() {
  pointer.x += (pointer.targetX - pointer.x) * 0.12;
  pointer.y += (pointer.targetY - pointer.y) * 0.12;
  if (Math.abs(pointer.targetX - pointer.x) > 0.1 || Math.abs(pointer.targetY - pointer.y) > 0.1) {
    dirty = true;
  }
  if (follower.active && follower.shown) {
    follower.x += (follower.targetX - follower.x) * 0.1;
    follower.y += (follower.targetY - follower.y) * 0.1;
    followerElement.style.transform = `translate3d(${follower.x.toFixed(1)}px, ${follower.y.toFixed(1)}px, 0)`;
  }
  if (dirty) {
    draw();
    dirty = false;
  }
  requestAnimationFrame(frame);
}

function scheduleDrift() {
  cancelDrift();
  driftCall = window.gsap.delayedCall(DRIFT_SECONDS, () => transitionTo(randomSeed(), true));
}

function cancelDrift() {
  if (driftCall) driftCall.kill();
  driftCall = null;
}

/** Fade the wordmark's transparency out over a deliberate few seconds, on its own
 *  clock so the letters stay translucent (contours showing through) before they
 *  settle opaque. */
function fadeWordIn() {
  window.gsap.killTweensOf(state, 'wordReveal');
  state.wordReveal = 0;
  window.gsap.to(state, {
    wordReveal: 1,
    duration: WORD_FADE_SECONDS,
    ease: 'power1.inOut',
    onUpdate: markDirty,
  });
}

/** Erase the current plot, then draw the next generation — new field, new ink. */
function transitionTo(seed, slow = false) {
  const timeline = window.gsap.timeline();
  timeline.to(state, { reveal: 0, duration: 0.55, ease: 'power2.in', onUpdate: markDirty });
  timeline.add(() => {
    newGeneration(seed, slow);
    fadeWordIn();
  });
  timeline.to(state, { reveal: 1, duration: 1.9, ease: 'power2.out', onUpdate: markDirty });
  timeline.add(scheduleDrift);
}

function revealLegend() {
  const items = document.querySelectorAll('[data-reveal]');
  window.gsap.set(items, { y: 10, opacity: 0 });
  // clearProps: a leftover transform would turn the islet wrap into a
  // containing block and break the fixed-position downloads sheet.
  window.gsap.to(items, {
    y: 0,
    opacity: 1,
    duration: 1.1,
    ease: 'power3.out',
    delay: 0.5,
    clearProps: 'transform',
  });
}

function bindPointer() {
  window.addEventListener(
    'pointermove',
    (event) => {
      pointer.targetX = (event.clientX / cssWidth - 0.5) * PARALLAX;
      pointer.targetY = (event.clientY / cssHeight - 0.5) * PARALLAX;
    },
    { passive: true },
  );
}

/** Decorative cartographic follower: trails the native cursor with the same
 *  inertia as the parallax. Never replaces the cursor, never reacts to hover.
 *  Fine pointers only; reduced motion removes it entirely (CSS). */
function bindFollower() {
  if (!followerElement || reducedMotion || !window.matchMedia('(pointer: fine)').matches) return;
  follower.active = true;
  window.addEventListener(
    'pointermove',
    (event) => {
      follower.targetX = event.clientX;
      follower.targetY = event.clientY;
      if (!follower.shown) {
        follower.x = event.clientX;
        follower.y = event.clientY;
        follower.shown = true;
      }
      followerElement.style.opacity = '0.85';
    },
    { passive: true },
  );
  document.documentElement.addEventListener('mouseleave', () => {
    follower.shown = false;
    followerElement.style.opacity = '0';
  });
}

// --- Device-orientation parallax (tilt), the lively input on mobile ---------

let orientationBound = false;

function handleOrientation(event) {
  if (event.gamma == null && event.beta == null) return;
  const tiltX = clamp((event.gamma || 0) / TILT_RANGE, -1, 1);
  const tiltY = clamp(((event.beta || 45) - 45) / TILT_RANGE, -1, 1);
  pointer.targetX = tiltX * PARALLAX;
  pointer.targetY = tiltY * PARALLAX;
}

function enableOrientation() {
  if (orientationBound || typeof window.DeviceOrientationEvent === 'undefined') return;
  orientationBound = true;
  window.addEventListener('deviceorientation', handleOrientation, { passive: true });
}

/** Grant/start the motion sensor. iOS gates it behind a user gesture; elsewhere
 *  just start listening. Safe to call repeatedly. */
function requestMotion() {
  const Sensor = window.DeviceOrientationEvent;
  if (!Sensor) return;
  if (typeof Sensor.requestPermission === 'function') {
    Sensor.requestPermission()
      .then((permission) => {
        if (permission === 'granted') enableOrientation();
      })
      .catch(() => {});
  } else {
    enableOrientation();
  }
}

function bindMotion() {
  const Sensor = window.DeviceOrientationEvent;
  if (!Sensor) return;
  if (typeof Sensor.requestPermission === 'function') {
    // iOS: ask from the visitor's first interaction (a gesture is required).
    window.addEventListener('pointerdown', requestMotion, { once: true });
  } else {
    enableOrientation();
  }
}

const RELEASES_API = 'https://api.github.com/repos/skvggor/hypso/releases/latest';
let downloadsResolved = false;

/** Point each platform link at the real asset of the latest release, whatever it
 *  is named. Progressive enhancement: the links otherwise open the releases page,
 *  so the page never depends on this request. */
async function resolveDownloads() {
  if (downloadsResolved) return;
  downloadsResolved = true;
  try {
    const response = await fetch(RELEASES_API, { headers: { Accept: 'application/vnd.github+json' } });
    if (!response.ok) return;
    const assets = (await response.json()).assets || [];
    const wire = (platform, test) => {
      const asset = assets.find((item) => test(item.name));
      const link = document.querySelector(`[data-platform="${platform}"]`);
      if (asset && link) {
        link.href = asset.browser_download_url;
        link.setAttribute('download', '');
      }
    };
    wire('linux-appimage', (name) => /\.AppImage$/i.test(name));
    wire('linux-tarball', (name) => /linux.*\.tar\.gz$/i.test(name));
    wire('windows', (name) => /\.zip$/i.test(name) && /win/i.test(name));
  } catch (error) {
    downloadsResolved = false; // let a later open retry
  }
}

/** The download component is revealed on demand; it lives inside the legend, so
 *  interacting with it never plots a new map. */
function bindDownloads() {
  if (!downloadToggle || !downloadsPanel) return;
  const close = () => {
    downloadsPanel.hidden = true;
    downloadToggle.setAttribute('aria-expanded', 'false');
    document.body.classList.remove('downloads-open');
    downloadToggle.focus();
  };
  downloadToggle.addEventListener('click', () => {
    const open = downloadsPanel.hidden;
    downloadsPanel.hidden = !open;
    downloadToggle.setAttribute('aria-expanded', String(open));
    document.body.classList.toggle('downloads-open', open);
    if (open) resolveDownloads();
  });
  if (downloadsClose) downloadsClose.addEventListener('click', close);
  window.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && !downloadsPanel.hidden) close();
  });
}

function bindReseed() {
  const trigger = () => {
    cancelDrift();
    transitionTo(randomSeed());
  };

  reseedButton.addEventListener('click', trigger);

  // The island and islet never reseed — only the map surface (and Space) does.
  document.addEventListener('click', (event) => {
    if (event.target.closest('a, button, .island, .islet-wrap')) return;
    trigger();
  });

  window.addEventListener('keydown', (event) => {
    if (event.code === 'Space' && event.target === document.body) {
      event.preventDefault();
      trigger();
    }
  });

  const hint = document.getElementById('hint');
  if (hint) {
    hint.innerHTML = window.matchMedia('(pointer: coarse)').matches
      ? 'Tap anywhere for another. Every plot is a new seed.'
      : 'Click the map or press <kbd>space</kbd> for another. Every plot is a new seed.';
  }
}

async function loadFont() {
  try {
    const response = await fetch('./assets/fonts/Montserrat-Black.ttf');
    const buffer = await response.arrayBuffer();
    otFont = window.opentype.parse(buffer);
    word.ready = true;
  } catch (error) {
    console.warn('Wordmark font unavailable:', error);
  }
  layoutWord();
  buildMask();
  buildWordOutline();
  replotCurrent(); // re-plot the current seed now that the letters can rise
  fadeWordIn(); // the wordmark's transparency resolves over a few seconds
}

/** Static poster in place of the live sheet — no animation, no layout shift.
 *  The island still renders, with a deterministic static silhouette. */
function fallback() {
  canvas.remove();
  poster.hidden = false;
  state.seed = randomSeed();
  updateIsland();
  document.querySelectorAll('[data-reveal]').forEach((element) => {
    element.style.opacity = '1';
  });
}

async function start() {
  if (reducedMotion || !hasGsap) {
    fallback();
    return;
  }

  let module;
  try {
    module = await import('../wasm/hypso.js');
    await module.default();
    generate = module.generate;
    generateRelief = module.generate_relief;
  } catch (error) {
    console.warn('Hypso WASM unavailable, showing poster:', error);
    fallback();
    return;
  }

  // Keep the plotter timeline time-accurate: don't let a heavy frame stall it.
  window.gsap.ticker.lagSmoothing(0);

  ctx = canvas.getContext('2d');
  state.palette = randomPalette();
  resize();
  window.addEventListener('resize', resize);
  loadFont();

  newGeneration(randomSeed());
  state.reveal = 0;

  revealLegend();
  bindPointer();
  bindFollower();
  bindMotion();
  bindReseed();
  bindDownloads();
  requestAnimationFrame(frame);

  window.gsap.to(state, {
    reveal: 1,
    duration: 2.8,
    ease: 'power2.out',
    onUpdate: markDirty,
    onComplete: scheduleDrift,
  });
}

start();

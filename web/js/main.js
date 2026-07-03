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

const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
const hasGsap = typeof window.gsap !== 'undefined';

root.classList.add('js');

const state = { lines: [], total: 0, reveal: 0, wordReveal: 0, seed: 0, palette: null };
const pointer = { targetX: 0, targetY: 0, x: 0, y: 0 };
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
 *  hue, at a random angle — bright enough to stay legible on the dark sheet. */
function randomPalette() {
  const hue = Math.random() * 360;
  const drift = hue + (Math.random() * 60 - 30);
  return {
    from: `hsl(${hue.toFixed(0)} ${(42 + Math.random() * 18).toFixed(0)}% ${(30 + Math.random() * 10).toFixed(0)}%)`,
    to: `hsl(${drift.toFixed(0)} ${(62 + Math.random() * 16).toFixed(0)}% ${(62 + Math.random() * 10).toFixed(0)}%)`,
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

function newGeneration(seed) {
  state.seed = seed;
  state.palette = randomPalette();
  const next = linesFor(seed);
  state.lines = next.lines;
  state.total = next.total;
  if (ctx) makeGradient();
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
  word.y = cssWidth < 40 * 16 ? cssHeight * 0.5 : cssHeight - margin() - word.size * 0.28;
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
  if (!otFont || !cssWidth || !cssHeight) return;
  const mw = FIELD.width;
  const mh = FIELD.height;
  maskCanvas.width = mw;
  maskCanvas.height = mh;
  const mctx = maskCanvas.getContext('2d');
  mctx.setTransform(1, 0, 0, 1, 0, 0);
  mctx.clearRect(0, 0, mw, mh);
  mctx.save();
  mctx.scale(mw / cssWidth, mh / cssHeight);
  const path = otFont.getPath(WORD, word.x, word.y, word.size);
  path.fill = '#ffffff';
  path.stroke = null;
  path.draw(mctx);
  mctx.restore();

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
  if (dirty) {
    draw();
    dirty = false;
  }
  requestAnimationFrame(frame);
}

function scheduleDrift() {
  cancelDrift();
  driftCall = window.gsap.delayedCall(DRIFT_SECONDS, () => transitionTo(randomSeed()));
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
function transitionTo(seed) {
  const timeline = window.gsap.timeline();
  timeline.to(state, { reveal: 0, duration: 0.55, ease: 'power2.in', onUpdate: markDirty });
  timeline.add(() => {
    newGeneration(seed);
    fadeWordIn();
  });
  timeline.to(state, { reveal: 1, duration: 1.9, ease: 'power2.out', onUpdate: markDirty });
  timeline.add(scheduleDrift);
}

function revealLegend() {
  const items = document.querySelectorAll('[data-reveal]');
  window.gsap.set(items, { y: 10, opacity: 0 });
  window.gsap.to(items, { y: 0, opacity: 1, duration: 1.1, ease: 'power3.out', delay: 0.5 });
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

function bindReseed() {
  const trigger = () => {
    cancelDrift();
    transitionTo(randomSeed());
  };

  reseedButton.addEventListener('click', trigger);

  document.addEventListener('click', (event) => {
    if (event.target.closest('a, button, .legend')) return;
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

/** Static poster in place of the live sheet — no animation, no layout shift. */
function fallback() {
  canvas.remove();
  poster.hidden = false;
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
  bindMotion();
  bindReseed();
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

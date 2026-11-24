// SPDX-License-Identifier: MIT OR Apache-2.0
// Browser demo for tpt-algo-wasm. Loads the wasm-pack `--target web` build from ./pkg/.
import init, { IsolationForest, Ekf } from "./pkg/tpt_algo_wasm.js";

const statusEl = document.getElementById("status");

function setStatus(text, isError = false) {
  statusEl.textContent = text;
  statusEl.classList.toggle("error", isError);
}

// Colours come from the page's CSS variables so the charts follow light and dark themes.
function cssVar(name, fallback) {
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return v || fallback;
}

// Seeded PRNG (mulberry32) so a given seed always produces the same data.
function rng(seed) {
  let s = seed >>> 0;
  return () => {
    s = (s + 0x6d2b79f5) >>> 0;
    let t = s;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

// Standard normal via Box-Muller, drawn from a uniform source.
function gaussian(uniform) {
  const u1 = Math.max(uniform(), 1e-12);
  const u2 = uniform();
  return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
}

function freeIfPossible(obj) {
  if (obj && typeof obj.free === "function") obj.free();
}

// ---------------------------------------------------------------- Isolation Forest

const FRAUD_WORLD = { minX: -4, maxX: 8, minY: -4, maxY: 8 };
const fraudCanvas = document.getElementById("fraud-canvas");
const fraudCtx = fraudCanvas.getContext("2d");
let fraudState = null; // { forest, points, scores, labels }

// Maps an anomaly score to a colour: blue (normal) through neutral to orange (anomalous).
function scoreColour(score) {
  const lo = 0.35;
  const hi = 0.75;
  const t = Math.min(1, Math.max(0, (score - lo) / (hi - lo)));
  const blue = [43, 108, 176];
  const neutral = [236, 236, 232];
  const orange = [192, 86, 33];
  const [a, b, u] = t < 0.5 ? [blue, neutral, t * 2] : [neutral, orange, (t - 0.5) * 2];
  const c = a.map((x, i) => Math.round(x + (b[i] - x) * u));
  return `rgb(${c[0]}, ${c[1]}, ${c[2]})`;
}

function toCanvas(x, y) {
  const px = ((x - FRAUD_WORLD.minX) / (FRAUD_WORLD.maxX - FRAUD_WORLD.minX)) * fraudCanvas.width;
  const py = fraudCanvas.height - ((y - FRAUD_WORLD.minY) / (FRAUD_WORLD.maxY - FRAUD_WORLD.minY)) * fraudCanvas.height;
  return [px, py];
}

function toWorld(px, py) {
  const x = FRAUD_WORLD.minX + (px / fraudCanvas.width) * (FRAUD_WORLD.maxX - FRAUD_WORLD.minX);
  const y = FRAUD_WORLD.minY + ((fraudCanvas.height - py) / fraudCanvas.height) * (FRAUD_WORLD.maxY - FRAUD_WORLD.minY);
  return [x, y];
}

function drawFraud() {
  if (!fraudState) return;
  const { points, scores, labels } = fraudState;
  const ctx = fraudCtx;
  ctx.clearRect(0, 0, fraudCanvas.width, fraudCanvas.height);

  ctx.strokeStyle = cssVar("--border", "#d9d9d4");
  ctx.lineWidth = 1;
  for (let v = -4; v <= 8; v += 2) {
    const [gx] = toCanvas(v, 0);
    const [, gy] = toCanvas(0, v);
    ctx.beginPath(); ctx.moveTo(gx, 0); ctx.lineTo(gx, fraudCanvas.height); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(0, gy); ctx.lineTo(fraudCanvas.width, gy); ctx.stroke();
  }

  for (let i = 0; i < points.length; i += 2) {
    const [px, py] = toCanvas(points[i], points[i + 1]);
    const k = i / 2;
    ctx.fillStyle = scoreColour(scores[k]);
    ctx.strokeStyle = labels[k] ? cssVar("--fg", "#1d1d1b") : "transparent";
    ctx.lineWidth = labels[k] ? 1.5 : 0;
    ctx.beginPath();
    ctx.arc(px, py, labels[k] ? 6 : 4, 0, Math.PI * 2);
    ctx.fill();
    if (labels[k]) ctx.stroke();
  }
}

function meanOf(values, keep) {
  let sum = 0;
  let n = 0;
  values.forEach((v, i) => {
    if (keep(i)) { sum += v; n += 1; }
  });
  return n === 0 ? NaN : sum / n;
}

function trainFraud() {
  const seed = Math.max(0, Math.floor(Number(document.getElementById("fraud-seed").value) || 0));
  const trees = Math.max(1, Math.floor(Number(document.getElementById("fraud-trees").value) || 100));
  const uniform = rng(seed);

  const points = [];
  const labels = [];
  for (let i = 0; i < 400; i++) {
    points.push(gaussian(uniform), gaussian(uniform));
    labels.push(false);
  }
  for (let i = 0; i < 12; i++) {
    points.push(4 + 3 * uniform(), 4 + 3 * uniform());
    labels.push(true);
  }

  const forest = IsolationForest.fit(Float64Array.from(points), 2, {
    n_trees: trees,
    subsample_size: 256,
    seed,
  });
  const scores = Array.from(forest.score_samples(Float64Array.from(points)));

  freeIfPossible(fraudState && fraudState.forest);
  fraudState = { forest, points, scores, labels };

  document.getElementById("fraud-inlier").textContent = meanOf(scores, (i) => !labels[i]).toFixed(3);
  document.getElementById("fraud-outlier").textContent = meanOf(scores, (i) => labels[i]).toFixed(3);
  document.getElementById("fraud-clicked").textContent = "–";
  drawFraud();
}

fraudCanvas.addEventListener("click", (event) => {
  if (!fraudState) return;
  const rect = fraudCanvas.getBoundingClientRect();
  const px = ((event.clientX - rect.left) / rect.width) * fraudCanvas.width;
  const py = ((event.clientY - rect.top) / rect.height) * fraudCanvas.height;
  const [x, y] = toWorld(px, py);
  const score = fraudState.forest.score(Float64Array.from([x, y]));
  document.getElementById("fraud-clicked").textContent = score.toFixed(3);
  drawFraud();
  const [cx, cy] = toCanvas(x, y);
  fraudCtx.strokeStyle = cssVar("--accent-2", "#c05621");
  fraudCtx.lineWidth = 2;
  fraudCtx.beginPath();
  fraudCtx.arc(cx, cy, 10, 0, Math.PI * 2);
  fraudCtx.stroke();
});

// ---------------------------------------------------------------- Extended Kalman filter

const SOC_STEPS = 3000; // 50 minutes at 1 s
const SOC_DT = 1.0;
const SOC_CURRENT = 1.0; // amps, discharge is positive
const CELL = { capacity_ah: 2.0, r0: 0.05, r1: 0.02, c1: 1000.0 };
// The simulated cell has slightly higher series resistance than the filter assumes,
// so the filter is also working with a model mismatch.
const TRUE_R0 = 0.06;
const OCV = [0.0, 3.0, 0.25, 3.55, 0.5, 3.7, 0.75, 3.85, 1.0, 4.15];

const socCanvas = document.getElementById("soc-canvas");
const socCtx = socCanvas.getContext("2d");
let socRun = null; // { truth, estimate, rmse }

function drawSoc() {
  if (!socRun) return;
  const ctx = socCtx;
  const W = socCanvas.width;
  const H = socCanvas.height;
  const pad = { left: 52, right: 32, top: 16, bottom: 34 };
  const plotW = W - pad.left - pad.right;
  const plotH = H - pad.top - pad.bottom;
  const x = (k) => pad.left + (k / SOC_STEPS) * plotW;
  const y = (s) => pad.top + (1 - s) * plotH;

  ctx.clearRect(0, 0, W, H);
  ctx.font = "12px system-ui, sans-serif";
  ctx.fillStyle = cssVar("--muted", "#6b6b66");
  ctx.strokeStyle = cssVar("--border", "#d9d9d4");
  ctx.lineWidth = 1;
  ctx.textAlign = "right";
  ctx.textBaseline = "middle";
  for (let s = 0; s <= 1.0001; s += 0.25) {
    ctx.beginPath(); ctx.moveTo(pad.left, y(s)); ctx.lineTo(W - pad.right, y(s)); ctx.stroke();
    ctx.fillText(`${Math.round(s * 100)}%`, pad.left - 8, y(s));
  }
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  for (let m = 0; m <= 50; m += 10) {
    const k = (m * 60) / SOC_DT;
    ctx.fillText(`${m} min`, x(k), H - pad.bottom + 8);
  }

  const drawLine = (values, colour, width) => {
    ctx.strokeStyle = colour;
    ctx.lineWidth = width;
    ctx.beginPath();
    values.forEach((s, k) => {
      if (k === 0) ctx.moveTo(x(k), y(s));
      else ctx.lineTo(x(k), y(s));
    });
    ctx.stroke();
  };
  drawLine(socRun.truth, cssVar("--muted", "#6b6b66"), 2);
  drawLine(socRun.estimate, cssVar("--accent-2", "#c05621"), 2);
}

function runSoc() {
  const seed = Math.max(0, Math.floor(Number(document.getElementById("soc-seed").value) || 0));
  const noise = Math.max(0, Number(document.getElementById("soc-noise").value) || 0);
  const gauss = gaussian.bind(null, rng(seed));

  // Filter: starts at 50% with a wide prior, so it must converge from a wrong guess.
  const filter = new Ekf(Float64Array.from(OCV), {
    ...CELL,
    soc0: 0.5,
    soc_variance: 0.01,
  });
  const ocvAt = (s) => filter.ocv_voltage(s);

  // Simulated truth: starts at 90% and uses the same equations as the model, with the
  // series resistance mismatch described above.
  let trueSoc = 0.9;
  let v1 = 0.0;
  const a = Math.exp(-SOC_DT / (CELL.r1 * CELL.c1));
  const truth = [];
  const estimate = [];
  let sq = 0;
  truth.push(trueSoc);
  estimate.push(filter.soc);

  for (let k = 1; k <= SOC_STEPS; k++) {
    trueSoc = Math.min(1, Math.max(0, trueSoc - (SOC_DT * SOC_CURRENT) / (3600 * CELL.capacity_ah)));
    v1 = v1 * a + CELL.r1 * (1 - a) * SOC_CURRENT;
    const measured = ocvAt(trueSoc) - v1 - TRUE_R0 * SOC_CURRENT + noise * gauss();

    filter.predict(SOC_CURRENT, SOC_DT);
    filter.update(measured, SOC_CURRENT);

    truth.push(trueSoc);
    estimate.push(filter.soc);
    sq += (filter.soc - trueSoc) ** 2;
  }

  freeIfPossible(filter);
  const rmse = Math.sqrt(sq / SOC_STEPS);
  socRun = { truth, estimate, rmse };

  document.getElementById("soc-final-true").textContent = `${(truth[SOC_STEPS] * 100).toFixed(1)}%`;
  document.getElementById("soc-final-est").textContent = `${(estimate[SOC_STEPS] * 100).toFixed(1)}%`;
  document.getElementById("soc-rmse").textContent = `${(rmse * 100).toFixed(2)} pts`;
  drawSoc();
}

// ---------------------------------------------------------------- start-up

async function main() {
  const started = performance.now();
  try {
    await init();
  } catch (err) {
    setStatus(
      "Could not load the WebAssembly module. Serve this folder over HTTP (for example, " +
        "`python -m http.server` in docs/demo) rather than opening the file directly. " +
        `Details: ${err}`,
      true,
    );
    return;
  }

  try {
    document.getElementById("fraud-section").hidden = false;
    document.getElementById("soc-section").hidden = false;
    trainFraud();
    runSoc();
    const ms = Math.round(performance.now() - started);
    setStatus(`Ready. Module loaded and both demos ran in ${ms} ms.`);
  } catch (err) {
    setStatus(`The demo failed: ${err && err.message ? err.message : err}`, true);
  }

  document.getElementById("fraud-run").addEventListener("click", () => {
    try { trainFraud(); } catch (err) { setStatus(`Isolation Forest failed: ${err.message || err}`, true); }
  });
  document.getElementById("soc-run").addEventListener("click", () => {
    try { runSoc(); } catch (err) { setStatus(`Kalman filter failed: ${err.message || err}`, true); }
  });
}

main();

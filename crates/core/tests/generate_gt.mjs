#!/usr/bin/env node
// Comprehensive ground truth generator for rnadraw
// Generates ~800 dot-bracket structures, runs them through reference WASM,
// and saves valid results to comprehensive_gt.json.
//
// Usage: node tests/generate_gt.mjs
// Requires: /tmp/ref_wasm_factory.js and /tmp/ref_wasm.wasm

import fs from 'fs';

// ── Seeded PRNG (Lehmer LCG) for reproducibility ────────────────────────────
class Rng {
  constructor(seed = 42) { this.state = seed >>> 0 || 1; }
  next() {
    this.state = Math.imul(this.state, 48271) % 0x7fffffff;
    return (this.state - 1) / 0x7ffffffe;
  }
  randInt(lo, hi) { return lo + Math.floor(this.next() * (hi - lo + 1)); }
  pick(arr) { return arr[this.randInt(0, arr.length - 1)]; }
  shuffle(arr) {
    for (let i = arr.length - 1; i > 0; i--) {
      const j = this.randInt(0, i);
      [arr[i], arr[j]] = [arr[j], arr[i]];
    }
    return arr;
  }
}

const rng = new Rng(20260223);

// ── Structure generators ─────────────────────────────────────────────────────

// Pad a structure with external unpaired bases to reach target length
function padToLength(s, targetLen) {
  const deficit = targetLen - s.length;
  if (deficit <= 0) return s;
  const left = rng.randInt(0, deficit);
  const right = deficit - left;
  return '.'.repeat(left) + s + '.'.repeat(right);
}

// Generate a hairpin: (((...)))
function genHairpin(len) {
  // At minimum: 2 stem + 1 loop = 3
  const maxStem = Math.floor((len - 1) / 2);
  const stemDepth = rng.randInt(1, Math.max(1, Math.min(maxStem, 15)));
  const loopSize = Math.max(1, len - 2 * stemDepth);
  return '('.repeat(stemDepth) + '.'.repeat(loopSize) + ')'.repeat(stemDepth);
}

// Generate an internal loop: ((..(..)))
function genInternalLoop(len) {
  if (len < 6) return genHairpin(len); // fallback
  // outer stem + left gap + inner stem + inner loop + inner stem + right gap + outer stem
  const outerStem = rng.randInt(1, Math.min(3, Math.floor(len / 4)));
  const remaining = len - 2 * outerStem;
  // left unpaired, inner stem, inner loop, right unpaired
  const leftGap = rng.randInt(1, Math.max(1, Math.min(4, remaining - 4)));
  const rightGap = rng.randInt(1, Math.max(1, Math.min(4, remaining - leftGap - 3)));
  const innerRemaining = remaining - leftGap - rightGap;
  const innerStem = rng.randInt(1, Math.max(1, Math.floor(innerRemaining / 2)));
  const innerLoop = Math.max(1, innerRemaining - 2 * innerStem);
  return '('.repeat(outerStem)
    + '.'.repeat(leftGap)
    + '('.repeat(innerStem)
    + '.'.repeat(innerLoop)
    + ')'.repeat(innerStem)
    + '.'.repeat(rightGap)
    + ')'.repeat(outerStem);
}

// Generate a bulge: ((.(..)))
function genBulge(len) {
  if (len < 5) return genHairpin(len);
  const outerStem = rng.randInt(1, Math.min(3, Math.floor(len / 4)));
  const remaining = len - 2 * outerStem;
  // bulge on one side only
  const bulgeSize = rng.randInt(1, Math.max(1, Math.min(3, remaining - 3)));
  const innerRemaining = remaining - bulgeSize;
  const innerStem = rng.randInt(1, Math.max(1, Math.floor(innerRemaining / 2)));
  const innerLoop = Math.max(1, innerRemaining - 2 * innerStem);
  // Put bulge on left or right side randomly
  if (rng.next() < 0.5) {
    return '('.repeat(outerStem)
      + '.'.repeat(bulgeSize)
      + '('.repeat(innerStem)
      + '.'.repeat(innerLoop)
      + ')'.repeat(innerStem)
      + ')'.repeat(outerStem);
  } else {
    return '('.repeat(outerStem)
      + '('.repeat(innerStem)
      + '.'.repeat(innerLoop)
      + ')'.repeat(innerStem)
      + '.'.repeat(bulgeSize)
      + ')'.repeat(outerStem);
  }
}

// Generate multiloop: outer stem with 2-4 children
function genMultiloop(len) {
  if (len < 10) return genHairpin(len);
  const outerStem = rng.randInt(1, Math.min(3, Math.floor(len / 6)));
  const innerLen = len - 2 * outerStem;
  const numChildren = rng.randInt(2, Math.min(4, Math.floor(innerLen / 3)));

  // Distribute length among children
  const childLens = [];
  let usedLen = 0;
  for (let i = 0; i < numChildren; i++) {
    const remaining = innerLen - usedLen - (numChildren - i - 1) * 2;
    const childLen = (i === numChildren - 1)
      ? Math.max(2, remaining)
      : rng.randInt(2, Math.max(2, Math.floor(remaining / 2)));
    childLens.push(childLen);
    usedLen += childLen;
  }

  // Build children as small hairpins, distribute remaining as unpaired
  let parts = [];
  let totalChildLen = 0;
  for (const cl of childLens) {
    const stem = Math.max(1, Math.floor(cl / 2));
    const loop = Math.max(0, cl - 2 * stem);
    parts.push('('.repeat(stem) + '.'.repeat(loop) + ')'.repeat(stem));
    totalChildLen += stem * 2 + loop;
  }

  // Distribute remaining length as unpaired between children
  const unpairedTotal = Math.max(0, innerLen - totalChildLen);
  const gaps = numChildren + 1; // before first, between each, after last
  let inner = '';
  for (let i = 0; i < parts.length; i++) {
    const gapSize = (i === 0)
      ? Math.floor(unpairedTotal / gaps)
      : rng.randInt(0, Math.max(0, Math.floor(unpairedTotal / gaps)));
    inner += '.'.repeat(Math.min(gapSize, unpairedTotal - inner.replace(/[()]/g, '').length + inner.length - totalChildLen));
    inner += parts[i];
  }
  // Any leftover unpaired at the end
  const usedUnpaired = inner.length - totalChildLen;
  if (usedUnpaired < unpairedTotal) {
    inner += '.'.repeat(unpairedTotal - usedUnpaired);
  }

  return '('.repeat(outerStem) + inner.slice(0, innerLen) + ')'.repeat(outerStem);
}

// Generate nested stem: (((((())))))
function genNestedStem(len) {
  const stemDepth = Math.max(1, Math.floor(len / 2));
  const loop = len - 2 * stemDepth;
  return '('.repeat(stemDepth) + '.'.repeat(Math.max(0, loop)) + ')'.repeat(stemDepth);
}

// Generate external unpaired: ..((..))..
function genExternalUnpaired(len) {
  if (len < 4) return '.'.repeat(len); // will be error, but we filter
  const stemLen = rng.randInt(2, Math.max(2, Math.min(len - 2, Math.floor(len * 0.7))));
  const inner = genHairpin(stemLen);
  return padToLength(inner, len);
}

// Generate combination: concatenate 2-3 small structures
function genCombination(len) {
  const numParts = rng.randInt(2, 3);
  const generators = [genHairpin, genNestedStem, genBulge];
  const parts = [];
  let remaining = len;

  for (let i = 0; i < numParts; i++) {
    const partLen = (i === numParts - 1)
      ? remaining
      : rng.randInt(2, Math.max(2, Math.floor(remaining / 2)));
    const gen = rng.pick(generators);
    parts.push(gen(Math.max(2, partLen)));
    remaining -= partLen;
  }
  return parts.join('');
}

// ── Validation ───────────────────────────────────────────────────────────────

function isValidDotBracketPlus(s) {
  // Check balanced parentheses per strand
  let depth = 0;
  let hasPair = false;
  for (const ch of s) {
    if (ch === '(') { depth++; hasPair = true; }
    else if (ch === ')') { depth--; if (depth < 0) return false; }
    else if (ch === '+') { /* strand break, depth carries over */ }
    else if (ch !== '.') return false;
  }
  return depth === 0 && hasPair;
}

// Ensure no empty strands (adjacent + or leading/trailing +)
function cleanStructure(s) {
  // Remove any characters that aren't valid
  s = s.replace(/[^().+]/g, '');
  // Remove double+ or leading/trailing +
  s = s.replace(/\+{2,}/g, '+').replace(/^\+/, '').replace(/\+$/, '');
  return s;
}

// ── Multi-strand helpers ─────────────────────────────────────────────────────

// Insert strand breaks into a valid structure for 2-strand
function makeMultiStrand2(structure) {
  // Find valid insertion points: between any two characters that aren't paired to each other
  const validPositions = [];
  for (let i = 1; i < structure.length; i++) {
    // Don't insert between a ( and its matching )
    validPositions.push(i);
  }
  if (validPositions.length === 0) return null;
  const pos = rng.pick(validPositions);
  return structure.slice(0, pos) + '+' + structure.slice(pos);
}

// Insert multiple strand breaks for 3+ strand
function makeMultiStrand3Plus(structure, numStrands) {
  const breaks = numStrands - 1;
  const positions = [];
  for (let i = 1; i < structure.length; i++) positions.push(i);
  if (positions.length < breaks) return null;

  // Pick unique positions
  rng.shuffle(positions);
  const chosen = positions.slice(0, breaks).sort((a, b) => b - a); // reverse order for insertion
  let result = structure;
  for (const pos of chosen) {
    result = result.slice(0, pos) + '+' + result.slice(pos);
  }
  return result;
}

// ── Main generation logic ────────────────────────────────────────────────────

const RANGES = [
  { name: 'XS',    lo: 2,   hi: 10,  count: 100 },
  { name: 'S',     lo: 11,  hi: 25,  count: 120 },
  { name: 'M',     lo: 26,  hi: 50,  count: 120 },
  { name: 'L',     lo: 51,  hi: 100, count: 100 },
  { name: 'XL',    lo: 101, hi: 200, count: 100 },
  { name: 'XXL',   lo: 201, hi: 400, count: 100 },
];

const PATTERN_DIST = [
  { gen: genHairpin,          weight: 20 },
  { gen: genInternalLoop,     weight: 15 },
  { gen: genBulge,            weight: 15 },
  { gen: genMultiloop,        weight: 20 },
  { gen: genNestedStem,       weight: 10 },
  { gen: genExternalUnpaired, weight: 10 },
  { gen: genCombination,      weight: 10 },
];

function pickPattern() {
  const totalWeight = PATTERN_DIST.reduce((s, p) => s + p.weight, 0);
  let r = rng.next() * totalWeight;
  for (const p of PATTERN_DIST) {
    r -= p.weight;
    if (r <= 0) return p.gen;
  }
  return PATTERN_DIST[0].gen;
}

function generateCases() {
  const seen = new Set();
  const allCases = [];

  // Single-strand cases by length range
  for (const range of RANGES) {
    let generated = 0;
    let attempts = 0;
    while (generated < range.count && attempts < range.count * 20) {
      attempts++;
      const len = rng.randInt(range.lo, range.hi);
      const gen = pickPattern();
      let structure;
      try {
        structure = gen(len);
      } catch {
        continue;
      }
      structure = cleanStructure(structure);
      // Verify actual length is in range (generators may not be exact)
      if (structure.length < range.lo || structure.length > range.hi + 10) continue;
      if (!isValidDotBracketPlus(structure)) continue;
      if (seen.has(structure)) continue;
      seen.add(structure);
      allCases.push({ structure, range: range.name });
      generated++;
    }
    console.error(`  ${range.name}: generated ${generated}/${range.count} (${attempts} attempts)`);
  }

  // 2-strand cases
  {
    let generated = 0;
    let attempts = 0;
    while (generated < 100 && attempts < 2000) {
      attempts++;
      const len = rng.randInt(4, 60);
      const gen = pickPattern();
      let base;
      try { base = gen(len); } catch { continue; }
      base = cleanStructure(base);
      if (!isValidDotBracketPlus(base)) continue;
      const ms = makeMultiStrand2(base);
      if (!ms || !isValidDotBracketPlus(ms)) continue;
      if (seen.has(ms)) continue;
      seen.add(ms);
      allCases.push({ structure: ms, range: 'MS-2' });
      generated++;
    }
    console.error(`  MS-2: generated ${generated}/100 (${attempts} attempts)`);
  }

  // 3+ strand cases (experimental, many will ERROR)
  {
    let generated = 0;
    let attempts = 0;
    while (generated < 100 && attempts < 5000) {
      attempts++;
      const numStrands = rng.pick([3, 5]);
      const len = rng.randInt(6, 40);
      const gen = pickPattern();
      let base;
      try { base = gen(len); } catch { continue; }
      base = cleanStructure(base);
      if (!isValidDotBracketPlus(base)) continue;
      const ms = makeMultiStrand3Plus(base, numStrands);
      if (!ms || !isValidDotBracketPlus(ms)) continue;
      if (seen.has(ms)) continue;
      seen.add(ms);
      allCases.push({ structure: ms, range: 'MS-3+' });
      generated++;
    }
    console.error(`  MS-3+: generated ${generated}/100 (${attempts} attempts)`);
  }

  return allCases;
}

// ── WASM loading ─────────────────────────────────────────────────────────────

async function loadReferenceWasm() {
  globalThis.window = { location: { origin: '' } };
  globalThis.document = {
    currentScript: { src: '' },
    querySelector: () => null,
    createElement: () => ({ style: {} }),
    getElementById: () => null,
  };
  globalThis.self = globalThis;
  globalThis.XMLHttpRequest = class {};

  let factoryCode = fs.readFileSync('/tmp/ref_wasm_factory.js', 'utf8').trimEnd();
  if (factoryCode.endsWith(',')) factoryCode = factoryCode.slice(0, -1);

  factoryCode = factoryCode.replace(
    /fetch\([^)]+credentials[^)]+\)/g,
    `(async () => {
      const _fs = await import('fs');
      const buf = _fs.default.readFileSync('/tmp/ref_wasm.wasm');
      return { ok: true, arrayBuffer: () => Promise.resolve(buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength)) };
    })()`
  );
  factoryCode = factoryCode.replace(
    /WebAssembly\.instantiateStreaming\([^)]+\)/g,
    `Promise.reject("no streaming")`
  );
  factoryCode = factoryCode.replace(/window\.location\.origin/g, '""');

  const getFactory = new Function(`${factoryCode}; return Ct;`);
  const Ct = getFactory();
  const mod = await Ct({ noInitialRun: true, noExitRuntime: true });
  return mod;
}

// ── Main ─────────────────────────────────────────────────────────────────────

async function main() {
  console.error('Loading reference WASM...');
  const mod = await loadReferenceWasm();
  if (!mod?.draw_structure) {
    console.error('ERROR: draw_structure not found in WASM module');
    process.exit(1);
  }
  console.error('WASM loaded. draw_structure available.');

  console.error('\nGenerating structures...');
  const cases = generateCases();
  console.error(`\nTotal structures generated: ${cases.length}`);

  console.error('\nRunning through reference WASM...');
  const results = {};
  const stats = { valid: 0, error: 0, exception: 0 };
  const rangeStats = {};

  for (const { structure, range } of cases) {
    if (!rangeStats[range]) rangeStats[range] = { valid: 0, error: 0 };
    try {
      const output = mod.draw_structure(structure);
      if (output.startsWith('ERROR') || output === 'undefined') {
        stats.error++;
        rangeStats[range].error++;
      } else {
        const parsed = JSON.parse(output);
        results[structure] = parsed;
        stats.valid++;
        rangeStats[range].valid++;
      }
    } catch (e) {
      stats.exception++;
      rangeStats[range].error++;
    }
  }

  console.error('\n── Results ──');
  console.error(`  Valid: ${stats.valid}`);
  console.error(`  Error: ${stats.error}`);
  console.error(`  Exception: ${stats.exception}`);
  console.error('\nPer range:');
  for (const [range, s] of Object.entries(rangeStats)) {
    console.error(`  ${range}: ${s.valid} valid, ${s.error} error`);
  }

  const outPath = new URL('fixtures/comprehensive_gt.json', import.meta.url).pathname;
  fs.writeFileSync(outPath, JSON.stringify(results, null, 2));
  console.error(`\nSaved ${Object.keys(results).length} cases to ${outPath}`);
}

main().catch(e => { console.error(e); process.exit(1); });

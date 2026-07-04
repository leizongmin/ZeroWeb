import { mkdirSync, writeFileSync } from 'node:fs';
import { join, dirname } from 'node:path';

const ARTIFACTS_ROOT = '.acceptance/artifacts';

export function evidenceDir(label) {
  const date = new Date().toISOString().slice(0, 10);
  return join(ARTIFACTS_ROOT, `${label}-${date}`);
}

export function ensureDir(dir) {
  mkdirSync(dir, { recursive: true });
}

export function saveJson(filePath, data) {
  const abs = filePath;
  ensureDir(dirname(abs));
  writeFileSync(abs, JSON.stringify(data, null, 2));
}

export function saveText(filePath, text) {
  const abs = filePath;
  ensureDir(dirname(abs));
  writeFileSync(abs, text);
}

export function step(label, fn) {
  console.log(`\n  ▶ ${label}`);
  const result = fn();
  console.log(`  ✓ ${label}`);
  return result;
}

export async function stepAsync(label, fn) {
  console.log(`\n  ▶ ${label}`);
  const result = await fn();
  console.log(`  ✓ ${label}`);
  return result;
}

export function assert(condition, message) {
  if (!condition) {
    const err = new Error(`ASSERT FAILED: ${message}`);
    console.error(err.message);
    throw err;
  }
}

export function assertEqual(actual, expected, label) {
  const ok = actual === expected;
  if (!ok) {
    const msg = `${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`;
    console.error(`  ✗ ${msg}`);
    throw new Error(msg);
  }
  console.log(`  ✓ ${label}: ${JSON.stringify(actual)}`);
}

export function assertClose(actual, expected, tolerance, label) {
  const diff = Math.abs(actual - expected);
  if (diff > tolerance) {
    const msg = `${label}: expected ~${expected} (tolerance ${tolerance}), got ${actual}`;
    console.error(`  ✗ ${msg}`);
    throw new Error(msg);
  }
  console.log(`  ✓ ${label}: ${actual} (expected ~${expected})`);
}

export async function retry(fn, retries = 3, delay = 1000) {
  for (let i = 0; i < retries; i++) {
    try {
      return await fn();
    } catch (e) {
      if (i === retries - 1) throw e;
      console.warn(`  ⚠ retry ${i + 1}/${retries}: ${e.message}`);
      await new Promise((r) => setTimeout(r, delay));
    }
  }
}

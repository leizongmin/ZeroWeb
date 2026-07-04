import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const ARTIFACTS = join('.acceptance', 'artifacts');

function loadSummary(label) {
  const dir = join(ARTIFACTS, `${label}-2026-07-04`);
  const path = join(dir, 'jsons', '_summary.json');
  if (!existsSync(path)) return { status: 'unknown' };
  return JSON.parse(readFileSync(path, 'utf-8'));
}

const recipes = [
  { name: 'smoke', label: 'smoke', file: 'smoke.js' },
  { name: 'perf', label: 'perf', file: 'perf.js' },
  { name: 'P0 regression', label: 'p0-regression', file: 'recipe1-p0-regression.js' },
  { name: 'parity diff', label: 'parity-diff', file: 'recipe2-parity-diff.js' },
  { name: 'full regression', label: 'full-regression', file: 'recipe3-full-regression.js' },
];

for (const r of recipes) {
  const summary = loadSummary(r.label);
  const pass = summary.failures ? summary.failures.length === 0 : true;
  r.status = pass ? 'pass' : 'fail';
  r.summary = summary;
  console.log(`${r.name}: ${r.status}`);
}

const result = {
  profile: 'desktop_cdp',
  timestamp: new Date().toISOString(),
  total: recipes.length,
  pass: recipes.filter(r => r.status === 'pass').length,
  fail: recipes.filter(r => r.status === 'fail').length,
  recipes,
};

writeFileSync(join(ARTIFACTS, '..', 'reports', 'acceptance-result.json'), JSON.stringify(result, null, 2));
console.log(`\nResult: ${result.pass}/${result.total} passed`);

#!/usr/bin/env node

import { cp, mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';

import { chromeCandidates } from './capture-chrome.mjs';
import { defaultComparatorPath } from './compare-evidence.mjs';
import { validateScenario } from './validate-scenario.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function run(command, args) {
  return new Promise((done) => {
    const child = spawn(command, args, { stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (data) => { stdout += data; });
    child.stderr.on('data', (data) => { stderr += data; });
    child.on('close', (code) => done({ code, stdout, stderr }));
  });
}

const scenario = {
  version: 1,
  name: 'self-test',
  url: 'file://${REPO_ROOT}/fixture.html',
  viewport: { width: 8, height: 8, dpr: 1 },
  thresholds: {
    maxDiffPercent: 5,
    maxRegionDiffPercent: 10,
    channelDiff: 8,
    pixelRadius: 1,
    maxGeometryDiffPx: 2,
  },
  observe: {
    selectors: ['#control'],
    stateExpression: '({ checked: true })',
    eventTypes: ['click'],
  },
  steps: [{ id: 'initial', action: { type: 'snapshot' } }],
};

async function main() {
  validateScenario(scenario);
  let rejected = false;
  try {
    validateScenario({ ...scenario, steps: [] });
  } catch {
    rejected = true;
  }
  assert(rejected, 'validator must reject an empty scenario');

  const windowsCandidates = chromeCandidates('win32', {
    PROGRAMFILES: 'C:\\Program Files',
    'PROGRAMFILES(X86)': 'C:\\Program Files (x86)',
    LOCALAPPDATA: 'C:\\LocalAppData',
  });
  assert(windowsCandidates.some((path) => path.endsWith('chrome.exe')), 'Windows Chrome candidates must use .exe');
  assert(
    chromeCandidates('darwin', {}).some((path) => path.includes('.app/Contents/MacOS/')),
    'macOS Chrome candidates must use app bundle executables',
  );
  assert(chromeCandidates('linux', {}).some((path) => path === '/usr/bin/chromium'), 'Linux Chromium path missing');
  assert(defaultComparatorPath('win32').endsWith('zero-wpt-runner.exe'), 'Windows comparator must use .exe');
  assert(!defaultComparatorPath('linux').endsWith('.exe'), 'Unix comparator must not use .exe');

  const root = await mkdtemp(resolve(tmpdir(), 'zeroweb-browser-chrome-parity-'));
  try {
    const chrome = resolve(root, 'chrome');
    const zeroweb = resolve(root, 'zeroweb');
    await mkdir(chrome);
    await mkdir(zeroweb);
    const scenarioPath = resolve(root, 'scenario.json');
    await writeFile(scenarioPath, `${JSON.stringify(scenario)}\n`);

    const placeholder = resolve(root, 'placeholder.png');
    await writeFile(placeholder, Buffer.from('not decoded by mock comparator'));
    for (const directory of [chrome, zeroweb]) {
      await cp(placeholder, resolve(directory, 'initial.png'));
      await cp(placeholder, resolve(directory, 'region.png'));
    }

    const step = {
      id: 'initial',
      action: { type: 'snapshot' },
      screenshot: 'initial.png',
      regions: { '#control': 'region.png' },
      state: { checked: true },
      events: [],
      geometry: { '#control': { x: 1, y: 1, width: 4, height: 4 } },
    };
    await writeFile(resolve(chrome, 'manifest.json'), `${JSON.stringify({
      schemaVersion: 1,
      scenario: 'self-test',
      engine: 'chrome',
      capturePath: 'chrome-cdp-gui',
      inputPath: 'browser-pointer',
      steps: [step],
    })}\n`);
    await writeFile(resolve(zeroweb, 'manifest.json'), `${JSON.stringify({
      schemaVersion: 1,
      scenario: 'self-test',
      engine: 'zeroweb',
      capturePath: 'production-window-gpu',
      inputPath: 'browser-pointer',
      steps: [step],
    })}\n`);

    const comparator = resolve(root, 'mock-comparator.mjs');
    await writeFile(comparator, 'console.log("PNG diff: 0/64 pixels = 0.00% (mock)");\n');
    const reportPath = resolve(root, 'report.json');
    const result = await run(process.execPath, [
      resolve(HERE, 'compare-evidence.mjs'),
      '--scenario', scenarioPath,
      '--chrome', chrome,
      '--zeroweb', zeroweb,
      '--out', reportPath,
      '--comparator', comparator,
      '--require-production',
    ]);
    assert(result.code === 0, `comparator self-test failed: ${result.stdout}\n${result.stderr}`);
    const report = JSON.parse(await readFile(reportPath, 'utf8'));
    assert(report.passed === true, 'matching production evidence must pass');

    const zeroManifestPath = resolve(zeroweb, 'manifest.json');
    const zeroManifest = JSON.parse(await readFile(zeroManifestPath, 'utf8'));
    zeroManifest.capturePath = 'renderer-only';
    await writeFile(zeroManifestPath, `${JSON.stringify(zeroManifest)}\n`);
    const rejectedReport = resolve(root, 'rejected-report.json');
    const rejectedResult = await run(process.execPath, [
      resolve(HERE, 'compare-evidence.mjs'),
      '--scenario', scenarioPath,
      '--chrome', chrome,
      '--zeroweb', zeroweb,
      '--out', rejectedReport,
      '--comparator', comparator,
      '--require-production',
    ]);
    assert(rejectedResult.code === 1, 'non-production evidence must fail');
    console.log('zeroweb-browser-chrome-parity self-test: PASS');
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});

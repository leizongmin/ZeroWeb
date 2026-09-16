#!/usr/bin/env node

import { cp, mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';

import { chromeCandidates, chromeLaunchArgs, detectCapturePath } from './capture-chrome.mjs';
import { defaultComparatorPath, validateManifest } from './compare-evidence.mjs';
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
  environment: { locale: 'en-US', colorScheme: 'light', reducedMotion: 'no-preference' },
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
  validateScenario({
    ...scenario,
    steps: [{ id: 'click-unobserved', action: { type: 'click', selector: '.not-observed' } }],
  });
  const genericTemplate = JSON.parse(await readFile(resolve(HERE, '../templates/generic-page.scenario.json'), 'utf8'));
  validateScenario(genericTemplate);
  assert(
    genericTemplate.observe.selectors.some((selector) => selector.includes('[name=')),
    'generic template must exercise a non-ID CSS selector',
  );
  let rejected = false;
  try {
    validateScenario({ ...scenario, steps: [] });
  } catch {
    rejected = true;
  }
  assert(rejected, 'validator must reject an empty scenario');
  rejected = false;
  try {
    validateScenario({ ...scenario, environment: { ...scenario.environment, reducedMotion: 'reduce' } });
  } catch {
    rejected = true;
  }
  assert(rejected, 'validator must reject an environment ZeroWeb cannot reproduce');

  const windowsCandidates = chromeCandidates('win32', {
    PROGRAMFILES: 'C:\\Program Files',
    'PROGRAMFILES(X86)': 'C:\\Program Files (x86)',
    LOCALAPPDATA: 'C:\\LocalAppData',
  });
  assert(windowsCandidates.some((path) => path.endsWith('chrome.exe')), 'Windows Chrome candidates must use .exe');
  const firstEdge = windowsCandidates.findIndex((path) => path.endsWith('msedge.exe'));
  assert(firstEdge >= 0, 'Windows Edge candidate missing');
  assert(
    windowsCandidates.slice(0, firstEdge).every((path) => path.endsWith('chrome.exe')),
    'Windows Edge candidates must follow Chrome/Chromium candidates',
  );
  const macCandidates = chromeCandidates('darwin', {});
  assert(
    macCandidates.some((path) => path.includes('.app/Contents/MacOS/')),
    'macOS Chrome candidates must use app bundle executables',
  );
  assert(macCandidates.some((path) => path.includes('Microsoft Edge.app')), 'macOS Edge candidate missing');
  const linuxCandidates = chromeCandidates('linux', {});
  assert(linuxCandidates.some((path) => path === '/usr/bin/chromium'), 'Linux Chromium path missing');
  assert(linuxCandidates.some((path) => path.endsWith('/microsoft-edge')), 'Linux Edge candidate missing');
  assert(defaultComparatorPath('win32').endsWith('zero-wpt-runner.exe'), 'Windows comparator must use .exe');
  assert(!defaultComparatorPath('linux').endsWith('.exe'), 'Unix comparator must not use .exe');
  assert(!chromeLaunchArgs('en-US', {}).includes('--no-sandbox'), 'sandbox must be enabled by default');
  assert(chromeLaunchArgs('en-US', { PARITY_CHROME_NO_SANDBOX: '1' }).includes('--no-sandbox'),
    'isolated runs may explicitly disable sandbox');
  for (const [version, args, expected] of [
    ['HeadlessChrome/127', ['--enable-automation'], 'chrome-headless'],
    ['Chrome/127', ['--enable-automation', '--headless=new'], 'chrome-headless'],
    ['Chrome/127', ['--enable-automation', '--headless'], 'chrome-headless'],
    ['Chrome/127', ['--enable-automation'], 'chrome-cdp-gui'],
    ['Chrome/127', [], 'chrome-cdp-unverified'],
    ['Chrome/127', null, 'chrome-cdp-unverified'],
  ]) {
    const browser = {
      version: async () => version,
      target: () => ({ createCDPSession: async () => ({
        send: async () => {
          if (args === null) throw new Error('Command unavailable');
          return { arguments: args };
        },
        detach: async () => {},
      }) }),
    };
    assert(await detectCapturePath(browser) === expected, `CDP mode must be ${expected}`);
  }

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
      engineVersion: 'Chrome/127.0.0.0',
      capturePath: 'chrome-cdp-gui',
      inputPath: 'browser-pointer',
      viewport: scenario.viewport,
      environment: scenario.environment,
      steps: [step],
    })}\n`);
    await writeFile(resolve(zeroweb, 'manifest.json'), `${JSON.stringify({
      schemaVersion: 1,
      scenario: 'self-test',
      engine: 'zeroweb',
      engineVersion: 'test-build',
      capturePath: 'production-window-gpu',
      inputPath: 'browser-pointer',
      viewport: scenario.viewport,
      environment: scenario.environment,
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

    // 每个缺证据反例独立从完整 fixture 派生；两端同时缺失也必须失败。
    const originalChrome = JSON.parse(await readFile(resolve(chrome, 'manifest.json'), 'utf8'));
    const originalZero = JSON.parse(await readFile(resolve(zeroweb, 'manifest.json'), 'utf8'));
    const clickScenario = structuredClone(scenario);
    clickScenario.steps[0].action = { type: 'click', selector: '#control' };
    const rustClick = structuredClone(originalZero);
    rustClick.steps[0].action = { ...clickScenario.steps[0].action, offset: null, jitter: null };
    validateManifest(rustClick, clickScenario, 'zeroweb');
    const nullState = structuredClone(originalZero);
    nullState.steps[0].state = null;
    validateManifest(nullState, scenario, 'zeroweb');
    const mutations = [
      ['missing state', (c, z) => { delete c.steps[0].state; delete z.steps[0].state; }],
      ['missing events', (c, z) => { delete c.steps[0].events; delete z.steps[0].events; }],
      ['missing geometry', (c, z) => { delete c.steps[0].geometry; delete z.steps[0].geometry; }],
      ['missing selector', (c, z) => { delete c.steps[0].geometry['#control']; delete z.steps[0].geometry['#control']; }],
      ['invalid geometry', (c, z) => { z.steps[0].geometry['#control'].x = '1'; }],
      ['invalid event', (c, z) => { z.steps[0].events = [{ type: 'click' }]; }],
      ['duplicate checkpoint', (c, z) => { z.steps.push(z.steps[0]); }],
      ['missing checkpoint', (c, z) => { z.steps = []; }],
      ['unexpected checkpoint', (c, z) => { z.steps.push({ ...z.steps[0], id: 'extra' }); }],
      ['wrong action', (c, z) => { z.steps[0].action = { type: 'key', key: 'Tab' }; }],
      ['missing viewport', (c, z) => { delete z.viewport; }],
      ['wrong viewport', (c, z) => { z.viewport.width += 1; }],
      ['wrong DPR', (c, z) => { z.viewport.dpr = 2; }],
      ['missing environment', (c, z) => { delete c.environment; delete z.environment; }],
      ['wrong environment', (c, z) => { z.environment.colorScheme = 'dark'; }],
      ['wrong engine', (c, z) => { z.engine = 'chrome'; }],
      ['missing version', (c, z) => { delete z.engineVersion; }],
      ['wrong Chrome input', c => { c.inputPath = 'script-dispatch'; }],
      ['unverified Chrome', c => { c.capturePath = 'chrome-cdp-unverified'; }],
      ['missing screenshot', (c, z) => { delete z.steps[0].screenshot; }],
      ['missing region', (c, z) => { delete z.steps[0].regions['#control']; }],
    ];
    for (const [label, mutate] of mutations) {
      const c = structuredClone(originalChrome);
      const z = structuredClone(originalZero);
      mutate(c, z);
      await writeFile(resolve(chrome, 'manifest.json'), JSON.stringify(c));
      await writeFile(resolve(zeroweb, 'manifest.json'), JSON.stringify(z));
      // 删除旧报告，防止异常退出后误读上次 PASS。
      await rm(reportPath, { force: true });
      const invalid = await run(process.execPath, [
        resolve(HERE, 'compare-evidence.mjs'), '--scenario', scenarioPath,
        '--chrome', chrome, '--zeroweb', zeroweb, '--out', reportPath,
        '--comparator', comparator, '--require-production',
      ]);
      assert(invalid.code === 1, `${label} must fail, got ${invalid.code}`);
      console.log(`rejected: ${label}`);
    }
    await writeFile(resolve(chrome, 'manifest.json'), JSON.stringify(originalChrome));
    await writeFile(resolve(zeroweb, 'manifest.json'), JSON.stringify(originalZero));

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

#!/usr/bin/env node

import { access, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawn } from 'node:child_process';
import { isDeepStrictEqual } from 'node:util';

import { loadScenario } from './validate-scenario.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, '../../../..');

export function defaultComparatorPath(platform) {
  return resolve(REPO_ROOT, `target/release/zero-wpt-runner${platform === 'win32' ? '.exe' : ''}`);
}

function parseArgs() {
  const args = process.argv.slice(2);
  const result = {
    scenario: null,
    chrome: null,
    zeroweb: null,
    out: null,
    requireProduction: false,
    comparator: process.env.PARITY_PNG_COMPARATOR
      || defaultComparatorPath(process.platform),
  };
  for (let index = 0; index < args.length; index += 1) {
    const option = args[index];
    if (option === '--scenario') result.scenario = args[++index];
    else if (option === '--chrome') result.chrome = args[++index];
    else if (option === '--zeroweb') result.zeroweb = args[++index];
    else if (option === '--out') result.out = args[++index];
    else if (option === '--comparator') result.comparator = args[++index];
    else if (option === '--require-production') result.requireProduction = true;
    else if (option === '--help') {
      console.log('用法: compare-evidence.mjs --scenario <json> --chrome <目录> --zeroweb <目录> --out <json> [--require-production]');
      process.exit(0);
    } else throw new Error(`unknown option: ${option}`);
  }
  for (const key of ['scenario', 'chrome', 'zeroweb', 'out']) {
    if (!result[key]) throw new Error(`--${key} is required`);
  }
  return result;
}

async function loadManifest(directory) {
  const path = resolve(directory, 'manifest.json');
  const manifest = JSON.parse(await readFile(path, 'utf8'));
  if (manifest.schemaVersion !== 1) throw new Error(`${path}: schemaVersion must be 1`);
  if (!Array.isArray(manifest.steps)) throw new Error(`${path}: steps must be an array`);
  return { path, directory: resolve(directory), manifest };
}

function canonicalEvents(events) {
  return (events || []).map((event) => ({
    type: event.type,
    target: event.target,
    defaultPrevented: Boolean(event.defaultPrevented),
  }));
}

function equalJson(left, right) {
  return isDeepStrictEqual(left, right);
}

function compareGeometry(chrome, zeroweb, limit) {
  const failures = [];
  let maxDelta = 0;
  const selectors = new Set([...Object.keys(chrome || {}), ...Object.keys(zeroweb || {})]);
  for (const selector of selectors) {
    const expected = chrome?.[selector];
    const actual = zeroweb?.[selector];
    if (!expected || !actual) {
      failures.push(`${selector}: missing geometry`);
      continue;
    }
    for (const field of ['x', 'y', 'width', 'height']) {
      const delta = Math.abs(Number(expected[field]) - Number(actual[field]));
      if (!Number.isFinite(delta)) {
        failures.push(`${selector}.${field}: non-numeric geometry`);
      } else {
        maxDelta = Math.max(maxDelta, delta);
        if (delta > limit) failures.push(`${selector}.${field}: ${delta.toFixed(3)}px > ${limit}px`);
      }
    }
  }
  return { passed: failures.length === 0, maxDeltaPx: maxDelta, failures };
}

function run(command, args) {
  return new Promise((done) => {
    const child = spawn(command, args, { cwd: REPO_ROOT, stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (data) => { stdout += data; });
    child.stderr.on('data', (data) => { stderr += data; });
    child.on('error', (error) => done({ code: null, stdout, stderr, error }));
    child.on('close', (code) => done({ code, stdout, stderr, error: null }));
  });
}

async function comparePng(options, actual, expected, maxDiffPercent) {
  await access(actual);
  await access(expected);
  const args = [
    'compare-png',
    actual,
    expected,
    '--max-diff',
    String(maxDiffPercent),
    '--channel-diff',
    String(options.thresholds.channelDiff),
    '--pixel-radius',
    String(options.thresholds.pixelRadius),
  ];
  if (options.padToUnion) args.push('--pad-to-union');
  if (options.ignoreInset !== undefined) {
    args.push('--ignore-inset', String(options.ignoreInset));
  }
  const comparatorIsNodeScript = options.comparator.endsWith('.mjs') || options.comparator.endsWith('.js');
  const result = comparatorIsNodeScript
    ? await run(process.execPath, [options.comparator, ...args])
    : await run(options.comparator, args);
  if (result.error) throw new Error(`failed to run PNG comparator: ${result.error.message}`);
  const output = `${result.stdout}\n${result.stderr}`.trim();
  const match = output.match(/PNG diff:\s*(\d+)\/(\d+)\s+pixels\s+=\s+([0-9.]+)%/);
  return {
    passed: result.code === 0,
    diffPixels: match ? Number(match[1]) : null,
    totalPixels: match ? Number(match[2]) : null,
    diffPercent: match ? Number(match[3]) : null,
    maxDiffPercent,
    output,
  };
}

async function main() {
  const cli = parseArgs();
  const scenario = await loadScenario(cli.scenario);
  const chrome = await loadManifest(cli.chrome);
  const zeroweb = await loadManifest(cli.zeroweb);
  if (chrome.manifest.scenario !== scenario.name || zeroweb.manifest.scenario !== scenario.name) {
    throw new Error('manifest scenario does not match the requested scenario');
  }

  const production = {
    chrome: chrome.manifest.capturePath === 'chrome-cdp-gui',
    zeroweb: zeroweb.manifest.capturePath === 'production-window-gpu'
      && zeroweb.manifest.inputPath === 'browser-pointer',
  };
  const steps = [];
  const chromeSteps = new Map(chrome.manifest.steps.map((step) => [step.id, step]));
  const zeroSteps = new Map(zeroweb.manifest.steps.map((step) => [step.id, step]));

  for (const expectedStep of scenario.steps) {
    const chromeStep = chromeSteps.get(expectedStep.id);
    const zeroStep = zeroSteps.get(expectedStep.id);
    if (!chromeStep || !zeroStep) {
      steps.push({
        id: expectedStep.id,
        passed: false,
        failures: [
          !chromeStep ? 'missing Chrome checkpoint' : null,
          !zeroStep ? 'missing ZeroWeb checkpoint' : null,
        ].filter(Boolean),
      });
      continue;
    }

    const state = {
      passed: equalJson(chromeStep.state, zeroStep.state),
      chrome: chromeStep.state,
      zeroweb: zeroStep.state,
    };
    const chromeEvents = canonicalEvents(chromeStep.events);
    const zeroEvents = canonicalEvents(zeroStep.events);
    const events = {
      passed: equalJson(chromeEvents, zeroEvents),
      chrome: chromeEvents,
      zeroweb: zeroEvents,
    };
    const geometry = compareGeometry(
      chromeStep.geometry,
      zeroStep.geometry,
      scenario.thresholds.maxGeometryDiffPx,
    );
    let pixels;
    try {
      pixels = await comparePng(
        {
          comparator: cli.comparator,
          thresholds: scenario.thresholds,
        },
        resolve(zeroweb.directory, zeroStep.screenshot),
        resolve(chrome.directory, chromeStep.screenshot),
        scenario.thresholds.maxDiffPercent,
      );
    } catch (error) {
      pixels = { passed: false, error: error.message };
    }

    const regions = {};
    for (const selector of scenario.observe.selectors) {
      const chromeRegion = chromeStep.regions?.[selector];
      const zeroRegion = zeroStep.regions?.[selector];
      if (!chromeRegion || !zeroRegion) {
        regions[selector] = { passed: false, error: 'missing target-region screenshot' };
        continue;
      }
      try {
        const unmasked = await comparePng(
          {
            comparator: cli.comparator,
            thresholds: scenario.thresholds,
            padToUnion: true,
          },
          resolve(zeroweb.directory, zeroRegion),
          resolve(chrome.directory, chromeRegion),
          scenario.thresholds.maxRegionDiffPercent,
        );
        const glyphMaskInsetPx = scenario.observe.glyphMaskInsetPx?.[selector];
        if (glyphMaskInsetPx === undefined) {
          regions[selector] = unmasked;
        } else {
          regions[selector] = {
            ...await comparePng(
              {
                comparator: cli.comparator,
                thresholds: scenario.thresholds,
                padToUnion: true,
                ignoreInset: glyphMaskInsetPx,
              },
              resolve(zeroweb.directory, zeroRegion),
              resolve(chrome.directory, chromeRegion),
              scenario.thresholds.maxRegionDiffPercent,
            ),
            glyphMaskInsetPx,
            unmasked,
          };
        }
      } catch (error) {
        regions[selector] = { passed: false, error: error.message };
      }
    }

    const failures = [];
    if (!state.passed) failures.push('state mismatch');
    if (!events.passed) failures.push('event sequence mismatch');
    if (!geometry.passed) failures.push('geometry mismatch');
    if (!pixels.passed) failures.push('full-frame pixel mismatch');
    for (const [selector, result] of Object.entries(regions)) {
      if (!result.passed) failures.push(`${selector} region mismatch`);
    }
    steps.push({
      id: expectedStep.id,
      passed: failures.length === 0,
      failures,
      state,
      events,
      geometry,
      pixels,
      regions,
    });
  }

  const productionPassed = !cli.requireProduction || (production.chrome && production.zeroweb);
  const report = {
    schemaVersion: 1,
    scenario: scenario.name,
    requireProduction: cli.requireProduction,
    production,
    passed: productionPassed && steps.every((step) => step.passed),
    thresholds: scenario.thresholds,
    steps,
  };
  await mkdir(dirname(resolve(cli.out)), { recursive: true });
  await writeFile(resolve(cli.out), `${JSON.stringify(report, null, 2)}\n`);
  console.log(`一致性报告: ${resolve(cli.out)}`);
  console.log(`结果: ${report.passed ? 'PASS' : 'FAIL'}`);
  if (!report.passed) process.exitCode = 1;
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  main().catch((error) => {
    console.error(error.stack || error.message);
    process.exit(1);
  });
}

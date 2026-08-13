#!/usr/bin/env node

import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const ACTION_FIELDS = {
  snapshot: [],
  click: ['selector'],
  type: ['text'],
  key: ['key'],
  wait: ['milliseconds'],
};

function fail(message) {
  throw new Error(`一致性场景无效: ${message}`);
}

function finitePositive(value, path) {
  if (typeof value !== 'number' || !Number.isFinite(value) || value <= 0) {
    fail(`${path} must be a positive finite number`);
  }
}

export function validateScenario(scenario) {
  if (!scenario || typeof scenario !== 'object' || Array.isArray(scenario)) {
    fail('root must be an object');
  }
  if (scenario.version !== 1) fail('version must be 1');
  if (typeof scenario.name !== 'string' || !scenario.name.trim()) fail('name is required');
  if (typeof scenario.url !== 'string' || !scenario.url.trim()) fail('url is required');

  const viewport = scenario.viewport;
  if (!viewport || typeof viewport !== 'object') fail('viewport is required');
  finitePositive(viewport.width, 'viewport.width');
  finitePositive(viewport.height, 'viewport.height');
  finitePositive(viewport.dpr, 'viewport.dpr');

  const thresholds = scenario.thresholds;
  if (!thresholds || typeof thresholds !== 'object') fail('thresholds is required');
  for (const key of [
    'maxDiffPercent',
    'maxRegionDiffPercent',
    'channelDiff',
    'pixelRadius',
    'maxGeometryDiffPx',
  ]) {
    if (typeof thresholds[key] !== 'number' || !Number.isFinite(thresholds[key]) || thresholds[key] < 0) {
      fail(`thresholds.${key} must be a non-negative finite number`);
    }
  }
  if (thresholds.maxDiffPercent <= 0 || thresholds.maxDiffPercent > 100) {
    fail('thresholds.maxDiffPercent must be in (0, 100]');
  }
  if (thresholds.maxRegionDiffPercent > 100) {
    fail('thresholds.maxRegionDiffPercent must be <= 100');
  }
  if (thresholds.channelDiff > 255) fail('thresholds.channelDiff must be <= 255');
  if (!Number.isInteger(thresholds.pixelRadius)) fail('thresholds.pixelRadius must be an integer');

  const observe = scenario.observe;
  if (!observe || typeof observe !== 'object') fail('observe is required');
  if (!Array.isArray(observe.selectors) || observe.selectors.some((value) => typeof value !== 'string')) {
    fail('observe.selectors must be an array of strings');
  }
  if (new Set(observe.selectors).size !== observe.selectors.length) {
    fail('observe.selectors must not contain duplicates');
  }
  if (observe.glyphMaskInsetPx !== undefined) {
    if (!observe.glyphMaskInsetPx || typeof observe.glyphMaskInsetPx !== 'object'
      || Array.isArray(observe.glyphMaskInsetPx)) {
      fail('observe.glyphMaskInsetPx must be an object');
    }
    for (const [selector, inset] of Object.entries(observe.glyphMaskInsetPx)) {
      if (!observe.selectors.includes(selector)) {
        fail(`observe.glyphMaskInsetPx selector is not observed: ${selector}`);
      }
      if (!Number.isInteger(inset) || inset < 0) {
        fail(`observe.glyphMaskInsetPx.${selector} must be a non-negative integer`);
      }
    }
  }
  if (typeof observe.stateExpression !== 'string' || !observe.stateExpression.trim()) {
    fail('observe.stateExpression is required');
  }
  if (!Array.isArray(observe.eventTypes) || observe.eventTypes.some((value) => typeof value !== 'string')) {
    fail('observe.eventTypes must be an array of strings');
  }
  if (scenario.environment?.chromeVersionPattern !== undefined) {
    if (typeof scenario.environment.chromeVersionPattern !== 'string') {
      fail('environment.chromeVersionPattern must be a string');
    }
    try {
      new RegExp(scenario.environment.chromeVersionPattern);
    } catch {
      fail('environment.chromeVersionPattern must be a valid regular expression');
    }
  }

  if (!Array.isArray(scenario.steps) || scenario.steps.length === 0) fail('steps must not be empty');
  const ids = new Set();
  for (const [index, step] of scenario.steps.entries()) {
    if (!step || typeof step !== 'object') fail(`steps[${index}] must be an object`);
    if (typeof step.id !== 'string' || !/^[a-z0-9][a-z0-9._-]*$/i.test(step.id)) {
      fail(`steps[${index}].id must be file-name safe`);
    }
    if (ids.has(step.id)) fail(`duplicate step id ${JSON.stringify(step.id)}`);
    ids.add(step.id);
    if (!step.action || typeof step.action !== 'object') fail(`steps[${index}].action is required`);
    const required = ACTION_FIELDS[step.action.type];
    if (!required) fail(`steps[${index}].action.type is unsupported`);
    for (const field of required) {
      if (step.action[field] === undefined) fail(`steps[${index}].action.${field} is required`);
    }
    if (step.action.type === 'wait') finitePositive(step.action.milliseconds, `steps[${index}].action.milliseconds`);
  }
  return scenario;
}

export async function loadScenario(path) {
  const absolute = resolve(path);
  const scenario = JSON.parse(await readFile(absolute, 'utf8'));
  return validateScenario(scenario);
}

async function main() {
  const path = process.argv[2];
  if (!path) {
    console.error('用法: validate-scenario.mjs <scenario.json>');
    process.exit(2);
  }
  const scenario = await loadScenario(path);
  console.log(`场景有效: ${scenario.name}（${scenario.steps.length} 个步骤）`);
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  main().catch((error) => {
    console.error(error.message);
    process.exit(1);
  });
}

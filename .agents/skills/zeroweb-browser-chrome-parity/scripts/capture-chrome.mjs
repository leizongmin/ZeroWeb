#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { access, mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { loadScenario } from './validate-scenario.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, '../../../..');
const PUPPETEER_PATH = resolve(
  REPO_ROOT,
  'tests/wpt-runner/scripts/node_modules/puppeteer-core/lib/puppeteer/puppeteer-core.js',
);

function parseArgs() {
  const args = process.argv.slice(2);
  const result = { scenario: null, out: null };
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] === '--scenario') result.scenario = args[++index];
    else if (args[index] === '--out') result.out = args[++index];
    else if (args[index] === '--help') {
      console.log('用法: capture-chrome.mjs --scenario <json> --out <目录>');
      process.exit(0);
    } else {
      throw new Error(`unknown option: ${args[index]}`);
    }
  }
  if (!result.scenario || !result.out) throw new Error('必须提供 --scenario 和 --out');
  return result;
}

function expandUrl(value) {
  const filePrefix = 'file://${REPO_ROOT}/';
  if (value.startsWith(filePrefix)) {
    const suffix = value.slice(filePrefix.length);
    const separator = suffix.search(/[?#]/);
    const relativePath = separator >= 0 ? suffix.slice(0, separator) : suffix;
    const trailing = separator >= 0 ? suffix.slice(separator) : '';
    return `${pathToFileURL(resolve(REPO_ROOT, relativePath)).href}${trailing}`;
  }
  return value.replaceAll('${REPO_ROOT}', REPO_ROOT);
}

async function loadPuppeteer() {
  try {
    return (await import(pathToFileURL(PUPPETEER_PATH).href)).default;
  } catch (error) {
    throw new Error(
      `puppeteer-core 不可用；请在 tests/wpt-runner/scripts 执行 npm ci（${error.message}）`,
    );
  }
}

async function firstExisting(paths) {
  for (const path of paths) {
    if (!path) continue;
    try {
      await access(path);
      return path;
    } catch {
      // Continue to the next platform candidate.
    }
  }
  return null;
}

export function chromeCandidates(platform, environment) {
  if (platform === 'win32') {
    return [
      environment.PROGRAMFILES && resolve(environment.PROGRAMFILES, 'Google/Chrome/Application/chrome.exe'),
      environment['PROGRAMFILES(X86)']
        && resolve(environment['PROGRAMFILES(X86)'], 'Google/Chrome/Application/chrome.exe'),
      environment.LOCALAPPDATA && resolve(environment.LOCALAPPDATA, 'Google/Chrome/Application/chrome.exe'),
      environment.PROGRAMFILES && resolve(environment.PROGRAMFILES, 'Chromium/Application/chrome.exe'),
      environment['PROGRAMFILES(X86)']
        && resolve(environment['PROGRAMFILES(X86)'], 'Microsoft/Edge/Application/msedge.exe'),
      environment.PROGRAMFILES && resolve(environment.PROGRAMFILES, 'Microsoft/Edge/Application/msedge.exe'),
      environment.LOCALAPPDATA && resolve(environment.LOCALAPPDATA, 'Microsoft/Edge/Application/msedge.exe'),
    ].filter(Boolean);
  }
  if (platform === 'darwin') {
    return [
      '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
      '/Applications/Chromium.app/Contents/MacOS/Chromium',
    ];
  }
  return [
    '/usr/bin/google-chrome',
    '/usr/bin/google-chrome-stable',
    '/usr/bin/chromium',
    '/usr/bin/chromium-browser',
  ];
}

async function resolveChromeExecutable() {
  if (process.env.PUPPETEER_EXECUTABLE_PATH) {
    const configured = resolve(process.env.PUPPETEER_EXECUTABLE_PATH);
    if (!await firstExisting([configured])) {
      throw new Error(`PUPPETEER_EXECUTABLE_PATH 不存在: ${configured}`);
    }
    return configured;
  }

  const candidates = chromeCandidates(process.platform, process.env);
  const executable = await firstExisting(candidates);
  if (!executable) {
    throw new Error('未找到 Chrome/Chromium/Edge；请设置 PUPPETEER_EXECUTABLE_PATH');
  }
  return executable;
}

async function connectBrowser(puppeteer, locale) {
  const cdpUrl = process.env.ORACLE_CDP_URL;
  if (cdpUrl) {
    const version = await fetch(`${cdpUrl.replace(/\/$/, '')}/json/version`).then((response) => {
      if (!response.ok) throw new Error(`CDP 版本端点返回 ${response.status}`);
      return response.json();
    });
    const browser = await puppeteer.connect({ browserWSEndpoint: version.webSocketDebuggerUrl });
    return { browser, capturePath: 'chrome-cdp-gui', close: () => browser.disconnect() };
  }

  const executablePath = await resolveChromeExecutable();
  const browser = await puppeteer.launch({
    executablePath,
    headless: 'new',
    args: [
      '--no-sandbox',
      '--disable-lcd-text',
      '--hide-scrollbars',
      `--lang=${locale}`,
    ],
  });
  return { browser, capturePath: 'chrome-headless', close: () => browser.close() };
}

async function installEventProbe(page, eventTypes) {
  await page.evaluate((types) => {
    globalThis.__browserParityEvents = [];
    const selectorFor = (element) => {
      if (element.id) return `#${element.id}`;
      const parts = [];
      for (let node = element; node; node = node.parentElement) {
        let part = node.tagName.toLowerCase();
        if (node.parentElement) {
          let index = 1;
          for (let sibling = node.previousElementSibling; sibling; sibling = sibling.previousElementSibling) {
            if (sibling.tagName === node.tagName) index += 1;
          }
          part += `:nth-of-type(${index})`;
        }
        parts.unshift(part);
      }
      return parts.join('>');
    };
    for (const type of types) {
      document.addEventListener(type, (event) => {
        const target = event.target instanceof Element ? selectorFor(event.target) : '';
        const record = {
          type: event.type,
          target,
          defaultPrevented: event.defaultPrevented,
        };
        globalThis.__browserParityEvents.push(record);
        queueMicrotask(() => {
          record.defaultPrevented = event.defaultPrevented;
        });
      }, true);
    }
  }, eventTypes);
}

async function waitForResources(page) {
  await page.evaluate(async () => {
    if (document.fonts?.ready) await document.fonts.ready;
    await Promise.all(Array.from(document.images || []).map((image) => {
      if (image.complete) return Promise.resolve();
      return new Promise((done) => {
        image.addEventListener('load', done, { once: true });
        image.addEventListener('error', done, { once: true });
        setTimeout(done, 1000);
      });
    }));
  });
}

async function waitForStableFrame(page) {
  const deadline = Date.now() + 3000;
  let previous = null;
  while (Date.now() < deadline) {
    const png = await page.screenshot({ type: 'png' });
    const signature = createHash('sha256').update(png).digest('hex');
    if (signature === previous) return;
    previous = signature;
    await new Promise((done) => setTimeout(done, 50));
  }
  throw new Error('页面在 3 秒内没有产生连续两张一致的稳定帧');
}

async function performAction(page, action) {
  switch (action.type) {
    case 'snapshot':
      return;
    case 'click': {
      const handle = await page.$(action.selector);
      if (!handle) throw new Error(`找不到点击目标: ${action.selector}`);
      const box = await handle.boundingBox();
      if (!box || box.width <= 0 || box.height <= 0) {
        throw new Error(`点击目标没有可见矩形: ${action.selector}`);
      }
      const offset = action.offset || { x: 0.5, y: 0.5 };
      const x = box.x + box.width * offset.x;
      const y = box.y + box.height * offset.y;
      await page.mouse.move(x, y);
      await page.mouse.down({ button: 'left' });
      if (action.jitter) await page.mouse.move(x + action.jitter.x, y + action.jitter.y);
      await page.mouse.up({ button: 'left' });
      return;
    }
    case 'type':
      await page.keyboard.type(action.text);
      return;
    case 'key':
      await page.keyboard.press(action.key);
      return;
    case 'wait':
      await new Promise((done) => setTimeout(done, action.milliseconds));
      return;
    default:
      throw new Error(`不支持的动作: ${action.type}`);
  }
}

async function observe(page, scenario) {
  return page.evaluate(({ selectors, stateExpression }) => {
    const selectorFor = (element) => {
      if (!element) return '';
      if (element.id) return `#${element.id}`;
      const parts = [];
      for (let node = element; node; node = node.parentElement) {
        let part = node.tagName.toLowerCase();
        if (node.parentElement) {
          let index = 1;
          for (let sibling = node.previousElementSibling; sibling; sibling = sibling.previousElementSibling) {
            if (sibling.tagName === node.tagName) index += 1;
          }
          part += `:nth-of-type(${index})`;
        }
        parts.unshift(part);
      }
      return parts.join('>');
    };
    const geometry = {};
    for (const selector of selectors) {
      const element = document.querySelector(selector);
      if (!element) {
        geometry[selector] = null;
        continue;
      }
      const rect = element.getBoundingClientRect();
      geometry[selector] = {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
      };
    }
    let state;
    try {
      state = (0, eval)(stateExpression);
    } catch (error) {
      throw new Error(`stateExpression failed: ${error.message}`);
    }
    const events = Array.isArray(state?.events)
      ? Array.from(state.events)
      : Array.from(globalThis.__browserParityEvents || []);
    if (state && typeof state === 'object' && !Array.isArray(state)) {
      const { events: _events, ...rest } = state;
      state = rest;
    }
    return {
      state,
      events,
      geometry,
      activeElement: selectorFor(document.activeElement),
      url: location.href,
    };
  }, {
    selectors: scenario.observe.selectors,
    stateExpression: scenario.observe.stateExpression,
  });
}

async function main() {
  const options = parseArgs();
  const scenario = await loadScenario(options.scenario);
  const output = resolve(options.out);
  await mkdir(output, { recursive: true });

  const puppeteer = await loadPuppeteer();
  const connection = await connectBrowser(puppeteer, scenario.environment?.locale || 'en-US');
  const browser = connection.browser;
  try {
    const engineVersion = await browser.version();
    const expectedVersion = scenario.environment?.chromeVersionPattern;
    if (expectedVersion && !new RegExp(expectedVersion).test(engineVersion)) {
      throw new Error(`Chrome 版本 ${JSON.stringify(engineVersion)} 不匹配 ${JSON.stringify(expectedVersion)}`);
    }
    const page = await browser.newPage();
    const cdp = await page.createCDPSession();
    await cdp.send('Emulation.setLocaleOverride', {
      locale: scenario.environment?.locale || 'en-US',
    });
    await page.setViewport({
      width: scenario.viewport.width,
      height: scenario.viewport.height,
      deviceScaleFactor: scenario.viewport.dpr,
    });
    await page.emulateMediaFeatures([
      { name: 'prefers-color-scheme', value: scenario.environment?.colorScheme || 'light' },
      { name: 'prefers-reduced-motion', value: scenario.environment?.reducedMotion || 'reduce' },
    ]);
    await page.goto(expandUrl(scenario.url), { waitUntil: 'load', timeout: 30000 });
    await waitForResources(page);
    await installEventProbe(page, scenario.observe.eventTypes);

    const steps = [];
    for (const step of scenario.steps) {
      await performAction(page, step.action);
      await page.evaluate(() => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done))));
      await waitForStableFrame(page);
      const observed = await observe(page, scenario);
      const screenshot = `${step.id}.png`;
      await page.screenshot({ path: resolve(output, screenshot), type: 'png' });
      const regions = {};
      for (const selector of scenario.observe.selectors) {
        const handle = await page.$(selector);
        const box = handle ? await handle.boundingBox() : null;
        if (!handle || !box || box.width <= 0 || box.height <= 0) continue;
        const regionPath = `${step.id}.region-${Buffer.from(selector).toString('hex')}.png`;
        const x = Math.floor(box.x);
        const y = Math.floor(box.y);
        const width = Math.ceil(box.x + box.width) - x;
        const height = Math.ceil(box.y + box.height) - y;
        await page.screenshot({
          path: resolve(output, regionPath),
          type: 'png',
          clip: { x, y, width, height },
        });
        regions[selector] = regionPath;
      }
      steps.push({
        id: step.id,
        action: step.action,
        screenshot,
        regions,
        ...observed,
      });
    }

    const manifest = {
      schemaVersion: 1,
      scenario: scenario.name,
      engine: 'chrome',
      engineVersion,
      capturePath: connection.capturePath,
      inputPath: 'browser-pointer',
      viewport: scenario.viewport,
      steps,
    };
    await writeFile(resolve(output, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
    console.log(`Chrome 证据: ${resolve(output, 'manifest.json')}`);
  } finally {
    await connection.close();
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  main().catch((error) => {
    console.error(error.stack || error.message);
    process.exit(1);
  });
}

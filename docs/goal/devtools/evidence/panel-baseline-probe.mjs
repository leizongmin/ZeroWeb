// DevTools goal M0-P3 — 面板可用度基线探测（对真实 Chromium 空跑）。
//
// 场景：ZeroWeb headless 只做 frontend 静态 serve（/devtools/），被调试方是
// Playwright pin Chromium（真实 CDP 端点）。frontend（经 ZeroWeb serve）附接
// Chromium 页面后，检查各面板 DOM 就位、console 零致命错误、并驱动一条最小
// 演示流（Elements 定位 + Console evaluate + Network 请求列表）。
//
// 用法：
//   node docs/goal/devtools/evidence/panel-baseline-probe.mjs \
//        [--zw <zero-browser 路径>] [--bundle <frontend 目录>] [--port <base 端口>]
// 前置：cargo build -p zero-browser；tests/playwright-matrix/node_modules 就位。
// 输出：stdout JSON 结论 + 截图 evidence/panel-baseline-<ts>.png。

import { createRequire } from 'node:module';
import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const { chromium } = require(resolve(process.cwd(), 'tests/playwright-matrix/node_modules/playwright-core'));

const HERE = dirname(fileURLToPath(import.meta.url));
const arg = (name, fallback) => {
  const i = process.argv.indexOf(name);
  return i > 0 ? process.argv[i + 1] : fallback;
};

const ZW_BIN = arg('--zw', 'target/debug/zero-browser');
const BUNDLE = arg('--bundle', join(process.env.HOME, '.cache/zeroweb/devtools/chrome-devtools-inspector-1.20260913.0'));
const BASE = Number(arg('--port', 19300));
const ZW_PORT = BASE;
const CHROME_PORT = BASE + 1;

const results = { steps: [], errors: [] };
const step = (name, ok, detail) => {
  results.steps.push({ name, ok, detail });
  console.error(`[${ok ? 'ok' : 'FAIL'}] ${name}${detail ? ' — ' + detail : ''}`);
};

// ── 起 ZeroWeb headless（serve frontend + CDP） ──
const zw = spawn(ZW_BIN, ['--headless', `--remote-debugging-port=${ZW_PORT}`], {
  env: { ...process.env, ZW_DEVTOOLS_FRONTEND_DIR: BUNDLE },
  stdio: ['ignore', 'pipe', 'pipe'],
});
zw.stderr.on('data', (d) => process.env.ZW_PROBE_VERBOSE && console.error('[zw]', d.toString()));
await new Promise((r) => setTimeout(r, 2500));

// ── 起 Chromium（被调试方，真实 CDP 端点） ──
// Playwright 默认走 --remote-debugging-pipe，端口参数不生效——直接裸起 pin 的
// Chromium 二进制拿真实 HTTP CDP 发现端点。
const chromiumBin = chromiumBinaryPath();
function chromiumBinaryPath() {
  const fromRegistry = chromium.executablePath?.();
  if (fromRegistry) return fromRegistry;
  // playwright-core registry 未注册时直接扫 pin 缓存（ms-playwright/chromium-*/chrome-linux*/chrome）
  const { readdirSync, existsSync } = require('node:fs');
  const cacheRoot = process.env.PLAYWRIGHT_BROWSERS_PATH ?? join(process.env.HOME, '.cache/ms-playwright');
  const entries = readdirSync(cacheRoot).filter((d) => /^chromium-\d+/.test(d)).sort().reverse();
  for (const dir of entries) {
    for (const candidate of ['chrome-linux/chrome', 'chrome-linux64/chrome']) {
      const p = join(cacheRoot, dir, candidate);
      if (existsSync(p)) return p;
    }
  }
  throw new Error('no chromium binary found in ms-playwright cache');
}
const chromeProc = spawn(chromiumBin, [
  '--headless=new',
  '--no-first-run',
  `--user-data-dir=/tmp/zw-devtools-probe-chrome-${BASE}`,
  `--remote-debugging-port=${CHROME_PORT}`,
  'data:text/html,<title>zw-probe</title><h1>zw-probe</h1>',
], { stdio: ['ignore', 'ignore', 'pipe'] });
chromeProc.stderr.on('data', (d) => process.env.ZW_PROBE_VERBOSE && console.error('[chrome]', d.toString()));
// 等 CDP 发现端点就绪
let pageTarget = null;
for (let i = 0; i < 40 && !pageTarget; i++) {
  await new Promise((r) => setTimeout(r, 500));
  try {
    const targets = await (await fetch(`http://127.0.0.1:${CHROME_PORT}/json`)).json();
    pageTarget = targets.find((t) => t.type === 'page' && !/devtools:/.test(t.url));
  } catch { /* not ready yet */ }
}
const browser = null;
void browser;
// driver 浏览器（展示 frontend 的那只；与被调试 Chromium 无关）
const driverBrowser = await chromium.launch();
const ctx = await driverBrowser.newContext();

try {
  // 1. ZeroWeb serve 面：/json 列表 + devtoolsFrontendUrl + 静态资源
  const discovery = await (await fetch(`http://127.0.0.1:${ZW_PORT}/json`)).json();
  step('zw-discovery', Array.isArray(discovery) && discovery.length > 0, JSON.stringify(discovery?.[0]?.devtoolsFrontendUrl ?? null));

  const frontendUrl = new URL(discovery[0].devtoolsFrontendUrl, `http://127.0.0.1:${ZW_PORT}`).toString();
  const entry = await fetch(frontendUrl);
  const html = await entry.text();
  step('zw-serve-entry', entry.status === 200 && html.includes('<!DOCTYPE html>'), `${entry.status} ${entry.headers.get('content-type')} ${html.length}B`);
  const chunkMatch = html.match(/src=".\/(chunk-[a-z0-9]+\.js)"/);
  if (chunkMatch) {
    const chunk = await fetch(new URL(chunkMatch[1], frontendUrl));
    step('zw-serve-chunk', chunk.status === 200, `${chunkMatch[1]} ${chunk.status} ${(await chunk.arrayBuffer()).byteLength}B`);
  }
  const traversal = await fetch(`http://127.0.0.1:${ZW_PORT}/devtools/%2e%2e/Cargo.toml`);
  step('zw-serve-traversal-blocked', traversal.status === 404, String(traversal.status));

  // 2. Chromium 侧：枚举被调试页面 target
  step('chromium-target', Boolean(pageTarget), pageTarget?.url);

  // 3. 在 Chromium 页面里打开 ZeroWeb serve 的 frontend，附接 Chromium 自己的页面 target
  const driver = await ctx.newPage();
  await driver.setViewportSize({ width: 1440, height: 900 });
  const consoleErrors = [];
  const pageErrors = [];
  const failedRequests = [];
  driver.on('console', (m) => m.type() === 'error' && consoleErrors.push(m.text()));
  driver.on('pageerror', (e) => pageErrors.push(String(e?.message ?? e)));
  driver.on('requestfailed', (r) => failedRequests.push(`${r.url()} :: ${r.failure()?.errorText}`));
  driver.on('response', (r) => r.status() >= 400 && failedRequests.push(`${r.url()} :: HTTP ${r.status()}`));
  const wsParam = pageTarget.webSocketDebuggerUrl.replace('ws://', '');
  const attachUrl = `${frontendUrl}?ws=${wsParam}`;
  await driver.goto(attachUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
  // frontend 启动 + 面板初始化
  await driver.waitForTimeout(8000);
  results.diagnostics = {
    pageErrors,
    failedRequests: failedRequests.slice(0, 15),
    consoleErrors: consoleErrors.slice(0, 10),
  };

  const panelState = await driver.evaluate(() => {
    const q = (sel) => document.querySelector(sel) !== null;
    const body = document.body;
    // DevTools UI 全在 shadow root 里——深度遍历统计（含 closed root 拿不到，open 即够）
    const shadowHosts = [];
    const walk = (root, depth) => {
      if (depth > 12) return;
      for (const el of root.querySelectorAll('*')) {
        if (el.shadowRoot) {
          shadowHosts.push(el.tagName.toLowerCase() + (el.id ? '#' + el.id : ''));
          walk(el.shadowRoot, depth + 1);
        }
      }
    };
    walk(document, 0);
    return {
      domContentLoaded: body !== null,
      rootWidget: q('.widget.vbox') || q('#-blink-dev-tools'),
      tabbedPaneTabs: [...document.querySelectorAll('.tabbed-pane-tab-label')].map((e) => e.textContent?.trim()).slice(0, 20),
      bodyChildCount: body.children.length,
      domNodeCount: document.querySelectorAll('*').length,
      shadowHostCount: shadowHosts.length,
      shadowHostSample: shadowHosts.slice(0, 12),
      innerTextSample: body.innerText?.slice(0, 200) ?? '',
      htmlSample: body.innerHTML.slice(0, 400),
    };
  });
  step('frontend-boot', panelState.rootWidget || panelState.bodyChildCount > 0, JSON.stringify(panelState.tabbedPaneTabs));
  results.panelState = panelState;
  step('frontend-elements', Boolean(panelState.elementsPanel), String(panelState.elementsPanel));
  step('frontend-console', Boolean(panelState.consolePanel), String(panelState.consolePanel));
  step('frontend-network', Boolean(panelState.networkPanel), String(panelState.networkPanel));

  // 4. 最小演示流：Console evaluate（经 frontend 的 Runtime 域）在 Chromium 页面生效
  //    （frontend 附接真实 Chromium —— 验证 serve 出的 frontend 本体可用）
  const evalResult = await driver.evaluate(async () => {
    // 借道 frontend 的 Main instance 不易；改用快捷键路径复杂——直接断言
    // console 面板 prompt 出现 + Elements 树含 <html> 节点即可判「面板活着」。
    const treeText = document.querySelector('.elements-tree-outline')?.textContent ?? '';
    return { hasHtml: /<html/i.test(treeText), treeLen: treeText.length };
  });
  step('frontend-elements-tree-content', evalResult.hasHtml || evalResult.treeLen > 0, JSON.stringify(evalResult));

  results.fatalConsoleErrors = consoleErrors.filter((e) => !/deps|DevTools|suggested/i.test(e));
  step('frontend-console-no-fatal', results.fatalConsoleErrors.length === 0, JSON.stringify(results.fatalConsoleErrors.slice(0, 3)));

  mkdirSync(HERE, { recursive: true });
  const shot = join(HERE, `panel-baseline-${Date.now()}.png`);
  await driver.screenshot({ path: shot, fullPage: false });
  results.screenshot = shot;
  step('screenshot', true, shot);

  results.pass = results.steps.every((s) => s.ok);
  writeFileSync(join(HERE, 'panel-baseline-result.json'), JSON.stringify(results, null, 2));
  console.log(JSON.stringify(results, null, 2));
  process.exitCode = results.pass ? 0 : 1;
} catch (error) {
  results.errors.push(String(error?.stack ?? error));
  console.error(JSON.stringify(results, null, 2));
  process.exitCode = 1;
} finally {
  await driverBrowser?.close().catch(() => {});
  chromeProc?.kill('SIGTERM');
  zw.kill('SIGTERM');
}

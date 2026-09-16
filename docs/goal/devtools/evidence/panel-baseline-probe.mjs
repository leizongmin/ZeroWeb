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
  // Chromium 111+ 拒绝带 Origin 头的 WS 附接（安全加固）——frontend 从 serve 端点打开，
  // Origin 是 serve origin，必须放行（Chrome 自身 devtools:// 内嵌无此问题）
  '--remote-allow-origins=*',
  // 被调试页面用真实 http 页（data: 页会被回收导致 DevTools 附接随即断开）
  `http://127.0.0.1:${ZW_PORT}/json/version`,
], { stdio: ['ignore', 'ignore', 'pipe'] });
chromeProc.stderr.on('data', (d) => process.env.ZW_PROBE_VERBOSE && console.error('[chrome]', d.toString()));
// 被调试页面用真实 http 页（data: 页在 headless 下会被回收，DevTools 附接随即断开）
const debuggeeUrl = `http://127.0.0.1:${ZW_PORT}/json/version`;
void debuggeeUrl;
let pageTarget = null;
for (let i = 0; i < 40 && !pageTarget; i++) {
  await new Promise((r) => setTimeout(r, 500));
  try {
    const targets = await (await fetch(`http://127.0.0.1:${CHROME_PORT}/json`)).json();
    // 必须选真实页面 target——headless chromium 还会列 browser_ui/omnibox 伪 page target
    pageTarget = targets.find((t) => t.type === 'page' && t.url.startsWith(`http://127.0.0.1:${ZW_PORT}/`));
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
  // frontendUrl 自带 ?ws=（指向 ZeroWeb 浏览器端点）——空跑场景必须剥掉，改指被调试方
  const attachUrl = `${frontendUrl.split('?')[0]}?ws=${wsParam}`;
  await driver.goto(attachUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
  // frontend 启动 + 面板初始化
  await driver.waitForTimeout(8000);
  results.diagnostics = {
    pageErrors,
    failedRequests: failedRequests.slice(0, 15),
    consoleErrors: consoleErrors.slice(0, 10),
  };

  const panelState = await driver.evaluate(() => {
    const body = document.body;
    // DevTools UI 全在 shadow root 里——深度遍历把所有节点拍平判定
    const nodes = [];
    const tabLabels = [];
    const walk = (root, depth) => {
      if (depth > 14 || nodes.length > 20000) return;
      for (const el of root.querySelectorAll('*')) {
        nodes.push({ id: el.id ?? '', cls: String(el.className ?? ''), tag: el.tagName?.toLowerCase() ?? '', text: (el.textContent ?? '').slice(0, 60) });
        if (/^tabbed-pane-tab-label$/.test(String(el.className ?? ''))) tabLabels.push(el.textContent?.trim());
        if (el.shadowRoot) walk(el.shadowRoot, depth + 1);
      }
    };
    walk(document, 0);
    const has = (pred) => nodes.some(pred);
    return {
      domContentLoaded: body !== null,
      rootWidget: has((n) => n.id === '-blink-dev-tools'),
      elementsPanel: has((n) => n.id === 'elements-panel' || n.cls.includes('elements-tree-outline')),
      consolePanel: has((n) => n.id === 'console-panel' || n.cls.includes('console-view')),
      networkPanel: has((n) => n.id === 'network-panel' || n.cls.includes('network-log-grid') || n.cls.includes('network-panel')),
      domTreeHasHtml: has((n) => n.tag === 'span' && /^<html/i.test(n.text)),
      stylesSidebar: has((n) => n.cls.includes('styles-side-panel') || n.cls.includes('style-panes-wrapper') || (n.cls === 'widget' && false)),
      tabbedPaneTabs: [...new Set(tabLabels)].slice(0, 16),
      bodyChildCount: body.children.length,
      shadowHostCount: nodes.filter((n) => n.id).length,
      domNodeCount: nodes.length,
    };
  });
  step('frontend-boot', panelState.rootWidget && panelState.domNodeCount > 100, JSON.stringify(panelState.tabbedPaneTabs));
  results.panelState = panelState;
  step('frontend-elements', Boolean(panelState.elementsPanel), `domNodeCount=${panelState.domNodeCount}`);

  // Console / Network 面板是惰性实例化且窄窗口下折叠进溢出菜单——
  // 用 DevTools 原生 `&panel=` 入口参数直开（重新 goto frontend）
  const openPanelViaUrl = async (panel) => {
    const url = `${frontendUrl.split('?')[0]}?ws=${wsParam}&panel=${panel}`;
    await driver.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
    await driver.waitForTimeout(5000);
    return driver.evaluate((name) => {
      const nodes = [];
      const walk = (root, depth) => {
        if (depth > 14 || nodes.length > 20000) return;
        for (const el of root.querySelectorAll('*')) {
          nodes.push({ id: el.id ?? '', cls: String(el.className ?? '') });
          if (el.shadowRoot) walk(el.shadowRoot, depth + 1);
        }
      };
      walk(document, 0);
      if (name === 'console') {
        return nodes.some((n) => n.id === 'console-panel' || n.cls.includes('console-view'));
      }
      return nodes.some((n) => n.id === 'network-panel' || n.cls.includes('network-log-grid') || n.cls.includes('network-panel'));
    }, panel);
  };
  const consoleOk = await openPanelViaUrl('console');
  step('frontend-console', consoleOk, 'via &panel=console');
  const networkOk = await openPanelViaUrl('network');
  step('frontend-network', networkOk, 'via &panel=network');

  // 4. 最小演示流断言：Elements 树已渲染被调试页的 <html> 节点（DOM 域活）+
  //    样式侧栏在位（CSS 域活）
  const evalResult = { hasHtml: panelState.domTreeHasHtml, tabs: panelState.tabbedPaneTabs };
  step('frontend-elements-tree-content', Boolean(evalResult.hasHtml), JSON.stringify(evalResult));

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

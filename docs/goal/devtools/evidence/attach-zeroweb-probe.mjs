// DevTools goal M1-S2 — DevTools frontend 附接 ZeroWeb 自身页面（对真实 frontend +
// 真实 ZeroWeb CDP 端点，零 mock）。
//
// 流程：
//   1. 起 ZeroWeb headless（serve frontend + CDP），raw WS Page.navigate 到真实外网页
//      （connectOverCDP 会占住单连接 accept 循环饿死后续 driver 连接——结构性限制，
//      master.md 记账）
//   2. driver Chromium 打开 `/devtools/inspector.html?ws=<addr>/devtools/page/zeroweb-tab-1`
//      （M1-S1 per-page 路径路由）——frontend 附接 ZeroWeb 页面
//   3. 逐面板断言（Playwright locator 天然穿透 open shadow root）：Elements 活 DOM /
//      Console REPL evaluate / Network 请求列表 + 收集 -32601 Unknown method 账本输入
//   4. 截图存证
//
// 用法：node attach-zeroweb-probe.mjs [--zw <bin>] [--bundle <dir>] [--port <base>] [--url <被调试页>]

import { createRequire } from 'node:module';
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const { chromium } = require(resolve(process.cwd(), 'tests/playwright-matrix/node_modules/playwright-core'));
const WebSocket = require(resolve(process.cwd(), 'tests/playwright-matrix/node_modules/ws'));

const HERE = dirname(fileURLToPath(import.meta.url));
const arg = (name, fallback) => {
  const i = process.argv.indexOf(name);
  return i > 0 ? process.argv[i + 1] : fallback;
};

const ZW_BIN = arg('--zw', 'target/debug/zero-browser');
const BUNDLE = arg('--bundle', join(process.env.HOME, '.cache/zeroweb/devtools/devtools-frontend-9bd6a496c3394422674c62a19e9faa627817c56e'));
const PORT = Number(arg('--port', 19520));
const DEBUGGEE_URL = arg('--url', 'https://example.com/');

const results = { steps: [], unknownMethods: new Set(), consoleErrors: [] };
const step = (name, ok, detail) => {
  results.steps.push({ name, ok, detail });
  console.error(`[${ok ? 'ok' : 'FAIL'}] ${name}${detail ? ' — ' + detail : ''}`);
};

const zw = spawn(ZW_BIN, ['--headless', `--remote-debugging-port=${PORT}`], {
  env: { ...process.env, ZW_DEVTOOLS_FRONTEND_DIR: BUNDLE },
  stdio: ['ignore', 'pipe', 'pipe'],
});
zw.stderr.on('data', (d) => process.env.ZW_PROBE_VERBOSE && console.error('[zw]', d.toString().trim()));
await new Promise((r) => setTimeout(r, 2500));

const driverBrowser = await chromium.launch();
try {
  // 1. ZeroWeb 发现面 + per-page devtoolsFrontendUrl（M1-S1 形态断言）
  const discovery = await (await fetch(`http://127.0.0.1:${PORT}/json`)).json();
  const frontendUrl = new URL(discovery?.[0]?.devtoolsFrontendUrl ?? '', `http://127.0.0.1:${PORT}`).toString();
  const perPageOk = frontendUrl.includes('/devtools/inspector.html?ws=') &&
    frontendUrl.includes(`/devtools/page/${discovery[0].id}`);
  step('zw-discovery-per-page-url', perPageOk, frontendUrl);
  const entryBase = frontendUrl.split('?')[0];

  // 2. 导航被调试页（真实 http 页）——raw WS flat Page.navigate，用完即断
  await new Promise((resolveNav) => {
    const ws = new WebSocket(`ws://127.0.0.1:${PORT}/`);
    ws.on('open', () => {
      ws.send(JSON.stringify({ id: 1, method: 'Page.navigate', params: { url: DEBUGGEE_URL } }));
    });
    ws.on('message', (m) => {
      const msg = JSON.parse(m.toString());
      if (msg.id === 1) {
        step('zw-debuggee-navigated', !msg.error, JSON.stringify(msg.result ?? msg.error).slice(0, 120));
        ws.close();
        resolveNav();
      }
    });
    ws.on('error', (e) => {
      step('zw-debuggee-navigated', false, e.message);
      resolveNav();
    });
  });
  await new Promise((r) => setTimeout(r, 2500));

  // 3. driver 打开 frontend（per-page ws 直连）——Elements 面板断言
  const driver = await driverBrowser.newPage();
  await driver.setViewportSize({ width: 1600, height: 1000 });
  const consoleErrors = [];
  driver.on('console', (m) => {
    const t = m.text();
    if (m.type() === 'error') consoleErrors.push(t);
    const unknown = t.match(/Request ([A-Za-z]+\\.[A-Za-z]+) failed/);
    if (unknown) results.unknownMethods.add(unknown[1]);
  });
  await driver.goto(`${entryBase}?ws=127.0.0.1:${PORT}/devtools/page/${discovery[0].id}`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  // frontend boot：根宿主 + tab 条出现（locator 穿透 shadow root）
  await driver.locator('#-blink-dev-tools').waitFor({ timeout: 20000 });
  await driver.waitForTimeout(4000);
  step('frontend-boot', (await driver.locator('#-blink-dev-tools').count()) > 0, '');
  const elementsTree = driver.locator('.elements-tree-outline');
  const elementsOk = (await elementsTree.count()) > 0;
  step('frontend-elements-panel', elementsOk, `tree=${await elementsTree.count()}`);
  const htmlNode = driver.locator('.elements-tree-outline', { hasText: '<html>' });
  step('frontend-elements-live-dom', (await htmlNode.count()) > 0, 'tree shows debuggee <html>');
  const exampleInTree = await driver.getByText('Example Domain').count();
  step('frontend-elements-debuggee-content', exampleInTree > 0, `matches=${exampleInTree}`);
  const shotElements = join(HERE, 'attach-zeroweb-elements.png');
  await driver.screenshot({ path: shotElements });
  step('screenshot-elements', true, shotElements);

  // 4. Console 面板直开（&panel=）+ REPL evaluate（DC-2 Console 判据最小演示流）
  await driver.goto(`${entryBase}?ws=127.0.0.1:${PORT}/devtools/page/${discovery[0].id}&panel=console`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await driver.locator('.console-view').waitFor({ timeout: 20000 }).catch(() => {});
  step('frontend-console-panel', (await driver.locator('.console-view').count()) > 0, '');
  // 点进 console 提示符并键入表达式（frontend REPL → Runtime.evaluate → ZeroWeb V8）
  const prompt = driver.locator('#console-prompt').first();
  const promptFallback = driver.locator('[contenteditable="true"]').first();
  if ((await prompt.count()) > 0) {
    await prompt.click();
  } else {
    await promptFallback.click();
  }
  await driver.keyboard.type('1+1', { delay: 40 });
  await driver.keyboard.press('Enter');
  await driver.waitForTimeout(2500);
  const replEcho = await driver.getByText('1+1').count();
  const replResult = await driver.getByText('2', { exact: true }).count();
  step('frontend-console-repl-evaluate', replEcho > 0 && replResult > 0, `echo=${replEcho} result2=${replResult}`);
  const shotConsole = join(HERE, 'attach-zeroweb-console.png');
  await driver.screenshot({ path: shotConsole });
  step('screenshot-console', true, shotConsole);

  // 5. 经 REPL 发起 fetch（被调试页请求 → ZeroWeb net 观测 → Network 域事件流），
  //    再直开 Network 面板看请求行
  if ((await prompt.count()) > 0) {
    await prompt.click();
  } else {
    await promptFallback.click();
  }
  await driver.keyboard.type(`fetch('/json/version')`, { delay: 20 });
  await driver.keyboard.press('Enter');
  await driver.waitForTimeout(3000);
  await driver.goto(`${entryBase}?ws=127.0.0.1:${PORT}/devtools/page/${discovery[0].id}&panel=network`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await driver.waitForTimeout(5000);
  const netPanelOk = (await driver.locator('#network-panel, .network-log-grid, .network-panel').count()) > 0;
  step('frontend-network-panel', netPanelOk, '');
  const requestRow = await driver.locator('.network-log-grid', { hasText: 'json/version' }).count()
    || (await driver.locator('.network-log-grid', { hasText: 'example.com' }).count());
  step('frontend-network-requests', requestRow > 0, requestRow > 0 ? 'request row visible' : 'no request row (Network 域事件缺口，见账本)');
  const shotNetwork = join(HERE, 'attach-zeroweb-network.png');
  await driver.screenshot({ path: shotNetwork });
  step('screenshot-network', true, shotNetwork);

  results.consoleErrors = consoleErrors.slice(0, 20);
  results.unknownMethods = [...results.unknownMethods];
  step('gap-ledger-collected', true, `${results.unknownMethods.length} unknown methods: ${results.unknownMethods.join(', ') || 'none'}`);

  results.pass = results.steps.every((s) => s.ok);
  writeFileSync(join(HERE, 'attach-zeroweb-result.json'), JSON.stringify(results, null, 2));
  console.log(JSON.stringify({ pass: results.pass, unknownMethods: results.unknownMethods }, null, 2));
  process.exitCode = results.pass ? 0 : 1;
} catch (error) {
  results.errors = [String(error?.stack ?? error)];
  console.error(JSON.stringify(results, null, 2));
  process.exitCode = 1;
} finally {
  await driverBrowser.close().catch(() => {});
  zw.kill('SIGTERM');
}

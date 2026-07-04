import { ChromeAcceptanceClient } from '../lib/client.js';
import { createRecorder } from '../lib/recorder.js';
import { assert, stepAsync } from '../lib/utils.js';

const PORT = parseInt(process.env.ZB_PORT || '9222', 10);
const client = new ChromeAcceptanceClient(`ws://127.0.0.1:${PORT}`, 120_000);
const rec = createRecorder('p0-regression');

async function main() {
  await client.connect();
  rec.log('Connected');
  const failures = [];

  await stepAsync('P0.1 — chrome 区域渲染', async () => {
    const shot = await rec.screenshot(client, 'p0-1-chrome-render');
    assert(shot != null, 'screenshot should succeed');
    assert(shot.meta.withChrome === true, 'screenshot should include chrome');
  });

  await stepAsync('P0.2 — 浏览器 chrome layout', async () => {
    const layout = await client.getLayout();
    rec.saveJson('p0-2-layout', layout);
    assert(layout.windowSize.width > 0, 'window width > 0');
    assert(layout.windowSize.height > 0, 'window height > 0');
    const vp = layout.viewport;
    assert(vp != null, 'viewport should exist');
    assert(vp.y > 0, 'viewport y should account for chrome height');
    rec.log(`  window: ${layout.windowSize.width}x${layout.windowSize.height}, viewport y=${vp.y}`);
  });

  await stepAsync('P0.3 — semantics 树完整', async () => {
    const sem = await client.getSemantics();
    rec.saveJson('p0-3-semantics', sem);
    assert(sem.id != null, 'root node has id');
    assert(sem.rect != null, 'root node has rect');
    assert(Array.isArray(sem.children), 'root node has children array');
    const buttons = await client.findWidgetByFlag('button');
    const focusable = await client.findWidgetByFlag('focusable');
    rec.log(`  nodes: ${countNodes(sem)}, buttons: ${buttons.length}, focusable: ${focusable.length}`);
    assert(buttons.length >= 1, 'expected at least 1 button');
  });

  await stepAsync('P0.4 — 用坐标点击 chrome 区域不崩溃', async () => {
    const result = await client.click(100, 30);
    rec.saveJson('p0-4-click-coord', result);
    assert(result.point != null, 'click returns point');
    rec.log(`  clicked at (${result.point.x}, ${result.point.y})`);
  });

  await stepAsync('P0.5 — 通过 widgetId 查找并获取坐标', async () => {
    const vpRect = await client.rectOf('viewport');
    rec.saveJson('p0-5-viewport-rect', { widgetId: 'viewport', rect: vpRect });
    assert(vpRect.y > 0, 'viewport y > 0');
    assert(vpRect.width > 0, 'viewport width > 0');
    assert(vpRect.height > 0, 'viewport height > 0');
  });

  await stepAsync('P0.6 — 查找 button 类 widget 并点击', async () => {
    const sem = await client.getSemantics();
    const firstButton = findFirstByFlag(sem, 'button');
    assert(firstButton != null, 'found at least one button widget');
    rec.log(`  found button: ${firstButton.id} (label: ${firstButton.label})`);
    const result = await client.click(null, null, firstButton.id);
    rec.saveJson('p0-6-button-click', result);
    assert(result.point != null, 'click on button returns point');
  });

  await stepAsync('P0.7 — emittedActions 可正常查询', async () => {
    const actions = await client.emittedActions();
    rec.saveJson('p0-7-emitted-actions', actions);
    assert(Array.isArray(actions), 'emitted actions should be an array');
  });

  rec.summary(failures);
  rec.log(`P0 REGRESSION ${failures.length === 0 ? 'PASS' : 'FAIL'}`);
  process.exit(failures.length === 0 ? 0 : 1);
}

function countNodes(node) {
  let count = 1;
  for (const child of (node.children || [])) count += countNodes(child);
  return count;
}

function findFirstByFlag(node, flag) {
  if ((node.flags || []).includes(flag)) return node;
  for (const child of (node.children || [])) {
    const found = findFirstByFlag(child, flag);
    if (found) return found;
  }
  return null;
}

main().catch((e) => {
  console.error('P0 regression error:', e);
  process.exit(1);
});

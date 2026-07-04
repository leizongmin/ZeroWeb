import { ChromeAcceptanceClient } from '../lib/client.js';
import { createRecorder } from '../lib/recorder.js';
import { assert, assertClose, stepAsync, retry } from '../lib/utils.js';

const PORT = parseInt(process.env.ZB_PORT || '9222', 10);
const client = new ChromeAcceptanceClient(`ws://127.0.0.1:${PORT}`, 120_000);
const rec = createRecorder('full-regression');

async function main() {
  await client.connect();
  rec.log('Connected');
  const failures = [];

  rec.log('── R1: Chrome Screenshot Pipeline ──');
  await stepAsync('R1.1 — screenshot with chrome flag', async () => {
    const shot = await retry(() => client.screenshot(), 2);
    rec.saveJson('r1-1-screenshot-meta', shot.meta);
    assert(shot.meta.withChrome === true, 'withChrome should be true');
    assert(shot.meta.width > 0, 'width > 0');
    assert(shot.meta.height > 0, 'height > 0');
    assert(shot.pngBytes.length > 0, 'png bytes non-empty');
  });

  rec.log('\n── R2: Layout & Viewport ──');
  await stepAsync('R2.1 — layout has window and viewport', async () => {
    const layout = await client.getLayout();
    rec.saveJson('r2-1-layout', layout);
    assert(layout.windowSize.width > 0, 'window width');
    assert(layout.windowSize.height > 0, 'window height');
    assert(layout.viewport != null, 'viewport present');
    assert(layout.viewport.y > 0, 'chrome occupies vertical space');
  });

  rec.log('\n── R3: Semantics Tree ──');
  await stepAsync('R3.1 — semantics root valid', async () => {
    const sem = await client.getSemantics();
    rec.saveJson('r3-1-semantics', sem);
    assert(sem.id != null, 'root id');
  });
  await stepAsync('R3.2 — find widgets by label', async () => {
    const navButtons = await client.findWidgetByFlag('button');
    rec.saveJson('r3-2-buttons', navButtons);
    assert(navButtons.length >= 1, 'at least one button');
  });
  await stepAsync('R3.3 — find focusable widgets', async () => {
    const focusable = await client.findWidgetByFlag('focusable');
    rec.saveJson('r3-3-focusable', focusable);
    assert(focusable.length >= 1, 'at least one focusable widget');
  });

  rec.log('\n── R4: Coordinate Click ──');
  await stepAsync('R4.1 — click at valid chrome region', async () => {
    const result = await client.click(200, 30);
    rec.saveJson('r4-1-click', result);
    assert(result.point != null, 'point in result');
    assertClose(result.point.x, 200, 1, 'click x coordinate');
    assertClose(result.point.y, 30, 1, 'click y coordinate');
  });

  rec.log('\n── R5: Widget Click ──');
  await stepAsync('R5.1 — find and click a button widget', async () => {
    const sem = await client.getSemantics();
    const firstButton = findFirstFlagged(sem, 'button');
    if (firstButton) {
      rec.log(`  clicking: ${firstButton.id} at ${JSON.stringify(firstButton.rect)}`);
      const result = await client.click(null, null, firstButton.id);
      rec.saveJson('r5-1-widget-click', result);
      assert(result.point != null, 'click returned point');
    } else {
      rec.log('  no button found, skipping');
    }
  });

  rec.log('\n── R6: Widget Geometry ──');
  await stepAsync('R6.1 — rectOf known widgets', async () => {
    const vpRect = await client.rectOf('viewport');
    rec.saveJson('r6-1-viewport-rect', { widgetId: 'viewport', rect: vpRect });
    assert(vpRect.y > 0, 'viewport y > 0');
    assert(vpRect.width > 0, 'viewport width > 0');
    assert(vpRect.height > 0, 'viewport height > 0');
  });

  rec.log('\n── R7: Emitted Actions ──');
  await stepAsync('R7.1 — emitted actions queryable', async () => {
    const actions = await client.emittedActions();
    rec.saveJson('r7-1-emitted-actions', actions);
    assert(Array.isArray(actions), 'actions array');
  });

  rec.log('\n── R8: Navigation + Chrome Persistence ──');
  await stepAsync('R8.1 — navigate then re-screenshot', async () => {
    await client.navigate('about:blank');
    const shot = await rec.screenshot(client, 'r8-1-after-navigate');
    assert(shot != null, 'screenshot after navigate');
    assert(shot.meta.withChrome === true, 'chrome still rendered after navigate');
  });

  rec.log('\n── R9: Browse Context Commands Stability ──');
  await stepAsync('R9.1 — getTree does not crash', async () => {
    const tree = await client.getTree();
    rec.saveJson('r9-1-tree', tree);
    assert(Array.isArray(tree.contexts), 'tree has contexts array');
  });

  rec.summary(failures);
  const pass = failures.length === 0;
  rec.log(`\nFULL REGRESSION ${pass ? 'PASS' : `FAIL (${failures.length} failures)`}`);
  process.exit(pass ? 0 : 1);
}

function findFirstFlagged(node, flag) {
  if ((node.flags || []).includes(flag)) return node;
  for (const child of (node.children || [])) {
    const found = findFirstFlagged(child, flag);
    if (found) return found;
  }
  return null;
}

main().catch((e) => {
  console.error('Full regression error:', e);
  process.exit(1);
});

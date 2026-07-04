import { ChromeAcceptanceClient } from '../lib/client.js';
import { createRecorder } from '../lib/recorder.js';
import { assert, stepAsync } from '../lib/utils.js';

const PORT = parseInt(process.env.ZB_PORT || '9222', 10);
const client = new ChromeAcceptanceClient(`ws://127.0.0.1:${PORT}`, 120_000);
const rec = createRecorder('smoke');

async function main() {
  rec.log(`Connecting to ws://127.0.0.1:${PORT} ...`);
  await client.connect();
  rec.log('Connected');

  const failures = [];

  await stepAsync('capture screenshot', async () => {
    const shot = await rec.screenshot(client, 'smoke-baseline');
    assert(shot != null, 'screenshot should succeed');
  });

  await stepAsync('get layout', async () => {
    const layout = await client.getLayout();
    rec.saveJson('layout', layout);
    assert(layout.windowSize != null, 'layout should have windowSize');
    rec.log(`  viewport: ${JSON.stringify(layout.viewport)}`);
  });

  await stepAsync('get semantics', async () => {
    const sem = await client.getSemantics();
    rec.saveJson('semantics', sem);
    assert(sem.id != null, 'semantics should have root id');
    const buttons = await client.findWidgetByFlag('button');
    const focusable = await client.findWidgetByFlag('focusable');
    rec.log(`  buttons: ${buttons.length}, focusable: ${focusable.length}`);
    assert(focusable.length >= 1, 'should have at least 1 focusable widget');
  });

  await stepAsync('rect of viewport', async () => {
    const vp = await client.rectOf('viewport');
    rec.log(`  viewport rect: ${JSON.stringify(vp)}`);
    assert(vp.y > 0, 'viewport y should be > 0 (chrome height)');
    assert(vp.width > 0, 'viewport width should be > 0');
    assert(vp.height > 0, 'viewport height should be > 0');
  });

  rec.summary(failures);
  rec.log(`SMOKE ${failures.length === 0 ? 'PASS' : 'FAIL'}`);
  process.exit(failures.length === 0 ? 0 : 1);
}

main().catch((e) => {
  console.error('Smoke test error:', e);
  process.exit(1);
});

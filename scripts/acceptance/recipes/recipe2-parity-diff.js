import { readFileSync, existsSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { ChromeAcceptanceClient } from '../lib/client.js';
import { createRecorder } from '../lib/recorder.js';
import { assert, stepAsync } from '../lib/utils.js';

const PORT = parseInt(process.env.ZB_PORT || '9222', 10);
const BASELINE_DIR = join('.acceptance', 'baselines');
const client = new ChromeAcceptanceClient(`ws://127.0.0.1:${PORT}`, 120_000);

async function main() {
  await client.connect();
  const rec = createRecorder('parity-diff');
  rec.log('Connected');
  const failures = [];

  await stepAsync('capture current screenshot', async () => {
    const shot = await rec.screenshot(client, 'current');
    assert(shot != null, 'screenshot should succeed');
  });

  await stepAsync('capture current layout', async () => {
    const layout = await client.getLayout();
    rec.saveJson('current-layout', layout);
  });

  await stepAsync('capture current semantics', async () => {
    const sem = await client.getSemantics();
    rec.saveJson('current-semantics', sem);
  });

  await stepAsync('diff: layout vs baseline', async () => {
    const baselinePath = join(BASELINE_DIR, 'layout.json');
    if (!existsSync(baselinePath)) {
      rec.log('  No baseline layout found, saving current as baseline');
      mkdirSync(BASELINE_DIR, { recursive: true });
      const layout = await client.getLayout();
      const { writeFileSync } = await import('node:fs');
      writeFileSync(baselinePath, JSON.stringify(layout, null, 2));
      return;
    }
    const baseline = JSON.parse(readFileSync(baselinePath, 'utf-8'));
    const current = await client.getLayout();
    rec.saveJson('layout-diff', { baseline, current });

    const vpBaseline = baseline.viewport;
    const vpCurrent = current.viewport;
    if (vpBaseline && vpCurrent) {
      const dx = Math.abs(vpCurrent.x - vpBaseline.x);
      const dy = Math.abs(vpCurrent.y - vpBaseline.y);
      const dw = Math.abs(vpCurrent.width - vpBaseline.width);
      const dh = Math.abs(vpCurrent.height - vpBaseline.height);
      rec.log(`  viewport delta: x=${dx}, y=${dy}, w=${dw}, h=${dh}`);
      if (dx > 1 || dy > 1 || dw > 1 || dh > 1) {
        failures.push(`viewport geometry changed: delta ${dx},${dy},${dw},${dh}`);
      }
    }
  });

  rec.summary(failures);
  const pass = failures.length === 0;
  rec.log(`PARITY DIFF ${pass ? 'PASS (in-sync)' : `FAIL (${failures.length} diffs)`}`);
  process.exit(pass ? 0 : 1);
}

main().catch((e) => {
  console.error('Parity diff error:', e);
  process.exit(1);
});

import { ChromeAcceptanceClient } from '../lib/client.js';

const PORT = parseInt(process.env.ZB_PORT || '9222', 10);
const client = new ChromeAcceptanceClient(`ws://127.0.0.1:${PORT}`, 300_000);

async function measure(label, fn) {
  const t0 = performance.now();
  let result;
  try {
    result = await fn();
  } catch (e) {
    const elapsed = ((performance.now() - t0) / 1000).toFixed(2);
    console.log(`  ${label}: ${elapsed}s → FAILED: ${e.message}`);
    return null;
  }
  const elapsed = ((performance.now() - t0) / 1000).toFixed(2);
  console.log(`  ${label}: ${elapsed}s → OK`);
  return result;
}

async function main() {
  console.log(`Connecting to ws://127.0.0.1:${PORT} ...`);
  await client.connect();
  console.log('Connected\n');

  console.log('── session.status ──');
  await measure('session.status', () => client._send('session.status'));

  console.log('\n── browsingContext.getDOMSnapshot ──');
  await measure('getDOMSnapshot', () => client._send('browsingContext.getDOMSnapshot'));

  console.log('\n── browsingContext.captureScreenshot (no chrome) ──');
  await measure('captureScreenshot', () => client.screenshot());

  console.log('\n── chrome.getLayout ──');
  await measure('getLayout', () => client.getLayout());

  console.log('\n── chrome.getSemantics ──');
  await measure('getSemantics', () => client.getSemantics());

  console.log('\n── chrome.click (by coords) ──');
  await measure('click', () => client.click(50, 25));

  console.log('\n── chrome.rectOf viewport ──');
  await measure('rectOf', () => client.rectOf('viewport'));

  console.log('\n── chrome.emittedActions ──');
  await measure('emittedActions', () => client.emittedActions());

  console.log('\n── full sequence (navigate → screenshot → layout → semantics) ──');
  await measure('full sequence', async () => {
    await client.navigate('about:blank');
    await client.screenshot();
    await client.getLayout();
    await client.getSemantics();
  });

  client.close();
  console.log('\nPerf test complete.');
}

main().catch((e) => {
  console.error('Perf test error:', e);
  process.exit(1);
});

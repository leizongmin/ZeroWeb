import { ChromeAcceptanceClient } from '../lib/client.js';
import { createRecorder } from '../lib/recorder.js';

const PORT = parseInt(process.env.ZB_PORT || '9222', 10);
const client = new ChromeAcceptanceClient(`ws://127.0.0.1:${PORT}`, 120_000);
const rec = createRecorder('exploratory');

async function main() {
  await client.connect();
  rec.log('Connected');

  rec.log('\n── E1: 逐个遍历所有 widget ──');
  const sem = await client.getSemantics();
  const allWidgets = flattenTree(sem);
  rec.saveJson('e1-all-widgets', allWidgets);
  rec.log(`  total widgets: ${allWidgets.length}`);
  for (const w of allWidgets) {
    rec.log(`    [${w.flags?.join(', ') || 'node'}] "${w.label || ''}" id=${w.id} rect=${JSON.stringify(w.rect)}`);
  }

  rec.log('\n── E2: 测试每个 button widget 的 rectOf 和 click ──');
  for (const w of allWidgets.filter((n) => (n.flags || []).includes('button'))) {
    try {
      const rect = await client.rectOf(w.id);
      rec.log(`  rectOf(${w.id}): ${JSON.stringify(rect)}`);
    } catch (e) {
      rec.log(`  rectOf(${w.id}) FAILED: ${e.message}`);
    }
  }

  rec.log('\n── E3: 多点坐标试探（不同区域）──');
  const testPoints = [
    { x: 10, y: 10, label: '左上角' },
    { x: 400, y: 10, label: '顶部中间' },
    { x: 780, y: 10, label: '右上角' },
    { x: 10, y: 300, label: '左侧中间' },
    { x: 400, y: 300, label: '中央' },
  ];
  for (const pt of testPoints) {
    try {
      const result = await client.click(pt.x, pt.y);
      rec.log(`  click(${pt.label} ${pt.x},${pt.y}): point=(${result.point.x},${result.point.y}) emitted=${result.emittedActions?.length || 0}`);
    } catch (e) {
      rec.log(`  click(${pt.label}) FAILED: ${e.message}`);
    }
  }

  rec.log('\n── E4: 连续快速命令测试 ──');
  for (let i = 0; i < 5; i++) {
    try {
      const shot = await client.screenshot();
      const layout = await client.getLayout();
      rec.log(`  iteration ${i + 1}: screenshot ${shot.meta.width}x${shot.meta.height}, viewport y=${layout.viewport?.y}`);
    } catch (e) {
      rec.log(`  iteration ${i + 1} FAILED: ${e.message}`);
    }
  }

  rec.log('\n── E5: 极端坐标点击 ──');
  const edgePoints = [
    { x: 0, y: 0, label: '(0,0)' },
    { x: -1, y: -1, label: '负坐标' },
    { x: 9999, y: 9999, label: '超大坐标' },
  ];
  for (const pt of edgePoints) {
    try {
      const result = await client.click(pt.x, pt.y);
      rec.log(`  click(${pt.label}): point=(${result.point?.x}, ${result.point?.y})`);
    } catch (e) {
      rec.log(`  click(${pt.label}): ${e.message}`);
    }
  }

  rec.summary([]);
  rec.log('\nExploratory test complete — review artifacts for findings.');
  client.close();
}

function flattenTree(node) {
  const result = [node];
  for (const child of (node.children || [])) {
    result.push(...flattenTree(child));
  }
  return result;
}

main().catch((e) => {
  console.error('Exploratory test error:', e);
  process.exit(1);
});

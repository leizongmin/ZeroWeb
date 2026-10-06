import WebSocket from 'file:///usr/share/nodejs/ws/index.js';
import fs from 'node:fs';
// slice37 NPO 探针 driver：CDP 连本地 headless（ZeroWeb 或 Chrome 均可）→ 导航 fixture →
// 采样 window.__probeResult + 渲染文本 + 截图。
// env: CDP_URL(http 基址)、URL(fixture)、OUT(json 前缀)、SHOT(png 前缀，可空)、WAIT(ms)
const CDP = process.env.CDP_URL;
const NAV_URL = process.env.URL || 'http://127.0.0.1:8123/npo-probe.html';
const OUT = process.env.OUT || '/tmp/s37/probe';
const SHOT = process.env.SHOT || '';
const WAIT = parseInt(process.env.WAIT || '3000', 10);
setTimeout(() => { console.error('PROBE-TIMEOUT'); process.exit(2); }, 60000);

const tabs = (await (await fetch(`${CDP}/json`)).json()).filter(t => t.type === 'page');
let ws = null, send = null;
for (const tab of tabs) {
  const c = new WebSocket(tab.webSocketDebuggerUrl, { perMessageDeflate: false, maxPayload: 32 * 1024 * 1024 });
  let cid = 0; const pen = new Map();
  c.on('message', d => {
    const m = JSON.parse(d);
    if (m.id && pen.has(m.id)) { pen.get(m.id)(m); pen.delete(m.id); }
  });
  const s = (method, params = {}) => new Promise(res => { const i = ++cid; pen.set(i, res); c.send(JSON.stringify({ id: i, method, params })); });
  const ok = await Promise.race([
    (async () => { await new Promise((r, j) => { c.on('open', r); c.on('error', j); }); await s('Runtime.enable'); return (await s('Runtime.evaluate', { expression: '1+1', returnByValue: true })).result?.result?.value === 2; })(),
    new Promise(r => setTimeout(() => r(false), 4000)),
  ]).catch(() => false);
  if (ok) { ws = c; send = s; break; } c.close();
}
if (!ws) { console.log('no tab'); process.exit(1); }
await send('Page.enable');
await send('Page.navigate', { url: NAV_URL });
await new Promise(r => setTimeout(r, WAIT));
const ev = await send('Runtime.evaluate', {
  expression: `JSON.stringify((function () {
    return {
      probe: window.__probeResult || null,
      outText: document.getElementById('out') ? document.getElementById('out').textContent : null,
    };
  })())`,
  returnByValue: true,
});
const val = ev.result?.result?.value;
console.log('PROBE-RESULT ' + val);
fs.writeFileSync(OUT + '.json', val ?? 'null');
if (SHOT) {
  const shot = await send('Page.captureScreenshot', { format: 'png' });
  if (shot.result?.data) fs.writeFileSync(SHOT + '.png', Buffer.from(shot.result.data, 'base64'));
}
process.exit(0);

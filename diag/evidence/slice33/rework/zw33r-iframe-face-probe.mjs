// slice33 返工 iframe 面探针（复用 zw33-shot.mjs 配方）：navigate 探针页 → 等 load
// → 求值 window.fr 判别面 → 截整页 PNG。用法:
//   CDP_URL=http://127.0.0.1:9222 OUT=/path/xxx node zw33r-iframe-face-probe.mjs
import WebSocket from 'file:///usr/share/nodejs/ws/index.js';
import fs from 'node:fs';

const CDP = process.env.CDP_URL || 'http://127.0.0.1:9222';
const URL_ = process.env.PROBE_URL || 'http://127.0.0.1:8123/iframe-face-probe.html';
const OUT = process.env.OUT || '/tmp/zw33r-iframe-face.png';
let cid = 0; const pen = new Map();
const ws = new WebSocket((await (await fetch(`${CDP}/json`)).json()).filter(t => t.type === 'page')[0].webSocketDebuggerUrl, { perMessageDeflate: false, maxPayload: 64 * 1024 * 1024 });
ws.on('message', d => { const m = JSON.parse(d); if (m.id && pen.has(m.id)) { pen.get(m.id)(m); pen.delete(m.id); } });
await new Promise((r, j2) => { ws.on('open', r); ws.on('error', j2); });
const send = (method, params = {}) => new Promise(res => { const i = ++cid; pen.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
const sleep = ms => new Promise(r => setTimeout(r, ms));
async function evaljs(expression) {
  const r = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
  if (r.result?.exceptionDetails) return { __err: r.result.exceptionDetails.exception?.description || r.result.exceptionDetails.text };
  return r.result?.result?.value;
}

await send('Page.enable');
await send('Page.navigate', { url: URL_ });
await sleep(3000);
const probe = await evaljs(`(() => {
  var el = document.querySelector('iframe');
  return {
    fr: String(window.fr),
    frTag: window.fr && window.fr.tagName ? window.fr.tagName : null,
    isElement: !!(window.fr && window.fr === el),
    isContentWindow: !!(el && window.fr === el.contentWindow),
    form: String(window.s28form),
    img: String(window.s28img),
    inputNamed: String(window.s28q),
  };
})()`);
console.log(JSON.stringify(probe, null, 2));
const shot = await send('Page.captureScreenshot', { format: 'png' });
fs.writeFileSync(OUT, Buffer.from(shot.result.data, 'base64'));
fs.writeFileSync(OUT.replace(/\.png$/, '.json'), JSON.stringify(probe, null, 2));
console.log(`saved ${OUT}`);
process.exit(0);

// slice33 RP-6 活体口径（同 slice32 probe-baidu.json 口径）：真实 baidu.com 载入后
// 核 title/bodyLen/inputs/kw 面(id=kw,name=wd)/su 面(id=su)/console 错误数。
// 用法: CDP_URL=http://127.0.0.1:9222 node zw33-baidu-rp6.mjs [OUT.json]
import WebSocket from 'file:///usr/share/nodejs/ws/index.js';
import fs from 'node:fs';

const CDP = process.env.CDP_URL || 'http://127.0.0.1:9222';
const OUT = process.argv[2] || '/tmp/zw33-baidu-rp6.json';
let cid = 0; const pen = new Map();
const tabs = async () => (await (await fetch(`${CDP}/json`)).json()).filter(t => t.type === 'page');
const ws = new WebSocket((await tabs())[0].webSocketDebuggerUrl, { perMessageDeflate: false, maxPayload: 64 * 1024 * 1024 });
ws.on('message', d => { const m = JSON.parse(d); if (m.id && pen.has(m.id)) { pen.get(m.id)(m); pen.delete(m.id); } });
await new Promise((r, j2) => { ws.on('open', r); ws.on('error', j2); });
const send = (method, params = {}) => new Promise(res => { const i = ++cid; pen.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
const sleep = ms => new Promise(r => setTimeout(r, ms));
async function evaljs(expression) {
  const r = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
  if (r.result?.exceptionDetails) return { __err: r.result.exceptionDetails.exception?.description || r.result.exceptionDetails.text };
  return r.result?.result?.value;
}
// console 错误收集
const errs = [];
ws.on('message', d => {
  const m = JSON.parse(d);
  if (m.method === 'Runtime.consoleAPICalled' && m.params?.type === 'error') {
    errs.push((m.params.args || []).map(a => a.value ?? a.description ?? '').join(' ').slice(0, 200));
  }
  if (m.method === 'Runtime.exceptionThrown') {
    errs.push(String(m.params?.exceptionDetails?.exception?.description || m.params?.exceptionDetails?.text || 'exception').slice(0, 200));
  }
});
await send('Runtime.enable');

await send('Page.enable');
await send('Page.navigate', { url: 'https://www.baidu.com/' });
await sleep(6000);

const probe = await evaljs(`(() => {
  const inputs = [...document.querySelectorAll('input')].map(i => i.name || '').filter(n => n !== '');
  const kwEl = document.querySelector('#kw');
  let kwNamed = false, suNamed = false, suName = '';
  try { kwNamed = globalThis.wd !== undefined; } catch (e) {}
  try { const su = document.querySelector('#su'); if (su) { suName = su.getAttribute('name') || ''; suNamed = globalThis[su.id] !== undefined; } } catch (e) {}
  return {
    title: document.title,
    bodyLen: (document.body ? document.body.innerHTML.length : 0),
    inputs,
    kw_name: kwEl ? (kwEl.getAttribute('name') || '') : null,
    kw_named: kwNamed,
    su_name: suName,
    su_named: suNamed,
    url: location.href,
  };
})()`);
probe.errCount = errs.length;
probe.errs = errs.slice(0, 10);

fs.writeFileSync(OUT, JSON.stringify(probe, null, 2));
console.log(JSON.stringify(probe, null, 2));
process.exit(0);

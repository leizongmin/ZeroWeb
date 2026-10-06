// slice33 残影二代探针：给每次出现的 window.mm 对象打 __zw33seq 代号，
// 区分「新集合装进 stale 成员」（查询/代理层）vs「旧集合长出 stale 成员」（维护层）。
// 用法: CDP_URL=... LIVE_URL=... node zw33-timeline2.mjs [轮数=8]
import WebSocket from 'file:///usr/share/nodejs/ws/index.js';
import fs from 'node:fs';

const CDP = process.env.CDP_URL || 'http://127.0.0.1:9222';
const LIVE = process.env.LIVE_URL || 'http://127.0.0.1:8123/zw-s32-integ-live.html';
const ROUNDS = parseInt(process.argv[2] || '8', 10);
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

const WRAP = `(() => {
  if (globalThis.__zw33Log) return 'already';
  globalThis.__zw33Log = []; globalThis.__zw33seq = 0;
  var t0 = Date.now();
  var mmInfo = function () {
    try {
      var c = window.mm;
      if (!c || typeof c.item !== 'function') return 'n/a';
      if (c.__zw33seq === undefined) c.__zw33seq = ++globalThis.__zw33seq;
      var its = []; for (var k = 0; k < c.length; k++) its.push((c[k].getAttribute('data-mark') || '?') + (c[k].isConnected ? '' : '~'));
      return 'col#' + c.__zw33seq + '[' + its.join(',') + ']';
    } catch (e) { return 'ERR'; }
  };
  var snap = function () {
    var marks = [];
    try {
      var nl = document.querySelectorAll('[id=mm]');
      for (var i = 0; i < nl.length; i++) marks.push(nl[i].getAttribute('data-mark') || '?');
    } catch (e) { marks = ['ERR']; }
    return { t: Date.now() - t0, marks: marks.join(','), mm: mmInfo() };
  };
  var r0 = globalThis.__zw_reset_pending_state;
  globalThis.__zw_reset_pending_state = function () { var s = snap(); s.ev = 'reset'; globalThis.__zw33Log.push(s); return r0.apply(this, arguments); };
  var i0 = globalThis.__zwInstallNamedAccess;
  globalThis.__zwInstallNamedAccess = function () {
    var before = mmInfo();
    var out = i0.apply(this, arguments);
    var s = snap(); s.ev = 'install'; s.before = before;
    globalThis.__zw33Log.push(s);
    return out;
  };
  // 盯住 window.mm 的赋值：谁在何时把 mm 换成新对象/新成员
  var cur = window.mm;
  setInterval(function () { var info = mmInfo(); if (info !== cur) { cur = info; globalThis.__zw33Log.push({ t: Date.now() - t0, ev: 'mmChange', mm: info }); } }, 50);
  return 'wrapped';
})()`;

const results = [];
for (let round = 1; round <= ROUNDS; round++) {
  await send('Page.navigate', { url: LIVE });
  await sleep(1500);
  await evaljs(WRAP);
  await evaljs(`(() => { const g = document.createElement('div'); g.id='mm'; g.setAttribute('data-mark','gamma-2'); g.textContent='gamma-2'; document.body.appendChild(g); return 1; })()`);
  await sleep(500);
  const re = await evaljs(`(() => {
    const g2 = document.querySelector('[data-mark=gamma-2]');
    if (g2) g2.remove();
    delete globalThis.mm;
    if (typeof globalThis.__zwInstallNamedAccess === 'function') { globalThis.__zwInstallNamedAccess(); return 'reinstalled'; }
    return 'unavailable';
  })()`);
  await sleep(400);
  await evaljs(`(() => { const g = document.createElement('div'); g.id='mm'; g.setAttribute('data-mark','gamma-3'); g.textContent='gamma-3'; document.body.appendChild(g); return 1; })()`);
  await sleep(900);
  const end = JSON.parse(await evaljs(`(() => { const c = window.mm; const isCol = !!(c && typeof c.item === 'function' && typeof c.length === 'number');
    return JSON.stringify({ len: isCol ? String(c.length) : 'n/a', items: isCol ? Array.prototype.map.call(c, function (e) { return (e.getAttribute('data-mark') || '') + (e.isConnected ? '' : '~disc'); }) : [], truth: String(document.querySelectorAll('[id=mm]').length), seq: isCol && c.__zw33seq !== undefined ? String(c.__zw33seq) : '?' }); })()`));
  const log = await evaljs(`JSON.stringify(globalThis.__zw33Log || [])`);
  const ghost = end.items.join('|') !== 'alpha|beta|gamma-3';
  results.push({ round, reinstall: re, end, ghost, log: typeof log === 'string' ? JSON.parse(log) : log });
  console.log(`round ${round}: ${ghost ? 'GHOST items=' + end.items.join('|') + ' col#' + end.seq + ' truth=' + end.truth : 'PASS'}`);
  if (ghost) {
    fs.writeFileSync('zw33-timeline2-ghost.json', JSON.stringify(results[results.length - 1], null, 1));
    console.log('saved zw33-timeline2-ghost.json');
    break;
  }
  await send('Page.navigate', { url: 'data:text/html,<body>interstitial ' + round + '</body>' });
  await sleep(600);
}
fs.writeFileSync('zw33-timeline2-all.json', JSON.stringify({ rounds: results }, null, 1));
process.exit(0);

// slice33 返工诊断：warm 面来源定位。
import WebSocket from 'file:///usr/share/nodejs/ws/index.js';
const CDP = process.env.CDP_URL || 'http://127.0.0.1:9222';
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
const out = await evaljs(`(() => {
  var el = document.querySelector('iframe');
  var d = Object.getOwnPropertyDescriptor(globalThis, 'fr') || {};
  var r = {};
  r.frTag = window.fr && window.fr.tagName ? window.fr.tagName : String(window.fr);
  r.desc_value_is_el = d.value === el;
  r.desc_writable = d.writable; r.desc_configurable = d.configurable;
  r.desc_get = typeof d.get;
  r.el_contentWindow = !!el.contentWindow;
  r.fr_is_cw = el && window.fr === el.contentWindow;
  r.store_keys = Object.keys(globalThis.__zwNAInstalledStore || {}).join(',');
  r.has_register = typeof globalThis.__zwRegisterNamedIframes;
  r.has_install = typeof globalThis.__zwInstallNamedAccess;
  r.install_src_has_iframe = String(globalThis.__zwInstallNamedAccess).indexOf('iframe[name') !== -1;
  return r;
})()`);
console.log(JSON.stringify(out, null, 2));
const out2 = await evaljs(`(() => {
  // 重跑安装器 + R139，看谁翻转 fr
  var el = document.querySelector('iframe');
  var before = window.fr === el ? 'element' : (el && window.fr === el.contentWindow ? 'contentWindow' : String(window.fr));
  if (typeof __zwInstallNamedAccess === 'function') { try { __zwInstallNamedAccess(); } catch (e) {} }
  var afterInstall = window.fr === el ? 'element' : (el && window.fr === el.contentWindow ? 'contentWindow' : String(window.fr));
  if (typeof __zwRegisterNamedIframes === 'function') { try { __zwRegisterNamedIframes(); } catch (e) {} }
  var afterR139 = window.fr === el ? 'element' : (el && window.fr === el.contentWindow ? 'contentWindow' : String(window.fr));
  return { before, afterInstall, afterR139 };
})()`);
console.log(JSON.stringify(out2, null, 2));
process.exit(0);

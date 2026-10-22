// BG goal CDP 场景驱动（node，零第三方依赖——最小 WebSocket 客户端与
// compositor-animation-comp04a-probe.mjs 同款连接面）。
//
// 对生产 headless 浏览器按场景驱动页面并输出结构化 JSON：
//   bg01-foreground  现有 tab（bg01-nested-timer）周期采样 __zw_iters
//   bg01-background  createTarget(bg02-static) 抢占 active 后采样 timer 页 iters
//   bg01-interval    现有 tab（bg01-interval-timer）周期采样 __zw_iters
//   bg02-anim        现有 tab（bg02-offscreen-anim，ZW_RAF_FRAME_DRIVEN=1）采样 __zw_raf
//   bg02-static      现有 tab（bg02-static）静置（CPU/呈现本底腿，无页面采样）
//   bg03-nav         Page.navigate source/target 交替 ×N，周期拉 fixture-server
//                    /__stats（stall 连接计数）+ evaluate 活性
//   bg04-cycle       Target.createTarget → closeTarget ×N，记录 target 事件与销毁回执
//
// 用法：node scripts/background-resource-probe.mjs --scenario <name>
//        --cdp-port 9222 --http-port <fixture-server 端口> --out <json>
//        [--wall-ms 30000] [--nav-cycles 30] [--reopen-cycles 20]
// 退出码恒 0（判定由消费方解读；连接失败退出 3）。
import fs from "node:fs";
import crypto from "node:crypto";
import net from "node:net";

// ── 最小 WebSocket 客户端（RFC 6455 文本帧，comp04a 同款）──
function wsConnect(url) {
  return new Promise((resolve, reject) => {
    const u = new URL(url);
    const key = crypto.randomBytes(16).toString("base64");
    const socket = net.connect(Number(u.port), u.hostname, () => {
      socket.write(
        `GET ${u.pathname || "/"} HTTP/1.1\r\n` +
          `Host: ${u.host}\r\n` +
          `Upgrade: websocket\r\nConnection: Upgrade\r\n` +
          `Sec-WebSocket-Key: ${key}\r\nSec-WebSocket-Version: 13\r\n\r\n`,
      );
    });
    let buffer = Buffer.alloc(0);
    let handshakeDone = false;
    const pending = [];
    const waiters = [];
    const conn = {
      send(text) {
        const payload = Buffer.from(text, "utf8");
        const mask = crypto.randomBytes(4);
        const masked = Buffer.from(payload);
        for (let i = 0; i < masked.length; i++) masked[i] ^= mask[i % 4];
        let header;
        if (payload.length < 126) {
          header = Buffer.from([0x81, 0x80 | payload.length]);
        } else {
          header = Buffer.alloc(4);
          header[0] = 0x81;
          header[1] = 0x80 | 126;
          header.writeUInt16BE(payload.length, 2);
        }
        socket.write(Buffer.concat([header, mask, masked]));
      },
      readTimeout(ms) {
        if (pending.length > 0) return Promise.resolve(pending.shift());
        return new Promise((res) => {
          const waiter = { res };
          waiter.timer = setTimeout(() => {
            const i = waiters.indexOf(waiter);
            if (i >= 0) waiters.splice(i, 1);
            res(null);
          }, ms);
          waiters.push(waiter);
        });
      },
      close() {
        try { socket.destroy(); } catch { /* 已关闭 */ }
      },
    };
    socket.on("data", (chunk) => {
      buffer = Buffer.concat([buffer, chunk]);
      if (!handshakeDone) {
        const idx = buffer.indexOf("\r\n\r\n");
        if (idx < 0) return;
        const head = buffer.subarray(0, idx).toString("latin1");
        const accept = crypto
          .createHash("sha1")
          .update(key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11")
          .digest("base64");
        if (!head.includes(accept)) {
          reject(new Error("websocket handshake rejected"));
          socket.destroy();
          return;
        }
        buffer = buffer.subarray(idx + 4);
        handshakeDone = true;
        resolve(conn);
      }
      while (buffer.length >= 2) {
        const opcode = buffer[0] & 0x0f;
        const masked = (buffer[1] & 0x80) !== 0;
        let len = buffer[1] & 0x7f;
        let offset = 2;
        if (len === 126) {
          if (buffer.length < 4) return;
          len = buffer.readUInt16BE(2);
          offset = 4;
        } else if (len === 127) {
          if (buffer.length < 10) return;
          len = Number(buffer.readBigUInt64BE(2));
          offset = 10;
        }
        const maskLen = masked ? 4 : 0;
        if (buffer.length < offset + maskLen + len) return;
        let payload = buffer.subarray(offset + maskLen, offset + maskLen + len);
        if (masked) {
          const mask = buffer.subarray(offset, offset + 4);
          const un = Buffer.from(payload);
          for (let i = 0; i < un.length; i++) un[i] ^= mask[i % 4];
          payload = un;
        }
        buffer = buffer.subarray(offset + maskLen + len);
        if (opcode === 0x1) {
          const text = payload.toString("utf8");
          if (waiters.length > 0) waiters.shift().res(text);
          else pending.push(text);
        } else if (opcode === 0x8) {
          socket.destroy();
        }
      }
    });
    socket.on("error", () => {});
    socket.on("close", () => {
      for (const w of waiters.splice(0)) {
        clearTimeout(w.timer);
        w.res(null);
      }
    });
  });
}

// ── CDP 辅助 ──
const args = process.argv.slice(2);
function argOf(name, dflt) {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : dflt;
}
const scenario = argOf("--scenario");
const cdpPort = Number(argOf("--cdp-port", "9222"));
const httpPort = Number(argOf("--http-port", "0"));
const startUrl = argOf("--start-url");
const outPath = argOf("--out");
const wallMs = Number(argOf("--wall-ms", "30000"));
const navCycles = Number(argOf("--nav-cycles", "30"));
const reopenCycles = Number(argOf("--reopen-cycles", "20"));
if (!scenario || !outPath) {
  console.error("usage: node background-resource-probe.mjs --scenario <name> --cdp-port <p> --http-port <p> --out <json> [...]");
  process.exit(2);
}

const sleep = (ms) => new Promise((res) => setTimeout(res, ms));
let nextId = 1;
const events = [];

async function call(conn, method, params, sessionId, timeoutMs = 15000) {
  const id = nextId++;
  const msg = { id, method, params };
  if (sessionId) msg.sessionId = sessionId;
  conn.send(JSON.stringify(msg));
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const remain = deadline - Date.now();
    if (remain <= 0) return { error: `timeout ${method}` };
    const raw = await conn.readTimeout(remain);
    if (raw === null) return { error: `connection closed (${method})` };
    const parsed = JSON.parse(raw);
    if (parsed.id === id) return parsed.error ? { error: `${method}: ${parsed.error.code} ${parsed.error.message}` } : { result: parsed.result };
    if (parsed.method) events.push({ t_ms: Date.now(), ...parsed });
  }
}

async function waitEvent(method, pred, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const i = events.findIndex((e) => e.method === method && (!pred || pred(e.params)));
    if (i >= 0) return events.splice(i, 1)[0];
    if (Date.now() >= deadline) return null;
    await readOne(conn, Math.min(100, deadline - Date.now()));
  }
}

async function readOne(conn, timeoutMs) {
  const raw = await conn.readTimeout(timeoutMs);
  if (raw === null) return null;
  const parsed = JSON.parse(raw);
  if (parsed.method) events.push({ t_ms: Date.now(), ...parsed });
  return "ok";
}

async function evaluate(conn, sessionId, expression, timeoutMs = 15000) {
  const r = await call(conn, "Runtime.evaluate", { expression, returnByValue: true }, sessionId, timeoutMs);
  if (r.error) return { error: r.error };
  const res = r.result && r.result.result;
  return { value: res && "value" in res ? res.value : undefined };
}

// headless 启动 URL 由驱动方控制（run_headless 忽略位置参数 URL，恒 about:blank）：
// attach 后显式 Page.navigate 并轮询 document.title 就绪（fixture title 均以 BG-0 开头）。
async function navigateAndWait(conn, sessionId, url, timeoutMs = 10000) {
  const nav = await call(conn, "Page.navigate", { url }, sessionId);
  if (nav.error) return { ok: false, error: nav.error };
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    await sleep(300);
    const t = await evaluate(conn, sessionId, "document.title", 5000);
    if (t.value && String(t.value).startsWith("BG-0")) return { ok: true, title: t.value };
  }
  return { ok: false, error: "title not ready" };
}

async function fetchStats(port) {
  if (!port) return null;
  try {
    const res = await fetch(`http://127.0.0.1:${port}/__stats`);
    return await res.json();
  } catch (e) {
    return { error: String(e) };
  }
}

// ── 连接与 autoAttach ──
async function attachAll(conn) {
  const r = await call(conn, "Target.setAutoAttach", { autoAttach: true, waitForDebuggerOnStart: false, flatten: true });
  if (r.error) throw new Error(`setAutoAttach: ${r.error}`);
  // 收现存 target 的 attachedToTarget（500ms 无新事件视为收齐）
  let lastCount = -1;
  const deadline = Date.now() + 3000;
  while (Date.now() < deadline) {
    await readOne(conn, 200);
    const n = events.filter((e) => e.method === "Target.attachedToTarget").length;
    if (n > 0 && n === lastCount) break;
    lastCount = n;
  }
}

function attachedSessions() {
  return events
    .filter((e) => e.method === "Target.attachedToTarget")
    .map((e) => ({ targetId: e.params.targetInfo.targetId, url: e.params.targetInfo.url, sessionId: e.params.sessionId }));
}

async function createTarget(conn, url) {
  const r = await call(conn, "Target.createTarget", { url });
  if (r.error) throw new Error(`createTarget: ${r.error}`);
  const targetId = r.result.targetId;
  const made = await waitEvent("Target.attachedToTarget", (p) => p.targetInfo && p.targetInfo.targetId === targetId, 5000);
  if (!made) throw new Error(`no attachedToTarget for ${targetId}`);
  return { targetId, sessionId: made.params.sessionId, url: made.params.targetInfo.url };
}

// ── 场景 ──
async function sampleLoop(conn, sessionId, expression, wallMs, intervalMs, samples, label) {
  const t0 = Date.now();
  while (Date.now() - t0 < wallMs) {
    await sleep(Math.min(intervalMs, Math.max(0, wallMs - (Date.now() - t0))));
    if (Date.now() - t0 >= wallMs) break;
    const r = await evaluate(conn, sessionId, expression);
    samples.push({ t_ms: Date.now(), label, ...r });
  }
}

async function run(conn) {
  const started = new Date().toISOString();
  const result = { scenario, started, wall_ms: wallMs, events_session: [], errors: [] };

  if (scenario === "bg01-foreground" || scenario === "bg01-interval" || scenario === "bg02-anim" || scenario === "bg02-rafstub" || scenario === "bg02-static") {
    await attachAll(conn);
    const sessions = attachedSessions();
    if (sessions.length === 0) throw new Error("no attached page target");
    const page = sessions[sessions.length - 1];
    const nav = await navigateAndWait(conn, page.sessionId, startUrl);
    if (!nav.ok) throw new Error(`navigateAndWait failed: ${nav.error}`);
    result.targets = sessions;
    result.start_nav = nav;
    const expr = scenario === "bg02-anim" || scenario === "bg02-rafstub" ? "window.__zw_raf" : "window.__zw_iters";
    const samples = [];
    if (scenario !== "bg02-static") {
      await sampleLoop(conn, page.sessionId, `${expr} === undefined ? null : ${expr}`, wallMs, 5000, samples, scenario);
    } else {
      await sleep(wallMs);
    }
    result.samples = samples;
  } else if (scenario === "bg01-background") {
    await attachAll(conn);
    const sessions = attachedSessions();
    const timerPage = sessions[sessions.length - 1];
    const nav = await navigateAndWait(conn, timerPage.sessionId, startUrl);
    if (!nav.ok) throw new Error(`navigateAndWait failed: ${nav.error}`);
    const preemptor = await createTarget(conn, `http://127.0.0.1:${httpPort}/bg02-static.html`);
    result.targets = [...sessions, preemptor];
    result.start_nav = nav;
    // 静置让 active 切换生效后再采样
    await sleep(1000);
    const samples = [];
    await sampleLoop(conn, timerPage.sessionId, "window.__zw_iters", wallMs, 5000, samples, "bg01-background");
    result.samples = samples;
  } else if (scenario === "bg03-nav") {
    await attachAll(conn);
    const sessions = attachedSessions();
    const page = sessions[sessions.length - 1];
    // --nav-src/--nav-dst 覆盖：M2-S2 隔离腿（纯文本页互导）复用同一编排，
    // 默认仍为 source（fetch+timer 持有形态）/ target（轻量后继页）。
    const src = `http://127.0.0.1:${httpPort}/${argOf("--nav-src", "bg03-nav-source.html")}`;
    const dst = `http://127.0.0.1:${httpPort}/${argOf("--nav-dst", "bg03-nav-target.html")}`;
    const nav0 = await navigateAndWait(conn, page.sessionId, src);
    if (!nav0.ok) throw new Error(`initial navigateAndWait failed: ${nav0.error}`);
    result.start_nav = nav0;
    const series = [];
    for (let i = 1; i <= navCycles; i++) {
      const url = i % 2 === 1 ? dst : src;
      const t0 = Date.now();
      const nav = await navigateAndWait(conn, page.sessionId, url);
      const stats = await fetchStats(httpPort);
      series.push({ cycle: i, navigated_to: url, nav_ok: nav.ok, nav_error: nav.error || null, title: nav.title ?? null, stats, nav_ms: Date.now() - t0 });
    }
    result.nav_series = series;
  } else if (scenario === "bg04-cycle") {
    await attachAll(conn);
    const url = `http://127.0.0.1:${httpPort}/bg03-nav-target.html`;
    const series = [];
    for (let i = 1; i <= reopenCycles; i++) {
      const t0 = Date.now();
      let entry = { cycle: i };
      try {
        const made = await createTarget(conn, url);
        entry.target_id = made.targetId;
        entry.open_ms = Date.now() - t0;
        await navigateAndWait(conn, made.sessionId, url, 5000);
        await sleep(1000);
        const close = await call(conn, "Target.closeTarget", { targetId: made.targetId });
        entry.close_error = close.error || null;
        // 等 targetDestroyed 回执（3s）
        const destroyedEv = await waitEvent("Target.targetDestroyed", (p) => p.targetId === made.targetId, 3000);
        entry.destroyed_event = destroyedEv !== null;
      } catch (e) {
        entry.error = String(e);
      }
      entry.cycle_ms = Date.now() - t0;
      series.push(entry);
    }
    const list = await call(conn, "Target.getTargets", {});
    result.final_targets = list.result ? list.result.targetInfos : null;
    result.reopen_series = series;
  } else {
    throw new Error(`unknown scenario: ${scenario}`);
  }
  result.finished = new Date().toISOString();
  return result;
}

let conn;
try {
  const ver = await fetch(`http://127.0.0.1:${cdpPort}/json/version`);
  const verJson = await ver.json();
  const wsUrl = verJson.webSocketDebuggerUrl;
  if (!wsUrl) throw new Error("no webSocketDebuggerUrl in /json/version");
  conn = await wsConnect(wsUrl);
  const result = await run(conn);
  fs.writeFileSync(outPath, JSON.stringify(result, null, 1));
  console.log(`probe done: ${outPath}`);
} catch (e) {
  console.error(`probe failed: ${e && e.stack || e}`);
  fs.writeFileSync(outPath, JSON.stringify({ scenario, fatal: String(e) }, null, 1));
  process.exit(3);
} finally {
  if (conn) conn.close();
}
process.exit(0);

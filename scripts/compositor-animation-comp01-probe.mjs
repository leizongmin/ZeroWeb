// COMP-01 帧连续性探针驱动（node，零第三方依赖——自带最小 WebSocket 客户端）。
//
// 对生产 headless 浏览器（zero-browser --headless）采样：
//   A. 动画通道：静止页面上 opacity/transform 无限动画，跨时间多次 GPU 截图
//      —— 哈希全同 = FROZEN（生产未推进动画），有差异 = ANIMATED。
//   B. 负载通道：触发页面内有界 JS 忙循环（4×220ms 片），记录采样与负载推进。
//   C. 阻塞诊断：同步 evaluate 忙循环期间第二条连接的截图请求是否被服务端
//      阻塞（记录时延，反映 headless mux/渲染主线程的阻塞形态）。
//   D. 正控通道：强制 DOM mutation 后应产出新帧（证明截图/发布链路本身活跃，
//      把「动画不推进」与「发布链路死亡」区分开）。
//
// 输出：<out-dir>/comp01-probe.json + comp01-s{N}.png；退出码恒 0（判定由消费方解读）。
//
// 用法：node scripts/compositor-animation-comp01-probe.mjs --url ws://127.0.0.1:PORT \
//        --fixture <html-path> --out <dir>
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import net from "node:net";

// ── 最小 WebSocket 客户端（RFC 6455 文本帧；仅本机回环、无扩展、无分片出站）──
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
    // 等待者以对象入列，超时自摘——弃用的 read 不会吞掉后续帧（run-rules 学习 #15 的
    // 「同一队列兼作等待与回退」反面模式在本客户端内的预防）。
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
      // 帧解析（服务端文本帧；掩码按规范可有可无，两者都解）。
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
    socket.on("connect", () => {});
    socket.on("ready", () => {});
    // 连接建立回调在 net.connect 的 handler 里完成；这里兜底 handshake 失败路径。
    socket.on("close", () => {
      for (const w of waiters.splice(0)) {
        clearTimeout(w.timer);
        w.res(null);
      }
    });
  });
}

// ── 探针逻辑 ──
const args = process.argv.slice(2);
function argOf(name) {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
}
const wsUrl = argOf("--url");
const fixturePath = argOf("--fixture");
const outDir = argOf("--out");
if (!wsUrl || !fixturePath || !outDir) {
  console.error("usage: node compositor-animation-comp01-probe.mjs --url <ws> --fixture <html> --out <dir>");
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });

// automation ExecuteScript 契约：脚本串被包成 `(function(){ SCRIPT }).apply(...)`
// —— 必须以 `return` 语句收尾才有返回值（automation.rs automation_value_from_script）。
const QUERY_STATE =
  "var o=document.getElementById('opacity-box'),t=document.getElementById('transform-box')," +
  "m=document.getElementById('load-marker');var so=window.getComputedStyle(o),st=window.getComputedStyle(t);" +
  "return JSON.stringify({opacity:so.opacity,transform:st.transform,marker:m.textContent," +
  "anims:(document.getAnimations?document.getAnimations().length:-1),pageNow:Math.round(performance.now())});";

const samples = [];
let nextId = 1;

// 单连接请求/响应；超时返回 null（不抛，方便阻塞诊断区分「被阻塞」与「出错」）。
async function rpcOn(conn, method, params, timeoutMs) {
  const id = nextId++;
  conn.send(JSON.stringify({ id, method, params }));
  const deadline = Date.now() + (timeoutMs ?? 30000);
  for (;;) {
    const remain = deadline - Date.now();
    if (remain <= 0) return null;
    const raw = await conn.readTimeout(remain);
    if (raw === null) return null;
    const msg = JSON.parse(raw);
    if (msg.id === id) {
      if (msg.error) throw new Error(`${method}: error ${msg.error.code} ${msg.error.message}`);
      return msg.result;
    }
    // 非本请求的响应（历史超时残 reply）：丢弃继续等。
  }
}

const sleep = (ms) => new Promise((res) => setTimeout(res, ms));

async function capture(conn, name) {
  const t0 = Date.now();
  const result = await rpcOn(conn, "browsingContext.captureScreenshot", {});
  if (result === null) return { name, error: "capture timeout" };
  const wallMs = Date.now() - t0;
  const png = result.data.png;
  const bytes = Buffer.from(png, "base64");
  fs.writeFileSync(path.join(outDir, `comp01-${name}.png`), bytes);
  const sha = crypto.createHash("sha256").update(bytes).digest("hex");
  return { name, wall_ms: wallMs, png_sha256: sha, png_bytes: bytes.length };
}

async function queryState(conn) {
  const t0 = Date.now();
  const result = await rpcOn(conn, "script.evaluate", { expression: QUERY_STATE });
  const wallMs = Date.now() - t0;
  const value = result && result.result ? result.result.value : null;
  let parsed = null;
  try { parsed = JSON.parse(value); } catch { parsed = { raw: value }; }
  return { wall_ms: wallMs, state: parsed };
}

async function record(conn, name) {
  const sample = { name, t_wall_ms: Date.now() - tProbeStart };
  sample.capture = await capture(conn, name);
  sample.query = await queryState(conn);
  samples.push(sample);
  const sha = sample.capture.png_sha256 ? sample.capture.png_sha256.slice(0, 12) : "n/a";
  console.log(`[probe] ${name}: sha=${sha} capture=${sample.capture.wall_ms ?? "?"}ms state=${JSON.stringify(sample.query.state)}`);
  return sample;
}

const tProbeStart = Date.now();
let conn;
try {
  conn = await wsConnect(wsUrl);
  const html = fs.readFileSync(fixturePath, "utf8");
  const load = await rpcOn(conn, "browsingContext.loadHtml", { html });
  if (load === null) throw new Error("loadHtml timeout");
  await sleep(300);

  // A. 动画通道：页面静止，动画若推进则帧哈希随时间变化。
  await record(conn, "s0");
  await sleep(1000);
  await record(conn, "s1");
  await sleep(1200);
  await record(conn, "s2");

  // B. 负载通道：触发有界忙负载（片内 marker 文本变化 → 预期哈希变化，由 query 归因）。
  await rpcOn(conn, "script.evaluate", { expression: "return window.__zwStartLoad();" });
  await sleep(150);
  await record(conn, "s3");
  await sleep(500);
  await record(conn, "s4");
  // 等负载结束（marker 变 done，最长 8s）。
  let loadDone = false;
  for (let i = 0; i < 40; i++) {
    const q = await queryState(conn);
    if (q.state && String(q.state.marker).includes("done")) { loadDone = true; break; }
    await sleep(200);
  }

  // C. 阻塞诊断：同步忙 evaluate 占住服务端（automation 往返在 mux 循环上阻塞），
  // 第二条连接的截图请求记录其被放行前的等待时延。
  const conn2 = await wsConnect(wsUrl);
  const busyExpr = "return window.__zwBusy(1200);";
  const busyT0 = Date.now();
  const busyId = nextId++;
  conn.send(JSON.stringify({ id: busyId, method: "script.evaluate", params: { expression: busyExpr } }));
  await sleep(150); // 让忙 evaluate 进入服务端处理
  let blockedCapture = null;
  const tBlock = Date.now();
  const blockResult = await rpcOn(conn2, "browsingContext.captureScreenshot", {}, 3000);
  if (blockResult === null) {
    blockedCapture = { served: false, waited_ms: Date.now() - tBlock, note: "capture not served within 3s while busy evaluate in flight" };
  } else {
    const bytes = Buffer.from(blockResult.data.png, "base64");
    blockedCapture = {
      served: true,
      waited_ms: Date.now() - tBlock,
      png_sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
    };
  }
  // 收忙 evaluate 的响应（等它返回，记录实际忙时长）。
  let busyReply = null;
  const busyDeadline = Date.now() + 10000;
  while (Date.now() < busyDeadline) {
    const raw = await conn.readTimeout(busyDeadline - Date.now());
    if (raw === null) break;
    const msg = JSON.parse(raw);
    if (msg.id === busyId) {
      busyReply = { wall_ms: Date.now() - busyT0, value: msg.result ? msg.result.result.value : null };
      break;
    }
  }
  conn2.close();

  // 阻塞解除后采样：若动画推进，此帧应与 s4 不同。
  await record(conn, "s5");

  // D. 正控通道：强制 DOM mutation → 应产出不同的帧。
  await rpcOn(conn, "script.evaluate", {
    expression:
      "var m=document.getElementById('load-marker');m.style.background='rgb(10,140,10)';" +
      "m.textContent='control=mutated';return 'ok';",
  });
  await sleep(300);
  await record(conn, "s6");

  const shaOf = (n) => {
    const s = samples.find((x) => x.name === n);
    return s && s.capture.png_sha256 ? s.capture.png_sha256 : "error";
  };
  const sampleOf = (n) => samples.find((x) => x.name === n);
  const animPropsStable = (a, b) => {
    const sa = sampleOf(a), sb = sampleOf(b);
    if (!sa || !sb || !sa.query.state || !sb.query.state) return null;
    return sa.query.state.opacity === sb.query.state.opacity &&
      sa.query.state.transform === sb.query.state.transform;
  };
  const animChannel =
    shaOf("s0") === shaOf("s1") && shaOf("s1") === shaOf("s2") ? "FROZEN" : "ANIMATED";
  // s4→s5 差异归因：opacity/transform 两查询恒定 → 差异来自 marker 文本 mutation
  // （负载片的可观测推进），不是动画推进；动画属性变化才算 ANIMATED。
  let postLoadChannel;
  if (shaOf("s4") === shaOf("s5")) {
    postLoadChannel = "FROZEN";
  } else if (animPropsStable("s4", "s5") === true) {
    postLoadChannel = "MUTATION_ATTRIBUTED";
  } else {
    postLoadChannel = "ANIMATED";
  }
  const controlChannel = shaOf("s5") === shaOf("s6") ? "NO_NEW_FRAME" : "OK";
  const verdict = {
    anim_idle: animChannel,
    anim_after_load: postLoadChannel,
    mutation_control: controlChannel,
    load_completed: loadDone,
    busy_reply: busyReply,
    blocked_capture: blockedCapture,
  };
  fs.writeFileSync(
    path.join(outDir, "comp01-probe.json"),
    JSON.stringify({ fixture: path.basename(fixturePath), started_at_wall: new Date(tProbeStart).toISOString(), samples, verdict }, null, 2),
  );
  console.log(`[probe] verdict: ${JSON.stringify(verdict)}`);
} catch (error) {
  fs.writeFileSync(
    path.join(outDir, "comp01-probe.json"),
    JSON.stringify({ error: String(error && error.stack ? error.stack : error), samples }, null, 2),
  );
  console.error(`[probe] FAILED: ${error}`);
  process.exitCode = 1;
} finally {
  if (conn) conn.close();
}

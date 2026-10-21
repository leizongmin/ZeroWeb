// COMP-04 worker canvas 基线探针驱动（node，零第三方依赖——自带最小 WebSocket 客户端）。
//
// 对生产 headless 浏览器（zero-browser --headless）加载 COMP-04 fixture，采样：
//   A. worker 能力面：worker 内 OffscreenCanvas 构造/getContext/draw 结果（fixture
//      自探针经 postMessage 外显）。
//   B. API 面：HTMLCanvasElement.transferControlToOffscreen 存在性。
//   C. 并发面：主线程 900ms 忙窗内 worker beat 停顿（同 realm 证明）+ canvas
//      绘制时间戳窗内计数（主线程绘制停摆）。
//   D. 正控通道：负载后 DOM mutation 应产出新帧（发布链路活跃）。
//
// 输出：<out-dir>/comp04-probe.json + comp04-s{N}.png；退出码恒 0（判定由消费方解读）。
//
// 用法：node scripts/compositor-animation-comp04-probe.mjs --url ws://127.0.0.1:PORT \
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
  console.error("usage: node compositor-animation-comp04-probe.mjs --url <ws> --fixture <html> --out <dir>");
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });

// automation ExecuteScript 契约：以 `return` 语句收尾才有返回值。
const QUERY_STATE =
  "var b=window.__zwBeats||[],lw=window.__zwLoadWindow,dl=window.__zwDrawLog||[],wp=window.__zwWorkerProbe;" +
  "var maxGapAll=0;for(var i=1;i<b.length;i++){var g=b[i]-b[i-1];if(g>maxGapAll)maxGapAll=g;}" +
  "var gapSpanLoad=0,beatsInLoad=0,drawsInLoad=0;" +
  "if(lw){for(var k=1;k<b.length;k++){if(b[k]>lw.start&&b[k-1]<lw.end){if(b[k]>lw.start&&b[k]<=lw.end)beatsInLoad++;" +
  "var lg=b[k]-b[k-1];if(lg>gapSpanLoad)gapSpanLoad=lg;}}" +
  "for(var j=0;j<dl.length;j++){if(dl[j]>=lw.start&&dl[j]<=lw.end)drawsInLoad++;}}" +
  "var drawLogSpans=!!(lw&&dl.length>1&&dl[0]<lw.start&&dl[dl.length-1]>lw.end);" +
  "var mc=document.getElementById('main-canvas');" +
  "return JSON.stringify({transfer:(window.__zwTransfer||'pending'),worker:(document.getElementById('strip-worker').textContent)," +
  "moc:(window.__zwMoc||'pending'),beats:b.length,maxGapAll:Math.round(maxGapAll)," +
  "gapSpanLoad:Math.round(gapSpanLoad),beatsInLoad:beatsInLoad,drawsInLoad:drawsInLoad," +
  "drawLogSpans:drawLogSpans,drawLogN:dl.length," +
  "loadWindow:lw,loadMarker:document.getElementById('strip-load').textContent});";

const samples = [];
let nextId = 1;

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
  fs.writeFileSync(path.join(outDir, `comp04-${name}.png`), bytes);
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
  await sleep(500);

  // A. 空闲通道：worker 探针回执 + 主线程 canvas 绘制推进（两采样间应活跃）。
  await record(conn, "s0");
  await sleep(900);
  await record(conn, "s1");

  // B. 负载通道：900ms 主线程忙窗（evaluate 调用本身阻塞至负载结束，墙钟即忙时长）。
  const tLoad = Date.now();
  await rpcOn(conn, "script.evaluate", { expression: "return window.__zwStartLoad();" });
  const loadWallMs = Date.now() - tLoad;
  await sleep(300);
  await record(conn, "s2");

  // C. 正控通道：DOM mutation → 应产出新帧（发布链路活跃正控）。
  await rpcOn(conn, "script.evaluate", {
    expression:
      "var m=document.getElementById('strip-load');m.style.background='rgb(10,140,10)';" +
      "m.textContent=m.textContent+' control=mutated';return 'ok';",
  });
  await sleep(300);
  await record(conn, "s3");

  const shaOf = (n) => {
    const s = samples.find((x) => x.name === n);
    return s && s.capture.png_sha256 ? s.capture.png_sha256 : "error";
  };
  const stateOf = (n) => {
    const s = samples.find((x) => x.name === n);
    return s && s.query.state ? s.query.state : null;
  };
  const s1 = stateOf("s1");
  const s2 = stateOf("s2");
  const verdict = {
    // 能力面：worker 内 OffscreenCanvas（预期 ctx=null——worker 影子上下文无 host 桥）。
    worker_probe: s1 ? s1.worker : "n/a",
    // API 面：transferControlToOffscreen（预期 undefined——入口 API 未实现）。
    transfer_control_api: s1 ? s1.transfer : "n/a",
    // 支持子集：主线程 OffscreenCanvas 2D（预期 pass）。
    main_offscreen_2d_subset: s1 ? s1.moc : "n/a",
    // 并发面：忙窗内 beat 停顿与绘制停摆（同 realm 证明）。
    // gap_spanning_load = 与忙窗相交的最大相邻 beat 间隔（窗内无 beat 到达时
    // 跨窗间隔即停顿时长）；draw_log_spans 防稀疏日志未覆盖忙窗的 vacuous 判定。
    load_wall_ms: loadWallMs,
    load_window: s2 ? s2.loadWindow : null,
    beat_gap_spanning_load_ms: s2 ? s2.gapSpanLoad : null,
    beat_gap_pre_load_ms: s1 ? s1.maxGapAll : null,
    beats_in_load: s2 ? s2.beatsInLoad : null,
    draws_in_load: s2 ? s2.drawsInLoad : null,
    draw_log_spans_load: s2 ? s2.drawLogSpans : null,
    draw_log_samples: s2 ? s2.drawLogN : null,
    beats_total: s2 ? s2.beats : null,
    // 帧活性：空闲期 canvas 动画应产出不同帧；负载后 mutation 应产出新帧。
    canvas_idle: shaOf("s0") !== shaOf("s1") ? "ANIMATED" : "FROZEN",
    mutation_control: shaOf("s2") !== shaOf("s3") ? "OK" : "NO_NEW_FRAME",
    load_completed: s2 ? String(s2.loadMarker || "").includes("done") : false,
  };
  fs.writeFileSync(
    path.join(outDir, "comp04-probe.json"),
    JSON.stringify({ fixture: path.basename(fixturePath), started_at_wall: new Date(tProbeStart).toISOString(), samples, verdict }, null, 2),
  );
  console.log(`[probe] verdict: ${JSON.stringify(verdict)}`);
} catch (error) {
  fs.writeFileSync(
    path.join(outDir, "comp04-probe.json"),
    JSON.stringify({ error: String(error && error.stack ? error.stack : error), samples }, null, 2),
  );
  console.error(`[probe] FAILED: ${error}`);
  process.exitCode = 1;
} finally {
  if (conn) conn.close();
}

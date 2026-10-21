// COMP-04a DC-4 探针驱动（node，零第三方依赖——自带最小 WebSocket 客户端，
// comp04 驱动同款连接面）。对生产 headless 浏览器加载 COMP-04a fixture，采样：
//   A. 模式面：worker_host 宿主桥存在性（flag on 判据）→ threaded / shim 分流。
//   B. 线程活性面：worker 侧 beat 时间戳（wt，Date.now()）最大相邻间隔——
//      threaded 预期 ~beat 周期量级（worker 线程不被忙窗阻塞）；shim 预期复现
//      comp04 基线 ~949ms（同 realm 与主线程同载）。页面侧送达间隔（at）两模式
//      都在忙窗停摆（主线程投递固有语义），作对照列。
//   C. 转移面：transferControlToOffscreen 调用与 worker 侧 getContext 回执
//      （threaded）；API 存在性（shim，只探不调）。
//   D. 生命周期面：terminate 后 beat 静默（beats_after_term=0）与错误路径回流。
//   像素连续性不在本驱动判定面——忙窗期呈现连续性由 frame_dump 采集 +
//   dc4-analyze.py 区域哈希分析承载（compositor 侧采集不依赖页面主线程）。
//
// 输出：<out-dir>/comp04a-probe.json + comp04a-s{0,1,2,3}.png；退出码恒 0（判定由消费方解读）。
//
// 用法：node scripts/compositor-animation-comp04a-probe.mjs --url ws://127.0.0.1:PORT \
//        --fixture <html-path> --out <dir>
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import net from "node:net";

// ── 最小 WebSocket 客户端（与 comp04-probe.mjs 完全同款，RFC 6455 文本帧）──
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
  console.error("usage: node compositor-animation-comp04a-probe.mjs --url <ws> --fixture <html> --out <dir>");
  process.exit(2);
}
fs.mkdirSync(outDir, { recursive: true });

// automation ExecuteScript 契约：以 `return` 语句收尾才有返回值。
// wt 间隔 = worker 侧发送时钟（线程活性）；at 间隔 = 页面送达时钟（主线程投递）。
// span 系列取「送达对跨忙窗」的相邻对（忙窗内送达为 0 时即跨窗停顿）。
const QUERY_STATE =
  "var b=window.__zwBeats||[],lw=window.__zwLoadWindow;" +
  "var wtMax=0,atMax=0,wtSpan=0,atSpan=0;" +
  "for(var i=1;i<b.length;i++){" +
  "var wg=b[i].wt-b[i-1].wt;if(wg>wtMax)wtMax=wg;" +
  "var ag=b[i].at-b[i-1].at;if(ag>atMax)atMax=ag;" +
  "if(lw&&b[i].at>lw.start&&b[i-1].at<lw.end){if(wg>wtSpan)wtSpan=wg;if(ag>atSpan)atSpan=ag;}}" +
  "var dm=window.__zwDrawsMain;" +
  "return JSON.stringify({mode:window.__zwMode,workerHost:window.__zwWorkerHost," +
  "transfer:window.__zwTransfer,beats:b.length," +
  "wtGapMax:Math.round(wtMax),atGapMax:Math.round(atMax)," +
  "wtGapSpanLoad:Math.round(wtSpan),atGapSpanLoad:Math.round(atSpan)," +
  "loadWindow:lw,drawsMain:(dm===undefined?null:dm)," +
  "term:window.__zwTerm,beatsAfterTerm:window.__zwBeatsAfterTerm," +
  "workerErr:window.__zwWorkerErr," +
  "ctrl:document.getElementById('strip-ctrl').textContent," +
  "pageNow:Math.round(performance.now())});";

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
  fs.writeFileSync(path.join(outDir, `comp04a-${name}.png`), bytes);
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

// 页面时钟锚点：performance.now() 的 time origin 不保证与 loadHtml 导航对齐
//（实测可继承前一文档/renderer 起点），时间线目标一律用「load 完成时读数 + 相对
// 偏移」表达，不写绝对值。
async function pageNow(conn) {
  const r = await rpcOn(conn, "script.evaluate", { expression: "return Math.round(performance.now());" }, 5000);
  return r && r.result ? r.result.value : null;
}

// 等到页面相对时间到达 offsetMs（自 load 完成锚点起算）。忙窗（锚点+3000 起
// ~900ms）期间避免发起 evaluate（会阻塞到窗结束）；连续失败有界退出（连接异常
// 时按墙钟兜底推进，缺失字段在 verdict 中显式可见，不无限挂起）。
async function waitPageOffset(conn, p0, offsetMs) {
  const target = p0 + offsetMs;
  const deadline = Date.now() + 30000;
  let failures = 0;
  for (;;) {
    const now = await pageNow(conn);
    if (now === null) {
      failures++;
      if (failures >= 3 || Date.now() > deadline) return null;
      await sleep(1000);
      continue;
    }
    if (now >= target) return now;
    if (Date.now() > deadline) return now;
    const remain = target - now;
    // 忙窗锚点+3000..+4000：跨入前先睡过整窗（+1200 余量），避免 evaluate 撞窗阻塞。
    if (now < p0 + 2900 && now + remain > p0 + 2900) {
      await sleep(p0 + 2900 - now + 1200);
    } else {
      await sleep(Math.min(remain, 500));
    }
  }
}

const tProbeStart = Date.now();
let conn;
try {
  conn = await wsConnect(wsUrl);
  const html = fs.readFileSync(fixturePath, "utf8");
  const load = await rpcOn(conn, "browsingContext.loadHtml", { html });
  if (load === null) throw new Error("loadHtml timeout");
  await sleep(500);
  // 页面时钟锚点（相对时间线基准，见 pageNow 注释）。
  const p0 = await pageNow(conn);

  // A. 空闲通道两采样（锚点+~700/+~1500ms，均在忙窗前）：shim 模式 canvas 应活跃。
  await record(conn, "s0");
  await sleep(700);
  await record(conn, "s1");

  // B. 等时间线走完（相对锚点）：忙窗 +3000–3900，terminate +6500，静默核查
  // +7500（threaded），s2 采样取 +8000。
  await waitPageOffset(conn, p0, 8000);
  await record(conn, "s2");

  // C. 正控通道：mutation → 应产出新帧（发布链路活跃正控；strip-ctrl 在分析区域外）。
  await rpcOn(conn, "script.evaluate", { expression: "return window.__zwCtrlFire();" });
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
  const mode = s2 ? s2.mode : null;
  const verdict = {
    // 模式面：threaded = S1a 宿主桥在（flag on）；shim = flag 关对照。
    mode,
    worker_host_present: s2 ? s2.workerHost : null,
    // 转移面：threaded 预期 transferred/sent + ctxOk=true；shim 预期 probe-only + typeof。
    transfer_state: s2 ? s2.transfer.state : null,
    transfer_ctx_ok: s2 ? s2.transfer.ctxOk : null,
    // 线程活性面：wt = worker 侧时钟。threaded 预期跨忙窗 wt 间隔 ~beat 周期量级
    //（≤200ms 门槛 = 周期 50ms 的 4 倍容忍）；shim 预期复现基线 ~900ms（同载）。
    // at = 页面送达时钟，两模式忙窗都停摆（主线程投递固有语义），对照列。
    beats_total: s2 ? s2.beats : null,
    wt_gap_max_ms: s2 ? s2.wtGapMax : null,
    wt_gap_spanning_load_ms: s2 ? s2.wtGapSpanLoad : null,
    at_gap_max_ms: s2 ? s2.atGapMax : null,
    at_gap_spanning_load_ms: s2 ? s2.atGapSpanLoad : null,
    load_window: s2 ? s2.loadWindow : null,
    draws_main: s2 ? s2.drawsMain : null,
    // 生命周期面（threaded）：terminate 后零 beat 回流。
    term: s2 ? s2.term : null,
    beats_after_term: s2 ? s2.beatsAfterTerm : null,
    // 错误路径面：worker 脚本抛错应回流页面 onerror（dc4-expected）。
    worker_err: s2 ? s2.workerErr : null,
    // 帧活性正控。
    canvas_idle: shaOf("s0") !== shaOf("s1") ? "ANIMATED" : "FROZEN",
    mutation_control: shaOf("s2") !== shaOf("s3") ? "OK" : "NO_NEW_FRAME",
    control_strip: s2 ? s2.ctrl : null,
  };
  // 判定（脚手架口径，呈现像素腿在 analyze）：
  //   shim：基线签名复现（wt 跨忙窗 ≥800ms）+ transfer API 在 + 正控 OK。
  //   threaded：wt 跨忙窗 ≤200ms + ctxOk + term 静默 + err 回流 + 正控 OK。
  if (mode === "shim") {
    verdict.state =
      (verdict.wt_gap_spanning_load_ms ?? 0) >= 800 &&
      verdict.transfer_state === "probe-only" &&
      verdict.mutation_control === "OK"
        ? "SHIM_BASELINE_PASS"
        : "SHIM_BASELINE_FAIL";
  } else if (mode === "threaded") {
    const gates = {
      wt_liveness: (verdict.wt_gap_spanning_load_ms ?? 1e9) <= 200,
      transfer_ctx: verdict.transfer_ctx_ok === true,
      term_silent: verdict.beats_after_term === 0,
      err_reported: String(verdict.worker_err || "").includes("dc4-expected"),
      control: verdict.mutation_control === "OK",
    };
    verdict.gates = gates;
    verdict.state = Object.values(gates).every(Boolean)
      ? "THREADED_PROBE_PASS"
      : "THREADED_PROBE_PARTIAL";
  } else {
    verdict.state = "UNKNOWN_MODE";
  }
  fs.writeFileSync(
    path.join(outDir, "comp04a-probe.json"),
    JSON.stringify({ fixture: path.basename(fixturePath), started_at_wall: new Date(tProbeStart).toISOString(), samples, verdict }, null, 2),
  );
  console.log(`[probe] verdict: ${JSON.stringify(verdict)}`);
} catch (error) {
  fs.writeFileSync(
    path.join(outDir, "comp04a-probe.json"),
    JSON.stringify({ error: String(error && error.stack ? error.stack : error), samples }, null, 2),
  );
  console.error(`[probe] FAILED: ${error}`);
  process.exitCode = 1;
} finally {
  if (conn) conn.close();
}

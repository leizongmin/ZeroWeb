#!/usr/bin/env node
// dom-ops 采集 runner — 跨浏览器 DOM/样式几何/Canvas 操作基准（协议 v1）。
//
// 流程：spawn ZeroWeb headless / Chrome headless → connectOverCDP → 本地 HTTP
// 服务 fixtures（COOP/COEP 头，Chrome 侧解锁细粒度 performance.now）→ 每个
// fixture 每个顺序遍历一次 goto，同 fixture 的子场景在同一页面会话内连续跑
// （forward = 协议顺序，reverse = 逆序——共享引擎状态，顺序效应可被检出）→
// 每个子场景一次 Runtime.evaluate（循环在页面内跑完，CDP 往返不进样本）→
// 预热 + 采样 + 页面内语义断言 → 写 per-browser JSON + 合并对比报告。
//
// 退出码：0 = 无语义断言 FAIL（INCONCLUSIVE 行如实标记在报告中，不改退出码）；
// 1 = 任一语义断言 FAIL；2 = 环境错误（二进制缺失、端点不可达、未知场景）或
// self-test 未捕获注入错误。
//
// 用法见同目录 README.md。依赖 playwright-core：从 tests/playwright-matrix 的
// node_modules 解析（pin 1.63.0，不新增安装）。

import fs from 'node:fs'
import net from 'node:net'
import os from 'node:os'
import path from 'node:path'
import { spawn, execFileSync } from 'node:child_process'
import { createServer as createHttpServer } from 'node:http'
import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'

// playwright-core 从 playwright-matrix 的 pin 安装解析，不新增 npm 依赖。
const require = createRequire(path.join(path.dirname(fileURLToPath(import.meta.url)), '..', '..', 'playwright-matrix', 'package.json'))
const { chromium } = require('playwright-core')

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = path.resolve(SCRIPT_DIR, '..', '..', '..')
const MATRIX_DIR = path.join(REPO_ROOT, 'tests', 'playwright-matrix')
const FIXTURES_DIR = path.join(SCRIPT_DIR, 'fixtures')
const DEFAULT_OUT = path.join(SCRIPT_DIR, 'results', timestamp())

function timestamp() {
  const d = new Date()
  const p = (n, w = 2) => String(n).padStart(w, '0')
  return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}_${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`
}

const fail = (msg) => {
  console.error(msg)
  process.exit(2)
}

// ── 协议常量（权威来源；README.md 引用本表，不改两份） ──

const PROTOCOL = {
  version: 'v1',
  viewport: { width: 1280, height: 720 },
  samples: 12,
  sampleGapMs: 25,
  sampleCapMs: 4000,
  warmupMax: 100,
  settleMs: 300,
  subWatchdogMs: 180_000,
  orderRatioAlert: 3.0,
  anomalyRatio: 10.0,
  notableRatio: 3.0,
  // per-op 值由样本时长派生，定时器分辨率影响的是样本总量：任一侧分辨率误差
  // 超过其最小样本的 10% → 该比较行 INCONCLUSIVE（ZeroWeb 实测 ~100µs 粒度）。
  timerErrorMaxFraction: 0.10,
  fixtureUrl: {
    'style-geometry': '/style-geometry.html',
    mutation: '/mutation.html',
    canvas: '/canvas.html',
  },
  sub: [
    { scenario: 'style-geometry', op: 'style.read.same', ops: 2000 },
    { scenario: 'style-geometry', op: 'style.read.rotate', ops: 2000 },
    { scenario: 'style-geometry', op: 'style.write.color', ops: 1000 },
    { scenario: 'style-geometry', op: 'geom.read.same', ops: 2000 },
    { scenario: 'style-geometry', op: 'geom.read.rotate', ops: 1000 },
    { scenario: 'style-geometry', op: 'geom.write_read.thrash', ops: 50 },
    { scenario: 'mutation', op: 'mut.replace.pool', ops: 20 },
    { scenario: 'mutation', op: 'mut.build.append', ops: 20 },
    { scenario: 'mutation', op: 'mut.remove.children', ops: 15 },
    { scenario: 'mutation', op: 'mut.move.identity', ops: 100 },
    { scenario: 'canvas', op: 'cv.state.fillStyle', ops: 2000 },
    { scenario: 'canvas', op: 'cv.state.saveRestore', ops: 500 },
    { scenario: 'canvas', op: 'cv.draw.rect', ops: 200 },
    { scenario: 'canvas', op: 'cv.read.imageData', ops: 200 },
  ],
}

// ── 页面内协议（必须自包含：经 page.evaluate 序列化，不引用外层作用域） ──

function runInPage(arg) {
  const { op, cfg } = arg
  const out = {
    samplesMs: [],
    checks: [],
    notes: [],
    truncated: false,
    error: null,
  }
  const N = 600

  function chk(name, pass, expected, actual) {
    out.checks.push({ name, pass: !!pass, expected, actual })
  }
  function closeTo(v, exp, tol) {
    return (
      Array.isArray(v) &&
      v.length === exp.length &&
      v.every((x, i) => Math.abs(x - exp[i]) <= tol)
    )
  }
  function parseColor(s) {
    if (typeof s !== 'string') return null
    s = s.trim().toLowerCase()
    let m = s.match(/^#([0-9a-f]{3})$/)
    if (m) return m[1].split('').map((c) => parseInt(c + c, 16))
    m = s.match(/^#([0-9a-f]{6})$/)
    if (m) return [0, 2, 4].map((i) => parseInt(m[1].slice(i, i + 2), 16))
    m = s.match(/^rgba?\(([^)]+)\)$/)
    if (m) {
      const parts = m[1].split(/[ ,/]+/).filter(Boolean)
      if (parts.length < 3) return null
      return parts.slice(0, 3).map((p) =>
        p.endsWith('%') ? Math.round(parseFloat(p) * 2.55) : Math.round(parseFloat(p))
      )
    }
    return null
  }

  const COLORS = ['#1a73e8', 'rgb(234, 67, 53)', '#0f9d58', '#f4b400']
  const WRITE_COLORS = ['rgb(255, 0, 0)', 'rgb(0, 128, 0)', 'rgb(0, 0, 255)']
  const S = { writes: 0, appended: 0, removed: 0 }

  // style-geometry 同页多子场景共享会话：setup 清掉其他子场景留下的内联样式，
  // 读断言不被写子场景污染（DC-4 同会话顺序采样的隔离前提）。
  function freshCells() {
    const cells = document.querySelectorAll('#board .cell')
    for (const el of cells) {
      el.style.color = ''
      el.style.width = ''
    }
    return cells
  }

  const OPS = {
    'style.read.same': {
      setup() {
        S.cells = freshCells()
      },
      op(i) {
        return getComputedStyle(S.cells[0]).color
      },
      verify() {
        const c0 = parseColor(getComputedStyle(S.cells[0]).color)
        chk('computed.color.c0', closeTo(c0, [26, 115, 232], 1), [26, 115, 232], c0)
        const c1 = parseColor(getComputedStyle(S.cells[1]).color)
        chk('computed.color.c1', closeTo(c1, [234, 67, 53], 1), [234, 67, 53], c1)
      },
    },
    'style.read.rotate': {
      setup() {
        S.cells = freshCells()
      },
      op(i) {
        return getComputedStyle(S.cells[i % N]).color
      },
      verify() {
        const c5 = parseColor(getComputedStyle(S.cells[N - 1]).color)
        chk('computed.color.c5', closeTo(c5, [158, 158, 158], 1), [158, 158, 158], c5)
      },
    },
    'style.write.color': {
      setup() {
        S.cells = freshCells()
        S.writes = 0
      },
      op(i) {
        S.cells[i % N].style.color = WRITE_COLORS[i % 3]
        S.writes++
      },
      verify() {
        const idx = (S.writes - 1) % N
        const got = parseColor(S.cells[idx].style.color)
        const exp = parseColor(WRITE_COLORS[(S.writes - 1) % 3])
        chk('style.write.roundtrip', closeTo(got, exp, 1), exp, got)
      },
    },
    'geom.read.same': {
      setup() {
        S.cells = freshCells()
      },
      op(i) {
        return S.cells[0].getBoundingClientRect()
      },
      verify() {
        const r0 = S.cells[0].getBoundingClientRect()
        const rN = S.cells[N - 1].getBoundingClientRect()
        chk(
          'geom.finite',
          Number.isFinite(r0.top) && r0.width > 0 && r0.height > 0,
          'finite, w>0, h>0',
          { top: r0.top, width: r0.width, height: r0.height }
        )
        chk('geom.order', rN.top >= r0.top, 'cells[599].top >= cells[0].top', {
          first: r0.top,
          last: rN.top,
        })
      },
    },
    'geom.read.rotate': {
      setup() {
        S.cells = freshCells()
      },
      op(i) {
        return S.cells[i % N].getBoundingClientRect()
      },
      verify() {
        const r = S.cells[N >> 1].getBoundingClientRect()
        chk('geom.finite.mid', Number.isFinite(r.top) && r.width > 0, 'finite, w>0', {
          top: r.top,
          width: r.width,
        })
      },
    },
    'geom.write_read.thrash': {
      setup() {
        S.cells = freshCells()
      },
      op(i) {
        S.cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
        return S.cells[(i + 1) % N].getBoundingClientRect().top
      },
      verify() {
        S.cells[3].style.width = '33px'
        const w = S.cells[3].getBoundingClientRect().width
        chk('geom.thrash.applied', Math.abs(w - 33) <= 2, 33, w)
      },
    },
    'mut.replace.pool': {
      setup() {
        S.box = document.getElementById('list-replace')
        S.box.replaceChildren()
        S.pool = []
        for (let i = 0; i < 50; i++) {
          const d = document.createElement('div')
          d.className = 'item'
          d.dataset.seq = String(i)
          S.pool.push(d)
        }
      },
      op(i) {
        S.box.replaceChildren(S.pool[0], S.pool[1], S.pool[2], S.pool[3], S.pool[4],
          S.pool[5], S.pool[6], S.pool[7], S.pool[8], S.pool[9], S.pool[10], S.pool[11],
          S.pool[12], S.pool[13], S.pool[14], S.pool[15], S.pool[16], S.pool[17],
          S.pool[18], S.pool[19], S.pool[20], S.pool[21], S.pool[22], S.pool[23],
          S.pool[24], S.pool[25], S.pool[26], S.pool[27], S.pool[28], S.pool[29],
          S.pool[30], S.pool[31], S.pool[32], S.pool[33], S.pool[34], S.pool[35],
          S.pool[36], S.pool[37], S.pool[38], S.pool[39], S.pool[40], S.pool[41],
          S.pool[42], S.pool[43], S.pool[44], S.pool[45], S.pool[46], S.pool[47],
          S.pool[48], S.pool[49])
      },
      verify() {
        // DC-4 self-test：注入错误期望（51 ≠ 实际 50）必须被检出为 FAIL。
        const expLen = cfg.injectFault ? 51 : 50
        const kids = S.box.children
        chk('mut.replace.count', kids.length === expLen, expLen, kids.length)
        let identityOk = kids.length === 50
        let firstBad = null
        for (let i = 0; i < kids.length && identityOk; i++) {
          if (kids[i] !== S.pool[i] || kids[i].dataset.seq !== String(i)) {
            identityOk = false
            firstBad = {
              index: i,
              identity: kids[i] === S.pool[i],
              seq: kids[i].dataset ? kids[i].dataset.seq : null,
              poolSeq: S.pool[i] ? S.pool[i].dataset.seq : null,
            }
          }
        }
        chk('mut.replace.identity_order', identityOk, 'children[i] === pool[i], seq 0..49',
          identityOk ? true : firstBad)
      },
    },
    'mut.build.append': {
      setup() {
        S.box = document.getElementById('list-append')
        S.box.replaceChildren()
        S.appended = 0
        S.lastAppended = null
      },
      op(i) {
        const d = document.createElement('div')
        d.className = 'item'
        S.box.appendChild(d)
        S.lastAppended = d
        S.appended++
      },
      verify() {
        chk('mut.append.count', S.box.children.length === S.appended, S.appended, S.box.children.length)
        chk('mut.append.last', S.box.lastElementChild === S.lastAppended,
          'last child is the last appended node', S.box.lastElementChild === S.lastAppended)
      },
    },
    'mut.remove.children': {
      setup() {
        S.box = document.getElementById('list-remove')
        S.box.replaceChildren()
        const frag = []
        for (let i = 0; i < 1500; i++) frag.push('<div class="item" data-seq="' + i + '"></div>')
        S.box.innerHTML = frag.join('')
        S.removed = 0
      },
      op(i) {
        S.box.removeChild(S.box.lastElementChild)
        S.removed++
      },
      verify() {
        chk('mut.remove.count', S.box.children.length === 1500 - S.removed, 1500 - S.removed,
          S.box.children.length)
      },
    },
    'mut.move.identity': {
      setup() {
        S.box = document.getElementById('list-move')
        S.box.replaceChildren()
        S.marker = null
        const frag = []
        for (let i = 0; i < 300; i++) {
          frag.push('<div class="item" data-seq="' + i + '"></div>')
        }
        S.box.innerHTML = frag.join('')
        for (const el of S.box.children) {
          if (el.dataset.seq === '150') S.marker = el
        }
      },
      op(i) {
        S.box.insertBefore(S.box.lastElementChild, S.box.firstElementChild)
      },
      verify() {
        let found = false
        for (const el of S.box.children) {
          if (el === S.marker) found = true
        }
        chk('mut.move.identity', S.marker.parentElement === S.box && found,
          'marker same node still in container', {
            parentIsBox: S.marker.parentElement === S.box,
            inContainer: found,
          })
        chk('mut.move.count', S.box.children.length === 300, 300, S.box.children.length)
      },
    },
    'cv.state.fillStyle': {
      setup() {
        S.ctx = document.getElementById('cv').getContext('2d')
      },
      op(i) {
        S.ctx.fillStyle = COLORS[i % 4]
      },
      verify() {
        S.ctx.fillStyle = '#1a73e8'
        const got = parseColor(S.ctx.fillStyle)
        chk('cv.fillStyle.roundtrip', closeTo(got, [26, 115, 232], 1), [26, 115, 232], got)
      },
    },
    'cv.state.saveRestore': {
      setup() {
        S.ctx = document.getElementById('cv').getContext('2d')
        S.ctx.fillStyle = '#000000'
      },
      op(i) {
        S.ctx.save()
        S.ctx.fillStyle = COLORS[i % 4]
        S.ctx.restore()
      },
      verify() {
        const got = parseColor(S.ctx.fillStyle)
        chk('cv.saveRestore.state', closeTo(got, [0, 0, 0], 1), [0, 0, 0], got)
      },
    },
    'cv.draw.rect': {
      setup() {
        S.ctx = document.getElementById('cv').getContext('2d')
        S.ctx.clearRect(0, 0, 256, 256)
      },
      op(i) {
        S.ctx.fillStyle = COLORS[i % 4]
        S.ctx.fillRect((i % 16) * 16, ((i >> 4) % 16) * 16, 8, 8)
      },
      verify() {
        const ctx = S.ctx
        ctx.clearRect(0, 0, 256, 256)
        ctx.fillStyle = 'rgb(255, 0, 0)'
        ctx.fillRect(10, 10, 20, 20)
        ctx.fillStyle = 'rgb(0, 0, 255)'
        ctx.fillRect(100, 100, 20, 20)
        const px = (x, y) => Array.from(ctx.getImageData(x, y, 1, 1).data.slice(0, 4))
        const red = px(20, 20)
        const blue = px(110, 110)
        const bg = px(200, 50)
        chk('cv.draw.red', closeTo(red, [255, 0, 0, 255], 2), [255, 0, 0, 255], red)
        chk('cv.draw.blue', closeTo(blue, [0, 0, 255, 255], 2), [0, 0, 255, 255], blue)
        chk('cv.draw.bg_alpha0', bg[3] === 0, 'alpha 0 outside fills', bg)
        const w = ctx.measureText('Hello benchmark').width
        chk('cv.measureText.finite', Number.isFinite(w) && w > 0, '>0 finite', w)
      },
    },
    'cv.read.imageData': {
      setup() {
        S.ctx = document.getElementById('cv').getContext('2d')
        S.ctx.clearRect(0, 0, 256, 256)
        S.ctx.fillStyle = 'rgb(255, 0, 0)'
        S.ctx.fillRect(10, 10, 20, 20)
      },
      op(i) {
        return S.ctx.getImageData(96, 96, 32, 32)
      },
      verify() {
        const d = Array.from(S.ctx.getImageData(20, 20, 1, 1).data.slice(0, 4))
        chk('cv.read.known_pixel', closeTo(d, [255, 0, 0, 255], 2), [255, 0, 0, 255], d)
      },
    },
  }

  const unit = OPS[op]
  if (!unit) {
    out.error = 'unknown op: ' + op
    return Promise.resolve(out)
  }
  unit.setup()
  const warmup = Math.min(cfg.opsPerSample, cfg.warmupMax)
  for (let i = 0; i < warmup; i++) unit.op(i)

  return new Promise((resolve) => {
    let s = 0
    const step = () => {
      if (s >= cfg.samples) return finish()
      setTimeout(() => {
        const t0 = performance.now()
        for (let i = 0; i < cfg.opsPerSample; i++) unit.op(i)
        const dt = performance.now() - t0
        out.samplesMs.push(dt)
        if (dt > cfg.sampleCapMs) {
          out.truncated = true
          out.notes.push('sample ' + s + ' exceeded cap ' + cfg.sampleCapMs + 'ms; truncated')
          return finish()
        }
        s++
        step()
      }, cfg.sampleGapMs)
    }
    const finish = () => {
      try {
        unit.verify()
      } catch (e) {
        chk('verify.noThrow', false, 'no throw', String(e && e.message ? e.message : e))
      }
      resolve(out)
    }
    step()
  })
}

// ── 定量分析（Node 侧） ──

function quantile(sorted, p) {
  if (!sorted.length) return null
  const idx = (sorted.length - 1) * p
  const lo = Math.floor(idx)
  const hi = Math.ceil(idx)
  return sorted[lo] + (sorted[hi] - sorted[lo]) * (idx - lo)
}

function summarize(run, ops) {
  if (!run || !run.samplesMs.length) return null
  const perOpUs = run.samplesMs.map((ms) => (ms / ops) * 1000).sort((a, b) => a - b)
  const samplesSorted = [...run.samplesMs].sort((a, b) => a - b)
  const p50Ms = quantile(samplesSorted, 0.5)
  return {
    samples_ms: run.samplesMs,
    truncated: !!run.truncated,
    per_op_us: {
      p05: quantile(perOpUs, 0.05),
      p50: quantile(perOpUs, 0.5),
      p95: quantile(perOpUs, 0.95),
    },
    ops_per_sec: (ops * 1000) / p50Ms,
  }
}

// 状态推导：evaluate 错误或样本不足一半的截断 → INCONCLUSIVE；任一语义断言
// 失败 → FAIL；双序齐备且断言全过 → PASS（degraded 截断只记 notes）。
function deriveStatus(result) {
  if (result.error) return 'INCONCLUSIVE'
  if (result.checks.some((c) => !c.pass)) return 'FAIL'
  const orders = [result.forward, result.reverse].filter(Boolean)
  if (orders.length < 2) return 'INCONCLUSIVE'
  for (const o of orders) {
    if (o.truncated && o.samples_ms.length < Math.ceil(result.samples_requested / 2)) {
      return 'INCONCLUSIVE'
    }
  }
  return 'PASS'
}

// ── 环境与浏览器管理 ──

function gitInfo() {
  try {
    const sha = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: REPO_ROOT }).toString().trim()
    const dirty = execFileSync('git', ['status', '--porcelain'], { cwd: REPO_ROOT }).toString().length > 0
    return { sha, dirty }
  } catch {
    return { sha: 'unknown', dirty: null }
  }
}

function freePort() {
  // 测试工具用途，竞态窗口可接受（同 tests/playwright-matrix/scripts 惯例）。
  return new Promise((resolve, reject) => {
    const srv = net.createServer()
    srv.listen(0, '127.0.0.1', () => {
      const port = srv.address().port
      srv.close(() => resolve(port))
    })
    srv.on('error', reject)
  })
}

async function waitEndpoint(port, timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/json/version`)
      if (res.ok) return await res.json()
    } catch { /* 未就绪 */ }
    await new Promise((r) => setTimeout(r, 200))
  }
  return null
}

function terminate(proc) {
  return new Promise((resolve) => {
    if (!proc || proc.exitCode !== null) return resolve()
    const timer = setTimeout(() => proc.kill('SIGKILL'), 5000)
    proc.once('exit', () => {
      clearTimeout(timer)
      resolve()
    })
    proc.kill('SIGTERM')
  })
}

function startContentServer() {
  return new Promise((resolve, reject) => {
    const server = createHttpServer((req, res) => {
      const name = new URL(req.url, 'http://x').pathname.replace(/^\/+/, '')
      const file = path.join(FIXTURES_DIR, name)
      // 信任边界：只允许本目录内的既有 fixture 文件（防路径穿越）。
      if (!name || path.dirname(file) !== FIXTURES_DIR || !fs.existsSync(file) || !fs.statSync(file).isFile()) {
        res.writeHead(404, { 'content-type': 'text/plain' })
        return res.end('not found')
      }
      res.writeHead(200, {
        'content-type': 'text/html; charset=utf-8',
        'cache-control': 'no-cache',
        // COOP/COEP：让 Chrome 页面进入 crossOriginIsolated，performance.now
        // 不被粗化到 100µs（非隔离页 Spectre 缓解）。fixture 无跨域子资源，
        // require-corp 无副作用。
        'cross-origin-opener-policy': 'same-origin',
        'cross-origin-embedder-policy': 'require-corp',
      })
      fs.createReadStream(file).pipe(res)
    })
    server.on('error', reject)
    server.listen(0, '127.0.0.1', () => resolve(server))
  })
}

async function launchBrowser(which) {
  const port = await freePort()
  let proc, binary, flags
  if (which === 'zeroweb') {
    binary = process.env.ZERO_WEB_DOM_OPS_BIN || path.join(REPO_ROOT, 'target', 'release', 'zero-browser')
    if (!fs.existsSync(binary)) fail(`zero-browser not built: ${binary} (build release first)`)
    flags = ['--headless', '--remote-debugging-port', String(port)]
    proc = spawn(binary, flags, { stdio: ['ignore', 'ignore', 'pipe'] })
  } else {
    binary = process.env.ZERO_WEB_DOM_OPS_CHROME || 'google-chrome'
    flags = [
      '--headless',
      `--remote-debugging-port=${port}`,
      `--user-data-dir=${fs.mkdtempSync(path.join(os.tmpdir(), 'dom-ops-chrome-'))}`,
      '--no-first-run',
      '--no-default-browser-check',
      '--disable-extensions',
      '--disable-background-timer-throttling',
      '--disable-backgrounding-occluded-windows',
      '--disable-renderer-backgrounding',
      '--window-size=1280,800',
    ]
    proc = spawn(binary, flags, { stdio: ['ignore', 'ignore', 'pipe'] })
  }
  let stderr = ''
  proc.stderr.on('data', (d) => (stderr += d))
  const version = await waitEndpoint(port)
  if (!version) {
    await terminate(proc)
    fail(`${which}: CDP endpoint not ready\n${stderr.slice(-500)}`)
  }
  return {
    which,
    proc,
    binary,
    flags,
    port,
    cdpProduct: version.Browser || 'unknown',
    tmpDirs: which === 'chrome' ? [flags.find((f) => f.startsWith('--user-data-dir=')).split('=')[1]] : [],
  }
}

// ── 采集 ──

async function runBrowserSession(page, subs, cfgCommon, baseUrl, acc) {
  // 同 fixture 子场景共享一次页面会话；forward 按协议顺序、reverse 逆序执行，
  // 共享引擎状态下的顺序效应可被检出（DC-4）。
  const groups = new Map()
  for (const s of subs) {
    if (!groups.has(s.scenario)) groups.set(s.scenario, [])
    groups.get(s.scenario).push(s)
  }
  const scenarioList = [...groups.keys()]
  const passes = [
    { order: 'forward', scenarios: scenarioList },
    { order: 'reverse', scenarios: [...scenarioList].reverse() },
  ]
  for (const pass of passes) {
    for (const scenario of pass.scenarios) {
      acc.harness.evaluate_calls++
      try {
        await page.goto(`${baseUrl}${PROTOCOL.fixtureUrl[scenario]}`, { waitUntil: 'load' })
        await new Promise((r) => setTimeout(r, cfgCommon.settleMs))
      } catch (e) {
        for (const sub of groups.get(scenario)) {
          acc.scenarios[`${scenario}/${sub.op}`].error = `goto failed: ${String(e?.message || e).slice(0, 200)}`
        }
        continue
      }
      acc.harness.evaluate_calls += groups.get(scenario).length
      // reverse 遍历同时逆转同组子场景顺序（顺序效应覆盖组内）。
      const groupSubs = pass.order === 'reverse' ? [...groups.get(scenario)].reverse() : groups.get(scenario)
      for (const sub of groupSubs) {
        const key = `${scenario}/${sub.op}`
        const result = acc.scenarios[key]
        console.log(`  -- ${pass.order} ${key}`)
        let watchdog
        try {
          const run = await Promise.race([
            page.evaluate(runInPage, {
              scenario,
              op: sub.op,
              cfg: { ...cfgCommon, opsPerSample: sub.ops },
            }),
            new Promise((_, rej) => {
              watchdog = setTimeout(
                () => rej(new Error(`watchdog ${PROTOCOL.subWatchdogMs}ms`)),
                PROTOCOL.subWatchdogMs
              )
            }),
          ])
          clearTimeout(watchdog)
          result[pass.order] = summarize(run, sub.ops)
          result.checks = result.checks.concat(run.checks)
          result.notes = result.notes.concat(run.notes)
        } catch (e) {
          clearTimeout(watchdog)
          result.notes.push(`${pass.order}: ${String(e?.message || e).slice(0, 200)}`)
          if (String(e?.message || e).includes('watchdog')) {
            result.error = result.error || 'evaluate watchdog timeout'
          }
        }
      }
    }
  }
  for (const result of Object.values(acc.scenarios)) {
    const fwd = result.forward?.per_op_us?.p50
    const rev = result.reverse?.per_op_us?.p50
    if (fwd != null && rev != null) {
      const spread = Math.abs(fwd - rev) / Math.min(fwd, rev)
      result.order_spread = spread
      if (spread > PROTOCOL.orderRatioAlert) {
        result.notes.push(
          `order effect: forward p50 ${fwd.toFixed(2)}µs vs reverse ${rev.toFixed(2)}µs (ratio ${spread.toFixed(1)})`
        )
      }
    }
    result.status = deriveStatus(result)
  }
}

async function collectBrowser(which, attachEndpoint, baseUrl, cfgCommon, subs) {
  let handle = null
  let browser
  if (attachEndpoint) {
    browser = await chromium.connectOverCDP(attachEndpoint)
  } else {
    handle = await launchBrowser(which)
    browser = await chromium.connectOverCDP(`http://127.0.0.1:${handle.port}`)
  }
  try {
    const context = browser.contexts()[0]
    const page = await context.newPage()
    await page.setViewportSize(PROTOCOL.viewport)
    await page.goto(`${baseUrl}${PROTOCOL.fixtureUrl['style-geometry']}`, { waitUntil: 'load' })
    const meta = await page.evaluate(() => ({
      ua: navigator.userAgent,
      iso: self.crossOriginIsolated === true,
      // 计时分辨率（µs）：相邻两次 performance.now 的最小正差。Chrome 非隔离页
      // 粗化到 100µs，会毁掉 µs 级行——据此把对应比较行判 INCONCLUSIVE。
      timerResUs: (() => {
        let res = Infinity
        for (let k = 0; k < 2000; k++) {
          const d = performance.now()
          const e = performance.now()
          const delta = e - d
          if (delta > 0 && delta < res) res = delta
        }
        return Number.isFinite(res) ? res * 1000 : null
      })(),
    }))
    const acc = { harness: { evaluate_calls: 0 }, scenarios: {} }
    for (const sub of subs) {
      acc.scenarios[`${sub.scenario}/${sub.op}`] = {
        ops_per_sample: sub.ops,
        samples_requested: cfgCommon.samples,
        warmup_ops: Math.min(sub.ops, cfgCommon.warmupMax),
        sample_cap_ms: cfgCommon.sampleCapMs,
        forward: null,
        reverse: null,
        checks: [],
        notes: [],
        error: null,
        status: 'INCONCLUSIVE',
      }
    }
    await runBrowserSession(page, subs, cfgCommon, baseUrl, acc)
    return {
      browser: which,
      // 报告记录相对路径（仓内二进制相对 REPO_ROOT，仓外保留原值），避免本机绝对路径入库。
      binary: handle
        ? (handle.binary.startsWith(REPO_ROOT) ? path.relative(REPO_ROOT, handle.binary) : handle.binary)
        : `attach:${attachEndpoint}`,
      flags: handle ? handle.flags : null,
      cdp_product: handle ? handle.cdpProduct : 'attach',
      user_agent: meta.ua,
      cross_origin_isolated: meta.iso,
      timer_resolution_us: meta.timerResUs,
      scenarios: acc.scenarios,
      harness: acc.harness,
      bridge_counters: 'not_exposed',
    }
  } finally {
    await browser.close().catch(() => {})
    if (handle) {
      await terminate(handle.proc)
      for (const d of handle.tmpDirs) fs.rmSync(d, { recursive: true, force: true })
    }
  }
}

function buildComparison(runs) {
  const zw = runs.find((r) => r.browser === 'zeroweb')
  const chrome = runs.find((r) => r.browser === 'chrome')
  const sampleTooCoarse = (run, r) => {
    if (!run || run.timer_resolution_us == null || !r) return false
    const arrs = [r.forward?.samples_ms, r.reverse?.samples_ms].filter(Boolean).flat()
    if (!arrs.length) return false
    const minSampleUs = Math.min(...arrs) * 1000
    return run.timer_resolution_us > PROTOCOL.timerErrorMaxFraction * minSampleUs
  }
  const comparison = {}
  for (const sub of PROTOCOL.sub) {
    const key = `${sub.scenario}/${sub.op}`
    const z = zw?.scenarios[key]
    const c = chrome?.scenarios[key]
    const avg = (r) =>
      r?.forward && r?.reverse ? (r.forward.per_op_us.p50 + r.reverse.per_op_us.p50) / 2 : null
    const zp = avg(z)
    const cp = avg(c)
    const row = {
      zeroweb_p50_us: zp,
      chrome_p50_us: cp,
      ratio_p50: zp != null && cp != null && cp > 0 ? zp / cp : null,
      status: 'MISSING',
      classification: null,
      inconclusive_reason: null,
    }
    const polluted = (r) =>
      r && (r.status !== 'PASS' || r.notes.some((n) => n.includes('order effect')))
    if (!z || !c) {
      row.inconclusive_reason = row.inconclusive_reason || 'missing side'
    } else if (polluted(z) || polluted(c)) {
      row.status = 'INCONCLUSIVE'
      row.inconclusive_reason =
        z.status !== 'PASS' ? `zeroweb:${z.status}` : c.status !== 'PASS' ? `chrome:${c.status}` : 'order effect'
    } else if (sampleTooCoarse(zw, z) || sampleTooCoarse(chrome, c)) {
      row.status = 'INCONCLUSIVE'
      row.inconclusive_reason = 'timer resolution error >10% of smallest sample'
    }
    if (row.ratio_p50 != null && row.status !== 'INCONCLUSIVE') {
      row.status = 'OK'
      row.classification =
        row.ratio_p50 >= PROTOCOL.anomalyRatio
          ? 'ANOMALY'
          : row.ratio_p50 >= PROTOCOL.notableRatio
            ? 'notable'
            : 'ok'
    }
    comparison[key] = row
  }
  return comparison
}

// ── 主流程 ──

function parseArgs() {
  const argv = process.argv.slice(2)
  const get = (name) => {
    const i = argv.indexOf(name)
    return i >= 0 ? argv[i + 1] : undefined
  }
  return {
    browser: get('--browser') || 'both',
    outDir: get('--out') || DEFAULT_OUT,
    scenario: get('--scenario'),
    cdpEndpoint: get('--cdp-endpoint'),
    selfTest: argv.includes('--self-test'),
  }
}

async function main() {
  const args = parseArgs()
  if (!['both', 'chrome', 'zeroweb'].includes(args.browser)) fail(`unknown --browser ${args.browser}`)
  if (!fs.existsSync(path.join(MATRIX_DIR, 'node_modules'))) {
    fail('tests/playwright-matrix/node_modules missing (run npm install there first)')
  }
  fs.mkdirSync(args.outDir, { recursive: true })

  let subs = PROTOCOL.sub
  if (args.selfTest) {
    subs = [PROTOCOL.sub.find((s) => s.op === 'mut.replace.pool')]
  } else if (args.scenario) {
    subs = PROTOCOL.sub.filter((s) => `${s.scenario}/${s.op}`.includes(args.scenario))
  }
  if (!subs.length) fail(`no sub-scenario matches --scenario ${args.scenario}`)

  const cfgCommon = {
    samples: PROTOCOL.samples,
    sampleGapMs: PROTOCOL.sampleGapMs,
    sampleCapMs: PROTOCOL.sampleCapMs,
    warmupMax: PROTOCOL.warmupMax,
    settleMs: PROTOCOL.settleMs,
    injectFault: args.selfTest,
  }

  const server = await startContentServer()
  const baseUrl = `http://127.0.0.1:${server.address().port}`
  const git = gitInfo()
  const env = {
    platform_class: `${os.platform()}-${os.arch()}`,
    cpu_model: os.cpus()[0]?.model,
    cpu_cores: os.cpus().length,
  }
  console.log(`dom-ops ${PROTOCOL.version} @ ${git.sha}${git.dirty ? ' (dirty)' : ''} → ${path.relative(REPO_ROOT, args.outDir)}`)

  const whiches = args.browser === 'both' ? ['zeroweb', 'chrome'] : [args.browser]
  const runs = []
  let hadFail = false
  try {
    for (const which of whiches) {
      console.log(`== ${which} ==`)
      const run = await collectBrowser(which, args.cdpEndpoint, baseUrl, cfgCommon, subs)
      runs.push(run)
      for (const [key, sc] of Object.entries(run.scenarios)) {
        const p50 = sc.forward?.per_op_us?.p50
        const p50r = sc.reverse?.per_op_us?.p50
        console.log(
          `  ${sc.status.padEnd(13)} ${key.padEnd(26)} fwd ${p50 != null ? p50.toFixed(2) + 'µs' : '—'} rev ${p50r != null ? p50r.toFixed(2) + 'µs' : '—'} checks ${sc.checks.filter((c) => c.pass).length}/${sc.checks.length}`
        )
        if (sc.status === 'FAIL') hadFail = true
      }
      fs.writeFileSync(
        path.join(args.outDir, `${which}.json`),
        JSON.stringify(
          {
            schema_version: 1,
            kind: 'dom-ops-run',
            protocol_version: PROTOCOL.version,
            generated_at: timestamp(),
            git_sha: git.sha,
            git_dirty: git.dirty,
            env,
            viewport: PROTOCOL.viewport,
            run,
          },
          null,
          2
        )
      )
    }
  } finally {
    server.close()
  }

  if (args.selfTest) {
    // harness 健康 = 注入错误期望在每次运行中都被检出为 FAIL。
    const caught = runs.length > 0 && runs.every((r) => r.scenarios['mutation/mut.replace.pool']?.status === 'FAIL')
    console.log(
      caught
        ? 'self-test PASS: injected wrong expectation detected as FAIL'
        : 'self-test FAIL: injected wrong expectation NOT detected'
    )
    process.exit(caught ? 0 : 2)
  }

  const comparison = whiches.length === 2 ? buildComparison(runs) : {}
  if (whiches.length === 2) {
    console.log('== comparison (p50 per-op, ZeroWeb/Chrome) ==')
    for (const [key, row] of Object.entries(comparison)) {
      if (row.ratio_p50 != null) {
        const tag = row.status === 'OK' ? ` [${row.classification}]` : ` [${row.inconclusive_reason}]`
        console.log(
          `  ${row.status.padEnd(13)} ${key.padEnd(26)} zw ${row.zeroweb_p50_us.toFixed(2)}µs / chrome ${row.chrome_p50_us.toFixed(2)}µs = ${row.ratio_p50.toFixed(1)}x${tag}`
        )
      } else {
        console.log(`  ${row.status.padEnd(13)} ${key}`)
      }
    }
  }

  fs.writeFileSync(
    path.join(args.outDir, 'report.json'),
    JSON.stringify(
      {
        schema_version: 1,
        kind: 'dom-ops-report',
        protocol_version: PROTOCOL.version,
        generated_at: timestamp(),
        git_sha: git.sha,
        git_dirty: git.dirty,
        env,
        runs,
        comparison,
      },
      null,
      2
    )
  )
  process.exit(hadFail ? 1 : 0)
}

main().catch((e) => fail(String(e?.stack || e)))

// P8e: replace.pool 机制构成——每臂独立新 tab。
//   p1: 裸池（无 className/dataset）240 op 紧循环   → 结构成本基线
//   p2: 属性池（基准同型）240 op 紧循环             → p2-p1 = 属性 overlay 贡献
//   p3: 池规模标定 replaceChildren(5)               → 成本是否 ∝ 子节点数
//   p4: 每条 mutation 记录价：createElement+className+dataset 挂文档内 vs 离屏
//   p5: 冷首 op 单独成本（fresh 文档第一个 replaceChildren）
import fs from 'node:fs'
import net from 'node:net'
import path from 'node:path'
import { spawn } from 'node:child_process'
import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import { createServer as createHttpServer } from 'node:http'

// 原始运行目录为本仓 checkout；经 import.meta.url 推导保持可复现（SL-008 不硬编码本机路径）。
const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..', '..', '..', '..')
const FIXTURES = path.join(REPO, 'tests/benchmarks/dom-ops/fixtures')
const require = createRequire(path.join(REPO, 'tests/playwright-matrix/package.json'))
const { chromium } = require('playwright-core')

function freePort() {
  return new Promise((resolve, reject) => {
    const srv = net.createServer()
    srv.listen(0, '127.0.0.1', () => { const p = srv.address().port; srv.close(() => resolve(p)) })
    srv.on('error', reject)
  })
}
async function waitEndpoint(port, ms = 30000) {
  const dl = Date.now() + ms
  while (Date.now() < dl) {
    try { const r = await fetch(`http://127.0.0.1:${port}/json/version`); if (r.ok) return true } catch {}
    await new Promise((r) => setTimeout(r, 200))
  }
  return false
}
function server() {
  return new Promise((res) => {
    const s = createHttpServer((req, resp) => {
      const name = new URL(req.url, 'http://x').pathname.replace(/^\/+/, '')
      const f = path.join(FIXTURES, name)
      if (!name || path.dirname(f) !== FIXTURES || !fs.existsSync(f)) { resp.writeHead(404); return resp.end() }
      resp.writeHead(200, { 'content-type': 'text/html; charset=utf-8' })
      fs.createReadStream(f).pipe(resp)
    })
    s.listen(0, '127.0.0.1', () => res(s))
  })
}

const which = process.argv[2] || 'zeroweb'
const port = await freePort()
let proc
if (which === 'zeroweb') {
  proc = spawn(path.join(REPO, 'target/release/zero-browser'), ['--headless', '--remote-debugging-port', String(port)], { stdio: ['ignore', 'ignore', 'pipe'] })
} else {
  proc = spawn('google-chrome', ['--headless', `--remote-debugging-port=${port}`, `--user-data-dir=${fs.mkdtempSync('/tmp/domops-probe8e-')}`, '--no-first-run', '--disable-background-timer-throttling'], { stdio: ['ignore', 'ignore', 'pipe'] })
}
if (!(await waitEndpoint(port))) { console.error('endpoint not ready'); process.exit(2) }
const http = await server()
const base = `http://127.0.0.1:${http.address().port}`
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`)
const ctx = browser.contexts()[0]
const settle = () => new Promise((r) => setTimeout(r, 300))

async function freshTab(fixture, fn, arg) {
  const page = await ctx.newPage()
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto(`${base}/${fixture}`, { waitUntil: 'load' })
  await settle()
  const r = await page.evaluate(fn, arg)
  await page.close().catch(() => {})
  return r
}

const out = { browser: which }

// replaceChildren 紧循环（自包含：attr 带属性池；count 子节点数；n op 数）。
const repLoop = (cfg) => {
  const box = document.getElementById('list-replace')
  box.replaceChildren()
  const pool = []
  for (let i = 0; i < cfg.count; i++) {
    const d = document.createElement('div')
    if (cfg.attr) { d.className = 'item'; d.dataset.seq = String(i) }
    pool.push(d)
  }
  const args = pool.slice(0, cfg.count)
  const cold0 = performance.now()
  box.replaceChildren(...args)
  const cold_ms = performance.now() - cold0
  const per = []
  for (let i = 0; i < cfg.n; i++) {
    const t = performance.now()
    box.replaceChildren(...args)
    per.push(performance.now() - t)
  }
  const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]
  return { cold_ms, med_us: med(per) * 1000 }
}

out.p1_bare = await freshTab('mutation.html', repLoop, { attr: false, count: 50, n: 240 })
out.p2_attr = await freshTab('mutation.html', repLoop, { attr: true, count: 50, n: 240 })
out.p3_small = await freshTab('mutation.html', repLoop, { attr: true, count: 5, n: 240 })

// p4: 每条 mutation 记录价——200 次 createElement+className+dataset，
// 挂到文档内容器 vs 离屏容器（离屏若不记录 mutation → 差值 = 记录成本）。
out.p4_mutation_price = await freshTab('mutation.html', () => {
  const host = document.getElementById('list-replace')
  const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]
  const price = (attach) => {
    const per = []
    for (let i = 0; i < 200; i++) {
      const t0 = performance.now()
      const d = document.createElement('div')
      d.className = 'item'
      d.dataset.seq = String(i)
      attach.appendChild(d)
      per.push(performance.now() - t0)
      if (i % 50 === 49) attach.replaceChildren()
    }
    return med(per) * 1000
  }
  return { attached_us: price(host), detached_us: price(document.createElement('div')) }
})

// p5: 冷首 op 复测 3 个 fresh 文档（首 op 单独计时分布）。
out.p5_cold_first = []
for (let i = 0; i < 3; i++) {
  out.p5_cold_first.push(await freshTab('mutation.html', () => {
    const box = document.getElementById('list-replace')
    const pool = []
    for (let j = 0; j < 50; j++) {
      const d = document.createElement('div')
      d.className = 'item'
      d.dataset.seq = String(j)
      pool.push(d)
    }
    const t0 = performance.now()
    box.replaceChildren(...pool)
    return performance.now() - t0
  }))
}

console.log(JSON.stringify(out))
await browser.close().catch(() => {})
proc.kill('SIGTERM')
http.close()
process.exit(0)

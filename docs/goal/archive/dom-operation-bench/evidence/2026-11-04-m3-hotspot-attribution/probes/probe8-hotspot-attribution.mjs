// P8: M3 热点归因微探针（临时脚本，结果入 evidence；不入仓运行产物）。
// 归因判别：
//   A（thrash）：pending gBCR 全链成本构成 + 每 op 成本随 op 序数是否增长
//     ——增长=有效视图 replay 随队列长度线性；平坦=固定全量重建。
//   B（replace.pool）：replaceChildren(50) 每 op 成长剖面 + 无属性池对照
//     ——属性 overlay 贡献 + 队列增长贡献。
// 小于 ~100µs 的量用批测（ZeroWeb performance.now ~100µs 粒度，v1 已记平台事实）。
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
    srvInit(resolve, reject)
  })
}
function srvInit(resolve, reject) {
  const srv = net.createServer()
  srv.listen(0, '127.0.0.1', () => { const p = srv.address().port; srv.close(() => resolve(p)) })
  srv.on('error', reject)
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
let proc, bin
if (which === 'zeroweb') {
  bin = path.join(REPO, 'target/release/zero-browser')
  proc = spawn(bin, ['--headless', '--remote-debugging-port', String(port)], { stdio: ['ignore', 'ignore', 'pipe'] })
} else {
  bin = 'google-chrome'
  proc = spawn(bin, ['--headless', `--remote-debugging-port=${port}`, `--user-data-dir=${fs.mkdtempSync('/tmp/domops-probe8-')}`, '--no-first-run', '--disable-background-timer-throttling'], { stdio: ['ignore', 'ignore', 'pipe'] })
}
if (!(await waitEndpoint(port))) { console.error('endpoint not ready'); process.exit(2) }
const http = await server()
const base = `http://127.0.0.1:${http.address().port}`
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`)
const page = await browser.contexts()[0].newPage()
await page.setViewportSize({ width: 1280, height: 720 })
await page.goto(`${base}/style-geometry.html`, { waitUntil: 'load' })
await new Promise((r) => setTimeout(r, 300))

const out = { browser: which }

// A1: drained gBCR 基线（批测 300 次取单次均摊；无 pending，走 snapshot 快路径）。
out.a1_drained_gbcr_us = await page.evaluate(() => {
  const el = document.querySelector('#board .cell')
  let sink = 0
  const t0 = performance.now()
  for (let i = 0; i < 300; i++) sink += el.getBoundingClientRect().width
  return (performance.now() - t0) * 1000 / 300 + (sink > 1e9 ? 0 : 0)
})

// A3: 同代际第二次读（RECT_LAYOUT_CACHE TLS 命中，批测 50 次）。
out.a3_same_gen_read_us = await page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  cells[3].style.width = '33px'
  cells[3].getBoundingClientRect() // 冷链首读（建立缓存）
  let sink = 0
  const t0 = performance.now()
  for (let i = 0; i < 50; i++) sink += cells[3].getBoundingClientRect().width
  const per = (performance.now() - t0) * 1000 / 50
  cells[3].style.width = ''
  return per + (sink > 1e9 ? 0 : 0)
})

// A2: thrash 每 op 成本成长剖面——单 evaluate 内 60 op，op i = 写 width + 计时 gBCR。
// 每 op 队列追加 2 条 mutation（SetStyle 空/值各一）；若每 op 成本随 op 序数增长，
// 则有效视图 replay（按队列长度线性）占主导；平坦则固定全量重建占主导。
out.a2_thrash_profile = await page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  const per = []
  for (let i = 0; i < 60; i++) {
    cells[3].style.width = i % 2 ? '33px' : ''
    const t0 = performance.now()
    cells[3].getBoundingClientRect()
    per.push(performance.now() - t0)
  }
  const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]
  return { first5: med(per.slice(0, 5)), last5: med(per.slice(-5)), mid: med(per.slice(27, 32)), all: per }
})

// B: replaceChildren(50) 每 op 成长剖面——同一 evaluate 内 50 op。
// 变体 1：池元素带 class+dataset（基准同型）；变体 2：裸 div（无属性）。
// 两者差 = 属性 overlay/attr 读链贡献；剖面斜率 = 队列/桶增长贡献。
for (const variant of ['attr', 'bare']) {
  await page.goto(`${base}/mutation.html`, { waitUntil: 'load' })
  await new Promise((r) => setTimeout(r, 200))
  out[`b_${variant}`] = await page.evaluate((variant) => {
    const box = document.getElementById('list-replace')
    const pool = []
    for (let i = 0; i < 50; i++) {
      const d = document.createElement('div')
      if (variant === 'attr') { d.className = 'item'; d.dataset.seq = String(i) }
      pool.push(d)
    }
    const per = []
    for (let i = 0; i < 50; i++) {
      const t0 = performance.now()
      box.replaceChildren(pool[0], pool[1], pool[2], pool[3], pool[4], pool[5], pool[6], pool[7], pool[8], pool[9],
        pool[10], pool[11], pool[12], pool[13], pool[14], pool[15], pool[16], pool[17], pool[18], pool[19],
        pool[20], pool[21], pool[22], pool[23], pool[24], pool[25], pool[26], pool[27], pool[28], pool[29],
        pool[30], pool[31], pool[32], pool[33], pool[34], pool[35], pool[36], pool[37], pool[38], pool[39],
        pool[40], pool[41], pool[42], pool[43], pool[44], pool[45], pool[46], pool[47], pool[48], pool[49])
      per.push(performance.now() - t0)
    }
    const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]
    return { first5: med(per.slice(0, 5)), last5: med(per.slice(-5)), mid: med(per.slice(22, 27)) }
  }, variant)
}

console.log(JSON.stringify(out))
await browser.close().catch(() => {})
proc.kill('SIGTERM')
http.close()
process.exit(0)

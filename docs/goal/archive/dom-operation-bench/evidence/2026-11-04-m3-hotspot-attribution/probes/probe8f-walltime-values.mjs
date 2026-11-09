// P8f: 终判别——probe8 序（a1→a3→a2 同页）vs probe8c e1 形（新页冷）的
// Node 侧 evaluate 墙钟 + 页内读值验证。判定 probe8 a2 便宜态的本质：
//   若墙钟也小 → 读确实没重建（队列空/键稳定，成本真省了）
//   若墙钟大而逐 op 小 → 成本落在未计时区（写/延迟失效），逐 op 计时是错觉
// 附读值验证：pending 路径应返 33px；stale snapshot 应返旧值。
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

const port = await freePort()
const proc = spawn(path.join(REPO, 'target/release/zero-browser'), ['--headless', '--remote-debugging-port', String(port)], { stdio: ['ignore', 'ignore', 'pipe'] })
if (!(await waitEndpoint(port))) { console.error('endpoint not ready'); process.exit(2) }
const http = await server()
const base = `http://127.0.0.1:${http.address().port}`
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`)
const ctx = browser.contexts()[0]
const settle = () => new Promise((r) => setTimeout(r, 300))
const out = { browser: 'zeroweb' }

// A2 形（带读值验证与总计）：60 op toggle+读。
const a2eval = async (page, tag) => {
  const t0 = Date.now()
  const r = await page.evaluate(() => {
    const cells = document.querySelectorAll('#board .cell')
    const per = []
    const widths = []
    for (let i = 0; i < 60; i++) {
      cells[3].style.width = i % 2 ? '33px' : ''
      const t = performance.now()
      const rect = cells[3].getBoundingClientRect()
      per.push(performance.now() - t)
      if (i % 2 === 0) widths.push(rect.width)
    }
    const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]
    return { med_us: med(per) * 1000, widths_first3: widths.slice(0, 3), widths_last3: widths.slice(-3) }
  })
  return { tag, wall_ms: Date.now() - t0, ...r }
}

// 序 1：probe8 序——同页 a1（300 drained 读）→ a3（写+读+清）→ a2。
{
  const page = await ctx.newPage()
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto(`${base}/style-geometry.html`, { waitUntil: 'load' })
  await settle()
  let t = Date.now()
  await page.evaluate(() => {
    const el = document.querySelector('#board .cell')
    let sink = 0
    for (let i = 0; i < 300; i++) sink += el.getBoundingClientRect().width
    return sink > 1e9 ? 0 : 0
  })
  out.seq1_a1_wall_ms = Date.now() - t
  t = Date.now()
  const a3 = await page.evaluate(() => {
    const cells = document.querySelectorAll('#board .cell')
    cells[3].style.width = '33px'
    const tc = performance.now()
    const coldRect = cells[3].getBoundingClientRect().width
    const cold_us = (performance.now() - tc) * 1000
    let sink = 0
    const t0 = performance.now()
    for (let i = 0; i < 50; i++) sink += cells[3].getBoundingClientRect().width
    const batch_us = (performance.now() - t0) * 1000 / 50
    cells[3].style.width = ''
    return { cold_us, batch_us, coldRect, warmRect: sink > 1e9 ? -1 : sink / 50 }
  })
  out.seq1_a3 = { wall_ms: Date.now() - t, ...a3 }
  out.seq1_a2 = await a2eval(page, 'after_a1_a3_same_page')
  await page.close().catch(() => {})
}

// 序 2：probe8c e1 形——新页冷，仅 a2。
{
  const page = await ctx.newPage()
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto(`${base}/style-geometry.html`, { waitUntil: 'load' })
  await settle()
  out.seq2_a2_cold = await a2eval(page, 'fresh_page_cold')
  await page.close().catch(() => {})
}

// 序 3：同页重复 a2 ×3（页内 evaluate 序数效应）。
{
  const page = await ctx.newPage()
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto(`${base}/style-geometry.html`, { waitUntil: 'load' })
  await settle()
  out.seq3_rep1 = await a2eval(page, 'rep1')
  out.seq3_rep2 = await a2eval(page, 'rep2')
  out.seq3_rep3 = await a2eval(page, 'rep3')
  await page.close().catch(() => {})
}

console.log(JSON.stringify(out))
await browser.close().catch(() => {})
proc.kill('SIGTERM')
http.close()
process.exit(0)

// P8d: 暖态武装因果隔离——probe8 A2 30µs vs probe8c e1 13.7ms 的修饰变量。
// 每臂独立新 tab，先跑不同「武装动作」再跑 A2 形（同元素 toggle+读 60 op 紧循环）：
//   X: 无武装（冷）          → 预期 ~13.7ms/op
//   Y: A1 drained 读 300     → 检验「纯读」是否武装
//   Z: A1 + A3（probe8 原序复刻）→ 预期 ~30µs（probe8 复现 run#2）
//   W: 写 1 条后立即结束 evaluate（宿主 apply 刷新快照）→ 检验「apply 循环」武装
//   V: 跑一次 A2 形后再跑一次（同文档第二次 evaluate）→ 检验「任意先前 evaluate」武装
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
  proc = spawn('google-chrome', ['--headless', `--remote-debugging-port=${port}`, `--user-data-dir=${fs.mkdtempSync('/tmp/domops-probe8d-')}`, '--no-first-run', '--disable-background-timer-throttling'], { stdio: ['ignore', 'ignore', 'pipe'] })
}
if (!(await waitEndpoint(port))) { console.error('endpoint not ready'); process.exit(2) }
const http = await server()
const base = `http://127.0.0.1:${http.address().port}`
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`)
const ctx = browser.contexts()[0]
const settle = () => new Promise((r) => setTimeout(r, 300))

// A2 形（同元素 toggle+读 60 op 紧循环）——evaluate 内自包含。
const runA2 = (page) => page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  const per = []
  for (let i = 0; i < 60; i++) {
    cells[3].style.width = i % 2 ? '33px' : ''
    const t0 = performance.now()
    cells[3].getBoundingClientRect()
    per.push(performance.now() - t0)
  }
  const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]
  return { med_us: med(per) * 1000 }
})

async function freshTab() {
  const page = await ctx.newPage()
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto(`${base}/style-geometry.html`, { waitUntil: 'load' })
  await settle()
  return page
}

const out = { browser: which }

// X: 冷。
{
  const p = await freshTab()
  out.x_cold = await runA2(p)
  await p.close().catch(() => {})
}

// Y: 纯 drained 读 300（无写）。
{
  const p = await freshTab()
  await p.evaluate(() => {
    const el = document.querySelector('#board .cell')
    let sink = 0
    for (let i = 0; i < 300; i++) sink += el.getBoundingClientRect().width
    return sink > 1e9 ? 0 : 0
  })
  out.y_after_drained_reads = await runA2(p)
  await p.close().catch(() => {})
}

// Z: probe8 原序复刻（A1 读 → A3 写+读+清 → A2 形）。
{
  const p = await freshTab()
  await p.evaluate(() => {
    const el = document.querySelector('#board .cell')
    let sink = 0
    for (let i = 0; i < 300; i++) sink += el.getBoundingClientRect().width
    return sink > 1e9 ? 0 : 0
  })
  await p.evaluate(() => {
    const cells = document.querySelectorAll('#board .cell')
    cells[3].style.width = '33px'
    cells[3].getBoundingClientRect()
    let sink = 0
    for (let i = 0; i < 50; i++) sink += cells[3].getBoundingClientRect().width
    cells[3].style.width = ''
    return sink > 1e9 ? 0 : 0
  })
  out.z_probe8_order = await runA2(p)
  await p.close().catch(() => {})
}

// W: 写 1 条（无读）后结束 evaluate → 宿主 apply 刷新快照。
{
  const p = await freshTab()
  await p.evaluate(() => {
    const cells = document.querySelectorAll('#board .cell')
    cells[3].style.width = '33px'
    return true
  })
  out.w_after_apply_cycle = await runA2(p)
  await p.close().catch(() => {})
}

// V: 同文档连续两次 A2 形。
{
  const p = await freshTab()
  out.v_first = await runA2(p)
  out.v_second = await runA2(p)
  out.v_third = await runA2(p)
  await p.close().catch(() => {})
}

console.log(JSON.stringify(out))
await browser.close().catch(() => {})
proc.kill('SIGTERM')
http.close()
process.exit(0)

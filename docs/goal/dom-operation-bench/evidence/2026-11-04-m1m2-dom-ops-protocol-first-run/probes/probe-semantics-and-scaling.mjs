// dom-ops 标定/探针 — 临时脚本，不入仓。
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
      resp.writeHead(200, {
        'content-type': 'text/html; charset=utf-8',
        'cross-origin-opener-policy': 'same-origin',
        'cross-origin-embedder-policy': 'require-corp',
      })
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
  proc = spawn(bin, ['--headless', `--remote-debugging-port=${port}`, `--user-data-dir=${fs.mkdtempSync('/tmp/domops-probe-')}`, '--no-first-run', '--disable-background-timer-throttling'], { stdio: ['ignore', 'ignore', 'pipe'] })
}
if (!(await waitEndpoint(port))) { console.error('endpoint not ready'); process.exit(2) }
const http = await server()
const base = `http://127.0.0.1:${http.address().port}`
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`)
const page = await browser.contexts()[0].newPage()
await page.setViewportSize({ width: 1280, height: 720 })

async function goto(f) {
  await page.goto(`${base}/${f}`, { waitUntil: 'load' })
  await new Promise((r) => setTimeout(r, 200))
}

// P1: thrash width — inline width 写入后几何是否反映
await goto('style-geometry.html')
const p1 = await page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  const r0 = cells[3].getBoundingClientRect().width
  cells[3].style.width = '33px'
  const stored = cells[3].style.width
  const r1 = cells[3].getBoundingClientRect().width
  const csWidth = getComputedStyle(cells[3]).width
  let offsetWidth = null
  try { offsetWidth = cells[3].offsetWidth } catch (e) { offsetWidth = 'ERR:' + e.message }
  const r0Again = cells[0].getBoundingClientRect().width
  cells[3].style.width = ''
  return { r0, stored, r1, csWidth, offsetWidth, r0Again }
})
console.log('P1 thrash-width:', JSON.stringify(p1))

// P2: replaceChildren 身份与顺序
await goto('mutation.html')
const p2 = await page.evaluate(() => {
  const box = document.getElementById('list-replace')
  box.replaceChildren()
  const pool = []
  for (let i = 0; i < 5; i++) { const d = document.createElement('div'); d.dataset.seq = String(i); pool.push(d) }
  box.replaceChildren(pool[0], pool[1], pool[2], pool[3], pool[4])
  const after = Array.from(box.children).map((c, i) => ({ ident: c === pool[i], seq: c.dataset.seq }))
  const len = box.children.length
  return { len, after }
})
console.log('P2 replace-identity:', JSON.stringify(p2))

// P3: append 成本随规模增长
await goto('mutation.html')
const p3 = await page.evaluate(() => {
  const box = document.getElementById('list-append')
  function timedAppend(n) {
    box.replaceChildren()
    const t0 = performance.now()
    for (let i = 0; i < n; i++) { const d = document.createElement('div'); box.appendChild(d) }
    return (performance.now() - t0) / n * 1000
  }
  return { n50: timedAppend(50), n200: timedAppend(200), n400: timedAppend(400), n800: timedAppend(800) }
})
console.log('P3 append-us/op by size:', JSON.stringify(p3))

// P4: remove 尾删成本随规模变化
const p4 = await page.evaluate(() => {
  const box = document.getElementById('list-remove')
  box.replaceChildren()
  for (let i = 0; i < 800; i++) box.appendChild(document.createElement('div'))
  const per = []
  for (const target of [800, 600, 400, 200]) {
    while (box.children.length > target) box.removeChild(box.lastElementChild)
    const t0 = performance.now()
    const batch = 100
    for (let i = 0; i < batch && box.children.length > 0; i++) box.removeChild(box.lastElementChild)
    per.push({ at: box.children.length, usPerOp: (performance.now() - t0) / batch * 1000 })
  }
  return per
})
console.log('P4 remove-us/op:', JSON.stringify(p4))

// P5: getElementById 唯一性（排除串容器）
const p5 = await page.evaluate(() => {
  const ids = ['list-replace', 'list-append', 'list-remove', 'list-move']
  return ids.map((id) => {
    const el = document.getElementById(id)
    const all = document.querySelectorAll('#' + id).length
    return { id, ok: !!el, count: all, children: el ? el.children.length : -1 }
  })
})
console.log('P5 get-by-id:', JSON.stringify(p5))

await browser.close().catch(() => {})
proc.kill('SIGTERM')
http.close()
process.exit(0)

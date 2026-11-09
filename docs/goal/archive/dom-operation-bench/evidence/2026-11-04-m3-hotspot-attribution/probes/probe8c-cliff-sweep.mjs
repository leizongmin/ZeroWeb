// P8c: 队列长度 cliff 扫描——每实验独立新 tab（隔离臂历史），紧循环逐 op 计时。
// 判别：每 op 成本 vs evaluate 内累计 mutation 数的曲线；cliff 位置 = 阈值；
// E5 单独检验「25ms 间隙是否武装昂贵路径」。
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
  proc = spawn('google-chrome', ['--headless', `--remote-debugging-port=${port}`, `--user-data-dir=${fs.mkdtempSync('/tmp/domops-probe8c-')}`, '--no-first-run', '--disable-background-timer-throttling'], { stdio: ['ignore', 'ignore', 'pipe'] })
}
if (!(await waitEndpoint(port))) { console.error('endpoint not ready'); process.exit(2) }
const http = await server()
const base = `http://127.0.0.1:${http.address().port}`
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`)
const ctx = browser.contexts()[0]
const settle = () => new Promise((r) => setTimeout(r, 300))

// 每实验独立 tab：goto + settle + evaluate + close。
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

// E1: probe8 A2 精确复刻（60 op 同元素 toggle+读，紧循环）——预期便宜（~30µs）。
out.e1_a2_fresh = await freshTab('style-geometry.html', () => {
  const cells = document.querySelectorAll('#board .cell')
  const per = []
  for (let i = 0; i < 60; i++) {
    cells[3].style.width = i % 2 ? '33px' : ''
    const t0 = performance.now()
    cells[3].getBoundingClientRect()
    per.push(performance.now() - t0)
  }
  return per
})

// E3: 同元素 toggle+读 300 op——队列 2/op 增长，看 cliff 位置。
out.e3_same_cell_300 = await freshTab('style-geometry.html', () => {
  const cells = document.querySelectorAll('#board .cell')
  const per = []
  for (let i = 0; i < 300; i++) {
    cells[3].style.width = i % 2 ? '33px' : ''
    const t0 = performance.now()
    cells[3].getBoundingClientRect()
    per.push(performance.now() - t0)
  }
  return per
})

// E4: 异元素写+读 300 op（基准 op 形，无 setup/warmup/gap）——队列 1/op。
out.e4_diff_cell_300 = await freshTab('style-geometry.html', () => {
  const cells = document.querySelectorAll('#board .cell')
  const N = cells.length
  const per = []
  for (let i = 0; i < 300; i++) {
    cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
    const t0 = performance.now()
    cells[(i + 1) % N].getBoundingClientRect().top
    per.push(performance.now() - t0)
  }
  return per
})

// E5: 间隙武装判别——写 1 条 → await 25ms → 60 次纯读（无新写）。
// 若纯读昂贵 → 间隙/await 使宿主做了什么（drain/重建）改变读路径。
out.e5_gap_then_reads = await freshTab('style-geometry.html', () => new Promise((resolve) => {
  const cells = document.querySelectorAll('#board .cell')
  cells[3].style.width = '33px'
  const pre = []
  const t0 = performance.now()
  cells[3].getBoundingClientRect()
  pre.push(performance.now() - t0)
  setTimeout(() => {
    const post = []
    for (let i = 0; i < 60; i++) {
      const t = performance.now()
      cells[3].getBoundingClientRect()
      post.push(performance.now() - t)
    }
    resolve({ pre_read_us: pre[0] * 1000, post_reads_us: post })
  }, 25)
}))

// E6: warmup 隔离——50 异元素写（无读）→ 60 op 同元素 toggle+读紧循环。
out.e6_warmup_then_samecell = await freshTab('style-geometry.html', () => {
  const cells = document.querySelectorAll('#board .cell')
  const N = cells.length
  for (let i = 0; i < 50; i++) cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
  const per = []
  for (let i = 0; i < 60; i++) {
    cells[3].style.width = i % 2 ? '33px' : ''
    const t0 = performance.now()
    cells[3].getBoundingClientRect()
    per.push(performance.now() - t0)
  }
  return per
})

// E2: t_no_setup 形状的 onset 剖面——warmup50 后首个样本 50 op 逐 op 计时。
out.e2_nosetup_onset = await freshTab('style-geometry.html', () => {
  const cells = document.querySelectorAll('#board .cell')
  const N = cells.length
  for (let i = 0; i < 50; i++) cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
  const per = []
  for (let i = 0; i < 50; i++) {
    cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
    const t0 = performance.now()
    cells[(i + 1) % N].getBoundingClientRect().top
    per.push(performance.now() - t0)
  }
  return per
})

console.log(JSON.stringify(out))
await browser.close().catch(() => {})
proc.kill('SIGTERM')
http.close()
process.exit(0)

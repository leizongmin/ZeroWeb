// P7: dataset.seq 在 runner 形态下的存活探针（临时，不入仓）
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

const port = await freePort()
const proc = spawn(path.join(REPO, 'target/release/zero-browser'), ['--headless', '--remote-debugging-port', String(port)], { stdio: ['ignore', 'ignore', 'pipe'] })
if (!(await waitEndpoint(port))) { console.error('endpoint not ready'); process.exit(2) }
const http = createHttpServer((req, resp) => {
  const name = new URL(req.url, 'http://x').pathname.replace(/^\/+/, '')
  const f = path.join(FIXTURES, name)
  if (!name || !fs.existsSync(f)) { resp.writeHead(404); return resp.end() }
  resp.writeHead(200, { 'content-type': 'text/html; charset=utf-8' })
  fs.createReadStream(f).pipe(resp)
})
await new Promise((r) => http.listen(0, '127.0.0.1', r))
const base = `http://127.0.0.1:${http.address().port}`
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`)
const page = await browser.contexts()[0].newPage()
await page.setViewportSize({ width: 1280, height: 720 })
await page.goto(`${base}/mutation.html`, { waitUntil: 'load' })
await new Promise((r) => setTimeout(r, 300))

const result = await page.evaluate((cfg) => {
  const log = []
  const box = document.getElementById('list-replace')
  box.replaceChildren()
  const pool = []
  for (let i = 0; i < 50; i++) {
    const d = document.createElement('div')
    d.className = 'item'
    d.dataset.seq = String(i)
    pool.push(d)
  }
  log.push({ phase: 'after-set', seq0: pool[0].dataset.seq, attr0: pool[0].getAttribute('data-seq'), keys: Object.keys(pool[0].dataset).length })
  box.replaceChildren(pool[0], pool[1], pool[2], pool[3], pool[4], pool[5], pool[6], pool[7], pool[8], pool[9], pool[10], pool[11], pool[12], pool[13], pool[14], pool[15], pool[16], pool[17], pool[18], pool[19], pool[20], pool[21], pool[22], pool[23], pool[24], pool[25], pool[26], pool[27], pool[28], pool[29], pool[30], pool[31], pool[32], pool[33], pool[34], pool[35], pool[36], pool[37], pool[38], pool[39], pool[40], pool[41], pool[42], pool[43], pool[44], pool[45], pool[46], pool[47], pool[48], pool[49])
  log.push({ phase: 'after-replace-sync', seq0: pool[0].dataset.seq, child0seq: box.children[0].dataset.seq, ident0: box.children[0] === pool[0] })
  return new Promise((resolve) => {
    let s = 0
    const step = () => {
      if (s >= 3) {
        log.push({ phase: 'after-async', seq0: pool[0].dataset.seq, child0seq: box.children[0].dataset.seq, ident0: box.children[0] === pool[0], attr0: pool[0].getAttribute('data-seq') })
        resolve({ log, cfgSeen: cfg })
        return
      }
      setTimeout(() => {
        box.replaceChildren(pool[0], pool[1], pool[2], pool[3], pool[4], pool[5], pool[6], pool[7], pool[8], pool[9], pool[10], pool[11], pool[12], pool[13], pool[14], pool[15], pool[16], pool[17], pool[18], pool[19], pool[20], pool[21], pool[22], pool[23], pool[24], pool[25], pool[26], pool[27], pool[28], pool[29], pool[30], pool[31], pool[32], pool[33], pool[34], pool[35], pool[36], pool[37], pool[38], pool[39], pool[40], pool[41], pool[42], pool[43], pool[44], pool[45], pool[46], pool[47], pool[48], pool[49])
        if (s === 0) log.push({ phase: 'in-timer-0', seq0: pool[0].dataset.seq, child0seq: box.children[0].dataset.seq })
        s++
        step()
      }, 25)
    }
    step()
  })
}, { injectFault: false })
console.log(JSON.stringify(result, null, 1))

// 判别读：fresh evaluate 重建 worker 视图 = snapshot(cached_html) + 空队列。
// 若 fresh 读到 data-seq → cached_html 完好，丢失仅在 await 循环内的 worker 视图；
// 若 fresh 读 null → renderer apply 路径丢失（cached_html 缺属性）。
const fresh = await page.evaluate(() => {
  const box = document.getElementById('list-replace')
  const c0 = box.children[0]
  return {
    childCount: box.children.length,
    child0seq: c0 ? c0.dataset.seq : 'NO_CHILD',
    child0attr: c0 ? c0.getAttribute('data-seq') : 'NO_CHILD',
    child0class: c0 ? c0.className : 'NO_CHILD',
  }
})
console.log('fresh-evaluate:', JSON.stringify(fresh))

await browser.close().catch(() => {})
proc.kill('SIGTERM')
http.close()
process.exit(0)

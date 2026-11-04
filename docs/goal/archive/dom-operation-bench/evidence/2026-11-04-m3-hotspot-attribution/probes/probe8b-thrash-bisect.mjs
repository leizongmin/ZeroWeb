// P8b: thrash / replace.pool 热点因子 bisect（临时脚本，结果入 evidence）。
// probe8 显示页内同元素写+读仅 ~30µs/op（zw），而基准 thrash 报 12.4ms/op——形状差在哪。
// 基准形状 = freshCells 清 600 cell（1200 mutation）+ 50 元素轮写 + warmup50
//           + 12 样本×50 op，样本间 setTimeout(25)（awaitPromise 泵 timer）。
// 逐臂单因子开关，每臂独立 evaluate（臂间宿主 drain，队列复位，镜像基准子场景隔离）。
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

const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]

const which = process.argv[2] || 'zeroweb'
const port = await freePort()
let proc
if (which === 'zeroweb') {
  proc = spawn(path.join(REPO, 'target/release/zero-browser'), ['--headless', '--remote-debugging-port', String(port)], { stdio: ['ignore', 'ignore', 'pipe'] })
} else {
  proc = spawn('google-chrome', ['--headless', `--remote-debugging-port=${port}`, `--user-data-dir=${fs.mkdtempSync('/tmp/domops-probe8b-')}`, '--no-first-run', '--disable-background-timer-throttling'], { stdio: ['ignore', 'ignore', 'pipe'] })
}
if (!(await waitEndpoint(port))) { console.error('endpoint not ready'); process.exit(2) }
const http = await server()
const base = `http://127.0.0.1:${http.address().port}`
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`)
const page = await browser.contexts()[0].newPage()
await page.setViewportSize({ width: 1280, height: 720 })
const settle = () => new Promise((r) => setTimeout(r, 300))

const out = { browser: which, arms: {} }
const arm = async (name, fixture, fn) => {
  await page.goto(`${base}/${fixture}`, { waitUntil: 'load' })
  await settle()
  out.arms[name] = await fn()
}

// ── style-geometry 臂 ──

// 基准镜像：setup 清 600 cell + warmup50 + 12×50 op（样本间 setTimeout 25ms）。
await arm('t_mirror', 'style-geometry.html', () => page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  const N = cells.length
  for (const el of cells) { el.style.color = ''; el.style.width = '' }
  for (let i = 0; i < 50; i++) cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
  return new Promise((resolve) => {
    const samples = []
    let s = 0
    const step = () => {
      if (s >= 6) return resolve({ samples_ms: samples, per_op: samples.map((d) => d * 1000 / 50) })
      setTimeout(() => {
        const t0 = performance.now()
        for (let i = 0; i < 50; i++) {
          cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
          cells[(i + 1) % N].getBoundingClientRect().top
        }
        samples.push(performance.now() - t0)
        s++
        step()
      }, 25)
    }
    step()
  })
}))

// 去因子·setup clear：只 warmup + 样本循环。
await arm('t_no_setup', 'style-geometry.html', () => page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  const N = cells.length
  for (let i = 0; i < 50; i++) cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
  return new Promise((resolve) => {
    const samples = []
    let s = 0
    const step = () => {
      if (s >= 6) return resolve({ samples_ms: samples, per_op: samples.map((d) => d * 1000 / 50) })
      setTimeout(() => {
        const t0 = performance.now()
        for (let i = 0; i < 50; i++) {
          cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
          cells[(i + 1) % N].getBoundingClientRect().top
        }
        samples.push(performance.now() - t0)
        s++
        step()
      }, 25)
    }
    step()
  })
}))

// 去因子·timer 间隙：setup + warmup + 12×50 紧循环（无 setTimeout）。
await arm('t_no_gap', 'style-geometry.html', () => page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  const N = cells.length
  for (const el of cells) { el.style.color = ''; el.style.width = '' }
  for (let i = 0; i < 50; i++) cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
  const samples = []
  for (let s = 0; s < 12; s++) {
    const t0 = performance.now()
    for (let i = 0; i < 50; i++) {
      cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
      cells[(i + 1) % N].getBoundingClientRect().top
    }
    samples.push(performance.now() - t0)
  }
  return { samples_ms: samples, per_op: samples.map((d) => d * 1000 / 50) }
}))

// 队列长度标定：清前 K 个 cell（2K mutation），同批紧循环 50 op 基准形读写。
await arm('t_scale', 'style-geometry.html', () => (async () => {
  const res = {}
  for (const K of [100, 300]) {
    // 每个K独立 evaluate：臂内 goto 由外层完成，这里用嵌套 evaluate 且队列随 K 累积——
    // 因此 K 递增顺序运行并记录累积事实（K=100 后队列已含 200 条）。
    res['k' + K] = await page.evaluate((K) => {
      const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]
      const cells = document.querySelectorAll('#board .cell')
      const N = cells.length
      for (let i = 0; i < K; i++) { cells[i].style.color = ''; cells[i].style.width = '' }
      for (let i = 0; i < 50; i++) cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
      const per = []
      for (let i = 0; i < 50; i++) {
        const t0 = performance.now()
        cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
        cells[(i + 1) % N].getBoundingClientRect().top
        per.push(performance.now() - t0)
      }
      return { med_us: med(per) * 1000, all5: per.slice(0, 3).map((v) => v * 1000) }
    }, K)
  }
  return res
})())

// 大队列·单脏元素：cells[3] 交替写 600 次（1200 mutation），其余干净，紧循环基准形 50 op。
await arm('t_bigqueue_samedirty', 'style-geometry.html', () => page.evaluate(() => {
  const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]
  const cells = document.querySelectorAll('#board .cell')
  const N = cells.length
  for (let j = 0; j < 600; j++) cells[3].style.width = j % 2 ? '33px' : ''
  for (let i = 0; i < 50; i++) cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
  const per = []
  for (let i = 0; i < 50; i++) {
    const t0 = performance.now()
    cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
    cells[(i + 1) % N].getBoundingClientRect().top
    per.push(performance.now() - t0)
  }
  return { med_us: med(per) * 1000 }
}))

// 便宜形状 + timer 间隙：同元素写+读（probe8 A2 形状）但带 12×50 样本与 25ms 间隙。
await arm('t_same_cell_gap', 'style-geometry.html', () => page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  return new Promise((resolve) => {
    const samples = []
    let s = 0
    const step = () => {
      if (s >= 6) return resolve({ samples_ms: samples, per_op: samples.map((d) => d * 1000 / 50) })
      setTimeout(() => {
        const t0 = performance.now()
        for (let i = 0; i < 50; i++) {
          cells[3].style.width = i % 2 ? '33px' : ''
          cells[3].getBoundingClientRect()
        }
        samples.push(performance.now() - t0)
        s++
        step()
      }, 25)
    }
    step()
  })
}))

// 前序子场景残留：先镜像 style.write.color（清 600 + 2000 色写），再跑完整镜像。
await page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  for (const el of cells) { el.style.color = ''; el.style.width = '' }
  const C = ['rgb(255, 0, 0)', 'rgb(0, 128, 0)', 'rgb(0, 0, 255)']
  for (let i = 0; i < 2000; i++) cells[i % 600].style.color = C[i % 3]
})
await settle()
out.arms.t_prior_write_then_mirror = await page.evaluate(() => {
  const cells = document.querySelectorAll('#board .cell')
  const N = cells.length
  for (const el of cells) { el.style.color = ''; el.style.width = '' }
  for (let i = 0; i < 50; i++) cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
  return new Promise((resolve) => {
    const samples = []
    let s = 0
    const step = () => {
      if (s >= 6) return resolve({ samples_ms: samples, per_op: samples.map((d) => d * 1000 / 50) })
      setTimeout(() => {
        const t0 = performance.now()
        for (let i = 0; i < 50; i++) {
          cells[i % N].style.width = 18 + (i % 5) * 2 + 'px'
          cells[(i + 1) % N].getBoundingClientRect().top
        }
        samples.push(performance.now() - t0)
        s++
        step()
      }, 25)
    }
    step()
  })
})

// ── mutation 臂 ──

// 基准镜像：setup + warmup20 + 12×20 op（样本间 setTimeout 25ms）。
await arm('r_mirror', 'mutation.html', () => page.evaluate(() => {
  const box = document.getElementById('list-replace')
  box.replaceChildren()
  const pool = []
  for (let i = 0; i < 50; i++) {
    const d = document.createElement('div')
    d.className = 'item'
    d.dataset.seq = String(i)
    pool.push(d)
  }
  const rep = () => box.replaceChildren(pool[0], pool[1], pool[2], pool[3], pool[4], pool[5], pool[6], pool[7], pool[8], pool[9], pool[10], pool[11], pool[12], pool[13], pool[14], pool[15], pool[16], pool[17], pool[18], pool[19], pool[20], pool[21], pool[22], pool[23], pool[24], pool[25], pool[26], pool[27], pool[28], pool[29], pool[30], pool[31], pool[32], pool[33], pool[34], pool[35], pool[36], pool[37], pool[38], pool[39], pool[40], pool[41], pool[42], pool[43], pool[44], pool[45], pool[46], pool[47], pool[48], pool[49])
  for (let i = 0; i < 20; i++) rep()
  return new Promise((resolve) => {
    const samples = []
    let s = 0
    const step = () => {
      if (s >= 6) return resolve({ samples_ms: samples, per_op: samples.map((d) => d * 1000 / 20) })
      setTimeout(() => {
        const t0 = performance.now()
        for (let i = 0; i < 20; i++) rep()
        samples.push(performance.now() - t0)
        s++
        step()
      }, 25)
    }
    step()
  })
}))

// 去因子·timer 间隙：setup + warmup20 + 12×20 紧循环。
await arm('r_no_gap', 'mutation.html', () => page.evaluate(() => {
  const box = document.getElementById('list-replace')
  box.replaceChildren()
  const pool = []
  for (let i = 0; i < 50; i++) {
    const d = document.createElement('div')
    d.className = 'item'
    d.dataset.seq = String(i)
    pool.push(d)
  }
  const rep = () => box.replaceChildren(pool[0], pool[1], pool[2], pool[3], pool[4], pool[5], pool[6], pool[7], pool[8], pool[9], pool[10], pool[11], pool[12], pool[13], pool[14], pool[15], pool[16], pool[17], pool[18], pool[19], pool[20], pool[21], pool[22], pool[23], pool[24], pool[25], pool[26], pool[27], pool[28], pool[29], pool[30], pool[31], pool[32], pool[33], pool[34], pool[35], pool[36], pool[37], pool[38], pool[39], pool[40], pool[41], pool[42], pool[43], pool[44], pool[45], pool[46], pool[47], pool[48], pool[49])
  for (let i = 0; i < 20; i++) rep()
  const samples = []
  for (let s = 0; s < 12; s++) {
    const t0 = performance.now()
    for (let i = 0; i < 20; i++) rep()
    samples.push(performance.now() - t0)
  }
  return { samples_ms: samples, per_op: samples.map((d) => d * 1000 / 20) }
}))

// 队列成长剖面：紧循环 240 op，首 20 / 末 20 每 op 对比（probe8 B 只测了 50 op）。
await arm('r_queue240', 'mutation.html', () => page.evaluate(() => {
  const box = document.getElementById('list-replace')
  box.replaceChildren()
  const pool = []
  for (let i = 0; i < 50; i++) {
    const d = document.createElement('div')
    d.className = 'item'
    d.dataset.seq = String(i)
    pool.push(d)
  }
  const rep = () => box.replaceChildren(pool[0], pool[1], pool[2], pool[3], pool[4], pool[5], pool[6], pool[7], pool[8], pool[9], pool[10], pool[11], pool[12], pool[13], pool[14], pool[15], pool[16], pool[17], pool[18], pool[19], pool[20], pool[21], pool[22], pool[23], pool[24], pool[25], pool[26], pool[27], pool[28], pool[29], pool[30], pool[31], pool[32], pool[33], pool[34], pool[35], pool[36], pool[37], pool[38], pool[39], pool[40], pool[41], pool[42], pool[43], pool[44], pool[45], pool[46], pool[47], pool[48], pool[49])
  const per = []
  for (let i = 0; i < 240; i++) {
    const t0 = performance.now()
    rep()
    per.push(performance.now() - t0)
  }
  const med = (a) => a.slice().sort((x, y) => x - y)[Math.floor(a.length / 2)]
  return { first20_us: med(per.slice(0, 20)) * 1000, last20_us: med(per.slice(-20)) * 1000 }
}))

// 语义 sanity（镜像臂后）：替换生效且 data-seq 完好。
out.sanity = await page.evaluate(() => {
  const box = document.getElementById('list-replace')
  return { children: box.children.length, seq0: box.children[0] ? box.children[0].dataset.seq : null }
})

console.log(JSON.stringify(out))
await browser.close().catch(() => {})
proc.kill('SIGTERM')
http.close()
process.exit(0)

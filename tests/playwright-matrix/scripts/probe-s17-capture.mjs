// S17 M5 复核：经捕获代理对 ZeroWeb 重跑全核心流 → 命令面与 Chromium 基线比对。
// 用法：node scripts/probe-s17-capture.mjs（spawn headless + 代理 + 子进程跑 capture-core-flow）
import fs from 'node:fs'
import path from 'node:path'
import { spawn } from 'node:child_process'
import net from 'node:net'
import { startCaptureProxy } from './cdp-capture-proxy.mjs'

const REPO = '../..'
const PORT = 9540
const OUT = 'out/zw-s17'

function freePort() {
  return new Promise((resolve, reject) => {
    const s = net.createServer()
    s.listen(0, '127.0.0.1', () => { const p = s.address().port; s.close(() => resolve(p)) })
    s.on('error', reject)
  })
}

const proc = spawn(`${REPO}/target/debug/zero-browser`, ['--headless', '--remote-debugging-port', String(PORT)], { stdio: ['ignore', 'ignore', 'inherit'] })
let version = null
for (let i = 0; i < 60; i++) {
  try { version = await (await fetch(`http://127.0.0.1:${PORT}/json/version`)).json(); break } catch {}
  await new Promise((r) => setTimeout(r, 250))
}
if (!version) { console.log('headless not ready'); proc.kill(); process.exit(2) }

fs.mkdirSync(OUT, { recursive: true })
const logPath = `${OUT}/cdp-traffic.jsonl`
fs.existsSync(logPath) && fs.unlinkSync(logPath)
const proxy = await startCaptureProxy({ targetHttp: `http://127.0.0.1:${PORT}`, logPath })
console.log(`proxy on ${proxy.port}`)

// 注意：必须用异步 spawn——execFileSync 会冻结本进程事件循环，而捕获代理就跑在
// 本进程里（flow 子进程的 discovery 请求将永远得不到服务 → 双向死锁，实测）。
try {
  const code = await new Promise((resolve, reject) => {
    const child = spawn('node', ['scripts/capture-core-flow.mjs'], {
      cwd: path.resolve(import.meta.dirname, '..'),
      env: { ...process.env, CDP_ENDPOINT_URL: `http://127.0.0.1:${proxy.port}`, CAPTURE_OUT: OUT },
      stdio: ['ignore', 'pipe', 'pipe'],
    })
    const timer = setTimeout(() => child.kill('SIGKILL'), 240_000)
    let out = '', errOut = ''
    child.stdout.on('data', (d) => (out += d))
    child.stderr.on('data', (d) => (errOut += d))
    child.on('exit', (code) => { clearTimeout(timer); resolve({ code, out, errOut }) })
    child.on('error', reject)
  }).then((r) => r)
  if (code.code !== 0) {
    console.log(`flow exited ${code.code}`)
    console.log('[flow stdout tail]', code.out.split('\n').slice(-4).join(' | '))
    if (code.errOut) console.log('[flow stderr]', code.errOut.slice(0, 300))
  }
} catch (err) {
  console.log('flow spawn error:', String(err?.message || err).slice(0, 200))
}

const terminate = () => new Promise((resolve) => {
  if (!proc || proc.exitCode !== null) return resolve()
  const t = setTimeout(() => { proc.kill('SIGKILL'); resolve() }, 5000)
  proc.once('exit', () => { clearTimeout(t); resolve() })
  proc.kill('SIGTERM')
})
await terminate()

// 汇总命令面
const lines = fs.readFileSync(logPath, 'utf8').trim().split('\n').map((l) => { try { return JSON.parse(l) } catch { return null } }).filter(Boolean)
const calls = new Map(), events = new Map()
for (const e of lines) {
  if (e.kind !== 'ws') continue
  let msg
  try { msg = JSON.parse(e.data) } catch { continue }
  if (msg.id !== undefined && msg.method) calls.set(msg.method, (calls.get(msg.method) || 0) + 1)
  else if (msg.id === undefined && msg.method) events.set(msg.method, (events.get(msg.method) || 0) + 1)
}
const totalCalls = [...calls.values()].reduce((a, b) => a + b, 0)
const totalEvents = [...events.values()].reduce((a, b) => a + b, 0)
const summary = {
  date: '2026-09-13',
  note: 'ZeroWeb S17 时点全核心流捕获（经捕获代理；frames×2 期望失败不影响命令面统计）',
  total_calls: totalCalls,
  unique_methods: calls.size,
  total_events: totalEvents,
  unique_events: events.size,
  methods: Object.fromEntries([...calls.entries()].sort()),
  events: Object.fromEntries([...events.entries()].sort()),
}
fs.writeFileSync(`${OUT}/zw-capture-summary.json`, JSON.stringify(summary, null, 2))
console.log(`ZeroWeb capture: ${totalCalls} calls / ${calls.size} methods; ${totalEvents} events / ${events.size} events`)
const base = JSON.parse(fs.readFileSync('../../docs/goal/cdp-protocol/evidence/chromium-capture-summary-2026-09-12.json', 'utf8'))
const baseMethods = new Set((base.methods || []).map((e) => e.method))
const baseEvents = new Set((base.browserEvents || []).map((e) => e.method))
const zwMethods = new Set(calls.keys())
const zwEvents = new Set(events.keys())
console.log('chromium baseline:', base.totals.commands, 'calls /', base.totals.uniqueMethods, 'methods;', base.totals.eventDeliveries, 'events /', base.totals.uniqueEvents, 'unique')
console.log('methods chromium-only (ZW 缺口面):', [...baseMethods].filter((m) => !zwMethods.has(m)).sort().join(', ') || '(none)')
console.log('methods zw-extra:', [...zwMethods].filter((m) => !baseMethods.has(m)).sort().join(', ') || '(none)')
console.log('events chromium-only:', [...baseEvents].filter((m) => !zwEvents.has(m)).sort().join(', ') || '(none)')
process.exit(0)

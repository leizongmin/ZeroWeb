// M5 收口预备 — cdp-e2e 确定性双跑验证 + 期望绿回归门。
//
// 流程：spawn 独立 ZeroWeb headless（每次跑全新进程，状态零残留）→ 跑全核心流 →
// 重复一次 → 对比两次步骤结果（deterministic 判定）+ 对照 expected-green.json
// 基线（回归门）→ 写 out/determinism-report.json。
//
// 退出码：0 = 双跑一致且期望绿全保；1 = 不一致或有期望绿回退（回归）；2 = 环境错误。
//
// 用法（make cdp-e2e 入口，test-guard 包裹）：
//   node scripts/verify-deterministic.mjs [--runs 2]

import fs from 'node:fs'
import net from 'node:net'
import path from 'node:path'
import { spawn, execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url))
const MATRIX_DIR = path.resolve(SCRIPT_DIR, '..')
const REPO_ROOT = path.resolve(MATRIX_DIR, '..', '..')
const ZERO_BROWSER = path.join(REPO_ROOT, 'target', 'debug', 'zero-browser')
const runsIdx = process.argv.indexOf('--runs')
const RUNS = runsIdx >= 0 ? Number(process.argv[runsIdx + 1]) || 2 : 2

const fail = (msg) => {
  console.error(msg)
  process.exit(2)
}

if (!fs.existsSync(ZERO_BROWSER)) fail(`zero-browser not built: ${ZERO_BROWSER}（先 cargo build -p zero-browser）`)
if (!fs.existsSync(path.join(MATRIX_DIR, 'node_modules'))) fail('node_modules missing（先 npm install）')

/** 找一个空闲端口（测试工具用途，竞态窗口可接受）。 */
function freePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer()
    server.listen(0, '127.0.0.1', () => {
      const port = server.address().port
      server.close(() => resolve(port))
    })
    server.on('error', reject)
  })
}

/** 等待 CDP 发现端点就绪。 */
async function waitEndpoint(port, timeoutMs = 20000) {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/json/version`)
      if (res.ok) return true
    } catch { /* 未就绪 */ }
    await new Promise((r) => setTimeout(r, 200))
  }
  return false
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

/** 单跑：spawn 全新 headless → 全核心流 → 读取步骤报告。 */
async function runOnce(index) {
  const port = await freePort()
  const proc = spawn(ZERO_BROWSER, ['--headless', '--remote-debugging-port', String(port)], {
    stdio: ['ignore', 'ignore', 'pipe'],
  })
  let stderr = ''
  proc.stderr.on('data', (d) => (stderr += d))
  try {
    if (!(await waitEndpoint(port))) fail(`run ${index}: CDP endpoint not ready\n${stderr.slice(-500)}`)
    try {
      execFileSync('node', [path.join(SCRIPT_DIR, 'capture-core-flow.mjs')], {
        cwd: MATRIX_DIR,
        env: { ...process.env, CDP_ENDPOINT_URL: `http://127.0.0.1:${port}` },
        timeout: 300_000,
        stdio: 'pipe',
      })
    } catch (err) {
      // 期望失败步骤（objectId 桥挂起项）使流程退出非零——以 steps-report 为准
      if (err.status === undefined || err.status > 2) throw err
      console.log(`  run ${index}: flow exited ${err.status}（含期望失败步骤）`)
    }
  } finally {
    await terminate(proc)
  }
  const report = JSON.parse(fs.readFileSync(path.join(MATRIX_DIR, 'out', 'steps-report.json'), 'utf8'))
  return report.steps.filter((s) => s.step !== 'observations')
}

// ── 主流程 ──

const baseline = JSON.parse(fs.readFileSync(path.join(MATRIX_DIR, 'expected-green.json'), 'utf8'))
const expectedGreen = new Set(baseline.expected_green)

const runs = []
for (let i = 1; i <= RUNS; i++) {
  console.log(`run ${i}/${RUNS} ...`)
  runs.push(await runOnce(i))
}

// deterministic 判定：步骤名集合 + 每步 ok 状态两次完全一致
const signature = (steps) =>
  JSON.stringify(
    steps
      .map((s) => `${s.step}:${s.ok ? 'ok' : 'FAIL'}`)
      .sort(),
  )
const deterministic = RUNS < 2 || signature(runs[0]) === signature(runs[1])

// 期望绿回归门：以第一次跑为准（双跑一致时即代表全部）
const okSteps = new Set(runs[0].filter((s) => s.ok).map((s) => s.step))
const regressions = [...expectedGreen].filter((step) => !okSteps.has(step))

const report = {
  date: new Date().toISOString(),
  runs: RUNS,
  deterministic,
  green_steps: [...okSteps].sort(),
  expected_green: baseline.expected_green,
  regressions,
  run_details: runs.map((steps, i) => ({
    run: i + 1,
    ok: steps.filter((s) => s.ok).map((s) => s.step),
    failed: steps.filter((s) => !s.ok).map((s) => s.step),
  })),
}
fs.mkdirSync(path.join(MATRIX_DIR, 'out'), { recursive: true })
fs.writeFileSync(
  path.join(MATRIX_DIR, 'out', 'determinism-report.json'),
  JSON.stringify(report, null, 2),
)

console.log(`deterministic: ${deterministic ? 'YES' : 'NO'}`)
console.log(`green steps (${okSteps.size}): ${[...okSteps].sort().join(', ')}`)
if (regressions.length > 0) console.log(`REGRESSIONS vs expected-green: ${regressions.join(', ')}`)

if (!deterministic) process.exit(1)
if (regressions.length > 0) process.exit(1)
console.log('cdp-e2e gate: PASS')
process.exit(0)

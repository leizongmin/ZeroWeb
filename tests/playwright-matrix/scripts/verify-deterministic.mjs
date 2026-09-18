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
      // 期望失败步骤（挂账项）使流程退出 1——以 steps-report 为准。exit 2 = 致命
      // 崩溃（如 page.new 失败，S78），绝不容忍：崩溃时 capture-core-flow 会写
      // `fatal` 报告，若照旧放行会与上一轮的陈旧 steps-report 叠加成假绿。
      // status null = 子进程被信号杀死（execFileSync 超时 SIGTERM）——此时无法
      // 区分「收尾挂起被杀且报告已完整」与「中途卡死被杀 + 读到陈旧报告」
      // （S1036/S1038 exited null 同族，S1038 修复），与 exit 2 同等不容忍。
      if (err.status === undefined || err.status === null || err.status > 1) {
        // S1075 取证增强：capture-core-flow 每步打 `  ok  <name>` 行，被杀前
        // stdout 尾部即最后进度（step 粒度心跳）；不 dump 则被 Node 默认错误
        // 打印截断（1236 字节只显 ~90），红例卡挂相位无法归因。仅落日志，
        // 判定语义不变（照旧 throw）。
        const tail = err.stdout ? err.stdout.toString().slice(-800) : '(no stdout)'
        console.log(
          `  run ${index}: killed (status ${err.status}, signal ${err.signal}), stdout tail:\n${tail}`
        )
        throw err
      }
      console.log(`  run ${index}: flow exited ${err.status}（含期望失败步骤）`)
    }
  } finally {
    await terminate(proc)
  }
  const report = JSON.parse(fs.readFileSync(path.join(MATRIX_DIR, 'out', 'steps-report.json'), 'utf8'))
  // S78：致命报告（崩溃时 capture-core-flow 兜底落盘）显式 fail，防陈旧报告假绿。
  if (report.fatal) fail(`run ${index}: flow fatal: ${report.fatal}`)
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

#!/usr/bin/env node
// to-bench-report.mjs — dom-ops 报告 → benchmark 门禁报告转换器（M4 消费接口）。
//
// 只做格式映射与入选行筛选；判定逻辑零新增（比较 = scripts/perf-gate.sh，
// measure/convert/gate 三段分离纪律与 bench-report.sh/perf-gate.sh 同款）。
//
// 入选行 = 确有价值指标（M3 归因明确 + v1/v3 两轮绝对值稳定 + 对修复面敏感）：
//   geom.write_read.thrash — M3 缺陷1 面：pending 读全量重建成本（10079x 异常哨兵）
//   geom.read.same         — drained 快路径成本（gBCR snapshot 哨兵）
//   mut.replace.pool       — M3 缺陷2 面：shim 逐节点结构/记录成本（358.8x 异常哨兵）
// 未入选行与理由：style.read.*/style.write.color（比值由 Chrome 亚 µs 分母驱动，
// 绝对值小且未归因）、geom.read.rotate（与 same 同链相关，冗余）、canvas 全行
// （29.8x 异常未归因，不入门禁防神秘失败）、timer-INCONCLUSIVE 行（计时分辨率）。
// 双浏览器都入：zeroweb = 回归目标；chrome = 同环境哨兵（环境漂移时先红 chrome 行）。
// 每 op 取正反序 p95 的较大者（保守；顺序效应由报告 order_spread 另行检出）。
// 单位统一转 ns（perf-gate 的 microbench_floor_ns 预算按 ns 语义设计，与既有
// 准则基线同单位；domops 行均为 µs 级，floor 分支不触发，纯 1.35 因子预算）。
//
// 语义断言 FAIL / 行缺失 → exit 2（不静默丢行——门禁不得在坏样本上判定）。
//
// 用法：node to-bench-report.mjs --report <dom-ops report.json> [--out <path>]
//   默认 out = tests/benchmarks/results/domops_bench_<report 生成时间>.json
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..')
const RESULTS_DIR = path.join(REPO, 'tests/benchmarks/results')

// 入选表（真实 id 见上方注释；scenario/op 与 runner PROTOCOL.sub 键一致）。
const WIRED_OPS = [
  { scenario: 'style-geometry', op: 'geom.write_read.thrash' },
  { scenario: 'style-geometry', op: 'geom.read.same' },
  { scenario: 'mutation', op: 'mut.replace.pool' },
]
const REQUIRED_BROWSERS = ['zeroweb', 'chrome']

const args = process.argv.slice(2)
let reportPath = ''
let outPath = ''
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--report') reportPath = args[++i]
  else if (args[i] === '--out') outPath = args[++i]
  else {
    console.error(`to-bench-report: 未知参数 ${args[i]}`)
    process.exit(2)
  }
}
if (!reportPath) {
  console.error('to-bench-report: FAIL — --report 必填（dom-ops report.json）')
  process.exit(2)
}

const domops = JSON.parse(fs.readFileSync(reportPath, 'utf8'))
if (domops.schema_version !== 1 || domops.kind !== 'dom-ops-report') {
  console.error(`to-bench-report: FAIL — schema 不符（schema_version=${domops.schema_version} kind=${domops.kind}），需要 dom-ops report v1`)
  process.exit(2)
}
if (!domops.config_hash) {
  console.error('to-bench-report: FAIL — 报告缺 config_hash（旧版 runner 采集），请用当前 runner 重新采集')
  process.exit(2)
}

const problems = []
const microbenches = []
for (const browser of REQUIRED_BROWSERS) {
  const run = domops.runs.find((r) => r.browser === browser)
  if (!run) {
    problems.push(`缺 ${browser} 运行（配对门禁需要双浏览器；缺测环境记 NOT_RUN 后补采）`)
    continue
  }
  for (const { scenario, op } of WIRED_OPS) {
    const key = `${scenario}/${op}`
    const sc = run.scenarios[key]
    if (!sc) {
      problems.push(`${browser}/${key}: 行缺失`)
      continue
    }
    if (sc.status !== 'PASS') {
      problems.push(`${browser}/${key}: status=${sc.status}（语义断言未全过，不入门禁）`)
      continue
    }
    const p95f = sc.forward?.per_op_us?.p95
    const p95r = sc.reverse?.per_op_us?.p95
    if (p95f == null || p95r == null) {
      problems.push(`${browser}/${key}: 分位数缺失`)
      continue
    }
    microbenches.push({
      id: `domops/${browser}/${scenario}/${op}`,
      p95: Math.max(p95f, p95r) * 1000,
      unit: 'ns',
    })
  }
}
if (problems.length > 0) {
  console.error('to-bench-report: FAIL — 入选行不可用：')
  for (const p of problems) console.error(`  - ${p}`)
  process.exit(2)
}
if (!outPath) {
  const stamp = String(domops.generated_at || 'undated').replace(/[^0-9]/g, '').slice(0, 15) || 'undated'
  outPath = path.join(RESULTS_DIR, `domops_bench_${stamp}.json`)
}
fs.mkdirSync(path.dirname(outPath), { recursive: true })
const bench = {
  schema_version: 1,
  kind: 'dom-ops-bench-report',
  source_report: path.basename(reportPath),
  generated_at: domops.generated_at,
  git_sha: domops.git_sha,
  git_dirty: domops.git_dirty,
  // dom-ops runner 暂无负载守卫（suspect 检测）——采集须在机器空闲窗口执行，
  // 限制与后续工作见 M4 evidence README。
  suspect: false,
  platform: {
    platform_class: domops.env.platform_class,
    cpu_model: domops.env.cpu_model,
    cpu_cores: domops.env.cpu_cores,
    os: String(domops.env.platform_class || '').split('-')[0],
  },
  run_config: { config_hash: domops.config_hash },
  microbenches,
  pages: [],
}
fs.writeFileSync(outPath, JSON.stringify(bench, null, 2))
console.log(`to-bench-report: OK — ${microbenches.length} 条目 → ${path.relative(REPO, outPath)}`)
for (const m of microbenches) console.log(`  ${m.id}  p95=${(m.p95 / 1000).toFixed(2)}µs`)

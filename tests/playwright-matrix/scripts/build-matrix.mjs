// 从 capture.jsonl 生成 CDP 命令矩阵资产。
//
// 输入：tests/playwright-matrix/out/capture.jsonl（cdp-capture-proxy 产出）
// 输出（CAPTURE_OUT 目录，默认 out/）：
//   chromium-capture-summary.json — 每方法调用数/成败/参数键/结果样例（机器可读）
//   chromium-capture.md           — 按域分组的人类可读明细（evidence/ 引用件）
//
// 三态登记（实现/部分/不实现 vs ZeroWeb）是人工判定件，落
// docs/goal/cdp-protocol/evidence/cdp-command-matrix.md；本脚本只产出捕获侧事实。

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url))
const OUT_DIR = process.env.CAPTURE_OUT || path.join(SCRIPT_DIR, '..', 'out')
const LOG_PATH = path.join(OUT_DIR, 'capture.jsonl')
const SAMPLE_LIMIT = 600

function main() {
  const lines = fs.readFileSync(LOG_PATH, 'utf8').split('\n').filter(Boolean)
  const methods = new Map() // method → 聚合
  const events = new Map() // 事件名 → 次数
  const discovery = []
  let malformed = 0

  const aggOf = (method) => {
    let a = methods.get(method)
    if (!a) {
      a = { count: 0, ok: 0, err: 0, errSamples: [], paramKeys: new Set(), sessionScoped: 0, resultSample: null }
      methods.set(method, a)
    }
    return a
  }

  // 响应 id → method（S2C 响应没有 method 字段，需与 C2S 请求配对）
  const idToMethod = new Map() // `${conn}:${id}` → method

  for (const line of lines) {
    let entry
    try {
      entry = JSON.parse(line)
    } catch {
      malformed += 1
      continue
    }

    if (entry.kind === 'http') {
      if (entry.event === 'discovery') discovery.push({ path: entry.path, method: entry.method })
      continue
    }
    if (entry.kind !== 'ws' || typeof entry.data !== 'string') continue

    let msg
    try {
      msg = JSON.parse(entry.data)
    } catch {
      malformed += 1
      continue
    }

    if (entry.dir === 'C2S') {
      if (typeof msg.method !== 'string') continue
      if (typeof msg.id === 'number') {
        // 命令调用
        idToMethod.set(`${entry.conn}:${msg.id}`, msg.method)
        const a = aggOf(msg.method)
        a.count += 1
        if (msg.sessionId !== undefined) a.sessionScoped += 1
        if (msg.params && typeof msg.params === 'object') {
          for (const k of Object.keys(msg.params)) a.paramKeys.add(k)
        }
      } else {
        // 客户端不太会发事件；仅计数
        events.set(`C2S:${msg.method}`, (events.get(`C2S:${msg.method}`) || 0) + 1)
      }
    } else if (entry.dir === 'S2C') {
      if (typeof msg.id === 'number') {
        const method = idToMethod.get(`${entry.conn}:${msg.id}`) || '(unmatched)'
        const a = aggOf(method)
        if (msg.error) {
          a.err += 1
          if (a.errSamples.length < 3) {
            a.errSamples.push({ code: msg.error.code, message: String(msg.error.message).slice(0, 200) })
          }
        } else {
          a.ok += 1
          if (a.resultSample === null) {
            a.resultSample = truncate(JSON.stringify(msg.result ?? null))
          }
        }
      } else if (typeof msg.method === 'string') {
        // 浏览器事件
        events.set(msg.method, (events.get(msg.method) || 0) + 1)
      }
    }
  }

  // ── 汇总结构 ──
  const methodList = [...methods.entries()]
    .map(([method, a]) => ({
      method,
      domain: method.split('.')[0],
      count: a.count,
      ok: a.ok,
      err: a.err,
      errSamples: a.errSamples,
      paramKeys: [...a.paramKeys].sort(),
      sessionScoped: a.sessionScoped > 0,
      resultSample: a.resultSample,
    }))
    .sort((x, y) => x.method.localeCompare(y.method))

  const eventList = [...events.entries()]
    .map(([method, count]) => ({ method, count }))
    .sort((x, y) => x.method.localeCompare(y.method))

  const summary = {
    capture: { source: 'capture.jsonl', lines: lines.length, malformed },
    totals: {
      commands: methodList.reduce((n, m) => n + m.count, 0),
      uniqueMethods: methodList.length,
      eventDeliveries: eventList.filter((e) => !e.method.startsWith('C2S:')).reduce((n, e) => n + e.count, 0),
      uniqueEvents: eventList.filter((e) => !e.method.startsWith('C2S:')).length,
    },
    discoveryEndpoints: [...new Set(discovery.map((d) => `${d.method} ${d.path}`))],
    methods: methodList,
    browserEvents: eventList.filter((e) => !e.method.startsWith('C2S:')),
  }
  fs.writeFileSync(path.join(OUT_DIR, 'chromium-capture-summary.json'), JSON.stringify(summary, null, 2))
  fs.writeFileSync(path.join(OUT_DIR, 'chromium-capture.md'), renderMarkdown(summary))
  console.log(`methods: ${summary.totals.uniqueMethods}, events: ${summary.totals.uniqueEvents}`)
  console.log(`→ ${OUT_DIR}/chromium-capture-summary.json`)
  console.log(`→ ${OUT_DIR}/chromium-capture.md`)
}

function truncate(s) {
  if (s === null || s.length <= SAMPLE_LIMIT) return s
  return s.slice(0, SAMPLE_LIMIT) + '…'
}

function renderMarkdown(summary) {
  const byDomain = new Map()
  for (const m of summary.methods) {
    if (!byDomain.has(m.domain)) byDomain.set(m.domain, [])
    byDomain.get(m.domain).push(m)
  }

  const out = []
  out.push('# Chromium CDP 捕获明细（生成物，勿手改）')
  out.push('')
  out.push(`- 源：\`tests/playwright-matrix/out/capture.jsonl\`（playwright-core 1.63.0 经捕获代理 connectOverCDP）`)
  out.push(`- 汇总：${summary.totals.commands} 次调用 / ${summary.totals.uniqueMethods} 个唯一方法；` +
    `${summary.totals.eventDeliveries} 次事件 / ${summary.totals.uniqueEvents} 个唯一事件`)
  out.push(`- HTTP 发现端点：${summary.discoveryEndpoints.map((d) => `\`${d}\``).join('、') || '（无）'}`)
  out.push('')

  for (const domain of [...byDomain.keys()].sort()) {
    out.push(`## ${domain}`)
    out.push('')
    out.push('| 方法 | 调用 | ok | err | sessionId | 参数键 | 结果样例（截断） | 错误样例 |')
    out.push('|------|------|----|-----|-----------|--------|------------------|----------|')
    for (const m of byDomain.get(domain)) {
      const errs = m.errSamples.map((e) => `\`${e.code}\` ${e.message}`).join('<br>') || ''
      out.push(`| \`${m.method}\` | ${m.count} | ${m.ok} | ${m.err} | ${m.sessionScoped ? 'Y' : ''} | ${m.paramKeys.map((k) => `\`${k}\``).join(' ')} | \`${m.resultSample ?? ''}\` | ${errs} |`)
    }
    out.push('')
  }

  const evs = summary.browserEvents
  if (evs.length > 0) {
    out.push('## 浏览器事件（S2C，无 id）')
    out.push('')
    out.push('| 事件 | 次数 |')
    out.push('|------|------|')
    for (const e of evs) out.push(`| \`${e.method}\` | ${e.count} |`)
    out.push('')
  }

  return out.join('\n')
}

main()

// CDP 捕获代理 — 记录 Playwright 客户端与浏览器之间的全部 CDP 流量。
//
// 职责：
// 1. HTTP 发现端点透传（/json/version、/json、/json/list）→ 记入 JSONL
// 2. WebSocket 升级与双向消息透传 → 逐条记入 JSONL
//
// 纯观测组件：不修改任何报文，仅旁路记录。目标浏览器由 targetHttp 指定
// （如 http://127.0.0.1:9333），消息逐条追加到 logPath（JSONL）。
//
// 用法：
//   const proxy = await startCaptureProxy({ targetHttp, logPath })
//   // proxy.port — 实际监听端口；Playwright connectOverCDP(`http://127.0.0.1:${proxy.port}`)
//   await proxy.close()

import http from 'node:http'
import fs from 'node:fs'
import { WebSocketServer, WebSocket } from 'ws'

const MAX_HTTP_BODY_LOG = 64 * 1024

/**
 * @param {object} opts
 * @param {string} opts.targetHttp 目标浏览器 HTTP 发现基址
 * @param {string} opts.logPath JSONL 输出路径
 * @param {string} [opts.host] 默认 127.0.0.1
 * @param {number} [opts.port] 默认 0（随机端口）
 */
export async function startCaptureProxy({ targetHttp, logPath, host = '127.0.0.1', port = 0 }) {
  const logStream = fs.createWriteStream(logPath, { flags: 'a' })
  let connSeq = 0

  const record = (entry) => {
    logStream.write(JSON.stringify(entry) + '\n')
  }

  // 上游 webSocketDebuggerUrl（从 /json/version 解析一次；调用前目标必须已就绪）
  const upstreamWsUrl = await resolveUpstreamWsUrl(targetHttp)

  const server = http.createServer((req, res) => {
    handleHttpDiscovery(req, res, targetHttp, record, () => `${host}:${actualPortRef.port}`)
  })

  const wss = new WebSocketServer({ noServer: true })
  server.on('upgrade', (req, socket, head) => {
    const connId = ++connSeq
    record({ ts: Date.now(), kind: 'ws', dir: 'META', conn: connId, event: 'ws-upgrade', path: req.url })

    wss.handleUpgrade(req, socket, head, (downstream) => {
      // 上游连接；open 前 C2S 消息先排队（CDP 客户端可能立刻发命令）
      const upstream = new WebSocket(upstreamWsUrl, { perMessageDeflate: false })
      const pending = []
      let upstreamReady = false

      downstream.on('message', (data) => {
        const raw = data.toString('utf8')
        record({ ts: Date.now(), kind: 'ws', dir: 'C2S', conn: connId, data: raw })
        if (upstreamReady) upstream.send(raw)
        else pending.push(raw)
      })
      downstream.on('close', () => upstream.close())

      upstream.on('open', () => {
        upstreamReady = true
        for (const msg of pending.splice(0)) upstream.send(msg)
      })
      upstream.on('message', (data) => {
        const raw = data.toString('utf8')
        record({ ts: Date.now(), kind: 'ws', dir: 'S2C', conn: connId, data: raw })
        if (downstream.readyState === downstream.OPEN) downstream.send(raw)
      })
      upstream.on('error', (err) => {
        record({ ts: Date.now(), kind: 'ws', dir: 'META', conn: connId, event: 'upstream-error', error: String(err) })
        downstream.close()
      })
      upstream.on('close', () => downstream.close())
    })
  })

  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(port, host, resolve)
  })

  const actualPortRef = { port: server.address().port }

  return {
    port: actualPortRef.port,
    endpoint: `http://${host}:${actualPortRef.port}`,
    close: async () => {
      for (const client of wss.clients) client.terminate()
      await new Promise((resolve) => server.close(resolve))
      logStream.end()
    },
  }
}

/** 从目标 /json/version 解析 webSocketDebuggerUrl（host:port 改写为目标直连地址）。 */
async function resolveUpstreamWsUrl(targetHttp) {
  const res = await fetch(`${targetHttp}/json/version`)
  if (!res.ok) throw new Error(`/json/version failed: ${res.status}`)
  const info = await res.json()
  const wsUrl = new URL(info.webSocketDebuggerUrl)
  const base = new URL(targetHttp)
  return `ws://${base.hostname}:${base.port}${wsUrl.pathname}`
}

/** HTTP 发现透传：请求 → 上游，记录请求行与响应体（截断）。
 *
 * 关键改写：发现响应里的 webSocketDebuggerUrl 指向浏览器直连地址 — 客户端
 * （connectOverCDP）会按响应体里的地址建 WS 连接。代理必须把它改写成自身
 * 地址（保留路径）才能让全部 WS 流量经过本代理被记录。
 */
function handleHttpDiscovery(req, res, targetHttp, record, proxyAddr) {
  const url = new URL(req.url, targetHttp)
  const upstreamReq = http.request(
    { hostname: url.hostname, port: url.port, path: url.pathname + url.search, method: req.method },
    (upstreamRes) => {
      const chunks = []
      upstreamRes.on('data', (c) => chunks.push(c))
      upstreamRes.on('end', () => {
        let body = Buffer.concat(chunks)
        if (upstreamRes.statusCode === 200 && body.includes('webSocketDebuggerUrl')) {
          const rewritten = body
            .toString('utf8')
            .replace(/(ws:\/\/)[^/\s"]+/g, `$1${proxyAddr()}`)
          body = Buffer.from(rewritten, 'utf8')
        }
        record({
          ts: Date.now(), kind: 'http', dir: 'C2S', event: 'discovery',
          method: req.method, path: req.url,
        })
        record({
          ts: Date.now(), kind: 'http', dir: 'S2C', event: 'discovery-response',
          path: req.url, status: upstreamRes.statusCode,
          body: body.length > MAX_HTTP_BODY_LOG
            ? body.subarray(0, MAX_HTTP_BODY_LOG).toString('utf8')
            : body.toString('utf8'),
        })
        res.writeHead(upstreamRes.statusCode || 502, {
          'content-type': upstreamRes.headers['content-type'] || 'application/json',
        })
        res.end(body)
      })
    },
  )
  upstreamReq.on('error', (err) => {
    record({ ts: Date.now(), kind: 'http', dir: 'META', event: 'upstream-error', path: req.url, error: String(err) })
    res.writeHead(502)
    res.end('proxy upstream error')
  })
  upstreamReq.end()
}

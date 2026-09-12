// Playwright (pin) 全核心流空跑 — CDP 命令全集捕获。
//
// 流程：启动 Chromium（--remote-debugging-port）→ 经捕获代理 connectOverCDP →
// 驱动 Playwright 核心流（navigate/evaluate/click/fill/keyboard/screenshot/cookies/
// console/network/dialog/frames/emulation/viewport/多页生命周期）→ 产出：
//   out/capture.jsonl    — 代理记录的全部 CDP 流量
//   out/steps-report.json — 逐步执行结果（ok/error），后续指向 ZeroWeb 时即差距清单
//
// 环境变量：
//   CDP_ENDPOINT_URL   直连既有 CDP 端点（跳过 Chromium 启动与代理；捕获依赖代理，直连时不记录流量）
//   ZERO_WEB_CDP_BROWSER 覆盖浏览器可执行文件（默认用 playwright-core pin 的 Chromium）
//   CAPTURE_OUT        输出目录（默认 <本目录>/../out）

import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { createServer as createHttpServer } from 'node:http'
import { chromium } from 'playwright-core'
import { startCaptureProxy } from './cdp-capture-proxy.mjs'

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url))
const OUT_DIR = process.env.CAPTURE_OUT || path.join(SCRIPT_DIR, '..', 'out')
const PLAYWRIGHT_DEFAULT_TIMEOUT_MS = 10_000

// ── 步骤记录 ──

const steps = []
async function step(name, fn) {
  const t0 = Date.now()
  try {
    const result = await fn()
    steps.push({ step: name, ok: true, ms: Date.now() - t0 })
    console.log(`  ok  ${name} (${Date.now() - t0}ms)`)
    return result
  } catch (err) {
    steps.push({ step: name, ok: false, error: String(err?.message || err).slice(0, 500), ms: Date.now() - t0 })
    console.log(`  FAIL ${name}: ${String(err?.message || err).slice(0, 300)}`)
    return undefined
  }
}

// ── 本地内容服务器（network/cookies/frames 需要 http://）──

const TINY_PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
  'base64',
)

const MAIN_PAGE = `<!DOCTYPE html>
<html><head><title>matrix-main</title></head>
<body>
  <h1 id="h">core-flow</h1>
  <input id="input" type="text" />
  <button id="btn">click-me</button>
  <button id="btn-alert">alert</button>
  <button id="btn-confirm">confirm</button>
  <button id="btn-prompt">prompt</button>
  <button id="btn-fetch">fetch</button>
  <button id="btn-xhr">xhr</button>
  <button id="btn-404">not-found</button>
  <button id="btn-redirect">redirect</button>
  <div id="clicked-log"></div>
  <iframe id="frame" src="/frame.html"></iframe>
  <script>
    console.log('boot', { a: 1 }, [1, 2]);
    console.warn('warn-msg');
    console.error('error-msg');
    let clicks = 0;
    document.getElementById('btn').addEventListener('click', () => {
      clicks += 1;
      console.log('clicked', { n: clicks });
      const d = document.createElement('div');
      d.id = 'clicked-' + clicks;
      d.textContent = 'clicked-' + clicks;
      document.getElementById('clicked-log').appendChild(d);
    });
    document.getElementById('btn-alert').addEventListener('click', () => alert('alert-text'));
    document.getElementById('btn-confirm').addEventListener('click', () =>
      console.log('confirm-result', window.confirm('confirm-text')));
    document.getElementById('btn-prompt').addEventListener('click', () =>
      console.log('prompt-result', window.prompt('prompt-text', 'default')));
    document.getElementById('btn-fetch').addEventListener('click', () =>
      fetch('/api/data').then(r => r.json()).then(j => console.log('fetch-ok', j)));
    document.getElementById('btn-xhr').addEventListener('click', () => {
      const x = new XMLHttpRequest();
      x.open('GET', '/api/data');
      x.onload = () => console.log('xhr-ok', x.responseText);
      x.send();
    });
    document.getElementById('btn-404').addEventListener('click', () =>
      fetch('/missing.json').catch(() => {}));
    document.getElementById('btn-redirect').addEventListener('click', () =>
      fetch('/redirect').then(r => r.json()).then(j => console.log('redirect-ok', j)));
  </script>
</body></html>`

const FRAME_PAGE = `<!DOCTYPE html>
<html><head><title>matrix-frame</title></head>
<body>
  <button id="frame-btn">frame-click</button>
  <script>
    console.log('frame-boot');
    document.getElementById('frame-btn').addEventListener('click', () =>
      parent.postMessage('frame-clicked', '*'));
  </script>
</body></html>`

function startContentServer() {
  return new Promise((resolve) => {
    const server = createHttpServer((req, res) => {
      const p = new URL(req.url, 'http://x').pathname
      const reply = (code, body, type = 'text/html; charset=utf-8') => {
        res.writeHead(code, { 'content-type': type })
        res.end(body)
      }
      if (p === '/') reply(200, MAIN_PAGE)
      else if (p === '/frame.html') reply(200, FRAME_PAGE)
      else if (p === '/api/data') reply(200, JSON.stringify({ ok: true, n: 1 }), 'application/json')
      else if (p === '/img.png') reply(200, '', 'image/png')
      else if (p === '/redirect') { res.writeHead(302, { location: '/api/data' }); res.end() }
      else reply(404, 'not found', 'text/plain')
    })
    server.listen(0, '127.0.0.1', () => resolve({ server, port: server.address().port }))
  })
}

// ── Chromium 启动 ──

function resolveBrowserExecutable() {
  if (process.env.ZERO_WEB_CDP_BROWSER) return process.env.ZERO_WEB_CDP_BROWSER
  try {
    const p = chromium.executablePath()
    if (p && fs.existsSync(p)) return p
  } catch { /* 回退系统浏览器 */ }
  for (const name of ['google-chrome', 'chromium', 'chromium-browser']) {
    // 粗查 PATH（capture 仅测试用，不值得引入 which 依赖）
    try {
      fs.accessSync(`/usr/bin/${name}`)
      return `/usr/bin/${name}`
    } catch { /* 下一个 */ }
  }
  throw new Error('no browser executable found')
}

function launchChromium() {
  const executablePath = resolveBrowserExecutable()
  const userDataDir = fs.mkdtempSync(path.join(os.tmpdir(), 'zw-cdp-matrix-'))
  const args = [
    '--headless=new',
    '--no-sandbox',
    '--disable-gpu',
    '--disable-dev-shm-usage',
    `--user-data-dir=${userDataDir}`,
    '--remote-debugging-port=0',
    'about:blank',
  ]
  const proc = spawn(executablePath, args, { stdio: ['ignore', 'pipe', 'pipe'] })
  const devToolsPortFile = path.join(userDataDir, 'DevToolsActivePort')
  const t0 = Date.now()
  return new Promise((resolve, reject) => {
    const poll = setInterval(() => {
      if (fs.existsSync(devToolsPortFile)) {
        clearInterval(poll)
        const [port] = fs.readFileSync(devToolsPortFile, 'utf8').split('\n')
        resolve({ proc, port: Number(port), userDataDir, executablePath })
      } else if (Date.now() - t0 > 20_000) {
        clearInterval(poll)
        reject(new Error('chromium devtools endpoint timeout (20s)'))
      }
    }, 100)
    proc.on('exit', (code) => { clearInterval(poll); reject(new Error(`chromium exited early: ${code}`)) })
  })
}

/** SIGTERM → 等待退出（上限 5s）→ 兜底 SIGKILL。 */
function terminateChromium(proc) {
  return new Promise((resolve) => {
    if (proc.exitCode !== null) { resolve(); return }
    const timer = setTimeout(() => { proc.kill('SIGKILL') }, 5000)
    proc.once('exit', () => { clearTimeout(timer); resolve() })
    proc.kill('SIGTERM')
  })
}

// ── 核心流 ──

async function runCoreFlow(browser, baseUrl) {
  const context = browser.contexts()[0]
  if (!context) throw new Error('no default context over CDP')
  context.setDefaultTimeout(PLAYWRIGHT_DEFAULT_TIMEOUT_MS)

  const consoleMsgs = []
  const requests = []
  const responses = []
  const finished = []
  const failed = []

  await step('context.default', async () => {
    if (!context.pages) throw new Error('default context missing')
  })

  const page = await step('page.new', () => context.newPage())
  if (!page) throw new Error('newPage failed — cannot continue flow')
  page.setDefaultTimeout(PLAYWRIGHT_DEFAULT_TIMEOUT_MS)

  await step('page.setViewportSize', () => page.setViewportSize({ width: 800, height: 600 }))
  await step('page.goto', async () => {
    const resp = await page.goto(`${baseUrl}/`, { waitUntil: 'load' })
    if (resp && resp.status() !== 200) throw new Error(`goto status ${resp.status()}`)
  })
  await step('page.title', async () => {
    const t = await page.title()
    if (t !== 'matrix-main') throw new Error(`unexpected title: ${t}`)
  })

  await step('console.collect', async () => {
    page.on('console', (msg) => consoleMsgs.push({ type: msg.type(), text: msg.text(), args: msg.args().length }))
    // 主文档已 boot 打过日志；重新触发一次可验证事件流
    await page.evaluate(() => console.log('late-log', 42))
    await page.waitForTimeout(200)
    if (consoleMsgs.length === 0) throw new Error('no console messages captured')
  })

  await step('evaluate.literal', async () => {
    const v = await page.evaluate('1 + 1')
    if (v !== 2) throw new Error(`expected 2, got ${JSON.stringify(v)}`)
  })
  await step('evaluate.function', async () => {
    const v = await page.evaluate(() => 41 + 1)
    if (v !== 42) throw new Error(`expected 42, got ${JSON.stringify(v)}`)
  })
  await step('evaluate.withArgs', async () => {
    const v = await page.evaluate(([a, b]) => a + b, [20, 22])
    if (v !== 42) throw new Error(`expected 42, got ${JSON.stringify(v)}`)
  })
  await step('evaluate.object', async () => {
    const v = await page.evaluate(() => ({ s: 'x', n: 1, arr: [1, 2], nested: { deep: true } }))
    if (v?.nested?.deep !== true) throw new Error(`unexpected object: ${JSON.stringify(v)}`)
  })
  await step('evaluate.async', async () => {
    const v = await page.evaluate(async () => {
      await new Promise((r) => setTimeout(r, 10))
      return 'async-done'
    })
    if (v !== 'async-done') throw new Error(`unexpected: ${JSON.stringify(v)}`)
  })

  await step('fill.input', async () => {
    await page.locator('#input').fill('hello world')
    const v = await page.locator('#input').inputValue()
    if (v !== 'hello world') throw new Error(`unexpected value: ${v}`)
  })

  await step('keyboard.type+press', async () => {
    await page.locator('#input').fill('')
    await page.keyboard.type('abc')
    await page.keyboard.press('Control+a')
    await page.keyboard.press('Enter')
    const v = await page.locator('#input').inputValue()
    if (!['abc', ''].includes(v)) throw new Error(`unexpected value: ${v}`)
  })

  await step('click.button', async () => {
    await page.locator('#btn').click()
    await page.waitForSelector('#clicked-1', { state: 'attached' })
  })
  await step('click.dblclick', () => page.locator('#h').dblclick())
  await step('click.withPosition', () => page.locator('#btn').click({ position: { x: 5, y: 5 } }))

  await step('locator.boundingBox', async () => {
    const box = await page.locator('#btn').boundingBox()
    if (!box || box.width <= 0) throw new Error(`unexpected box: ${JSON.stringify(box)}`)
  })

  await step('network.events', async () => {
    page.on('request', (r) => requests.push(r.url()))
    page.on('response', (r) => responses.push({ url: r.url(), status: r.status() }))
    page.on('requestfinished', (r) => finished.push(r.url()))
    page.on('requestfailed', (r) => failed.push(r.url()))
    await page.locator('#btn-fetch').click()
    await page.locator('#btn-xhr').click()
    await page.locator('#btn-404').click()
    await page.locator('#btn-redirect').click()
    await page.waitForTimeout(500)
    if (requests.length < 4) throw new Error(`only ${requests.length} requests observed: ${requests.join(',')}`)
  })

  await step('cookies.roundtrip', async () => {
    await context.addCookies([{ name: 'zw', value: '1', url: baseUrl }])
    const cookies = await context.cookies(`${baseUrl}/`)
    if (!cookies.some((c) => c.name === 'zw')) throw new Error(`cookie missing: ${JSON.stringify(cookies)}`)
    await context.clearCookies()
    const after = await context.cookies()
    if (after.length !== 0) throw new Error(`clearCookies left ${after.length}`)
  })

  await step('dialog.accept', async () => {
    page.once('dialog', (d) => d.accept())
    await page.locator('#btn-alert').click()
    await page.waitForTimeout(200)
  })
  await step('dialog.confirm+prompt', async () => {
    page.once('dialog', (d) => d.accept())
    await page.locator('#btn-confirm').click()
    page.once('dialog', (d) => d.accept('prompt-answer'))
    await page.locator('#btn-prompt').click()
    await page.waitForTimeout(300)
  })

  await step('frames.access', async () => {
    const frames = page.frames()
    if (frames.length < 2) throw new Error(`expected iframe frame, got ${frames.length}`)
  })
  await step('frames.click+evaluate', async () => {
    await page.frameLocator('#frame').locator('#frame-btn').click()
    const v = await page.frames()[1].evaluate('6 * 7')
    if (v !== 42) throw new Error(`frame evaluate: ${JSON.stringify(v)}`)
  })

  await step('emulation.media', async () => {
    await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' })
    const dark = await page.evaluate('matchMedia("(prefers-color-scheme: dark)").matches')
    if (dark !== true) throw new Error(`colorScheme not applied: ${dark}`)
  })

  await step('screenshot.viewport', async () => {
    const buf = await page.screenshot()
    if (buf.length < 1000) throw new Error(`suspiciously small screenshot: ${buf.length}`)
  })
  await step('screenshot.fullPage', () => page.screenshot({ fullPage: true }))
  await step('screenshot.element', () => page.locator('#btn').screenshot())

  await step('page.setContent', async () => {
    await page.setContent('<h1 id="hs">setContent-ok</h1>')
    const t = await page.locator('#hs').textContent()
    if (t !== 'setContent-ok') throw new Error(`unexpected: ${t}`)
  })

  await step('viewport.verified', async () => {
    await page.setViewportSize({ width: 500, height: 400 })
    const [w, h] = await page.evaluate('[window.innerWidth, window.innerHeight]')
    if (w !== 500 || h !== 400) throw new Error(`viewport not applied: ${w}x${h}`)
  })

  await step('page.second.lifecycle', async () => {
    const p2 = await context.newPage()
    await p2.goto(`${baseUrl}/frame.html`)
    const t = await p2.title()
    if (t !== 'matrix-frame') throw new Error(`unexpected title: ${t}`)
    await p2.close()
  })

  // 观测数据收编进步骤报告（不作为判定项）
  steps.push({
    step: 'observations', ok: true,
    consoleCount: consoleMsgs.length,
    consoleTypes: consoleMsgs.map((m) => m.type),
    requestCount: requests.length,
    responseCount: responses.length,
    finishedCount: finished.length,
    failedCount: failed.length,
  })
}

// ── 主流程 ──

async function main() {
  fs.mkdirSync(OUT_DIR, { recursive: true })
  const logPath = path.join(OUT_DIR, 'capture.jsonl')
  fs.rmSync(logPath, { force: true })

  const { server: contentServer, port: contentPort } = await startContentServer()
  const baseUrl = `http://127.0.0.1:${contentPort}`

  let chromiumProc = null
  let userDataDir = null
  let proxy = null
  let connectUrl = process.env.CDP_ENDPOINT_URL
  let browserInfo = { endpoint: connectUrl || 'launched' }

  try {
    if (!connectUrl) {
      const launched = await launchChromium()
      chromiumProc = launched.proc
      userDataDir = launched.userDataDir
      browserInfo = { executablePath: launched.executablePath, cdpPort: launched.port }
      proxy = await startCaptureProxy({ targetHttp: `http://127.0.0.1:${launched.port}`, logPath })
      connectUrl = proxy.endpoint
      console.log(`chromium: ${launched.executablePath} (cdp :${launched.port}, proxy :${proxy.port})`)
    } else {
      console.log(`connecting to existing endpoint: ${connectUrl} (无代理，不记录流量)`)
    }

    const browser = await chromium.connectOverCDP(connectUrl)
    try {
      browserInfo.browserVersion = browser.version()
      console.log(`connected: ${browserInfo.browserVersion}`)
      await runCoreFlow(browser, baseUrl)
    } finally {
      await browser.close().catch((e) => console.log(`browser.close: ${e.message}`))
    }
  } finally {
    if (proxy) await proxy.close()
    contentServer.close()
    if (chromiumProc) await terminateChromium(chromiumProc)
    if (userDataDir) {
      try {
        fs.rmSync(userDataDir, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 })
      } catch (e) {
        console.log(`cleanup: profile dir left at ${userDataDir} (${e.message})`)
      }
    }
  }

  const report = { browser: browserInfo, playwright: '1.63.0', baseUrl, steps }
  fs.writeFileSync(path.join(OUT_DIR, 'steps-report.json'), JSON.stringify(report, null, 2))
  const okCount = steps.filter((s) => s.step !== 'observations' && s.ok).length
  const failCount = steps.filter((s) => s.step !== 'observations' && !s.ok).length
  console.log(`\ndone: ${okCount} ok, ${failCount} failed → ${OUT_DIR}`)
  process.exit(failCount === 0 ? 0 : 1)
}

main().catch((err) => {
  console.error('capture failed:', err)
  process.exit(2)
})

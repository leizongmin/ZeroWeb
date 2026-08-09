#!/usr/bin/env node
// 产品静态页 chromium Oracle 截图（DC-13 product-smoke 配套）。
//
// 对任意产品 HTML（如 morning-work/article.html）用 headless chromium 渲染并截图，
// 产出 PNG 作为 product-smoke 的独立 chromium oracle。与 chromium-oracle-shot.mjs
// 同 HTTP-server 模式（R388：http:// 取代 file://，使 @font-face 本地字体与相对路径
// 资源正确解析），但面向产品 fixture 而非 WPT reftest。
//
// 用法：
//   node product-oracle-shot.mjs --root <dir> --html <relpath> --out <png> \
//       [--width 800 --height 600 --selector <css> --wait 300]
//
//   --root      静态 server 根目录（HTML 的相对资源从此解析）。
//   --html      相对 root 的 HTML 路径（如 morning-work/article.html）。
//   --out       输出 PNG 路径。
//   --width/--height  视口尺寸（默认 800×600，与 product-smoke 一致）。
//   --selector  可选：只截该元素（默认全视口）。
//   --wait      networkidle0 后额外等待 ms（默认 300，等字体/webfont 解码）。
//
// 依赖：puppeteer-core（tests/wpt-runner/scripts/package.json）+ 系统 chromium（/usr/bin/chromium）。
// 外部资源（ads/disqus CDN 等）任其超时失败 —— 仅本地静态内容入 oracle。
import { readFile, mkdir } from 'node:fs/promises';
import { createReadStream, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { join, dirname, extname, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import puppeteer from 'puppeteer-core';

const HERE = dirname(fileURLToPath(import.meta.url));
const CHROMIUM = process.env.PUPPETEER_EXECUTABLE_PATH
  || (process.platform === 'darwin'
    ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
    : '/usr/bin/chromium');

const MIME = {
  '.html': 'text/html; charset=utf-8', '.htm': 'text/html; charset=utf-8',
  '.xht': 'application/xhtml+xml; charset=utf-8', '.xhtml': 'application/xhtml+xml; charset=utf-8',
  '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.ttf': 'font/ttf', '.otf': 'font/otf', '.woff': 'font/woff', '.woff2': 'font/woff2',
  '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
};

function parseArgs() {
  const a = process.argv.slice(2);
  const o = { root: null, html: null, out: null, width: 800, height: 600, selector: null, wait: 300 };
  for (let i = 0; i < a.length; i++) {
    if (a[i] === '--root') o.root = a[++i];
    else if (a[i] === '--html') o.html = a[++i];
    else if (a[i] === '--out') o.out = a[++i];
    else if (a[i] === '--width') o.width = parseInt(a[++i], 10);
    else if (a[i] === '--height') o.height = parseInt(a[++i], 10);
    else if (a[i] === '--selector') o.selector = a[++i];
    else if (a[i] === '--wait') o.wait = parseInt(a[++i], 10);
    else if (a[i] === '--help') { console.log('See header comment.'); process.exit(0); }
  }
  if (!o.root || !o.html || !o.out) {
    console.error('Usage: product-oracle-shot.mjs --root <dir> --html <rel> --out <png> [--width 800 --height 600 --selector <css> --wait 300]');
    process.exit(2);
  }
  return o;
}

async function statSafe(p) {
  try { const s = statSync(p); return [true, s.isFile()]; } catch { return [false, false]; }
}

async function startStaticServer(rootDir) {
  const root = resolve(rootDir);
  const server = createServer((req, res) => {
    const decoded = decodeURIComponent((req.url || '/').split('?')[0]);
    const filePath = normalize(join(root, decoded));
    if (!filePath.startsWith(root)) { res.writeHead(403); res.end('403'); return; }
    statSafe(filePath).then(([ok, isFile]) => {
      if (!ok || !isFile) { res.writeHead(404); res.end('404'); return; }
      res.writeHead(200, { 'Content-Type': MIME[extname(filePath).toLowerCase()] || 'application/octet-stream' });
      createReadStream(filePath).pipe(res);
    }).catch(() => { res.writeHead(404); res.end('404'); });
  });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const { port } = server.address();
  return { url: `http://127.0.0.1:${port}`, close: () => new Promise((r) => server.close(r)) };
}

async function main() {
  const opts = parseArgs();
  const server = await startStaticServer(opts.root);
  await mkdir(dirname(opts.out), { recursive: true });
  // chromium 默认 font-render-hinting（与既有 welcome/WPT oracle 一致——R1069 证
  // full-hinting 匹配）。不传 --font-render-hinting=none（否则 oracle 与 ZW FreeType
  // DEFAULT hinted 路径不一致）。
  // WSL2 兼容（R1348）：若 ORACLE_CDP_URL 已设（run-oracle-capture.sh 启动的非 headless
  // chromium），经 CDP 连接复用——headless 'new' 在 WSL2 + chromium 150 渲染 SIGTRAP
  //（见 run-oracle-capture.sh）。共享浏览器只 disconnect 不 close。
  const cdpUrl = process.env.ORACLE_CDP_URL;
  let browser;
  let shouldCloseBrowser;
  if (cdpUrl) {
    const ver = await fetch(`${cdpUrl}/json/version`).then((r) => r.json());
    browser = await puppeteer.connect({ browserWSEndpoint: ver.webSocketDebuggerUrl });
    shouldCloseBrowser = false;
  } else {
    browser = await puppeteer.launch({
      executablePath: CHROMIUM,
      headless: 'new',
      args: ['--no-sandbox', '--disable-gpu'],
    });
    shouldCloseBrowser = true;
  }
  try {
    console.log(`chromium source: ${cdpUrl || CHROMIUM} (${await browser.version()})`);
    const page = await browser.newPage();
    await page.setViewport({ width: opts.width, height: opts.height, deviceScaleFactor: 1 });
    const url = `${server.url}/${opts.html.replace(/^\//, '')}`;
    try {
      await page.goto(url, { waitUntil: 'networkidle0', timeout: 10000 });
    } catch (e) {
      console.error(`goto warning: ${e.message}（外部资源超时，本地内容仍截图）`);
    }
    if (opts.wait > 0) await new Promise((r) => setTimeout(r, opts.wait));
    // 等待所有 <img> 解码完成（R745 race 修复），与 chromium-oracle-shot.mjs 一致。
    await page.evaluate(async () => {
      const imgs = Array.from(document.images || []);
      await Promise.all(imgs.map((im) =>
        (im.complete && im.naturalWidth > 0) ? Promise.resolve()
          : new Promise((res) => { im.onload = im.onerror = () => res(); setTimeout(res, 800); })
      ));
    }).catch(() => {});
    const shotOpts = { path: opts.out, type: 'png' };
    if (opts.selector) {
      const el = await page.$(opts.selector);
      if (el) await el.screenshot(shotOpts);
      else await page.screenshot(shotOpts);
    } else {
      await page.screenshot(shotOpts);
    }
    console.log(`wrote oracle: ${opts.out} (${opts.width}x${opts.height})`);
  } finally {
    if (shouldCloseBrowser) {
      await browser.close();
    } else {
      browser.disconnect();
    }
    await server.close();
  }
}

main().catch((e) => { console.error(e); process.exit(1); });

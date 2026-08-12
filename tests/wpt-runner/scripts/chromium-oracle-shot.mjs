#!/usr/bin/env node
// chromium 独立 Oracle 截图（DC-14 anti-false-pass）。
//
// 对（抽样）上游 reftest 的 **test** 文件用 headless chromium 渲染并截图，
// 产出 {safe_id}.png 作为独立参考（C_test），供与 ZeroWeb 的 Z_test 离线对比。
// safe_id = test 路径的 / \ . 替换为 _，与 reftest.rs REFTEST_DUMP 命名一致。
//
// 用法：
//   node chromium-oracle-shot.mjs [--per-dir N] [--all] [--out DIR]
//
// 依赖：puppeteer-core（npm install）+ 系统 chromium（/usr/bin/chromium）。
//
// **R388 oracle-invalidation 修复**：原实现用 `file://` 加载 test 文件，
// 但上游 WPT reftest 的样式表与字体用**绝对路径**引用（如
// `<link href="/fonts/ahem.css">`、ahem.css 内 `url("../../fonts/Ahem.ttf")`）。
// `file://` 把 `/fonts/...` 解析为文件系统根（不存在）→ Ahem.css / 字体加载失败
// → chromium 退回 fallback 字体 → Ahem 方块字形（应为实心 1em 方块）变成细 X
// → 几何崩溃、底色大面积外露（ifc-008 oracle 实测 85% 红底 vs 正确 0% 红）。
// 后果：108 个 Ahem 依赖 reftest 的 oracle 全部损坏，cross-validate 把
// ZeroWeb 的正确渲染误判为「chromium 不一致」（ifc-008 Z_vs_chr 7.93% 是假发散，
// 正确 oracle 下仅 0.52%）。
//
// 修复：脚本内嵌一个 root=DATA_ROOT 的本地静态 HTTP server，用 `http://localhost`
// 加载 test。此时 `/fonts/ahem.css` → DATA_ROOT/fonts/ahem.css、
// `../../fonts/Ahem.ttf`（相对 /fonts/ahem.css）→ /fonts/Ahem.ttf =
// DATA_ROOT/fonts/Ahem.ttf，均存在 → @font-face 正常加载 Ahem → oracle 正确。
// 此方案自包含、不依赖系统是否安装 Ahem，避免 oracle 静默损坏复发。
import { readFile, mkdir } from 'node:fs/promises';
import { createReadStream, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { join, dirname, basename, extname, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const DATA_ROOT = join(HERE, '..', 'wpt-data');
const MANIFEST = join(DATA_ROOT, 'reftest-manifest.json');

function parseArgs() {
  const a = process.argv.slice(2);
  const o = { perDir: 3, all: false, out: join(HERE, '..', 'oracle-shots') };
  for (let i = 0; i < a.length; i++) {
    if (a[i] === '--per-dir') o.perDir = parseInt(a[++i], 10);
    else if (a[i] === '--all') o.all = true;
    else if (a[i] === '--out') o.out = a[++i];
    else if (a[i] === '--help') {
      console.log('Usage: chromium-oracle-shot.mjs [--per-dir N] [--all] [--out DIR]');
      process.exit(0);
    }
  }
  return o;
}

const safeId = (p) => p.replace(/[\\/.]/g, '_');
const categoryOf = (p) => {
  const parts = p.split('/');
  const i = parts.indexOf('css');
  return i >= 0 && parts[i + 1] ? 'css/' + parts[i + 1] : '(other)';
};

// 内嵌静态 HTTP server（root=DATA_ROOT）。R388：用 http:// 取代 file://，
// 让上游 reftest 的绝对路径 `/fonts/ahem.css` 与相对 `../../fonts/Ahem.ttf`
// 都解析到 DATA_ROOT 下的真实文件（见文件头注释）。返回 { url, close }。
const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.htm': 'text/html; charset=utf-8',
  '.xht': 'application/xhtml+xml; charset=utf-8',
  '.xhtml': 'application/xhtml+xml; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.ttf': 'font/ttf',
  '.otf': 'font/otf',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
};
async function startStaticServer(rootDir) {
  const root = resolve(rootDir);
  const server = createServer((req, res) => {
    // 解码 + 规范化，禁止路径逃逸到 root 之外
    const decoded = decodeURIComponent((req.url || '/').split('?')[0]);
    const filePath = normalize(join(root, decoded));
    if (!filePath.startsWith(root)) {
      res.writeHead(403); res.end('403'); return;
    }
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
// stat 包裹（避免 throw；返回 [exists, isFile]）
async function statSafe(p) {
  try { const s = statSync(p); return [true, s.isFile()]; } catch { return [false, false]; }
}

// 等待页面所有 <img> 加载完成（complete && naturalWidth>0），防 SVG/图片解码 race
// 致截图时 img 未就绪（R388/R692 oracle 损坏：blank broken-img placeholder）。
// 无 img 或已全就绪则立即返回；超时则放弃等待（不阻塞）。
async function waitForImages(page, timeoutMs = 400) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const ready = await page.evaluate(() => {
      const imgs = Array.from(document.querySelectorAll('img'));
      return imgs.length === 0 || imgs.every((i) => i.complete && i.naturalWidth > 0);
    }).catch(() => true);
    if (ready) return;
    await new Promise((r) => setTimeout(r, 20));
  }
}

// `networkidle0` 不保证 CSS Font Loading 已完成字体解码和 face swap。
// bounded 等待 FontFaceSet.ready，坏字体或不支持该 API 时仍继续批量捕获。
async function waitForFonts(page, timeoutMs = 2000) {
  await Promise.race([
    page.evaluate(() => document.fonts ? document.fonts.ready.then(() => true) : true),
    new Promise((resolve) => setTimeout(resolve, timeoutMs)),
  ]).catch(() => {});
}

async function main() {
  const opts = parseArgs();
  const manifest = JSON.parse(await readFile(MANIFEST, 'utf-8'));
  const entries = manifest.entries || [];

  // 按目录分组抽样，保证跨目录覆盖
  const byCat = new Map();
  for (const e of entries) {
    const c = categoryOf(e.test);
    if (!byCat.has(c)) byCat.set(c, []);
    byCat.get(c).push(e);
  }
  let sample = [];
  for (const [c, list] of byCat) {
    const n = opts.all ? list.length : Math.min(opts.perDir, list.length);
    // 均匀抽样（取首/中/尾）而非全取头部，避免偏差
    const idx = list.length <= n ? list.map((_, i) => i)
      : Array.from({ length: n }, (_, k) => Math.floor(k * list.length / n));
    for (const i of idx) sample.push(list[i]);
  }
  console.log(`chromium Oracle shot: ${sample.length} cases (${opts.all ? 'all' : opts.perDir + '/dir'}) -> ${opts.out}`);

  const puppeteer = await import('puppeteer-core');
  const executablePath = process.env.PUPPETEER_EXECUTABLE_PATH || '/usr/bin/chromium';
  const browser = await puppeteer.default.launch({
    headless: true, executablePath,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });
  await mkdir(opts.out, { recursive: true });

  // R388：启动本地静态 server（root=DATA_ROOT），用 http:// 加载 test。
  const server = await startStaticServer(DATA_ROOT);

  let ok = 0, fail = 0;
  for (const e of sample) {
    const page = await browser.newPage();
    await page.setViewport({ width: 800, height: 600 });
    try {
      const url = `${server.url}/${e.test}`;
      await page.goto(url, { waitUntil: 'networkidle0', timeout: 8000 });
      await waitForImages(page);
      await waitForFonts(page);
      await new Promise(r => setTimeout(r, 80));
      await page.screenshot({ path: join(opts.out, safeId(e.test) + '.png'), type: 'png' });
      ok++;
    } catch (err) {
      console.error(`  FAIL ${e.test}: ${err.message}`);
      fail++;
    } finally {
      await page.close();
    }
  }
  await browser.close();
  await server.close();
  console.log(`Done: ${ok} captured, ${fail} failed -> ${opts.out}`);
}

main().catch(e => { console.error('Fatal:', e); process.exit(1); });

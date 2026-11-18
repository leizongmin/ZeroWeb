#!/usr/bin/env node
// R5524: border-collapse 域 chromium CDP 几何捕获（R4031-N 用户 2026-09-08 点名放行的捕获流程）。
//
// 背景：R4025-N/R4026-N/R4031-N 六轮像素逆向穷尽后，collapse 中心线迁移的前置 =
// chromium 实测几何（html/td 盒 + 边框带；007 参照蓝带 220px 纯蓝无法从 CSS 语义推导）。
// 本脚本对显式给定的 wpt-data 相对路径逐案 evaluate 盒几何 + computed border/padding/
// border-collapse，JSON 产物写 tests/wpt-runner/geometry-shots/（人工甄别后归档 evidence/）。
// 只写新文件、无删除路径（guidelines #31：不可误伤既有语料/产物）。
//
// 用法：
//   node capture-collapse-geometry.mjs <wpt-data 相对路径> [...]
//   # 复用已运行的 CDP 实例：ORACLE_CDP_URL=http://localhost:9227 node ...
//
// 依赖：puppeteer-core + 系统 chromium；ORACLE_CDP_URL 未设时 headless 自启
// （chromium 156 起 headless 渲染路径可用，R1253 时代的 SIGTRAP 见
// scripts/run-oracle-capture.sh 头注；几何提取只需 layout，不依赖 paint）。
import { join, dirname, extname, normalize, relative, resolve } from 'node:path';
import { createReadStream, statSync, existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = join(HERE, '..', '..', '..');
const DATA = join(HERE, '..', 'wpt-data');
const OUT = join(HERE, '..', 'geometry-shots');
mkdirSync(OUT, { recursive: true });
const MIME = {
  '.html': 'text/html; charset=utf-8', '.htm': 'text/html; charset=utf-8',
  '.xht': 'application/xhtml+xml; charset=utf-8', '.xhtml': 'application/xhtml+xml; charset=utf-8',
  '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.gif': 'image/gif',
  '.svg': 'image/svg+xml', '.ttf': 'font/ttf', '.otf': 'font/otf', '.woff': 'font/woff', '.woff2': 'font/woff2',
};

const files = process.argv.slice(2);
if (files.length === 0) {
  console.error('Usage: capture-collapse-geometry.mjs <wpt-data 相对路径> [...]');
  process.exit(1);
}

// 等待 FontFaceSet.ready，避免几何捕获偶发落在 fallback 字体上（同 capture-oracle-per-dir.mjs）。
async function waitForFonts(page, timeoutMs = 2000) {
  await Promise.race([
    page.evaluate(() => (document.fonts ? document.fonts.ready.then(() => true) : true)),
    new Promise((resolve) => setTimeout(resolve, timeoutMs)),
  ]).catch(() => {});
}

// 盒几何 + computed 边框三元组（宽/样式/色）——边框带实测与冲突消解胜者都从这三元组读出。
function extractionSnippet() {
  return () => {
    const box = (el) => {
      const r = el.getBoundingClientRect();
      const cs = getComputedStyle(el);
      return {
        tag: el.tagName.toLowerCase(),
        id: el.id || undefined,
        cls: el.className || undefined,
        rect: { x: +r.x.toFixed(2), y: +r.y.toFixed(2), w: +r.width.toFixed(2), h: +r.height.toFixed(2) },
        client: { w: el.clientWidth, h: el.clientHeight },
        offset: { w: el.offsetWidth, h: el.offsetHeight },
        border: {
          width: [cs.borderTopWidth, cs.borderRightWidth, cs.borderBottomWidth, cs.borderLeftWidth],
          style: [cs.borderTopStyle, cs.borderRightStyle, cs.borderBottomStyle, cs.borderLeftStyle],
          color: [cs.borderTopColor, cs.borderRightColor, cs.borderBottomColor, cs.borderLeftColor],
        },
        padding: [cs.paddingTop, cs.paddingRight, cs.paddingBottom, cs.paddingLeft],
        borderCollapse: cs.borderCollapse,
        display: cs.display,
      };
    };
    // 真表格标签 ∪ computed display 属 table-* 族（R5526：CSS 表布局案用 div+display:
    // table-column 等，tag 选择器抓不到——按 computed display 兜住，保持文档序）。
    const tableDisplays = new Set([
      'table', 'inline-table', 'table-row', 'table-row-group', 'table-header-group',
      'table-footer-group', 'table-column', 'table-column-group', 'table-cell', 'table-caption',
    ]);
    const els = [];
    for (const el of document.querySelectorAll('*')) {
      const isTag = /^(table|caption|colgroup|col|tbody|thead|tfoot|tr|td|th)$/i.test(el.tagName);
      const isTableDisplay = tableDisplays.has(getComputedStyle(el).display);
      if (isTag || isTableDisplay) els.push(el);
    }
    return {
      url: location.pathname,
      title: document.title,
      html: document.documentElement ? box(document.documentElement) : null,
      body: document.body ? box(document.body) : null,
      boxes: els.map(box),
      doc: {
        clientW: document.documentElement.clientWidth,
        clientH: document.documentElement.clientHeight,
        scrollW: document.documentElement.scrollWidth,
        scrollH: document.documentElement.scrollHeight,
      },
    };
  };
}

const root = resolve(DATA);
const srv = createServer((req, res) => {
  const fp = normalize(join(root, decodeURIComponent((req.url || '/').split('?')[0])));
  if (!fp.startsWith(root)) { res.writeHead(403); res.end(); return; }
  try {
    const s = statSync(fp);
    if (!s.isFile()) { res.writeHead(404); res.end(); return; }
    res.writeHead(200, { 'Content-Type': MIME[extname(fp).toLowerCase()] || 'application/octet-stream' });
    createReadStream(fp).pipe(res);
  } catch { res.writeHead(404); res.end(); }
});
await new Promise((r) => srv.listen(0, '127.0.0.1', r));
const { port } = srv.address();
const base = `http://127.0.0.1:${port}`;

const puppeteer = await import('puppeteer-core');
const browser = process.env.ORACLE_CDP_URL
  ? await puppeteer.default.connect({ browserURL: process.env.ORACLE_CDP_URL })
  : await puppeteer.default.launch({
      headless: true, executablePath: process.env.PUPPETEER_EXECUTABLE_PATH || '/usr/bin/chromium',
      args: ['--no-sandbox', '--disable-setuid-sandbox'],
    });

const results = [];
for (const rel of files) {
  if (!existsSync(join(DATA, rel))) {
    results.push({ file: rel, ok: false, error: 'file not found in wpt-data' });
    continue;
  }
  const page = await browser.newPage();
  await page.setViewport({ width: 800, height: 600 });
  try {
    await page.goto(`${base}/${rel}`, { waitUntil: 'networkidle0', timeout: 8000 });
    await waitForFonts(page);
    await new Promise((r) => setTimeout(r, 80));
    const geometry = await page.evaluate(extractionSnippet());
    const outPath = join(OUT, rel.replace(/[\\/]/g, '_').replace(/\.(html|xht|htm)$/, '') + '.json');
    writeFileSync(outPath, JSON.stringify(geometry, null, 1) + '\n');
    // SL-008（1bb7c6ca6 先例）：产物回执只记仓库相对路径，不落本机绝对路径。
    results.push({ file: rel, ok: true, out: relative(REPO_ROOT, outPath) });
  } catch (e) {
    results.push({ file: rel, ok: false, error: String(e).slice(0, 200) });
  }
  await page.close();
}

const manifest = {
  capturedAt: new Date().toISOString().slice(0, 10),
  chromiumVersion: await browser.version(),
  userAgent: await (await browser.newPage()).evaluate(() => navigator.userAgent).catch(() => null),
  viewport: { width: 800, height: 600 },
  tool: 'tests/wpt-runner/scripts/capture-collapse-geometry.mjs',
  purpose: 'border-collapse 域 CDP 几何捕获（R4031-N 2026-09-08 用户点名放行）',
  cases: results,
};
writeFileSync(join(OUT, 'manifest.json'), JSON.stringify(manifest, null, 1) + '\n');

for (const r of results) console.log(`${r.ok ? 'ok' : 'FAIL'}  ${r.file}${r.ok ? '' : '  ' + r.error}`);
console.log(`chromium ${manifest.chromiumVersion}; ${results.filter((r) => r.ok).length}/${results.length} captured -> ${OUT}`);
await browser.close();
await new Promise((r) => srv.close(r));

#!/usr/bin/env node
// 全量捕获指定目录的所有 reftest **test** 文件的 chromium Oracle 截图（DC-14 全量分母）。
//
// 区别于 chromium-oracle-shot.mjs（按目录抽样 per-dir N）：本脚本捕获指定目录的**全部**
// test 文件，供 DC-14 去子集化的全量 chromium-Oracle 测量。
//
// 用法：
//   node capture-oracle-per-dir.mjs --category css/css-position [--category css/css-tables ...]
//
// 依赖：puppeteer-core + 系统 chromium；经 ~/use-proxy 代理（chromium 抓上游无关，本地 HTTP）。
import { join, dirname, extname, normalize, relative, resolve } from 'node:path';
import { createReadStream, statSync, readdirSync } from 'node:fs';
import { createServer } from 'node:http';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const DATA = join(HERE, '..', 'wpt-data');
const OUT = join(HERE, '..', 'oracle-shots');
const MIME = {
  '.html': 'text/html; charset=utf-8', '.htm': 'text/html; charset=utf-8',
  '.xht': 'application/xhtml+xml; charset=utf-8', '.xhtml': 'application/xhtml+xml; charset=utf-8',
  '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.gif': 'image/gif',
  '.svg': 'image/svg+xml', '.ttf': 'font/ttf', '.otf': 'font/otf', '.woff': 'font/woff', '.woff2': 'font/woff2',
};

const categories = [];
for (let i = 2; i < process.argv.length; i++) {
  if (process.argv[i] === '--category') categories.push(process.argv[++i]);
}
if (categories.length === 0) { console.error('Usage: capture-oracle-per-dir.mjs --category <dir> [...]'); process.exit(1); }

// 递归收集 category 目录下所有 test 文件（相对 category 的路径）。
// css-text / CSS2 的 test 散落在子目录（white-space/、box/...），顶层 readdirSync 会漏掉。
// 扁平目录（grid/flex/...）退化为仅文件名，行为不变。
function collectTests(dir, base = dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) {
      out.push(...collectTests(full, base));
    } else if ((name.endsWith('.html') || name.endsWith('.xht'))
               && !name.includes('-ref') && !name.includes('notref') && !name.includes('reference')) {
      out.push(relative(base, full));
    }
  }
  return out;
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
await new Promise(r => srv.listen(0, '127.0.0.1', r));
const { port } = srv.address();
const base = `http://127.0.0.1:${port}`;

const puppeteer = await import('puppeteer-core');
const browser = await puppeteer.default.launch({
  headless: true, executablePath: process.env.PUPPETEER_EXECUTABLE_PATH || '/usr/bin/chromium',
  args: ['--no-sandbox', '--disable-setuid-sandbox'],
});

let totalOk = 0, totalFail = 0;
for (const cat of categories) {
  const dir = join(DATA, cat);
  let tests;
  try { tests = collectTests(dir); }
  catch { console.error(`dir not found: ${cat}`); continue; }
  let ok = 0, fail = 0;
  for (const t of tests) {
    const page = await browser.newPage();
    await page.setViewport({ width: 800, height: 600 });
    try {
      await page.goto(`${base}/${cat}/${t}`, { waitUntil: 'networkidle0', timeout: 8000 });
      await new Promise(r => setTimeout(r, 80));
      const safe = cat.replace(/[\\/]/g, '_') + '_' + t.replace(/[\\/.]/g, '_');
      await page.screenshot({ path: join(OUT, safe + '.png'), type: 'png' });
      ok++;
    } catch { fail++; }
    await page.close();
  }
  console.log(`${cat}: ${ok} ok, ${fail} fail`);
  totalOk += ok; totalFail += fail;
}
await browser.close();
await new Promise(r => srv.close(r));
console.log(`TOTAL: ${totalOk} ok, ${totalFail} fail`);

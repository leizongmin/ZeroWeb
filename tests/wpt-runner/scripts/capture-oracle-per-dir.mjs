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
import { createReadStream, statSync, readdirSync, existsSync, mkdirSync } from 'node:fs';
import { createServer } from 'node:http';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const DATA = join(HERE, '..', 'wpt-data');
const OUT = join(HERE, '..', 'oracle-shots');
// R2423：创建输出目录（chromium-oracle-shot.mjs 已 mkdir recursive；本脚本此前假定目录存在，
// 缺失时 page.screenshot({path}) 写入失败 → 全部捕获静默失败 0 ok/N fail，阻塞 oracle 测量）。
mkdirSync(OUT, { recursive: true });
const MIME = {
  '.html': 'text/html; charset=utf-8', '.htm': 'text/html; charset=utf-8',
  '.xht': 'application/xhtml+xml; charset=utf-8', '.xhtml': 'application/xhtml+xml; charset=utf-8',
  '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.gif': 'image/gif',
  '.svg': 'image/svg+xml', '.ttf': 'font/ttf', '.otf': 'font/otf', '.woff': 'font/woff', '.woff2': 'font/woff2',
};

const categories = [];
let skipExisting = false;
for (let i = 2; i < process.argv.length; i++) {
  if (process.argv[i] === '--category') categories.push(process.argv[++i]);
  else if (process.argv[i] === '--skip-existing') skipExisting = true;
}
if (categories.length === 0) { console.error('Usage: capture-oracle-per-dir.mjs --category <dir> [...] [--skip-existing]'); process.exit(1); }

// 等待页面所有 <img> 加载完成（complete && naturalWidth>0），防 SVG/图片解码 race
// 致截图时 img 未就绪（R388/R692 oracle 损坏：blank broken-img placeholder）。
// 无 img 或已全就绪则立即返回；超时则放弃等待（不阻塞，回到固定延时兜底）。
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
// M3: headless chromium 在 WSL2（无 /dev/dri + chromium 150）渲染 SIGTRAP。
// 用 ORACLE_CDP_URL 连接预启动的非 headless chromium（GUI 渲染路径，--user-data-dir 独立
// profile + --remote-debugging-port + --ozone-platform=x11），绕过 headless 崩溃。
const browser = process.env.ORACLE_CDP_URL
  ? await puppeteer.default.connect({ browserURL: process.env.ORACLE_CDP_URL })
  : await puppeteer.default.launch({
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
    const safe = cat.replace(/[\\/]/g, '_') + '_' + t.replace(/[\\/.]/g, '_');
    const outPath = join(OUT, safe + '.png');
    // 断点续传：--skip-existing 时跳过已存在的 PNG（chromium 渲染确定性，跨会话恢复安全）。
    if (skipExisting && existsSync(outPath)) { ok++; continue; }
    const page = await browser.newPage();
    await page.setViewport({ width: 800, height: 600 });
    try {
      await page.goto(`${base}/${cat}/${t}`, { waitUntil: 'networkidle0', timeout: 8000 });
      await waitForImages(page);
      await new Promise(r => setTimeout(r, 80));
      await page.screenshot({ path: outPath, type: 'png' });
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

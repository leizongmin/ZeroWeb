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
import { readFile, mkdir } from 'node:fs/promises';
import { join, dirname, basename } from 'node:path';
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

  let ok = 0, fail = 0;
  for (const e of sample) {
    const page = await browser.newPage();
    await page.setViewport({ width: 800, height: 600 });
    try {
      const url = 'file://' + join(DATA_ROOT, e.test);
      await page.goto(url, { waitUntil: 'networkidle0', timeout: 8000 });
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
  console.log(`Done: ${ok} captured, ${fail} failed -> ${opts.out}`);
}

main().catch(e => { console.error('Fatal:', e); process.exit(1); });

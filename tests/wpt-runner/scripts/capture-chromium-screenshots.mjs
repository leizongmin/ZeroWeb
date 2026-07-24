#!/usr/bin/env node
// WPT Reftest Chromium 参考截图工具
//
// 使用 Puppeteer 在 headless Chromium 中渲染 reftest HTML 并截图，
// 作为 ZeroWeb reftest 的参考基线。
//
// 用法：
//   node capture-chromium-screenshots.mjs --data-dir ../wpt-data [--viewport 800x600] [--filter css/CSS2]
//
// 环境要求：
//   - Node.js >= 18
//   - puppeteer-core 包（npm install puppeteer-core）+ 系统 chromium（默认 /usr/bin/chromium，
//     可用 PUPPETEER_EXECUTABLE_PATH 覆盖）

import { readdir, readFile, writeFile, mkdir } from 'node:fs/promises';
import { join, resolve, basename, dirname } from 'node:path';
import { existsSync } from 'node:fs';

// 解析命令行参数
function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    dataDir: resolve(dirname(import.meta.url.replace('file://', '')), '..', 'wpt-data'),
    viewport: '800x600',
    filter: null,
    output: null,
    media: null,
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--data-dir':
        opts.dataDir = resolve(args[++i]);
        break;
      case '--viewport':
        opts.viewport = args[++i];
        break;
      case '--filter':
        opts.filter = args[++i];
        break;
      case '--output':
        opts.output = resolve(args[++i]);
        break;
      case '--media':
        opts.media = args[++i];
        break;
      case '--help':
        console.log(`Usage: node capture-chromium-screenshots.mjs [options]
Options:
  --data-dir <path>    Path to WPT test data directory (default: ../wpt-data)
  --viewport <WxH>     Viewport size (default: 800x600)
  --filter <prefix>    Only process tests matching path prefix
  --output <path>      Output directory for screenshots (default: <data-dir>/screenshots)
  --media <type>       Emulated media type: screen|print (default: screen; for @media print oracle capture)
  --help               Show this help`);
        process.exit(0);
    }
  }

  const [w, h] = opts.viewport.split('x').map(Number);
  opts.width = w || 800;
  opts.height = h || 600;
  opts.output = opts.output || join(opts.dataDir, 'screenshots');

  return opts;
}

// 递归查找所有 HTML 文件
async function findHtmlFiles(dir, prefix = '') {
  const results = [];
  if (!existsSync(dir)) return results;

  const entries = await readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = join(dir, entry.name);
    const relPath = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) {
      results.push(...await findHtmlFiles(fullPath, relPath));
    } else if (entry.name.endsWith('.html') || entry.name.endsWith('.htm') ||
               entry.name.endsWith('.xht') || entry.name.endsWith('.xhtml')) {
      results.push({ fullPath, relPath });
    }
  }
  return results;
}

// 提取 reftest 链接
function extractReftestLinks(html) {
  const refs = [];
  const linkRegex = /<link\s+[^>]*rel\s*=\s*["']?(match|mismatch)["']?[^>]*>/gi;
  let match;
  while ((match = linkRegex.exec(html)) !== null) {
    const tag = match[0];
    const relation = match[1];
    const hrefMatch = tag.match(/href\s*=\s*["']([^"']+)["']/i);
    if (hrefMatch) {
      refs.push({
        href: hrefMatch[1],
        relation: relation === 'match' ? '==' : '!=',
      });
    }
  }
  return refs;
}

async function main() {
  const opts = parseArgs();

  console.log('WPT Reftest Chromium Screenshot Capture');
  console.log(`  Data dir: ${opts.dataDir}`);
  console.log(`  Viewport: ${opts.width}x${opts.height}`);
  console.log(`  Output:   ${opts.output}`);
  console.log('');

  // 动态导入 puppeteer-core（不自带 chromium，复用系统 /usr/bin/chromium）
  let puppeteer;
  try {
    puppeteer = await import('puppeteer-core');
  } catch {
    console.error('Error: puppeteer-core not installed. Run: npm install puppeteer-core');
    process.exit(1);
  }

  const executablePath = process.env.PUPPETEER_EXECUTABLE_PATH || '/usr/bin/chromium';
  const browser = await puppeteer.launch({
    headless: true,
    executablePath,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  const page = await browser.newPage();
  await page.setViewport({ width: opts.width, height: opts.height });
  // R1991：模拟媒体类型（screen|print）以抓 @media print oracle shot（与 ZW --media 对齐）。
  if (opts.media) {
    await page.emulateMediaType(opts.media);
  }

  // 查找所有 HTML 文件
  let htmlFiles = await findHtmlFiles(opts.dataDir);

  // 应用过滤器
  if (opts.filter) {
    htmlFiles = htmlFiles.filter(f => f.relPath.startsWith(opts.filter));
  }

  console.log(`Found ${htmlFiles.length} HTML files`);

  // 确保输出目录存在
  await mkdir(opts.output, { recursive: true });

  let captured = 0;
  let skipped = 0;

  for (const file of htmlFiles) {
    const html = await readFile(file.fullPath, 'utf-8');

    // 只处理包含 reftest 链接的文件
    const refs = extractReftestLinks(html);
    if (refs.length === 0) {
      skipped++;
      continue;
    }

    try {
      // 渲染并截图测试页面
      await page.goto(`file://${file.fullPath}`, { waitUntil: 'networkidle0', timeout: 5000 });
      await new Promise(r => setTimeout(r, 100)); // 等待渲染完成

      const outName = basename(file.relPath).replace(/\.html?$/, '') + '-ref.png';
      const outPath = join(opts.output, outName);
      await page.screenshot({ path: outPath, type: 'png' });
      captured++;

      if (captured % 10 === 0) {
        console.log(`  Captured ${captured}/${htmlFiles.length}...`);
      }
    } catch (err) {
      console.error(`  Error capturing ${file.relPath}: ${err.message}`);
    }
  }

  await browser.close();

  console.log(`\nDone: ${captured} screenshots captured, ${skipped} non-reftest files skipped`);
  console.log(`Screenshots saved to: ${opts.output}`);
}

main().catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});

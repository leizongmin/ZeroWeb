#!/usr/bin/env node
// Verify generated pages with real browser geometry and native UI interactions.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { resolve, join, extname, sep } from 'node:path';
import { createRequire } from 'node:module';

const { chromium } = createRequire(import.meta.url)('playwright');
const root = resolve(process.argv[2] || '');
const evidence = process.argv[3] && resolve(process.argv[3]);
const mime = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png' };
const server = createServer(async (request, response) => {
  const path = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
  const file = resolve(root, `.${path}`, path.endsWith('/') ? 'index.html' : '');
  if (!file.startsWith(root + sep)) { response.writeHead(403).end(); return; }
  try {
    const body = await readFile(file);
    response.writeHead(200, { 'Content-Type': mime[extname(file)] || 'application/octet-stream' }).end(body);
  } catch { response.writeHead(404).end(); }
});
await new Promise((done) => server.listen(0, '127.0.0.1', done));
const base = `http://127.0.0.1:${server.address().port}`;
let browser;
const results = [];
const errors = [];

// Measure the viewport instead of masking overflow on the document.
async function geometry(page) {
  return page.evaluate(() => ({
    viewport: innerWidth,
    width: document.documentElement.scrollWidth,
    height: document.documentElement.scrollHeight,
    clipped: [...document.querySelectorAll('.summary-primary strong, h1')].some((node) => node.scrollWidth > node.clientWidth + 1)
  }));
}

try {
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ reducedMotion: 'reduce' });
  page.on('pageerror', (error) => errors.push(error.message));
  const sitemap = await readFile(join(root, 'sitemap.xml'), 'utf8');
  const paths = [...sitemap.matchAll(/<loc>(.*?)<\/loc>/g)].map((match) => new URL(match[1]).pathname);
  assert(paths.length, 'Sitemap has no pages');
  if (evidence) await mkdir(evidence, { recursive: true });
  for (const width of [375, 390, 768, 1024, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    for (const path of paths) {
      const response = await page.goto(base + path);
      assert.equal(response.status(), 200, path);
      const closed = await geometry(page);
      assert(closed.width <= width + 1, `${path} at ${width}px overflows to ${closed.width}px`);
      assert(!closed.clipped, `${path} at ${width}px clips its heading or metric`);
      // Use native summary activation so expanded data is tested too.
      for (const summary of await page.locator('details > summary').all()) {
        if (!await summary.evaluate((node) => node.parentElement.open)) await summary.click();
      }
      const expanded = await geometry(page);
      assert(expanded.width <= width + 1, `${path} expanded at ${width}px overflows to ${expanded.width}px`);
      results.push({ path, viewport: width, width: closed.width, closedHeight: closed.height, expandedHeight: expanded.height });
      if (evidence && [390, 1440].includes(width) && ['/', '/zh/', '/progress.html', '/zh/progress.html', '/updates.html', '/zh/updates.html', '/updates/2026-10-25-weekly/', '/zh/updates/2026-10-25-weekly/', '/guides/rust-browser-architecture/', '/zh/guides/rust-browser-architecture/'].includes(path)) {
        // Collapse via the same native controls before recording the default layout.
        for (const summary of (await page.locator('details > summary').all()).reverse()) {
          if (await summary.evaluate((node) => node.parentElement.open)) await summary.click();
        }
        await page.goto(base + path);
        await page.screenshot({ path: join(evidence, `${path.replaceAll('/', '_')}-${width}.png`) });
      }
    }
    console.log(`PASS: ${paths.length} pages at ${width}px, default and expanded layouts`);
  }
  for (const prefix of ['', '/zh']) {
    await page.goto(`${base}${prefix}/updates/2026-08-23-weekly/`);
    await page.locator('.article-toc a[href="#completed"]').click();
    assert.equal(new URL(page.url()).hash, '#completed');
    assert(Math.abs(await page.locator('#completed').evaluate((node) => node.getBoundingClientRect().top)) < 100);
    await page.locator(`.language-switcher a[hreflang="${prefix ? 'en' : 'zh'}"]`).click();
    assert.equal(new URL(page.url()).pathname, `${prefix ? '' : '/zh'}/updates/2026-08-23-weekly/`);
    await page.goto(`${base}${prefix}/updates.html`);
    await page.locator('.article-toc a').last().click();
    assert(new URL(page.url()).hash.startsWith('#month-'));
    await page.goto(`${base}${prefix}/progress.html`);
    const summary = page.locator('details > summary').first();
    await summary.focus();
    await page.keyboard.press('Enter');
    assert(await summary.evaluate((node) => node.parentElement.open));
    await page.waitForFunction(() => metricsData !== null);
    const chart = page.locator('#performance-chart svg');
    const before = await chart.getAttribute('aria-label');
    await page.locator('#metric-select').selectOption('resource/peak_rss_mb');
    assert.notEqual(await chart.getAttribute('aria-label'), before);
    assert.match(await page.locator('#performance-table').textContent(), /MB/);
    await page.goto(`${base}${prefix}/guides/rust-browser-architecture/`);
    await page.locator('.article-toc a').nth(1).click();
    assert.equal(new URL(page.url()).hash, '#section-2');
    assert.equal(await page.locator('.guide-pipeline li').count(), 5);
  }
  for (const mode of ['no-js', 'fetch-failure', 'dark']) {
    const context = await browser.newContext({ javaScriptEnabled: mode !== 'no-js', colorScheme: mode === 'dark' ? 'dark' : 'light', reducedMotion: 'reduce', viewport: { width: 390, height: 900 } });
    if (mode === 'fetch-failure') await context.route('**/metrics.json', (route) => route.abort());
    const fallback = await context.newPage();
    fallback.on('pageerror', (error) => errors.push(error.message));
    for (const prefix of ['', '/zh']) {
      await fallback.goto(`${base}${prefix}/progress.html`);
      assert.equal(await fallback.locator('#metric-summary .metric-card').count(), 6);
      await fallback.locator('details > summary').first().click();
      assert(await fallback.locator('#wptfyi-table table').isVisible());
      assert((await geometry(fallback)).width <= 391);
      await fallback.goto(`${base}${prefix}/guides/zerowebview-getting-started/`);
      await fallback.locator('.article-toc a').first().click();
      assert.equal(new URL(fallback.url()).hash, '#section-1');
      if (evidence && mode === 'dark') await fallback.screenshot({ path: join(evidence, `guide-${prefix ? 'zh' : 'en'}-dark.png`), fullPage: true });
      if (evidence && mode === 'dark') {
        for (const path of ['/', '/progress.html', '/updates/2026-10-25-weekly/']) {
          await fallback.goto(base + prefix + path);
          await fallback.screenshot({ path: join(evidence, `${prefix ? 'zh' : 'en'}-${path.replaceAll('/', '_')}-dark.png`) });
        }
      }
    }
    await context.close();
  }
  assert.deepEqual(errors, [], 'Browser JavaScript errors');
  if (evidence) await writeFile(join(evidence, 'results.json'), JSON.stringify({ results, errors, interactions: 'PASS', fallback: 'PASS' }, null, 2));
  console.log('PASS: language links, article/month anchors, keyboard details, metric switch, guides, no-JS, fetch failure and dark mode');
} finally {
  await browser?.close();
  await new Promise((done) => server.close(done));
}

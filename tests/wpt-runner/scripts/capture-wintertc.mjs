#!/usr/bin/env node
// 录制 wintertc.org 首页为 DC-13 产品 smoke fixture（DC-13 WinterTC 图片密集首页）。
//
// 用 headless chromium 加载 https://wintertc.org/（经代理），等 Twind 运行生成 CSS，
// 导出「已解析 DOM + 内联 Twind <style>」的静态 HTML（供 ZeroBrowser 无需 JS 渲染），
// 同时截 800×600 chromium Oracle 截图供与 ZeroWeb 离线对比。
//
// 依赖：tests/wpt-runner/scripts/node_modules/puppeteer-core + 系统 /usr/bin/chromium。
// 用法：node capture-wintertc.mjs
//   产物：apps/browser/assets/wintertc/index.html（已解析）+ evidence/.../wintertc/wintertc-chromium.png
import { writeFile, mkdir } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import puppeteer from 'puppeteer-core';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = join(HERE, '..', '..', '..');
const FIXTURE_DIR = join(REPO, 'apps', 'browser', 'assets', 'wintertc');
const EVIDENCE_DIR = join(REPO, 'docs', 'goal', 'rendering-compat', 'evidence', 'product-static', 'wintertc');

const PROXY = process.env.PROXY || 'http://proxy.example.local:7078';
const URL = 'https://wintertc.org/';
const VIEWPORT = { width: 800, height: 600 };

async function main() {
  await mkdir(FIXTURE_DIR, { recursive: true });
  await mkdir(EVIDENCE_DIR, { recursive: true });

  const browser = await puppeteer.launch({
    headless: true,
    executablePath: process.env.PUPPETEER_EXECUTABLE_PATH || '/usr/bin/chromium',
    args: [
      '--no-sandbox',
      '--disable-setuid-sandbox',
      `--proxy-server=${PROXY}`,
      '--hide-scrollbars',
    ],
  });
  try {
    const page = await browser.newPage();
    await page.setViewport(VIEWPORT);
    console.log(`loading ${URL} via ${PROXY} ...`);
    await page.goto(URL, { waitUntil: 'networkidle0', timeout: 45000 }).catch((e) => {
      console.warn('goto warn:', e.message);
    });
    // 给 Twind（CSS-in-JS）足够时间扫描 DOM 生成 <style>
    await new Promise((r) => setTimeout(r, 2500));

    // 导出已解析 DOM（含 Twind 生成的 <style>）
    const resolvedHtml = await page.content();
    await writeFile(join(FIXTURE_DIR, 'index.html'), resolvedHtml);
    console.log(`wrote index.html (${resolvedHtml.length} bytes)`);

    // chromium Oracle 截图
    await page.screenshot({ path: join(EVIDENCE_DIR, 'wintertc-chromium.png'), type: 'png' });
    console.log('wrote wintertc-chromium.png');

    // 报告页面引用的 /static/ 资源（供后续 curl 拉取）
    const refs = await page.evaluate(() => {
      const out = new Set();
      document.querySelectorAll('img[src],link[href],script[src]').forEach((el) => {
        const v = el.getAttribute('src') || el.getAttribute('href');
        if (v && v.startsWith('/static/')) out.add(v);
      });
      return [...out];
    });
    console.log('static refs:', JSON.stringify(refs));
    await writeFile(join(EVIDENCE_DIR, 'static-refs.json'), JSON.stringify(refs, null, 2));
  } finally {
    await browser.close();
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});

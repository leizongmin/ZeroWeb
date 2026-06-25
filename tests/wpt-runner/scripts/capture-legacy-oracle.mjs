#!/usr/bin/env node
// R656: 截取本地 legacy-html fixture 的 chromium Oracle PNG（DC-13 legacy-html smoke）。
// 依赖：tests/wpt-runner/scripts/node_modules/puppeteer-core + 系统 /usr/bin/chromium。
// 用法：node capture-legacy-oracle.mjs <input.html> <output.png>
import puppeteer from 'puppeteer-core';
import { resolve } from 'node:path';

const [, , input, output] = process.argv;
if (!input || !output) {
  console.error('usage: node capture-legacy-oracle.mjs <input.html> <output.png>');
  process.exit(1);
}

const VIEWPORT = { width: 800, height: 600, deviceScaleFactor: 1 };

const browser = await puppeteer.launch({
  headless: 'new',
  executablePath: process.env.PUPPETEER_EXECUTABLE_PATH || '/usr/bin/chromium',
  args: ['--no-sandbox', '--disable-dev-shm-usage', '--force-color-profile=srgb'],
});
try {
  const page = await browser.newPage();
  await page.setViewport(VIEWPORT);
  const url = 'file://' + resolve(input);
  await page.goto(url, { waitUntil: 'load', timeout: 30000 }).catch((e) => {
    console.warn('goto warn:', e.message);
  });
  await page.screenshot({ path: output, type: 'png', clip: { x: 0, y: 0, ...VIEWPORT } });
  console.log('wrote', output);
} finally {
  await browser.close();
}

#!/usr/bin/env node
// AA 基准 chromium 截图：渲染任意 file:// HTML → PNG（与 ZeroWeb product-smoke 同输入对比）。
import puppeteer from 'puppeteer-core';
import { realpathSync } from 'node:fs';
const url = 'file://' + realpathSync(process.argv[2]);
const out = process.argv[3];
const b = await puppeteer.launch({
  headless: true,
  executablePath: process.env.PUPPETEER_EXECUTABLE_PATH || '/usr/bin/chromium',
  args: ['--no-sandbox', '--disable-setuid-sandbox'],
});
const p = await b.newPage();
await p.setViewport({ width: 800, height: 600 });
await p.goto(url, { waitUntil: 'networkidle0', timeout: 8000 });
await new Promise(r => setTimeout(r, 120));
await p.screenshot({ path: out, type: 'png' });
await b.close();
console.log('shot ->', out);

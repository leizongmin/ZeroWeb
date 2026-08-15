#!/usr/bin/env node
// 离线运行冻结的 HTML5test fixture，并检查其结果页已完成生成。

import { createReadStream } from 'node:fs';
import { access, mkdir, readFile, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import puppeteer from './node_modules/puppeteer-core/lib/puppeteer/puppeteer-core.js';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, '../../..');
const FIXTURE_ROOT = resolve(REPO_ROOT, 'tests/wpt-runner/fixtures/html5test-frozen');
const DEFAULT_OUTPUT = resolve(REPO_ROOT, '.acceptance/html5test-frozen');
const MIME = {
  '.css': 'text/css; charset=utf-8', '.eot': 'application/vnd.ms-fontobject',
  '.html': 'text/html; charset=utf-8', '.ico': 'image/x-icon', '.js': 'text/javascript; charset=utf-8',
  '.jpg': 'image/jpeg', '.png': 'image/png', '.svg': 'image/svg+xml', '.ttf': 'font/ttf', '.woff': 'font/woff',
};

function parseArgs() {
  const result = { out: DEFAULT_OUTPUT, parity: false, zeroweb: false };
  const args = process.argv.slice(2);
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] === '--out') result.out = resolve(args[++index]);
    else if (args[index] === '--parity') result.parity = true;
    else if (args[index] === '--zeroweb') result.zeroweb = true;
    else if (args[index] === '--help') {
      console.log('Usage: node html5test-frozen-smoke.mjs [--out <directory>] [--parity] [--zeroweb]');
      process.exit(0);
    } else throw new Error(`unknown option: ${args[index]}`);
  }
  return result;
}

async function startFixtureServer() {
  const server = createServer((request, response) => {
    const requestPath = decodeURIComponent((request.url || '/').split('?')[0]);
    const file = normalize(join(FIXTURE_ROOT, requestPath));
    if (!file.startsWith(FIXTURE_ROOT)) {
      response.writeHead(403).end();
      return;
    }
    createReadStream(file)
      .on('error', () => response.writeHead(404).end())
      .once('open', () => response.writeHead(200, { 'Content-Type': MIME[extname(file)] || 'application/octet-stream' }))
      .pipe(response);
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  return {
    url: `http://127.0.0.1:${server.address().port}`,
    close: () => new Promise((done) => server.close(done)),
  };
}

function chromePath() {
  return process.env.PUPPETEER_EXECUTABLE_PATH || 'C:/Program Files/Google/Chrome/Application/chrome.exe';
}

async function runSmoke(url, output) {
  const diagnostics = [];
  let page;
  const browser = await puppeteer.launch({
    executablePath: chromePath(),
    headless: 'new',
    args: ['--no-sandbox', '--disable-lcd-text'],
  });
  try {
    page = await browser.newPage();
    page.on('pageerror', (error) => diagnostics.push(`pageerror: ${error.message}`));
    page.on('console', (message) => {
      if (message.type() === 'error') diagnostics.push(`console: ${message.text()}`);
    });
    await page.setViewport({ width: 1280, height: 900, deviceScaleFactor: 1 });
    await page.goto(`${url}/index.html`, { waitUntil: 'load', timeout: 15000 });
    await page.waitForFunction(() => {
      const contents = document.querySelector('#contents');
      return contents && getComputedStyle(contents).visibility === 'visible' && document.querySelector('#score .pointsPanel');
    }, { timeout: 5000 });
    await page.screenshot({ path: join(output, 'chrome-smoke.png'), type: 'png' });
    // The archived page still requests discontinued analytics and advertising
    // hosts.  These requests are intentionally blocked in the offline fixture;
    // the visible result state above is the regression contract.
  } catch (error) {
    if (page) {
      diagnostics.push(await page.evaluate(() => JSON.stringify({
        contentsVisibility: getComputedStyle(document.querySelector('#contents')).visibility,
        score: document.querySelector('#score').innerHTML,
        resultCategories: document.querySelectorAll('#results .category').length,
        loadingDisplay: getComputedStyle(document.querySelector('#loading')).display,
      })).catch((inspectError) => `inspection: ${inspectError.message}`));
    }
    await writeFile(join(output, 'chrome-smoke-diagnostics.txt'), `${diagnostics.join('\n')}\n${error.stack || error.message}\n`);
    throw error;
  } finally {
    await browser.close();
  }
}

async function runParity(url, output, allowMismatch = false) {
  const scenario = JSON.parse(await readFile(join(FIXTURE_ROOT, 'parity.scenario.json'), 'utf8'));
  scenario.url = `${url}/index.html`;
  const scenarioPath = join(output, 'html5test-frozen.scenario.json');
  await writeFile(scenarioPath, `${JSON.stringify(scenario, null, 2)}\n`);
  const evidence = join(output, 'parity');
  const command = process.execPath;
  const args = [join(REPO_ROOT, '.agents/skills/zeroweb-browser-chrome-parity/scripts/run-parity.mjs'), scenarioPath, evidence];
  await new Promise((done, fail) => {
    const child = spawn(command, args, {
      cwd: REPO_ROOT,
      env: {
        ...process.env,
        PARITY_ORACLE_MODE: 'gui',
        ZEROWEB_EVIDENCE_COMMAND: JSON.stringify([
          'target/release/zero-browser.exe', '--renderer=gpu', '--scale=1',
          '--parity-scenario', '${PARITY_SCENARIO}', '--parity-output-dir', '${PARITY_OUTPUT_DIR}',
        ]),
      },
      stdio: 'inherit',
    });
    child.once('error', fail);
    child.once('exit', (code) => {
      if (code === 0 || allowMismatch) done();
      else fail(new Error(`parity runner exited ${code}`));
    });
  });
}

async function assertZeroWebReport(output) {
  const report = JSON.parse(await readFile(join(output, 'parity', 'report.json'), 'utf8'));
  const state = report.steps?.at(-1)?.state?.zeroweb;
  if (!report.production?.zeroweb) throw new Error('ZeroWeb capture was not production GPU evidence');
  if (!state?.ready) throw new Error(`HTML5test report did not become visible: ${JSON.stringify(state)}`);
  if (!/^\d+$/.test(String(state.score || ''))) throw new Error(`HTML5test score is missing: ${JSON.stringify(state)}`);
  if (state.categories < 8) throw new Error(`HTML5test categories are incomplete: ${JSON.stringify(state)}`);
  if (state.supportedResults < 1) throw new Error(`HTML5test supported API results are missing: ${JSON.stringify(state)}`);
  if (state.unsupportedResults < 1) throw new Error(`HTML5test unsupported API results are missing: ${JSON.stringify(state)}`);
  if (state.warning) throw new Error(`HTML5test reported a page error: ${state.warning}`);
}

async function main() {
  const options = parseArgs();
  await access(join(FIXTURE_ROOT, 'index.html'));
  await mkdir(options.out, { recursive: true });
  const server = await startFixtureServer();
  try {
    await runSmoke(server.url, options.out);
    if (options.parity) await runParity(server.url, options.out);
    if (options.zeroweb) {
      await runParity(server.url, options.out, true);
      await assertZeroWebReport(options.out);
      console.log('ZeroWeb frozen HTML5test smoke passed (score and 8 categories rendered).');
    }
  } finally {
    await server.close();
  }
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});

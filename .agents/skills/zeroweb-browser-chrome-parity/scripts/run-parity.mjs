#!/usr/bin/env node

import { access, mkdir } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, '../../../..');

function usage() {
  console.error(`用法: node run-parity.mjs <scenario.json> <evidence-dir>

必需环境变量:
  ZEROWEB_EVIDENCE_COMMAND  JSON 字符串数组，例如 ["cargo","run","--bin","producer"]

可选环境变量:
  ORACLE_CDP_URL            GUI Chrome DevTools 地址；完整生产验收必须设置
  PARITY_TIMEOUT            每个生产器的墙钟超时秒数，默认 180
  PARITY_PNG_COMPARATOR     zero-wpt-runner 可执行文件路径`);
}

function parseProducerCommand() {
  const source = process.env.ZEROWEB_EVIDENCE_COMMAND;
  if (!source) {
    throw new Error('缺少 ZEROWEB_EVIDENCE_COMMAND；静态截图不能证明交互一致性');
  }
  let command;
  try {
    command = JSON.parse(source);
  } catch (error) {
    throw new Error(`ZEROWEB_EVIDENCE_COMMAND 必须是 JSON 字符串数组: ${error.message}`);
  }
  if (!Array.isArray(command) || command.length === 0 || command.some((value) => typeof value !== 'string')) {
    throw new Error('ZEROWEB_EVIDENCE_COMMAND 必须是非空 JSON 字符串数组');
  }
  return command;
}

function timeoutMilliseconds() {
  const seconds = Number(process.env.PARITY_TIMEOUT || '180');
  if (!Number.isInteger(seconds) || seconds <= 0) {
    throw new Error('PARITY_TIMEOUT 必须是正整数');
  }
  return seconds * 1000;
}

function expandProducerArguments(command, values) {
  return command.map((argument) => argument
    .replaceAll('${PARITY_SCENARIO}', values.scenario)
    .replaceAll('${PARITY_OUTPUT_DIR}', values.outputDir)
    .replaceAll('${PARITY_REPO_ROOT}', values.repoRoot));
}

async function terminateTree(child) {
  if (child.exitCode !== null) return;
  if (process.platform === 'win32') {
    await new Promise((done) => {
      const killer = spawn('taskkill', ['/pid', String(child.pid), '/T', '/F'], {
        stdio: 'ignore',
        windowsHide: true,
      });
      killer.once('error', done);
      killer.once('exit', done);
    });
  } else {
    try {
      process.kill(-child.pid, 'SIGTERM');
    } catch {
      child.kill('SIGTERM');
    }
    await new Promise((done) => setTimeout(done, 1000));
    if (child.exitCode === null) {
      try {
        process.kill(-child.pid, 'SIGKILL');
      } catch {
        child.kill('SIGKILL');
      }
    }
  }
}

function runBounded(command, args, options, timeoutMs) {
  return new Promise((done, reject) => {
    const child = spawn(command, args, {
      cwd: options.cwd,
      env: options.env,
      shell: false,
      stdio: 'inherit',
      detached: process.platform !== 'win32',
      windowsHide: true,
    });
    let timedOut = false;
    const timer = setTimeout(async () => {
      timedOut = true;
      await terminateTree(child);
    }, timeoutMs);
    child.once('error', (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.once('exit', (code, signal) => {
      clearTimeout(timer);
      if (timedOut) {
        reject(new Error(`${command} 超过 ${timeoutMs / 1000} 秒墙钟上限`));
      } else if (code !== 0) {
        reject(new Error(`${command} 失败: code=${code} signal=${signal || 'none'}`));
      } else {
        done();
      }
    });
  });
}

async function main() {
  if (process.argv.length !== 4) {
    usage();
    process.exit(2);
  }
  const scenario = resolve(process.argv[2]);
  const output = resolve(process.argv[3]);
  const chromeDir = resolve(output, 'chrome');
  const zerowebDir = resolve(output, 'zeroweb');
  const timeoutMs = timeoutMilliseconds();
  const producer = expandProducerArguments(parseProducerCommand(), {
    scenario,
    outputDir: zerowebDir,
    repoRoot: REPO_ROOT,
  });

  await mkdir(chromeDir, { recursive: true });
  await mkdir(zerowebDir, { recursive: true });
  await runBounded(
    process.execPath,
    [resolve(HERE, 'validate-scenario.mjs'), scenario],
    { cwd: REPO_ROOT, env: process.env },
    timeoutMs,
  );
  await runBounded(
    process.execPath,
    [resolve(HERE, 'capture-chrome.mjs'), '--scenario', scenario, '--out', chromeDir],
    { cwd: REPO_ROOT, env: process.env },
    timeoutMs,
  );
  await runBounded(
    producer[0],
    producer.slice(1),
    {
      cwd: REPO_ROOT,
      env: {
        ...process.env,
        PARITY_SCENARIO: scenario,
        PARITY_OUTPUT_DIR: zerowebDir,
        PARITY_REPO_ROOT: REPO_ROOT,
      },
    },
    timeoutMs,
  );

  const manifest = resolve(zerowebDir, 'manifest.json');
  try {
    await access(manifest);
  } catch {
    throw new Error(`ZeroWeb 生产器未写出 ${manifest}`);
  }

  const compareArgs = [
    resolve(HERE, 'compare-evidence.mjs'),
    '--scenario', scenario,
    '--chrome', chromeDir,
    '--zeroweb', zerowebDir,
    '--out', resolve(output, 'report.json'),
    '--require-production',
  ];
  if (process.env.PARITY_PNG_COMPARATOR) {
    compareArgs.push('--comparator', process.env.PARITY_PNG_COMPARATOR);
  }
  await runBounded(
    process.execPath,
    compareArgs,
    { cwd: REPO_ROOT, env: process.env },
    timeoutMs,
  );
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});

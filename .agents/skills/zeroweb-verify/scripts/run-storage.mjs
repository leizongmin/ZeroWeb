import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..');
const assets = path.join(root, '.agents/skills/zeroweb-verify');
const id = process.argv[2];
assert.match(id ?? '', /^[a-zA-Z0-9][a-zA-Z0-9_-]{0,79}$/, 'Expected a unique run ID');
assert(['darwin', 'linux'].includes(process.platform), 'Requires macOS/Linux ps and kill');

/** 执行只读检查；工具或依赖缺失时停止，不驱动功能。 */
function read(command, args) {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', timeout: 10000 });
  assert.equal(result.status, 0, `${command} failed: ${result.stderr}`);
  return result.stdout.trim();
}

const head = read('git', ['rev-parse', 'HEAD']);
assert.equal(read('git', ['rev-parse', '--show-toplevel']), root);
const relevant = ['Cargo.toml', 'Cargo.lock', '.cargo', 'crates/storage', 'crates/runtime-config'];
const before = read('git', ['status', '--porcelain', '--', ...relevant]);
const ignored = read('git', ['check-ignore', '--no-index', '-v',
  '.agents/skills/zeroweb-verify/.verify-task/probe']);
assert(ignored.includes('.verify-task/'), 'Missing local evidence ignore rule');
const versions = {
  node: process.version,
  rustc: read('rustc', ['--version']),
  cargo: read('cargo', ['--version']),
  os: process.platform === 'darwin' ? read('sw_vers', []) : read('uname', ['-sr']),
  arch: process.arch,
};
read('ps', ['-ax', '-o', 'pid=,ppid=,rss=']);
const run = path.join(assets, '.verify-task', id);
fs.mkdirSync(path.dirname(run), { recursive: true });
fs.mkdirSync(run); // 拒绝复用已有目录，避免覆盖证据或误删既有数据。
const temporary = path.join(run, 'tmp');
const target = path.join(run, 'target');
fs.mkdirSync(temporary);
const env = { ...process.env, TMPDIR: temporary, TMP: temporary, TEMP: temporary,
  CARGO_TARGET_DIR: target, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '2' };
const guard = path.join(run, 'test-guard');
const commands = [];
const evidence = [];
const report = { head, startedAt: new Date().toISOString(), versions, relevantDiffBefore: before,
  initialWorkspace: read('git', ['status', '--porcelain']), ignored, commands,
  scope: 'F07: zero-storage production persistence API; no JS/IPC/GUI coverage',
  result: 'BLOCKED', evidence };
const writeReport = () => fs.writeFileSync(path.join(run, 'result.json'),
  `${JSON.stringify(report, null, 2)}\n`);
writeReport();

/** 收集真实子进程输出；仅守卫自举时监控进程树，后续复用 test-guard。 */
async function execute(command, args, logName, bootstrap = false) {
  commands.push({ command: path.relative(root, command) || command, args, logName });
  const logPath = path.join(run, logName);
  const fd = fs.openSync(logPath, 'wx');
  evidence.push(logName);
  let child;
  let monitor;
  let failure;
  try {
    child = spawn(command, args, { cwd: root, env, detached: true, stdio: ['ignore', fd, fd] });
    const terminate = (reason) => {
      failure = reason;
      try { process.kill(-child.pid, 'SIGKILL'); } catch {}
    };
    if (bootstrap) {
      const deadline = Date.now() + 60000;
      monitor = setInterval(() => {
        if (Date.now() > deadline) return terminate('Guard bootstrap timeout: 60s');
        const snapshot = spawnSync('ps', ['-ax', '-o', 'pid=,ppid=,rss='],
          { encoding: 'utf8', timeout: 2000 });
        if (snapshot.status !== 0) return terminate('Cannot monitor bootstrap memory');
        const rows = snapshot.stdout.trim().split('\n').map(line => line.trim().split(/\s+/).map(Number));
        const descendants = new Set([child.pid]);
        let previous;
        do {
          previous = descendants.size;
          for (const [pid, ppid] of rows) if (descendants.has(ppid)) descendants.add(pid);
        } while (previous !== descendants.size);
        const rss = rows.filter(([pid]) => descendants.has(pid)).map(row => row[2]);
        if (rss.some(kb => kb > 4 * 1024 * 1024) ||
            rss.reduce((a, b) => a + b, 0) > 8 * 1024 * 1024) {
          terminate('Guard bootstrap memory exceeded: 4/8 GiB');
        }
      }, 250);
    }
    const code = await new Promise((resolve, reject) => {
      child.once('error', reject);
      child.once('close', resolve);
    });
    assert(!failure, failure);
    assert.equal(code, 0, `${logName}: command exited ${code}`);
    return fs.readFileSync(logPath, 'utf8');
  } finally {
    clearInterval(monitor);
    fs.closeSync(fd);
    writeReport();
  }
}

try {
  console.log(`Preparing isolated production storage tests: ${id}`);
  await execute('rustc', ['--edition=2024', '-O', 'scripts/test-guard.rs', '-o', guard],
    'guard-build.log', true);
  const limits = ['--per-proc-mem', '4', '--total-mem', '8', '--time-limit', '600', '--'];
  const compiled = await execute(guard, [...limits, 'cargo', 'test', '--offline', '--locked',
    '-p', 'zero-storage', '--lib', '--no-run', '--message-format=json'], 'compile.log');
  const artifacts = compiled.split('\n').filter(line => line.startsWith('{')).map(line => JSON.parse(line))
    .filter(item => item.reason === 'compiler-artifact' && item.package_id.includes('zero-storage') &&
      item.profile.test && item.executable);
  assert.equal(artifacts.length, 1, 'Expected exactly one zero-storage test executable');
  const binary = artifacts[0].executable;
  assert(binary.startsWith(`${target}${path.sep}`), 'Artifact must be in this run target');
  report.binary = { path: path.relative(root, binary),
    sha256: createHash('sha256').update(fs.readFileSync(binary)).digest('hex') };
  const module = 'storage_manager::persistence_tests::';
  const listing = await execute(guard, [...limits, binary, module, '--list'], 'test-list.log');
  const names = [...listing.matchAll(/^(storage_manager::persistence_tests::\S+): test$/gm)]
    .map(match => match[1]);
  assert.equal(names.length, 7, 'Persistence test scope changed; review its contract before running');
  report.tests = names;
  report.result = 'FAIL';
  const output = await execute(guard, [...limits, binary, module, '--test-threads=1', '--nocapture'],
    'test.log');
  assert.match(output, /test result: ok\. 7 passed; 0 failed; 0 ignored;/);
  for (const name of names) assert(output.includes(`test ${name} ... ok`), `Missing result: ${name}`);
  report.temporaryResidue = fs.readdirSync(temporary);
  assert.equal(report.temporaryResidue.length, 0, 'TestDirectory cleanup left temporary data');
  report.relevantDiffAfter = read('git', ['status', '--porcelain', '--', ...relevant]);
  assert.equal(report.relevantDiffAfter, before, 'Relevant source changed during verification');
  assert.equal(read('git', ['rev-parse', 'HEAD']), head, 'HEAD changed during verification');
  report.result = 'PASS';
} catch (error) {
  report.error = error.message;
  console.error(error.message);
} finally {
  // 只删除本次独占目录里的编译/临时资源，保留输出、判据与二进制身份。
  for (const owned of [temporary, target, guard]) fs.rmSync(owned, { recursive: true, force: true });
  report.cleanup = { removed: ['tmp', 'target', 'test-guard'],
    remainingOwnedResources: [temporary, target, guard].filter(item => fs.existsSync(item)) };
  report.finishedAt = new Date().toISOString();
  report.evidenceHashes = Object.fromEntries(evidence.map(name => [name,
    createHash('sha256').update(fs.readFileSync(path.join(run, name))).digest('hex')]));
  writeReport();
  assert.equal(report.cleanup.remainingOwnedResources.length, 0);
  const retained = JSON.parse(fs.readFileSync(path.join(run, 'result.json'), 'utf8'));
  assert.equal(retained.result, report.result);
  console.log(`${report.result}: ${path.relative(root, run)}/result.json; evidence readable after cleanup`);
}
process.exitCode = report.result === 'PASS' ? 0 : 1;

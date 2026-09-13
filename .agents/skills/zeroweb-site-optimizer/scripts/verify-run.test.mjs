import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { verifyRun } from './verify-run.mjs';

const now = Date.parse('2026-01-01T00:10:00Z');
const hash = data => createHash('sha256').update(data).digest('hex');
async function fixture(t) {
  const root = await mkdtemp(path.join(os.tmpdir(), 'zeroweb-skill-eval-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const state = JSON.parse(await readFile(new URL('../templates/checkpoint.json', import.meta.url)));
  state.activity = { state: 'running', executor: { kind: 'host_task', ref: 'synthetic-task' },
    checked_at: state.updated_at };
  state.budget.validation_estimate_seconds = 900;
  async function save(name, value) {
    const data = JSON.stringify(value);
    await writeFile(path.join(root, name), data);
    return { path: name, sha256: hash(data) };
  }
  state.candidate_manifest = await save('manifest.json', { source: 'synthetic-candidate', binaries: [] });
  state.versions.original = 'a'.repeat(64);
  state.versions.best = state.versions.original;
  state.versions.trial = state.candidate_manifest.sha256;
  const artifact = await save('raw.json', { synthetic: true });
  const reports = {};
  for (const [id, kind, checks] of [
    ['font', 'target', ['geometry', 'target-region', 'whole-frame']],
    ['bench', 'engineering', ['relative-budget', 'hard-budget']],
  ]) {
    state.required_gates.push({ id, kind, checks, result: null });
    reports[id] = { schema_version: 1, subject: state.versions.trial, exit_code: 0,
      completed: true, comparable: true, checks: checks.map(id => ({ id, status: 'PASS' })),
      artifacts: [artifact] };
  }
  async function flush() {
    for (const gate of state.required_gates) {
      gate.result = reports[gate.id] ? await save(`${gate.id}.json`, reports[gate.id]) : null;
    }
    await save('checkpoint.json', state);
    return path.join(root, 'checkpoint.json');
  }
  return { root, state, reports, save, flush, check: async () => verifyRun(await flush(), now) };
}

test('complete evidence passes, but never proves a live task', async t => {
  const f = await fixture(t);
  const result = await f.check();
  assert.equal(result.delivery_verdict, 'ready');
  assert.equal(result.target_verdict, 'PASS');
  assert.equal(result.can_start_candidate, true);
  assert.equal(result.live_verified, false);
});

test('exit zero with a skipped comparison is not green', async t => {
  const f = await fixture(t);
  f.reports.bench.checks[0].status = 'SKIPPED';
  assert.equal((await f.check()).delivery_verdict, 'not_ready');
  assert.equal((await f.check()).target_verdict, 'PASS');
});

test('whole-frame pass cannot hide a failing text region', async t => {
  const f = await fixture(t);
  f.reports.font.checks[1].status = 'FAIL';
  assert.equal((await f.check()).target_verdict, 'FAIL');
  assert.equal((await f.check()).delivery_verdict, 'not_ready');
});

test('waivers match exact check, candidate and delivery scope; raw failure survives', async t => {
  const f = await fixture(t);
  f.reports.bench.checks[0].status = 'FAIL';
  f.reports.bench.exit_code = 2;
  const waiver = { gate_id: 'bench', check_id: 'relative-budget', delivery_scope: 'branch:other-pr',
    subject: f.state.versions.trial, reason: 'Synthetic explicit approval', approval_ref: 'user-message-1' };
  f.state.waivers = [waiver];
  assert.equal((await f.check()).delivery_verdict, 'not_ready');
  waiver.delivery_scope = f.state.delivery_scope;
  waiver.subject = 'b'.repeat(64);
  assert.equal((await f.check()).delivery_verdict, 'not_ready');
  waiver.subject = f.state.versions.trial;
  let result = await f.check();
  assert.equal(result.delivery_verdict, 'ready_with_waivers');
  assert.equal(result.gates[1].status, 'FAIL');
  f.reports.bench.checks[1].status = 'FAIL';
  assert.equal((await f.check()).delivery_verdict, 'not_ready');
  f.reports.bench.checks[1].status = 'PASS';
  f.reports.bench.comparable = false;
  result = await f.check();
  assert.equal(result.delivery_verdict, 'not_ready');
});

test('resume near deadline preserves trial and refuses another candidate', async t => {
  const f = await fixture(t);
  delete f.reports.font;
  const checkpoint = await f.flush();
  const result = await verifyRun(checkpoint, Date.parse('2026-01-01T01:50:00Z'));
  assert.equal(result.can_start_candidate, false);
  assert.equal(result.delivery_verdict, 'not_ready');
  const saved = JSON.parse(await readFile(checkpoint));
  assert.equal(saved.versions.best, 'a'.repeat(64));
  assert.equal(saved.versions.trial, f.state.versions.trial);
});

test('unknown or larger measured validation cost reserves enough time', async t => {
  const f = await fixture(t);
  f.state.budget.validation_estimate_seconds = null;
  assert.equal((await f.check()).can_start_candidate, false);
  f.state.budget.validation_estimate_seconds = 6600;
  assert.equal((await f.check()).reserve_seconds, 6900);
  assert.equal((await f.check()).can_start_candidate, false);
});

for (const state of ['stopped', 'awaiting_user', 'unknown']) {
  test(`${state} never permits a new candidate`, async t => {
    const f = await fixture(t);
    f.state.activity.state = state;
    assert.equal((await f.check()).can_start_candidate, false);
  });
}

for (const mutation of ['missing', 'duplicate', 'extra', 'unknown-status', 'subject', 'no-artifact']) {
  test(`reject malformed gate: ${mutation}`, async t => {
    const f = await fixture(t);
    const report = f.reports.bench;
    if (mutation === 'missing') report.checks.pop();
    if (mutation === 'duplicate') report.checks.push(report.checks[0]);
    if (mutation === 'extra') report.checks.push({ id: 'unplanned', status: 'PASS' });
    if (mutation === 'unknown-status') report.checks[0].status = 'NEW';
    if (mutation === 'subject') report.subject = 'b'.repeat(64);
    if (mutation === 'no-artifact') report.artifacts = [];
    await assert.rejects(f.check());
  });
}

test('incomplete, incomparable and contradictory executions cannot pass', async t => {
  const f = await fixture(t);
  f.reports.bench.completed = false;
  assert.equal((await f.check()).delivery_verdict, 'not_ready');
  f.reports.bench.completed = true;
  f.reports.bench.comparable = false;
  assert.equal((await f.check()).delivery_verdict, 'not_ready');
  f.reports.bench.comparable = true;
  f.reports.bench.exit_code = 1;
  assert.equal((await f.check()).delivery_verdict, 'not_ready');
});

test('changed evidence and traversal are rejected', async t => {
  const f = await fixture(t);
  const checkpoint = await f.flush();
  await writeFile(path.join(f.root, 'raw.json'), 'changed');
  await assert.rejects(verifyRun(checkpoint, now), /hash mismatch/);
  f.reports.bench.artifacts[0].path = '../outside.json';
  f.reports.font.artifacts = [await f.save('safe.json', {})];
  await assert.rejects(f.check(), /inside the run directory/);
});

test('symlink escape is rejected before reading the external file', async t => {
  const f = await fixture(t);
  const other = await fixture(t);
  await symlink(path.join(other.root, 'manifest.json'), path.join(f.root, 'link.json'));
  f.reports.font.artifacts = [{ path: 'link.json', sha256: f.state.versions.trial }];
  await assert.rejects(f.check(), /symlink escapes/);
});

test('empty initial plan and exhausted counters cannot authorize work', async t => {
  const f = await fixture(t);
  f.state.required_gates = [];
  f.state.budget.candidates_used = 5;
  const result = await f.check();
  assert.equal(result.delivery_verdict, 'not_ready');
  assert.equal(result.can_start_candidate, false);
});

test('missing executor, future heartbeat and invalid counters are rejected', async t => {
  const f = await fixture(t);
  const executor = f.state.activity.executor;
  f.state.activity.executor = null;
  await assert.rejects(f.check(), /checked executor/);
  f.state.activity.executor = executor;
  f.state.activity.checked_at = '2026-01-01T00:20:00Z';
  await assert.rejects(f.check(), /activity check time/);
  f.state.activity.checked_at = f.state.updated_at;
  f.state.budget.candidates_used = -1;
  await assert.rejects(f.check(), /budget counters/);
});

test('a standalone target or engineering gate cannot claim delivery readiness', async t => {
  const f = await fixture(t);
  const allGates = f.state.required_gates;
  for (const gate of allGates) {
    f.state.required_gates = [gate];
    assert.equal((await f.check()).delivery_verdict, 'not_ready');
  }
});

test('CLI returns 0, 1 and 2 for ready, incomplete and invalid evidence', async t => {
  const f = await fixture(t);
  const cli = fileURLToPath(new URL('./verify-run.mjs', import.meta.url));
  const invoke = async () => spawnSync(process.execPath, [cli, await f.flush()],
    { encoding: 'utf8', timeout: 5000 });
  assert.equal((await invoke()).status, 0);
  f.reports.bench.checks[0].status = 'SKIPPED';
  assert.equal((await invoke()).status, 1);
  f.state.waivers = [{ gate_id: 'bench', check_id: 'relative-budget',
    delivery_scope: f.state.delivery_scope, subject: f.state.versions.trial,
    reason: 'Synthetic explicit approval', approval_ref: 'user-message-1' }];
  const waived = await invoke();
  assert.equal(waived.status, 1);
  assert.equal(JSON.parse(waived.stdout).delivery_verdict, 'ready_with_waivers');
  f.reports.bench.subject = 'wrong';
  const invalid = await invoke();
  assert.equal(invalid.status, 2);
  assert.ok(!invalid.stderr.includes(f.root));
});

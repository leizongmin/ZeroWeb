import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { verifyWorkflow } from './verify-workflow.mjs';

const now = Date.parse('2026-01-01T00:10:00Z');
const hash = data => createHash('sha256').update(data).digest('hex');

/** 构造独立的合成目标、候选证据与累计预算，不启动产品或访问网络。 */
async function fixture(t) {
  const root = await mkdtemp(path.join(os.tmpdir(), 'zeroweb-workflow-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  let sequence = 0;
  async function save(value) {
    const data = JSON.stringify(value);
    const name = `evidence-${sequence++}.json`;
    await writeFile(path.join(root, name), data);
    return { path: name, sha256: hash(data) };
  }
  const cp = JSON.parse(await readFile(new URL('../templates/checkpoint.json', import.meta.url)));
  cp.activity = { state: 'running', executor: { kind: 'host_task', ref: 'controller' },
    checked_at: cp.updated_at };
  cp.budget.validation_estimate_seconds = 900;
  cp.budget.next_step_estimate_seconds = 300;
  cp.budget.candidate_limit = null;
  cp.budget.exploration_period_limit = null;
  cp.budget.candidates_used = 8;
  cp.budget.exploration_periods_used = 25;
  cp.budget.resources = [];
  cp.candidate_manifest = await save({ source: 'synthetic-best' });
  cp.versions = { original: cp.candidate_manifest.sha256, best: cp.candidate_manifest.sha256, trial: null };
  const raw = await save({ synthetic: true });
  for (const kind of ['target', 'engineering']) {
    cp.required_gates.push({ id: kind, kind, checks: ['check'], result: await save({
      schema_version: 1, subject: cp.versions.best, exit_code: 0, completed: true, comparable: true,
      checks: [{ id: 'check', status: 'PASS' }], artifacts: [raw],
    }) });
  }
  const workflow = {
    schema_version: 1, run_id: cp.run_id, revision: 1,
    contract_ref: await save({ objective: 'Search and read', scope: 'synthetic site' }),
    checkpoint_ref: null, verification_mode: 'independent',
    goals: [
      { id: 'search', description: 'Search works', verification: 'Real search returns expected content' },
      { id: 'read', description: 'Reading works', verification: 'Open result and return to search' },
    ],
    tasks: [
      { id: 'search-fix', goal_ids: ['search'], depends_on: [], description: 'Fix search',
        status: 'pending', pause_reason: null, evidence: null },
      { id: 'read-fix', goal_ids: ['read'], depends_on: [], description: 'Fix reading',
        status: 'pending', pause_reason: null, evidence: null },
    ],
    operations: [], final_acceptance: null, stop_reason: null,
  };
  async function flush() {
    workflow.checkpoint_ref = await save(cp);
    const ref = await save(workflow);
    return path.join(root, ref.path);
  }
  async function finish() {
    for (const task of workflow.tasks) {
      task.status = 'done';
      task.evidence = raw;
    }
    workflow.final_acceptance = await save({
      schema_version: 1, subject: cp.versions.best, contract_sha256: workflow.contract_ref.sha256,
      checks: workflow.goals.map(goal => ({ id: goal.id, status: 'PASS' })),
      reviewer: { executor_ref: 'fresh-reviewer', independent: true }, artifacts: [raw],
    });
  }
  return { root, cp, workflow, raw, save, flush, finish,
    check: async previous => verifyWorkflow(await flush(), previous, now) };
}

test('more than five candidates and twenty discovery periods still continue within budget', async t => {
  const f = await fixture(t);
  const result = await f.check();
  assert.equal(result.next, 'continue');
  assert.deepEqual(result.ready_tasks, ['search-fix', 'read-fix']);
  assert.equal(result.goal_verdict, 'INCOMPLETE');
});

test('local gates passing cannot establish global completion', async t => {
  const f = await fixture(t);
  f.workflow.stop_reason = 'completed';
  await assert.rejects(f.check(), /completion evidence/);
});

test('a paused cluster does not block an independent task', async t => {
  const f = await fixture(t);
  f.workflow.tasks[0].pause_reason = 'Need a new hypothesis';
  assert.deepEqual((await f.check()).ready_tasks, ['read-fix']);
  f.workflow.tasks[1].depends_on = ['search-fix'];
  assert.equal((await f.check()).next, 'replan');
});

for (const status of ['intended', 'running']) {
  test(`recover ${status} operation before dispatching again`, async t => {
    const f = await fixture(t);
    f.workflow.tasks[0].status = 'implementing';
    f.workflow.operations.push({ id: 'op-1', task_id: 'search-fix', kind: 'implement',
      status, executor_ref: status === 'running' ? 'worker-1' : null, result: null });
    assert.equal((await f.check()).next, 'wait_or_recover');
    const previous = await f.flush();
    f.workflow.revision++;
    f.workflow.operations.push({ id: 'op-2', task_id: 'read-fix', kind: 'implement',
      status: 'intended', executor_ref: null, result: null });
    await assert.rejects(f.check(previous), /unresolved operation/);
  });
}

test('finished slices require final acceptance; a failing journey returns to planning', async t => {
  const f = await fixture(t);
  await f.finish();
  const acceptance = f.workflow.final_acceptance;
  f.workflow.final_acceptance = null;
  assert.equal((await f.check()).next, 'final_acceptance');
  const report = JSON.parse(await readFile(path.join(f.root, acceptance.path)));
  report.checks[1].status = 'FAIL';
  f.workflow.final_acceptance = await f.save(report);
  assert.equal((await f.check()).next, 'replan');
});

test('complete evidence on current best permits a clean stop', async t => {
  const f = await fixture(t);
  await f.finish();
  f.workflow.stop_reason = 'completed';
  f.cp.activity.state = 'stopped';
  assert.equal((await f.check()).goal_verdict, 'PASS');
  assert.equal((await f.check()).next, 'stop');
  f.cp.versions.trial = 'b'.repeat(64);
  await assert.rejects(f.check(), /completion evidence/);
});

test('stale final evidence and missing goals cannot pass', async t => {
  const f = await fixture(t);
  await f.finish();
  const report = JSON.parse(await readFile(path.join(f.root, f.workflow.final_acceptance.path)));
  report.subject = 'b'.repeat(64);
  f.workflow.final_acceptance = await f.save(report);
  assert.equal((await f.check()).goal_verdict, 'INCOMPLETE');
  report.subject = f.cp.versions.best;
  report.checks.pop();
  f.workflow.final_acceptance = await f.save(report);
  assert.equal((await f.check()).goal_verdict, 'INCOMPLETE');
});

test('implementer cannot supply independent final review', async t => {
  const f = await fixture(t);
  await f.finish();
  f.workflow.operations.push({ id: 'op-1', task_id: 'search-fix', kind: 'implement',
    status: 'completed', executor_ref: 'fresh-reviewer', result: f.raw });
  assert.equal((await f.check()).goal_verdict, 'INCOMPLETE');
});

test('deadline and explicitly configured iteration limits prevent new work', async t => {
  const f = await fixture(t);
  f.cp.deadline_at = '2026-01-01T00:20:00Z';
  assert.equal((await f.check()).next, 'stop');
  assert.equal((await f.check()).reason, 'budget_exhausted');
  f.cp.deadline_at = '2026-01-01T02:00:00Z';
  f.cp.budget.candidate_limit = 8;
  assert.equal((await f.check()).reason, 'iteration_limit');
});

test('stop barriers persist across recovery and forbid a new operation', async t => {
  const f = await fixture(t);
  f.workflow.stop_reason = 'user_stopped';
  assert.equal((await f.check()).next, 'stop');
  const previous = await f.flush();
  f.workflow.revision++;
  f.workflow.stop_reason = null;
  await assert.rejects(f.check(previous), /Stop barrier/);
  f.workflow.stop_reason = 'user_stopped';
  f.workflow.operations.push({ id: 'new-op', task_id: 'search-fix', kind: 'implement',
    status: 'intended', executor_ref: null, result: null });
  await assert.rejects(f.check(previous), /stopped workflow/);
});

for (const mutation of ['goals', 'budget', 'deadline', 'tasks', 'best', 'status']) {
  test(`transition rejects ${mutation} history rewrite`, async t => {
    const f = await fixture(t);
    const previous = await f.flush();
    f.workflow.revision++;
    if (mutation === 'goals') f.workflow.goals.pop();
    if (mutation === 'budget') f.cp.budget.candidates_used = 0;
    if (mutation === 'deadline') f.cp.deadline_at = '2026-01-02T02:00:00Z';
    if (mutation === 'tasks') f.workflow.tasks.pop();
    if (mutation === 'best') {
      f.cp.candidate_manifest = await f.save({ source: 'unverified' });
      f.cp.versions.best = f.cp.candidate_manifest.sha256;
      f.cp.required_gates = [];
    }
    if (mutation === 'status') {
      f.workflow.tasks[0].status = 'done';
      f.workflow.tasks[0].evidence = f.raw;
    }
    await assert.rejects(f.check(previous));
  });
}

test('resource budget includes pending reservations, next step and final reserve', async t => {
  const f = await fixture(t);
  f.cp.budget.resources = [{ unit: 'tokens', limit: 1000, used: 600, reserved: 200,
    next_step_estimate: 150, handoff_reserve: 100 }];
  assert.equal((await f.check()).reason, 'budget_exhausted');
  f.cp.budget.resources[0].used = null;
  assert.equal((await f.check()).reason, 'budget_unknown');
});

test('unknown estimates cannot bypass an expired deadline or duplicate a live worker', async t => {
  const f = await fixture(t);
  f.cp.budget.next_step_estimate_seconds = null;
  f.cp.deadline_at = '2026-01-01T00:05:00Z';
  assert.equal((await f.check()).reason, 'budget_exhausted');
  f.cp.deadline_at = '2026-01-01T02:00:00Z';
  f.workflow.tasks[0].status = 'implementing';
  f.workflow.operations.push({ id: 'live', task_id: 'search-fix', kind: 'implement',
    status: 'running', executor_ref: 'worker', result: null });
  assert.equal((await f.check()).next, 'wait_or_recover');
});

test('successful task dispatch and completion preserve immutable receipts', async t => {
  const f = await fixture(t);
  let previous = await f.flush();
  f.workflow.revision++;
  f.workflow.tasks[0].status = 'implementing';
  f.workflow.operations.push({ id: 'worker', task_id: 'search-fix', kind: 'implement',
    status: 'intended', executor_ref: null, result: null });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
  previous = await f.flush();
  f.workflow.revision++;
  f.workflow.operations[0].status = 'running';
  f.workflow.operations[0].executor_ref = 'worker-session';
  await f.check(previous);
  previous = await f.flush();
  f.workflow.revision++;
  f.workflow.operations[0].status = 'completed';
  f.workflow.operations[0].result = f.raw;
  f.workflow.tasks[0].status = 'verifying';
  await f.check(previous);
  previous = await f.flush();
  f.workflow.revision++;
  f.workflow.tasks[0].status = 'done';
  f.workflow.tasks[0].evidence = f.raw;
  assert.deepEqual((await f.check(previous)).ready_tasks, ['read-fix']);
  previous = await f.flush();
  f.workflow.revision++;
  f.workflow.operations[0].result = await f.save({ changed: true });
  await assert.rejects(f.check(previous), /Completed operation changed/);
});

test('same candidate cannot lose required checks during recovery', async t => {
  const f = await fixture(t);
  const previous = await f.flush();
  f.workflow.revision++;
  f.cp.required_gates.pop();
  await assert.rejects(f.check(previous), /coverage removed/);
});

test('known spending cannot be erased through an unknown intermediate value', async t => {
  const f = await fixture(t);
  f.cp.budget.resources = [{ unit: 'tokens', limit: 10000, used: 600, reserved: 0,
    next_step_estimate: 100, handoff_reserve: 100 }];
  const previous = await f.flush();
  f.workflow.revision++;
  f.cp.budget.resources[0].used = null;
  await assert.rejects(f.check(previous), /Resource budget reset/);
});

test('dependency cycles, uncovered goals and malformed task states are rejected', async t => {
  for (const mutation of ['cycle', 'uncovered', 'status']) {
    const f = await fixture(t);
    if (mutation === 'cycle') {
      f.workflow.tasks[0].depends_on = ['read-fix'];
      f.workflow.tasks[1].depends_on = ['search-fix'];
    }
    if (mutation === 'uncovered') f.workflow.tasks.pop();
    if (mutation === 'status') f.workflow.tasks[0].status = 'success';
    await assert.rejects(f.check());
  }
});

test('CLI separates ready records, incomplete work and invalid evidence', async t => {
  const f = await fixture(t);
  const cli = new URL('./verify-workflow.mjs', import.meta.url);
  const invoke = async () => spawnSync(process.execPath, [fileURLToPath(cli), await f.flush()],
    { encoding: 'utf8', timeout: 5000 });
  assert.equal((await invoke()).status, 1);
  await f.finish();
  assert.equal((await invoke()).status, 0);
  await writeFile(path.join(f.root, f.workflow.contract_ref.path), 'changed');
  const invalid = await invoke();
  assert.equal(invalid.status, 2);
  assert.ok(!invalid.stderr.includes(f.root));
});

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

/** 模拟阶段 PR 的服务端回执；测试不访问 GitHub、不产生远端副作用。 */
async function remoteFixture(t) {
  const f = await fixture(t);
  f.workflow.delivery_mode = 'auto_merge';
  const task = f.workflow.tasks[0];
  task.status = 'pr_review';
  task.delivery = { kind: 'pr', repo: 'example/browser', pr: 1, base_branch: 'integration',
    base_sha: 'a'.repeat(40), head_sha: 'b'.repeat(40),
    review: null, merge: null, integration: null };
  f.workflow.tasks[1].depends_on = [task.id];
  const identity = () => Object.fromEntries(['repo', 'pr', 'base_branch', 'base_sha', 'head_sha']
    .map(key => [key, task.delivery[key]]));
  const receipt = fields => f.save({ schema_version: 1, task_id: task.id, artifacts: [f.raw], ...fields });
  async function validateCandidate() {
    task.delivery.candidate_manifest = await f.save({ source_sha: task.delivery.head_sha,
      base_sha: task.delivery.base_sha, dirty_patch: null });
    f.cp.candidate_manifest = task.delivery.candidate_manifest;
    f.cp.versions.best = f.cp.candidate_manifest.sha256;
    for (const gate of f.cp.required_gates) {
      const report = JSON.parse(await readFile(path.join(f.root, gate.result.path)));
      report.subject = f.cp.versions.best;
      gate.result = await f.save(report);
    }
  }
  await validateCandidate();
  async function review(fields = {}) {
    task.delivery.review = await receipt({ ...identity(), subject: task.delivery.candidate_manifest.sha256,
      verdict: 'PASS', open_findings: [],
      reviewer: { executor_ref: 'code-reviewer', independent: true }, ...fields });
  }
  async function merge() {
    task.delivery.merge = await receipt({ ...identity(), confirmed: true, commit: 'c'.repeat(40),
      review_sha256: task.delivery.review.sha256 });
  }
  async function integrate(status = 'PASS') {
    task.delivery.integration = await receipt({ status, commit: 'c'.repeat(40),
      merge_commit: 'c'.repeat(40), contains_merge: true, subject: f.cp.versions.best });
  }
  return { ...f, task, identity, receipt, validateCandidate, review, merge, integrate };
}

test('long-running default continues without resetting earlier explicit deadline', async t => {
  const f = await fixture(t);
  assert.equal(f.cp.deadline_at, null);
  assert.equal((await verifyWorkflow(await f.flush(), null,
    Date.parse('2026-02-01T00:00:00Z'))).next, 'continue');
  f.cp.deadline_at = '2026-01-01T02:00:00Z';
  const previous = await f.flush();
  f.workflow.revision++;
  f.cp.deadline_at = null;
  await assert.rejects(f.check(previous), /Budget identity changed/);
});

test('review findings lead to repair and fresh review of the same PR', async t => {
  const f = await remoteFixture(t);
  await f.review({ verdict: 'CHANGES_REQUIRED', open_findings: ['Navigation regression'] });
  assert.equal((await f.check()).next, 'continue');
  let previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'merging';
  await assert.rejects(f.check(previous), /independent review/);
  f.task.status = 'needs_fix';
  await f.check(previous);
  previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'implementing';
  await f.check(previous);
  previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'verifying';
  f.task.delivery.head_sha = 'd'.repeat(40);
  await f.check(previous);
  previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'pr_review';
  await f.validateCandidate();
  await f.review();
  await f.check(previous);
  previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'merging';
  f.workflow.operations.push({ id: 'merge-1', task_id: f.task.id, kind: 'merge',
    status: 'intended', executor_ref: null, result: null, subject: f.identity() });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
  assert.equal(f.task.delivery.pr, 1);
});

for (const change of ['head_sha', 'base_sha', 'implementer', 'controller', 'findings']) {
  test(`auto merge rejects stale or non-independent review: ${change}`, async t => {
    const f = await remoteFixture(t);
    await f.review();
    if (change.endsWith('_sha')) f.task.delivery[change] = 'd'.repeat(40);
    if (change === 'implementer') f.workflow.operations.push({ id: 'impl', task_id: f.task.id,
      kind: 'implement', status: 'completed', executor_ref: 'code-reviewer', result: f.raw });
    if (change === 'controller') await f.review({
      reviewer: { executor_ref: 'controller', independent: true } });
    if (change === 'findings') await f.review({ open_findings: ['Unfixed bug'] });
    f.task.status = 'merging';
    await assert.rejects(f.check(), /independent review/);
  });
}

test('unknown merge blocks dependencies, confirmed integration releases the next task', async t => {
  const f = await remoteFixture(t);
  await f.review();
  f.task.status = 'merging';
  f.workflow.operations.push({ id: 'merge', task_id: f.task.id, kind: 'merge', status: 'running',
    executor_ref: 'controller', result: null, subject: f.identity() });
  assert.equal((await f.check()).next, 'wait_or_recover');
  assert.deepEqual((await f.check()).ready_tasks, []);
  f.task.status = 'integrating';
  await assert.rejects(f.check(), /Confirmed merge/);
  await f.merge();
  f.workflow.operations[0].status = 'completed';
  f.workflow.operations[0].result = f.task.delivery.merge;
  assert.deepEqual((await f.check()).ready_tasks, [f.task.id]);
  let previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'done';
  f.task.evidence = f.raw;
  await assert.rejects(f.check(previous), /Passing integration/);
  await f.integrate();
  assert.deepEqual((await f.check(previous)).ready_tasks, ['read-fix']);
  previous = await f.flush();
  f.workflow.revision++;
  f.task.delivery.head_sha = 'e'.repeat(40);
  await assert.rejects(f.check(previous));
});

test('failed merged task gets a separate repair without dependency deadlock', async t => {
  const f = await remoteFixture(t);
  await f.review();
  await f.merge();
  await f.integrate('FAIL');
  f.task.status = 'integrating';
  f.task.pause_reason = 'Integration failed; repair needed';
  const previous = await f.flush();
  f.workflow.revision++;
  f.workflow.tasks.push({ id: 'repair', goal_ids: ['search'], depends_on: [],
    description: 'Repair merged navigation regression', status: 'pending', pause_reason: null,
    evidence: null, delivery: null, repairs_task_id: f.task.id });
  assert.deepEqual((await f.check(previous)).ready_tasks, ['repair']);
  const repair = f.workflow.tasks.at(-1);
  repair.depends_on = ['read-fix'];
  await assert.rejects(f.check(previous), /Repair cannot depend/);
  repair.depends_on = [];
  f.task.pause_reason = null;
  f.task.status = 'done';
  f.task.evidence = f.raw;
  await assert.rejects(f.check(previous), /Passing integration/);
});

test('remote mode cannot silently downgrade, and PR-only never permits merge dispatch', async t => {
  const f = await remoteFixture(t);
  const previous = await f.flush();
  f.workflow.revision++;
  f.workflow.delivery_mode = 'local';
  f.task.status = 'verifying';
  await assert.rejects(f.check(previous), /Workflow contract changed/);
  f.workflow.delivery_mode = 'pr_only';
  f.task.status = 'pr_review';
  await f.review();
  const prOnly = await f.flush();
  f.workflow.revision++;
  f.task.status = 'merging';
  await assert.rejects(f.check(prOnly), /auto-merge contract/);
  f.task.status = 'done';
  f.task.evidence = f.raw;
  assert.deepEqual((await f.check(prOnly)).ready_tasks, ['read-fix']);
});

test('user stop forbids merge, and merge intent must match exact reviewed identity', async t => {
  const f = await remoteFixture(t);
  await f.review();
  f.workflow.stop_reason = 'user_stopped';
  let previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'merging';
  f.workflow.operations.push({ id: 'merge', task_id: f.task.id, kind: 'merge',
    status: 'intended', executor_ref: null, result: null, subject: f.identity() });
  await assert.rejects(f.check(previous), /stopped workflow/);
  f.workflow.stop_reason = null;
  f.workflow.operations = [];
  previous = await f.flush();
  f.workflow.revision++;
  f.workflow.operations.push({ id: 'merge', task_id: f.task.id, kind: 'merge',
    status: 'intended', executor_ref: null, result: null,
    subject: { ...f.identity(), head_sha: 'e'.repeat(40) } });
  await assert.rejects(f.check(previous), /bind reviewed PR/);
});

test('merged tasks still require the full latest journey before stopping', async t => {
  const f = await remoteFixture(t);
  await f.review();
  await f.merge();
  await f.integrate();
  f.workflow.tasks[1].delivery = { kind: 'no_change', reason: 'Reading already works on integration' };
  await f.finish();
  assert.equal((await f.check()).reason, 'completed');
  const report = JSON.parse(await readFile(path.join(f.root, f.workflow.final_acceptance.path)));
  report.checks[1].status = 'FAIL';
  f.workflow.final_acceptance = await f.save(report);
  assert.equal((await f.check()).next, 'replan');
  assert.equal((await f.check()).goal_verdict, 'INCOMPLETE');
});

test('integration of a separate repair unblocks the original and its dependent', async t => {
  const f = await remoteFixture(t);
  await f.review();
  await f.merge();
  await f.integrate('FAIL');
  f.task.status = 'integrating';
  f.task.pause_reason = 'Repair in progress';
  const repair = { id: 'repair', goal_ids: ['search'], depends_on: [], description: 'Fix integration',
    status: 'pending', pause_reason: null, evidence: null, delivery: null, repairs_task_id: f.task.id };
  f.workflow.tasks.push(repair);
  async function advance(status) {
    const previous = await f.flush();
    f.workflow.revision++;
    repair.status = status;
    return previous;
  }
  await f.check(await advance('implementing'));
  await f.check(await advance('verifying'));
  let previous = await advance('pr_review');
  const identity = { ...f.identity(), pr: 2, base_sha: 'c'.repeat(40), head_sha: 'd'.repeat(40) };
  const receipt = fields => f.save({ schema_version: 1, task_id: repair.id, artifacts: [f.raw], ...fields });
  const manifest = await f.save({ source_sha: identity.head_sha, base_sha: identity.base_sha, dirty_patch: null });
  repair.delivery = { kind: 'pr', ...identity, candidate_manifest: manifest, review: await receipt({ ...identity,
    subject: manifest.sha256,
    verdict: 'PASS', open_findings: [], reviewer: { executor_ref: 'repair-reviewer', independent: true } }),
  merge: null, integration: null };
  await f.check(previous);
  await f.check(await advance('merging'));
  previous = await advance('integrating');
  repair.delivery.merge = await receipt({ ...identity, confirmed: true, commit: 'e'.repeat(40),
    review_sha256: repair.delivery.review.sha256 });
  await f.check(previous);
  previous = await advance('done');
  repair.evidence = f.raw;
  repair.delivery.integration = await receipt({ status: 'PASS', commit: 'e'.repeat(40),
    merge_commit: 'e'.repeat(40), contains_merge: true, subject: f.cp.versions.best });
  assert.equal((await f.check(previous)).next, 'replan');
  previous = await f.flush();
  f.workflow.revision++;
  f.task.pause_reason = null;
  f.task.status = 'done';
  f.task.evidence = f.raw;
  f.task.delivery.integration = await f.receipt({ status: 'PASS', commit: 'e'.repeat(40),
    merge_commit: 'c'.repeat(40), contains_merge: true, subject: f.cp.versions.best });
  assert.deepEqual((await f.check(previous)).ready_tasks, ['read-fix']);
});

test('merge gates, receipt identity and integration best cannot be substituted', async t => {
  const f = await remoteFixture(t);
  await f.review();
  let previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'merging';
  f.workflow.operations.push({ id: 'merge', task_id: f.task.id, kind: 'merge',
    status: 'intended', executor_ref: null, result: null, subject: f.identity() });
  const gate = f.cp.required_gates[1];
  const result = gate.result;
  gate.result = null;
  await assert.rejects(f.check(previous), /passing candidate gates/);
  gate.result = result;
  await f.merge();
  f.workflow.operations[0].status = 'completed';
  f.workflow.operations[0].result = f.task.delivery.merge;
  f.task.status = 'integrating';
  previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'done';
  f.task.evidence = f.raw;
  f.task.delivery.integration = await f.receipt({ status: 'PASS', commit: 'c'.repeat(40),
    merge_commit: 'c'.repeat(40), contains_merge: true, subject: 'd'.repeat(64) });
  await assert.rejects(f.check(previous), /verified integration best/);
  await f.integrate();
  const merge = JSON.parse(await readFile(path.join(f.root, f.task.delivery.merge.path)));
  merge.review_sha256 = 'e'.repeat(64);
  f.task.delivery.merge = await f.save(merge);
  await assert.rejects(f.check(previous), /Confirmed merge/);
});

test('regression: a newly reviewed PR head cannot reuse old candidate gates', async t => {
  const f = await remoteFixture(t);
  await f.review();
  const previous = await f.flush();
  f.workflow.revision++;
  f.task.delivery.head_sha = 'f'.repeat(40);
  f.task.delivery.candidate_manifest = await f.save({ source_sha: f.task.delivery.head_sha,
    base_sha: f.task.delivery.base_sha, dirty_patch: null });
  await f.review({ subject: f.task.delivery.candidate_manifest.sha256 });
  f.task.status = 'merging';
  f.workflow.operations.push({ id: 'new-head-merge', task_id: f.task.id, kind: 'merge',
    status: 'intended', executor_ref: null, result: null, subject: f.identity() });
  await assert.rejects(f.check(previous), /candidate.*PR|PR.*candidate/);
});

test('regression: final acceptance has a recoverable goal-level operation', async t => {
  const f = await fixture(t);
  await f.finish();
  const report = f.workflow.final_acceptance;
  f.workflow.final_acceptance = null;
  let previous = await f.flush();
  f.workflow.revision++;
  const op = { id: 'final-1', task_id: null, kind: 'final_acceptance', status: 'intended',
    executor_ref: null, result: null,
    subject: { manifest_sha256: f.cp.versions.best, contract_sha256: f.workflow.contract_ref.sha256 } };
  f.workflow.operations.push(op);
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
  previous = await f.flush();
  f.workflow.revision++;
  op.status = 'running';
  op.executor_ref = 'fresh-reviewer';
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
  previous = await f.flush();
  f.workflow.revision++;
  op.status = 'completed';
  op.result = report;
  f.workflow.final_acceptance = report;
  assert.equal((await f.check(previous)).reason, 'completed');
});

test('regression: empty version state can record an evidenced failing original once', async t => {
  const f = await fixture(t);
  const baseline = structuredClone(f.cp);
  f.cp.versions = { original: null, best: null, trial: null };
  f.cp.candidate_manifest = null;
  f.cp.required_gates = [];
  let previous = await f.flush();
  f.workflow.revision++;
  Object.assign(f.cp, baseline);
  const gate = f.cp.required_gates[0];
  const report = JSON.parse(await readFile(path.join(f.root, gate.result.path)));
  report.checks[0].status = 'FAIL';
  gate.result = await f.save(report);
  assert.equal((await f.check(previous)).delivery_verdict, 'not_ready');
  previous = await f.flush();
  f.workflow.revision++;
  f.cp.versions.original = 'f'.repeat(64);
  await assert.rejects(f.check(previous), /Original version changed/);
});

test('regression: infrastructure recovery preserves the contract and requires fresh evidence', async t => {
  const f = await fixture(t);
  f.workflow.stop_reason = 'infrastructure_blocked';
  const previous = await f.flush();
  const revision = f.workflow.revision++;
  f.workflow.stop_reason = null;
  await assert.rejects(f.check(previous), /recovery|Stop barrier/);
  f.workflow.recovery_ref = await f.save({ schema_version: 1, run_id: f.workflow.run_id,
    previous_revision: revision, resolved: true, checked_at: f.cp.updated_at, artifacts: [f.raw] });
  assert.equal((await f.check(previous)).next, 'continue');
  f.cp.budget.candidates_used = 0;
  await assert.rejects(f.check(previous), /counter decreased/);
});

test('regression: the last allowed candidate can still be reviewed and merged', async t => {
  const f = await remoteFixture(t);
  f.cp.budget.candidate_limit = f.cp.budget.candidates_used;
  await f.review();
  assert.deepEqual((await f.check()).ready_tasks, [f.task.id]);
  const previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'merging';
  f.workflow.operations.push({ id: 'last-merge', task_id: f.task.id, kind: 'merge',
    status: 'intended', executor_ref: null, result: null, subject: f.identity() });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
});

for (const mutation of ['base', 'dirty', 'review-subject']) {
  test(`PR evidence cannot substitute candidate identity: ${mutation}`, async t => {
    const f = await remoteFixture(t);
    if (mutation === 'base') f.task.delivery.base_sha = 'e'.repeat(40);
    if (mutation === 'dirty') {
      f.task.delivery.candidate_manifest = await f.save({ source_sha: f.task.delivery.head_sha,
        base_sha: f.task.delivery.base_sha, dirty_patch: 'e'.repeat(64) });
    }
    await f.review(mutation === 'review-subject' ? { subject: 'e'.repeat(64) } : {});
    f.task.status = 'merging';
    await assert.rejects(f.check(), /independent review/);
  });
}

test('final operation preserves failed evidence and rejects a substituted reviewer', async t => {
  const f = await fixture(t);
  await f.finish();
  const report = JSON.parse(await readFile(path.join(f.root, f.workflow.final_acceptance.path)));
  report.checks[0].status = 'FAIL';
  f.workflow.final_acceptance = await f.save(report);
  f.workflow.operations.push({ id: 'final', task_id: null, kind: 'final_acceptance', status: 'completed',
    executor_ref: report.reviewer.executor_ref, result: f.workflow.final_acceptance,
    subject: { manifest_sha256: f.cp.versions.best, contract_sha256: f.workflow.contract_ref.sha256 } });
  assert.equal((await f.check()).next, 'replan');
  f.workflow.operations[0].executor_ref = 'different-reviewer';
  await assert.rejects(f.check(), /operation result mismatch/);
});

test('new final evidence cannot skip recording the acceptance operation', async t => {
  const f = await fixture(t);
  await f.finish();
  const report = f.workflow.final_acceptance;
  f.workflow.final_acceptance = null;
  const previous = await f.flush();
  f.workflow.revision++;
  f.workflow.final_acceptance = report;
  await assert.rejects(f.check(previous), /requires a completed operation/);
});

for (const mutation of ['pending-task', 'stale-best', 'user-stop']) {
  test(`final acceptance cannot bypass its dispatch boundary: ${mutation}`, async t => {
    const f = await fixture(t);
    await f.finish();
    f.workflow.final_acceptance = null;
    if (mutation === 'pending-task') {
      f.workflow.tasks[1].status = 'pending';
      f.workflow.tasks[1].evidence = null;
    }
    if (mutation === 'user-stop') f.workflow.stop_reason = 'user_stopped';
    const previous = await f.flush();
    f.workflow.revision++;
    f.workflow.operations.push({ id: 'final', task_id: null, kind: 'final_acceptance', status: 'intended',
      executor_ref: null, result: null,
      subject: { manifest_sha256: mutation === 'stale-best' ? 'e'.repeat(64) : f.cp.versions.best,
        contract_sha256: f.workflow.contract_ref.sha256 } });
    await assert.rejects(f.check(previous), /not ready for dispatch|stopped workflow/);
  });
}

for (const reason of ['user_stopped', 'safety_blocked', 'infrastructure_blocked']) {
  test(`recovery evidence cannot clear unrelated or stale stop: ${reason}`, async t => {
    const f = await fixture(t);
    f.workflow.stop_reason = reason;
    f.workflow.revision = 2;
    const previous = await f.flush();
    f.workflow.revision++;
    f.workflow.stop_reason = null;
    f.workflow.recovery_ref = await f.save({ schema_version: 1, run_id: f.workflow.run_id,
      previous_revision: reason === 'infrastructure_blocked' ? 1 : 2,
      resolved: true, checked_at: f.cp.updated_at, artifacts: [f.raw] });
    await assert.rejects(f.check(previous), /Stop barrier/);
  });
}

test('iteration cap permits final acceptance but not another implementation or deadline overrun', async t => {
  const f = await fixture(t);
  f.cp.budget.candidate_limit = f.cp.budget.candidates_used;
  f.cp.budget.exploration_period_limit = f.cp.budget.exploration_periods_used;
  let previous = await f.flush();
  f.workflow.revision++;
  f.workflow.tasks[0].status = 'implementing';
  f.workflow.operations.push({ id: 'extra', task_id: f.workflow.tasks[0].id, kind: 'implement',
    status: 'intended', executor_ref: null, result: null });
  await assert.rejects(f.check(previous), /Dispatch budget/);
  f.workflow.operations = [];
  await f.finish();
  f.workflow.final_acceptance = null;
  assert.equal((await f.check()).next, 'final_acceptance');
  previous = await f.flush();
  f.workflow.revision++;
  f.workflow.operations.push({ id: 'final', task_id: null, kind: 'final_acceptance', status: 'intended',
    executor_ref: null, result: null,
    subject: { manifest_sha256: f.cp.versions.best, contract_sha256: f.workflow.contract_ref.sha256 } });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
  f.cp.deadline_at = '2026-01-01T00:05:00Z';
  assert.equal((await f.check()).reason, 'budget_exhausted');
});

import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { chmod, mkdtemp, readFile, realpath, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { verifyWorkflow } from './verify-workflow.mjs';
import { reviewSection } from './pr-body.mjs';
import { recoveryPacket } from './recover.mjs';

const now = Date.parse('2026-01-01T00:10:00Z');
const hash = data => createHash('sha256').update(data).digest('hex');

/** 构造独立的合成目标、候选证据与累计预算，不启动产品或访问网络。 */
async function fixture(t) {
  const root = await realpath(await mkdtemp(path.join(os.tmpdir(), 'zeroweb-workflow-')));
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

/** 保存合成双审查及正文，真实派发转换由专门的相邻状态测试覆盖。 */
async function recordReview(f, task, fields = {}) {
  const identity = Object.fromEntries(['repo', 'pr', 'base_branch', 'base_sha', 'head_sha']
    .map(key => [key, task.delivery[key]]));
  const common = { schema_version: 1, task_id: task.id, ...identity,
    candidate_manifest: task.delivery.candidate_manifest, subject: task.delivery.candidate_manifest.sha256,
    verdict: 'PASS', open_findings: [], artifacts: [f.raw] };
  const artifacts = [];
  for (const role of ['test_validity', 'defects']) {
    const report = { ...common, stage: 'first', role,
      reviewer: { executor_ref: `${task.id}-${role}`, independent: true } };
    const ref = await f.save(report);
    artifacts.push(ref);
    if (!f.workflow.operations.some(op => op.result?.sha256 === ref.sha256)) {
      f.workflow.operations.push({ id: `review-${f.workflow.operations.length}`, task_id: task.id,
        kind: 'review', status: 'completed', executor_ref: report.reviewer.executor_ref, result: ref });
    }
  }
  const summary = { ...common, stage: 'summary', review_scope: 'dual', artifacts,
    reviewer: { executor_ref: 'code-reviewer', independent: true }, ...fields };
  task.delivery.review = await f.save(summary);
  f.workflow.operations.push({ id: `summary-${f.workflow.operations.length}`, task_id: task.id,
    kind: 'review', status: 'completed', executor_ref: summary.reviewer.executor_ref, result: task.delivery.review });
  await recordPresentation(f, task);
}

/** 模拟已回读的无产品画面 PR，不用真实图片或网络为测试凑证据。 */
async function recordPresentation(f, task) {
  const body = ['## 解决了什么\n修复导航。', '## 怎么验证\n行为回归。',
    '## 还有什么问题\n无已知问题。', '## 优化前后截图\n纯合成测试，无产品画面。',
    await reviewSection(f.root, task, f.workflow)].join('\n\n');
  const name = `body-${randomUUID()}.md`;
  await writeFile(path.join(f.root, name), body);
  const bodyRef = { path: name, sha256: hash(body) };
  task.delivery.presentation = await f.save({ schema_version: 1, task_id: task.id,
    ...Object.fromEntries(['repo', 'pr', 'base_branch', 'base_sha', 'head_sha'].map(key => [key, task.delivery[key]])),
    review_sha256: task.delivery.review.sha256, body_ref: bodyRef, artifacts: [f.raw],
    screenshots: { status: 'not_applicable', reason: '纯合成测试，无产品画面。', evidence_ref: f.raw } });
}

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
    await recordReview(f, task, fields);
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

test('default seven-day wall clock includes downtime and preserves explicit old deadlines', async t => {
  const f = await fixture(t);
  assert.equal(Date.parse(f.cp.deadline_at) - Date.parse(f.cp.started_at), 7 * 86400000);
  assert.equal((await verifyWorkflow(await f.flush(), null,
    Date.parse('2026-01-04T00:00:00Z'))).next, 'continue');
  assert.equal((await verifyWorkflow(await f.flush(), null,
    Date.parse('2026-01-08T00:00:00Z'))).reason, 'budget_exhausted');
  f.cp.deadline_at = null;
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
  f.cp.budget.pr_iteration_limit = 1;
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
  await f.check();
  previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'merging';
  f.workflow.operations.push({ id: 'merge-1', task_id: f.task.id, kind: 'merge',
    status: 'intended', executor_ref: null, result: null, subject: f.identity() });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
  assert.equal(f.task.delivery.pr, 1);
  assert.equal((await f.check()).pr_iterations_used, 1);
});

test('PR cap blocks new work but allows implementation and publication within the same PR', async t => {
  const f = await remoteFixture(t);
  f.cp.budget.pr_iteration_limit = 1;
  f.task.status = 'needs_fix';
  f.workflow.tasks[1].depends_on = [];
  let result = await f.check();
  assert.equal(result.pr_iterations_used, 1);
  assert.equal(result.pr_iteration_available, false);
  assert.deepEqual(result.ready_tasks, [f.task.id]);
  let previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'implementing';
  f.workflow.operations.push({ id: 'repair', task_id: f.task.id, kind: 'implement',
    status: 'intended', executor_ref: null, result: null });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
  f.workflow.operations[0].status = 'completed';
  f.workflow.operations[0].executor_ref = 'worker';
  f.workflow.operations[0].result = f.raw;
  previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'verifying';
  f.workflow.operations.push({ id: 'update-pr', task_id: f.task.id, kind: 'publish',
    status: 'intended', executor_ref: null, result: null });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
});

for (const kind of ['implement', 'publish']) {
  test(`PR cap rejects dispatch for a new PR: ${kind}`, async t => {
    const f = await remoteFixture(t);
    f.cp.budget.pr_iteration_limit = 1;
    f.task.pause_reason = 'Waiting for review';
    const task = f.workflow.tasks[1];
    task.depends_on = [];
    task.status = kind === 'publish' ? 'verifying' : 'pending';
    assert.equal((await f.check()).reason, 'iteration_limit');
    const previous = await f.flush();
    f.workflow.revision++;
    if (kind === 'implement') task.status = 'implementing';
    f.workflow.operations.push({ id: 'new-pr', task_id: task.id, kind,
      status: 'intended', executor_ref: null, result: null });
    await assert.rejects(f.check(previous), /PR iteration limit/);
  });
}

test('distinct PRs count once and cannot exceed the cap during creation', async t => {
  const f = await remoteFixture(t);
  f.cp.budget.pr_iteration_limit = 1;
  f.task.status = 'verifying';
  const task = f.workflow.tasks[1];
  task.status = 'verifying';
  task.depends_on = [];
  const previous = await f.flush();
  f.workflow.revision++;
  task.delivery = { ...f.task.delivery, repo: 'EXAMPLE/browser' };
  assert.equal((await f.check(previous)).pr_iterations_used, 1);
  task.delivery.pr = 2;
  await assert.rejects(f.check(previous), /PR iteration limit/);
  f.cp.budget.pr_iteration_limit = 2;
  assert.equal((await f.check()).pr_iterations_used, 2);
});

for (const mutation of ['limit', 'pr', 'repo', 'remove']) {
  test(`PR iteration history cannot be reset: ${mutation}`, async t => {
    const f = await remoteFixture(t);
    f.cp.budget.pr_iteration_limit = 2;
    f.task.status = 'verifying';
    const previous = await f.flush();
    f.workflow.revision++;
    if (mutation === 'limit') f.cp.budget.pr_iteration_limit = null;
    if (mutation === 'pr') f.task.delivery.pr = 2;
    if (mutation === 'repo') f.task.delivery.repo = 'example/other';
    if (mutation === 'remove') f.task.delivery = null;
    await assert.rejects(f.check(previous), /Budget limit|PR identity|PR task/);
  });
}

test('local work has no PR iterations; old snapshots without a PR limit remain valid', async t => {
  const f = await fixture(t);
  f.cp.budget.pr_iteration_limit = 0;
  assert.equal((await f.check()).pr_iterations_used, 0);
  assert.equal((await f.check()).next, 'continue');
  delete f.cp.budget.pr_iteration_limit;
  assert.equal((await f.check()).pr_iteration_limit, null);
  f.cp.budget.pr_iteration_limit = -1;
  await assert.rejects(f.check(), /PR iteration limit/);
});

test('PR cap permits no-change validation and goal-level final acceptance', async t => {
  const f = await remoteFixture(t);
  f.cp.budget.pr_iteration_limit = 1;
  await f.review();
  await f.merge();
  await f.integrate();
  const task = f.workflow.tasks[1];
  task.delivery = { kind: 'no_change', reason: 'Reading already works' };
  task.depends_on = [];
  task.status = 'verifying';
  assert.ok((await f.check()).ready_tasks.includes(task.id));
  await f.finish();
  f.workflow.final_acceptance = null;
  assert.equal((await f.check()).next, 'final_acceptance');
  const previous = await f.flush();
  f.workflow.revision++;
  f.workflow.operations.push({ id: 'final', task_id: null, kind: 'final_acceptance', status: 'intended',
    executor_ref: null, result: null,
    subject: { manifest_sha256: f.cp.versions.best, contract_sha256: f.workflow.contract_ref.sha256 } });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
});

for (const change of ['head_sha', 'base_sha', 'implementer', 'controller', 'findings']) {
  test(`auto merge rejects stale or non-independent review: ${change}`, async t => {
    const f = await remoteFixture(t);
    await f.review();
    if (change.endsWith('_sha')) f.task.delivery[change] = 'd'.repeat(40);
    if (change === 'implementer') f.workflow.operations.unshift({ id: 'impl', task_id: f.task.id,
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
  f.workflow.operations.at(-1).status = 'completed';
  f.workflow.operations.at(-1).result = f.task.delivery.merge;
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
  f.workflow.operations.pop();
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
  repair.delivery = { kind: 'pr', ...identity, candidate_manifest: manifest,
    review: null, merge: null, integration: null };
  await recordReview(f, repair);
  await f.check();
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
  f.workflow.operations.at(-1).status = 'completed';
  f.workflow.operations.at(-1).result = f.task.delivery.merge;
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
  f.task.delivery.head_sha = 'f'.repeat(40);
  f.task.delivery.candidate_manifest = await f.save({ source_sha: f.task.delivery.head_sha,
    base_sha: f.task.delivery.base_sha, dirty_patch: null });
  await f.review({ subject: f.task.delivery.candidate_manifest.sha256 });
  const previous = await f.flush();
  f.workflow.revision++;
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

/** 新汇总另记终态，保留所有旧报告与操作；用于定向破坏证据链。 */
async function newSummary(f, report) {
  const id = `summary-${randomUUID()}`;
  f.task.delivery.review = await f.save({ ...report, operation_id: id });
  f.workflow.operations.push({ id, task_id: f.task.id,
    kind: 'review', status: 'completed', executor_ref: report.reviewer.executor_ref,
    result: f.task.delivery.review });
  await recordPresentation(f, f.task);
}

for (const mutation of ['missing-role', 'same-executor', 'missing-operation', 'different-candidate']) {
  test(`review completeness rejects ${mutation}`, async t => {
    const f = await remoteFixture(t);
    await f.review();
    const summary = JSON.parse(await readFile(path.join(f.root, f.task.delivery.review.path)));
    if (mutation === 'missing-role') summary.artifacts.pop();
    if (mutation === 'missing-operation') f.workflow.operations.shift();
    if (mutation === 'same-executor' || mutation === 'different-candidate') {
      const ref = summary.artifacts[1];
      const report = JSON.parse(await readFile(path.join(f.root, ref.path)));
      if (mutation === 'same-executor') report.reviewer.executor_ref = `${f.task.id}-test_validity`;
      else {
        report.head_sha = 'e'.repeat(40);
        report.candidate_manifest = await f.save({ source_sha: report.head_sha,
          base_sha: report.base_sha, dirty_patch: null });
        report.subject = report.candidate_manifest.sha256;
      }
      const replacement = await f.save(report);
      const op = f.workflow.operations.find(item => item.result?.sha256 === ref.sha256);
      op.result = replacement;
      op.executor_ref = report.reviewer.executor_ref;
      summary.artifacts[1] = replacement;
    }
    await newSummary(f, summary);
    f.task.status = 'merging';
    await assert.rejects(f.check(), /roles required|reviewers must differ|matching completed operation|candidates must match/);
  });
}

test('changed head retains first reports and requires non-author recheck of every patch', async t => {
  const f = await remoteFixture(t);
  await f.review();
  const summary = JSON.parse(await readFile(path.join(f.root, f.task.delivery.review.path)));
  const originalFirst = structuredClone(summary.artifacts);
  const from = { ...f.identity(), subject: summary.subject };
  f.workflow.operations.push({ id: 'repair', task_id: f.task.id, kind: 'implement',
    status: 'completed', executor_ref: 'patch-author', result: f.raw });
  f.task.delivery.head_sha = 'd'.repeat(40);
  await f.validateCandidate();
  Object.assign(summary, f.identity(), { subject: f.task.delivery.candidate_manifest.sha256 });
  await newSummary(f, summary);
  f.task.status = 'merging';
  await assert.rejects(f.check(), /Changed candidate requires recheck/);
  const recheck = { schema_version: 1, task_id: f.task.id, ...f.identity(),
    subject: summary.subject, candidate_manifest: f.task.delivery.candidate_manifest,
    stage: 'recheck', role: 'defects', from, covers_operations: ['repair'],
    reviewer: { executor_ref: 'patch-author', independent: true },
    verdict: 'PASS', open_findings: [], artifacts: [f.raw] };
  async function appendRecheck() {
    const ref = await f.save(recheck);
    f.workflow.operations.push({ id: `recheck-${f.workflow.operations.length}`, task_id: f.task.id,
      kind: 'review', status: 'completed', executor_ref: recheck.reviewer.executor_ref, result: ref });
    summary.artifacts = [...originalFirst, ref];
    await newSummary(f, summary);
  }
  await appendRecheck();
  await assert.rejects(f.check(), /non-author/);
  recheck.reviewer.executor_ref = 'non-author';
  recheck.covers_operations = [];
  await appendRecheck();
  await assert.rejects(f.check(), /Missing patch recheck/);
  recheck.covers_operations = ['repair'];
  await appendRecheck();
  assert.equal((await f.check()).next, 'continue');
  for (const ref of originalFirst) {
    const report = JSON.parse(await readFile(path.join(f.root, ref.path)));
    assert.equal(report.head_sha, 'b'.repeat(40));
  }
});

test('new repair can persist intent before new review exists', async t => {
  const f = await remoteFixture(t);
  await f.review();
  f.task.status = 'needs_fix';
  const previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'implementing';
  f.workflow.operations.push({ id: 'repair-intent', task_id: f.task.id, kind: 'implement',
    status: 'intended', executor_ref: null, result: null });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
  f.task.status = 'merging';
  await assert.rejects(f.check(), /independent review/);
});

test('editorial exemption still requires an independent completed summary and scope evidence', async t => {
  const f = await remoteFixture(t);
  await f.review();
  const summary = JSON.parse(await readFile(path.join(f.root, f.task.delivery.review.path)));
  summary.review_scope = 'editorial';
  summary.artifacts = [f.raw];
  summary.scope_reason = 'Spelling only';
  await newSummary(f, summary);
  f.task.status = 'merging';
  await assert.rejects(f.check(), /scope evidence/);
  summary.scope_evidence = f.raw;
  await newSummary(f, summary);
  assert.equal((await f.check()).next, 'continue');
});

test('reviewed PR cannot merge or finish PR-only without current presentation', async t => {
  const f = await remoteFixture(t);
  await f.review();
  f.task.delivery.presentation = null;
  f.task.status = 'merging';
  await assert.rejects(f.check(), /presentation required/);
  f.workflow.delivery_mode = 'pr_only';
  f.task.status = 'done';
  f.task.evidence = f.raw;
  await assert.rejects(f.check(), /presentation required/);
});

test('recovery reloads stage rules while preserving stop, deadline and unknown merge', async t => {
  const f = await remoteFixture(t);
  await f.review();
  f.task.status = 'merging';
  f.workflow.stop_reason = 'user_stopped';
  f.workflow.operations.push({ id: 'merge-unknown', task_id: f.task.id, kind: 'merge',
    status: 'running', executor_ref: 'controller', result: null, subject: f.identity() });
  const packet = await recoveryPacket(await f.flush(), null, now);
  assert.equal(packet.stop_reason, 'user_stopped');
  assert.equal(packet.deadline_at, f.cp.deadline_at);
  assert.equal(packet.verdict.next, 'stop');
  assert.equal(packet.unresolved_operations[0].id, 'merge-unknown');
  assert.ok(packet.required_reads.some(item => item.path.endsWith('github-delivery.md') && item.sha256));
  f.workflow.revision++;
  const missingPrevious = await recoveryPacket(await f.flush(), null, now);
  assert.equal(missingPrevious.verdict.next, 'repair_records');
  assert.equal(missingPrevious.stop_reason, 'user_stopped');
});

test('operation chronology cannot be rewritten across snapshots', async t => {
  const f = await remoteFixture(t);
  await f.review();
  const previous = await f.flush();
  f.workflow.revision++;
  [f.workflow.operations[0], f.workflow.operations[1]] = [f.workflow.operations[1], f.workflow.operations[0]];
  await recordPresentation(f, f.task);
  await assert.rejects(f.check(previous), /Operation order changed/);
});

test('dual review advances through persisted intents, results, presentation and merge intent', async t => {
  const f = await remoteFixture(t);
  await f.review();
  const operations = structuredClone(f.workflow.operations);
  const review = f.task.delivery.review;
  f.workflow.operations = [];
  f.task.delivery.review = null;
  f.task.delivery.presentation = null;
  for (const completed of operations) {
    let previous = await f.flush();
    f.workflow.revision++;
    const op = { ...completed, status: 'intended', executor_ref: null, result: null };
    f.workflow.operations.push(op);
    assert.equal((await f.check(previous)).next, 'wait_or_recover');
    previous = await f.flush();
    f.workflow.revision++;
    op.status = 'running';
    op.executor_ref = completed.executor_ref;
    await f.check(previous);
    previous = await f.flush();
    f.workflow.revision++;
    Object.assign(op, completed);
    if (completed.result.sha256 === review.sha256) {
      f.task.delivery.review = review;
      await recordPresentation(f, f.task);
    }
    await f.check(previous);
  }
  const previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'merging';
  f.workflow.operations.push({ id: 'merge', task_id: f.task.id, kind: 'merge',
    status: 'intended', executor_ref: null, result: null, subject: f.identity() });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
});

test('delivery CLI reads remote body, rejects edits and checks stop before querying',
  { skip: process.platform === 'win32' ? 'Fake gh uses a POSIX shell' : false }, async t => {
  const f = await remoteFixture(t);
  await f.review();
  f.cp.deadline_at = null;
  const presentation = JSON.parse(await readFile(path.join(f.root, f.task.delivery.presentation.path)));
  const body = await readFile(path.join(f.root, presentation.body_ref.path), 'utf8');
  const remote = { number: f.task.delivery.pr, baseRefName: f.task.delivery.base_branch,
    headRefOid: f.task.delivery.head_sha, baseRefOid: f.task.delivery.base_sha,
    state: 'OPEN', isDraft: true, body };
  const remoteFile = path.join(f.root, 'remote.json');
  const marker = path.join(f.root, 'queried');
  const gh = path.join(f.root, 'gh');
  // 仅测试子进程使用这个假 gh，不访问网络、不替换系统工具。
  await writeFile(gh, '#!/bin/sh\nprintf queried > "$QUERY_MARKER"\ncat "$FAKE_PR"\n');
  await chmod(gh, 0o700);
  const cli = fileURLToPath(new URL('./delivery-check.mjs', import.meta.url));
  async function invoke() {
    await writeFile(remoteFile, JSON.stringify(remote));
    return spawnSync(process.execPath, [cli, 'ready', await f.flush(), f.task.id],
      { encoding: 'utf8', timeout: 5000, env: { ...process.env,
        PATH: `${f.root}${path.delimiter}${process.env.PATH}`,
        FAKE_PR: remoteFile, QUERY_MARKER: marker } });
  }
  assert.equal((await invoke()).status, 0);
  remote.body += '\nRemote edit\n';
  assert.equal((await invoke()).status, 2);
  await rm(marker);
  f.workflow.stop_reason = 'user_stopped';
  assert.equal((await invoke()).status, 2);
  await assert.rejects(readFile(marker), { code: 'ENOENT' });
  const render = spawnSync(process.execPath, [cli, 'render', await f.flush(), f.task.id],
    { encoding: 'utf8', timeout: 5000 });
  assert.equal(render.status, 0);
  assert.ok(render.stdout.includes('测试有效性首轮 | PASS'));
  assert.ok(!render.stdout.includes('{{用 delivery-check'));
});

test('review-stage body update can persist a publish operation without reopening implementation', async t => {
  const f = await remoteFixture(t);
  await f.review();
  const previous = await f.flush();
  f.workflow.revision++;
  f.task.delivery.presentation = null;
  f.workflow.operations.push({ id: 'publish-review', task_id: f.task.id, kind: 'publish',
    status: 'intended', executor_ref: null, result: null });
  assert.equal((await f.check(previous)).next, 'wait_or_recover');
});

test('a new summary invalidates old presentation without blocking its publication', async t => {
  const f = await remoteFixture(t);
  await f.review();
  const presentation = f.task.delivery.presentation;
  const report = JSON.parse(await readFile(path.join(f.root, f.task.delivery.review.path)));
  await newSummary(f, report);
  f.task.delivery.presentation = presentation;
  assert.equal((await f.check()).next, 'continue');
  f.task.status = 'merging';
  await assert.rejects(f.check(), /presentation required/);
});

/** 还原升级前的汇总结构；旧原始证据原样保留，不伪造新双审查。 */
async function legacyReview(f) {
  f.task.delivery.review = await f.receipt({ ...f.identity(), subject: f.task.delivery.candidate_manifest.sha256,
    verdict: 'PASS', open_findings: [], reviewer: { executor_ref: 'legacy-reviewer', independent: true } });
  delete f.task.delivery.presentation;
}

for (const stage of ['done', 'integrating']) {
  test(`legacy ${stage} evidence remains historical across upgrade`, async t => {
    const f = await remoteFixture(t);
    await legacyReview(f);
    await f.merge();
    await f.integrate();
    f.task.status = stage;
    if (stage === 'done') f.task.evidence = f.raw;
    const previous = await f.flush();
    f.workflow.revision++;
    if (stage === 'integrating') {
      f.task.status = 'done';
      f.task.evidence = f.raw;
    }
    const result = await f.check(previous);
    assert.deepEqual(result.legacy_delivery_tasks, [f.task.id]);
    assert.ok(result.ready_tasks.includes('read-fix'));
    // 不能在没有历史参照时凭旧格式认定新交付已完成。
    await assert.rejects(f.check());
  });
}

test('legacy active review can be cleared for upgrade but cannot authorize a new merge', async t => {
  const f = await remoteFixture(t);
  await legacyReview(f);
  const previous = await f.flush();
  f.workflow.revision++;
  f.task.status = 'merging';
  await assert.rejects(f.check(previous));
  f.task.status = 'pr_review';
  f.task.delivery.review = null;
  assert.equal((await f.check(previous)).next, 'continue');
});

test('review summary permits original Markdown attachments beside role reports', async t => {
  const f = await remoteFixture(t);
  await f.review();
  const report = JSON.parse(await readFile(path.join(f.root, f.task.delivery.review.path)));
  const content = '# Original review notes\nObserved behavior and evidence.\n';
  await writeFile(path.join(f.root, 'notes.md'), content);
  report.artifacts.push({ path: 'notes.md', sha256: hash(content) });
  await newSummary(f, report);
  f.task.status = 'merging';
  assert.equal((await f.check()).next, 'continue');
});

test('visible history does not impersonate a pending current summary', async t => {
  const f = await remoteFixture(t);
  await f.review();
  const originalReview = f.task.delivery.review;
  f.task.delivery.review = null;
  let body = await reviewSection(f.root, f.task, f.workflow);
  assert.ok(body.includes('PASS（历史报告）'));
  assert.ok(body.includes('| 汇总 | 待执行 |'));
  assert.ok(body.includes('未解决阻断项：未汇总'));
  f.task.delivery.review = originalReview;
  f.task.delivery.head_sha = 'f'.repeat(40);
  body = await reviewSection(f.root, f.task, f.workflow);
  assert.ok(body.includes('| 汇总 | 待执行 |'));
  assert.ok(body.includes('未解决阻断项：未汇总'));
});

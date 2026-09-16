import path from 'node:path';
import { realpath } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { evidence, readJson, verifyRun } from './verify-run.mjs';

const text = value => typeof value === 'string' && value.trim().length > 0;
const equal = (left, right) => JSON.stringify(left) === JSON.stringify(right);
const stages = {
  pending: ['pending', 'implementing'],
  implementing: ['implementing', 'verifying'],
  verifying: ['verifying', 'needs_fix', 'done'],
  needs_fix: ['needs_fix', 'implementing'],
  done: ['done'],
};
const stops = ['completed', 'budget_exhausted', 'iteration_limit', 'infrastructure_blocked',
  'safety_blocked', 'user_stopped'];
function requireValue(condition, message) {
  if (!condition) throw new Error(message);
}
function unique(items, label) {
  requireValue(Array.isArray(items) && items.every(item => item && text(item.id))
    && new Set(items.map(item => item.id)).size === items.length, `Invalid ${label}`);
}
function strings(items) {
  return Array.isArray(items) && items.every(text) && new Set(items).size === items.length;
}

/** 读取不可变检查点引用，复用候选检查器的路径、摘要和门禁核验。 */
async function load(file, now) {
  const root = path.dirname(await realpath(file));
  const state = await readJson(file);
  requireValue(state.schema_version === 1 && text(state.run_id)
    && Number.isSafeInteger(state.revision) && state.revision > 0, 'Invalid workflow identity');
  await evidence(root, state.contract_ref);
  const checkpointFile = await evidence(root, state.checkpoint_ref);
  const checkpoint = await readJson(checkpointFile);
  requireValue(checkpoint.run_id === state.run_id, 'Checkpoint run mismatch');
  // 运行证据以同一根解析，避免两份账本或嵌套目录产生不同含义。
  requireValue(path.dirname(checkpointFile) === root, 'Checkpoint must share workflow root');
  const gates = await verifyRun(checkpointFile, now);
  requireValue(['independent', 'deterministic'].includes(state.verification_mode), 'Invalid verification mode');
  requireValue(state.stop_reason === null || stops.includes(state.stop_reason), 'Invalid stop reason');
  unique(state.goals, 'goals');
  requireValue(state.goals.length > 0 && state.goals.every(goal =>
    text(goal.description) && text(goal.verification)), 'Missing goal acceptance');
  unique(state.tasks, 'tasks');
  const tasks = new Map(state.tasks.map(task => [task.id, task]));
  const goalIds = state.goals.map(goal => goal.id);
  for (const task of state.tasks) {
    requireValue(Object.hasOwn(stages, task.status) && text(task.description)
      && strings(task.goal_ids) && task.goal_ids.length > 0
      && task.goal_ids.every(id => goalIds.includes(id))
      && strings(task.depends_on) && task.depends_on.every(id => tasks.has(id) && id !== task.id)
      && (task.pause_reason === null || text(task.pause_reason)), 'Invalid task');
    requireValue(task.status !== 'done' || (task.evidence !== null && task.pause_reason === null),
      'Done task requires evidence');
    if (task.evidence !== null) await evidence(root, task.evidence);
  }
  requireValue(goalIds.every(id => state.tasks.some(task => task.goal_ids.includes(id))), 'Uncovered goal');
  // 逐节点 DFS 拒绝循环依赖；暂停的前置项不会被当成完成。
  const visited = new Set();
  const visiting = new Set();
  function visit(id) {
    requireValue(!visiting.has(id), 'Dependency cycle');
    if (visited.has(id)) return;
    visiting.add(id);
    tasks.get(id).depends_on.forEach(visit);
    visiting.delete(id);
    visited.add(id);
  }
  tasks.forEach(task => visit(task.id));
  unique(state.operations, 'operations');
  for (const operation of state.operations) {
    requireValue(tasks.has(operation.task_id)
      && ['implement', 'review'].includes(operation.kind)
      && ['intended', 'running', 'completed'].includes(operation.status)
      && (operation.executor_ref === null || text(operation.executor_ref))
      && (operation.status !== 'running' || text(operation.executor_ref))
      && (operation.status !== 'completed' || operation.result !== null), 'Invalid operation');
    if (operation.result !== null) await evidence(root, operation.result);
  }
  requireValue(state.operations.filter(op => op.status !== 'completed').length <= 1,
    'More than one unresolved operation');
  let finalPassed = false;
  let finalCurrent = false;
  if (state.final_acceptance !== null) {
    const report = await readJson(await evidence(root, state.final_acceptance));
    unique(report.checks, 'final acceptance checks');
    requireValue(report.schema_version === 1 && report.checks.every(check =>
      ['PASS', 'FAIL', 'INCONCLUSIVE', 'SKIPPED'].includes(check.status))
      && report.reviewer && text(report.reviewer.executor_ref)
      && typeof report.reviewer.independent === 'boolean'
      && Array.isArray(report.artifacts) && report.artifacts.length > 0, 'Invalid final acceptance');
    for (const ref of report.artifacts) await evidence(root, ref);
    finalCurrent = report.subject === checkpoint.versions.best
      && report.contract_sha256 === state.contract_ref.sha256
      && report.checks.length === goalIds.length && report.checks.every(check => goalIds.includes(check.id));
    const implementers = state.operations.filter(op => op.kind === 'implement').map(op => op.executor_ref);
    const independent = report.reviewer.independent
      && report.reviewer.executor_ref !== checkpoint.activity.executor?.ref
      && !implementers.includes(report.reviewer.executor_ref);
    finalPassed = finalCurrent && report.checks.every(check => check.status === 'PASS')
      && (state.verification_mode === 'deterministic' || independent);
  }
  return { root, state, checkpoint, gates, finalPassed, finalCurrent };
}

/** 校验相邻快照的不可回退历史；预算/范围扩展须先核验单独的授权修订。 */
function transition(previous, current) {
  const a = previous.state;
  const b = current.state;
  const before = previous.checkpoint;
  const after = current.checkpoint;
  requireValue(a.run_id === b.run_id && b.revision === a.revision + 1
    && equal(a.contract_ref, b.contract_ref) && equal(a.goals, b.goals)
    && a.verification_mode === b.verification_mode, 'Workflow contract changed');
  requireValue(a.stop_reason === null || a.stop_reason === b.stop_reason, 'Stop barrier removed');
  requireValue(before.started_at === after.started_at && before.deadline_at === after.deadline_at
    && before.delivery_scope === after.delivery_scope
    && Date.parse(after.updated_at) >= Date.parse(before.updated_at), 'Budget identity changed');
  for (const key of ['candidate_limit', 'exploration_period_limit']) {
    requireValue(before.budget[key] === after.budget[key], 'Budget limit changed');
  }
  for (const key of ['candidates_used', 'exploration_periods_used']) {
    requireValue(after.budget[key] >= before.budget[key], 'Budget counter decreased');
  }
  const oldResources = before.budget.resources ?? [];
  const resources = after.budget.resources ?? [];
  requireValue(oldResources.length === resources.length, 'Resource budget removed');
  for (const old of oldResources) {
    const value = resources.find(item => item.unit === old.unit);
    requireValue(value && value.limit === old.limit
      && (old.used === null || (value.used !== null && value.used >= old.used)), 'Resource budget reset');
  }
  requireValue(before.versions.original === after.versions.original, 'Original version changed');
  if (before.candidate_manifest?.sha256 === after.candidate_manifest?.sha256) {
    for (const old of before.required_gates) {
      const gate = after.required_gates.find(item => item.id === old.id);
      requireValue(gate && gate.kind === old.kind && old.checks.every(id => gate.checks.includes(id)),
        'Required gate coverage removed');
    }
  }
  if (before.versions.best !== after.versions.best) {
    requireValue(after.versions.best !== null
      && after.candidate_manifest?.sha256 === after.versions.best
      && current.gates.delivery_verdict === 'ready', 'Unverified best promotion');
  }
  for (const old of a.tasks) {
    const task = b.tasks.find(item => item.id === old.id);
    requireValue(task && equal(old.goal_ids, task.goal_ids) && old.description === task.description,
      'Task history removed or rewritten');
    requireValue(stages[old.status].includes(task.status), 'Invalid task transition');
    requireValue(old.status !== 'done' || equal(old, task), 'Completed task changed');
    requireValue(old.status === 'pending' || equal(old.depends_on, task.depends_on),
      'Active dependencies changed');
  }
  for (const task of b.tasks.filter(task => !a.tasks.some(old => old.id === task.id))) {
    requireValue(task.status === 'pending', 'New task must be pending');
  }
  for (const old of a.operations) {
    const op = b.operations.find(item => item.id === old.id);
    requireValue(op && op.task_id === old.task_id && op.kind === old.kind
      && (old.executor_ref === null || old.executor_ref === op.executor_ref)
      && ['intended', 'running', 'completed'].indexOf(op.status)
        >= ['intended', 'running', 'completed'].indexOf(old.status), 'Operation history changed');
    requireValue(old.status !== 'completed' || equal(old, op), 'Completed operation changed');
  }
  const added = b.operations.filter(op => !a.operations.some(old => old.id === op.id));
  requireValue(added.length <= 1, 'Multiple new operations');
  if (added.length) {
    requireValue(a.stop_reason === null && b.stop_reason === null, 'Cannot dispatch in stopped workflow');
    requireValue(!a.operations.some(op => op.status !== 'completed'), 'Recover unresolved operation first');
    requireValue(current.gates.can_start_candidate, 'Dispatch budget or activity unavailable');
    const op = added[0];
    const task = b.tasks.find(item => item.id === op.task_id);
    requireValue(op.status === 'intended' && op.executor_ref === null && op.result === null,
      'Persist dispatch intent before execution');
    requireValue(task.pause_reason === null
      && task.status === (op.kind === 'implement' ? 'implementing' : 'verifying')
      && task.depends_on.every(id => b.tasks.find(item => item.id === id).status === 'done'),
    'Task is not ready for dispatch');
  }
}

/** 只读判断目标证据及下一阶段；不启动任务、不查询宿主、不授予执行权限。 */
export async function verifyWorkflow(file, previousFile = null, now = Date.now()) {
  const current = await load(file, now);
  if (previousFile !== null) transition(await load(previousFile, now), current);
  const { state, checkpoint, gates, finalPassed, finalCurrent } = current;
  const unresolved = state.operations.some(op => op.status !== 'completed');
  const completed = state.tasks.every(task => task.status === 'done')
    && finalPassed && !unresolved && checkpoint.versions.trial === null
    && checkpoint.candidate_manifest?.sha256 === checkpoint.versions.best
    && gates.delivery_verdict === 'ready';
  requireValue(state.stop_reason !== 'completed' || completed, 'Missing completion evidence');
  const ready = state.tasks.filter(task => task.status !== 'done' && task.pause_reason === null
    && task.depends_on.every(id => state.tasks.find(item => item.id === id).status === 'done'));
  let next;
  let reason = state.stop_reason;
  // 停止屏障先于恢复/派发；停止后仅允许收回已有操作并保存真实终态。
  if (reason !== null) next = 'stop';
  else if (completed) { next = 'stop'; reason = 'completed'; }
  else if (gates.remaining_seconds === 0 || (gates.budget_known && !gates.budget_available)) {
    next = 'stop'; reason = 'budget_exhausted';
  }
  else if (!gates.iteration_available) { next = 'stop'; reason = 'iteration_limit'; }
  else if (unresolved) next = 'wait_or_recover';
  else if (!gates.budget_known) { next = 'estimate_budget'; reason = 'budget_unknown'; }
  else if (checkpoint.activity.state !== 'running') next = 'verify_executor';
  else if (ready.length) next = 'continue';
  else if (state.tasks.every(task => task.status === 'done') && !finalCurrent) next = 'final_acceptance';
  else next = 'replan';
  return {
    schema_version: 1, run_id: state.run_id, revision: state.revision,
    goal_verdict: completed ? 'PASS' : 'INCOMPLETE', next, reason,
    ready_tasks: next === 'continue' ? ready.map(task => task.id) : [],
    live_verified: false, delivery_verdict: gates.delivery_verdict,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  if (![3, 4].includes(process.argv.length)) {
    console.error('Usage: node verify-workflow.mjs <workflow.json> [previous-workflow.json]');
    process.exitCode = 2;
  } else {
    try {
      const result = await verifyWorkflow(process.argv[2], process.argv[3] ?? null);
      console.log(JSON.stringify(result, null, 2));
      process.exitCode = result.goal_verdict === 'PASS' ? 0 : 1;
    } catch {
      console.error('Invalid workflow, transition or evidence; inspect local records.');
      process.exitCode = 2;
    }
  }
}

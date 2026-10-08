import path from 'node:path';
import { realpath } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { evidence, readJson, verifyRun } from './verify-run.mjs';
import { deliveryIdentity, verifyDelivery } from './verify-delivery.mjs';

const text = value => typeof value === 'string' && value.trim().length > 0;
const digest = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
const equal = (left, right) => JSON.stringify(left) === JSON.stringify(right);
const stages = {
  pending: ['pending', 'implementing'],
  implementing: ['implementing', 'verifying'],
  verifying: ['verifying', 'needs_fix', 'pr_review', 'done'],
  needs_fix: ['needs_fix', 'implementing'],
  pr_review: ['pr_review', 'needs_fix', 'merging', 'done'],
  merging: ['merging', 'pr_review', 'integrating'],
  integrating: ['integrating', 'done'],
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

/** PR 以仓库和编号去重；同一 PR 的候选、提交和返修不重复计次。 */
function prIdentity(delivery) {
  return `${delivery.repo.toLowerCase()}#${delivery.pr}`;
}

/** 达到 PR 上限后仍允许已有 PR 收尾和无需代码改动的验证。 */
function withinPrBudget(current, task) {
  return (current.state.delivery_mode ?? 'local') === 'local'
    || current.prAvailable || ['pr', 'no_change'].includes(task?.delivery?.kind);
}

/** 读取不可变检查点引用，复用候选检查器的路径、摘要和门禁核验。 */
async function load(file, now, previous = null, readingPrevious = false) {
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
  // 旧快照未声明远端模式时保留本地语义，不能迁移成新增远端授权。
  const deliveryMode = state.delivery_mode ?? 'local';
  requireValue(['local', 'pr_only', 'auto_merge'].includes(deliveryMode), 'Invalid delivery mode');
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
    requireValue(!['merging', 'integrating'].includes(task.status) || deliveryMode === 'auto_merge',
      'Merge stage requires auto-merge contract');
    requireValue(task.status !== 'pr_review' || deliveryMode !== 'local',
      'PR stage requires remote contract');
    requireValue(task.repairs_task_id == null || (tasks.has(task.repairs_task_id)
      && task.repairs_task_id !== task.id), 'Invalid repair target');
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
  function dependsOn(id, target) {
    return tasks.get(id).depends_on.some(parent => parent === target || dependsOn(parent, target));
  }
  for (const task of state.tasks) {
    requireValue(task.repairs_task_id == null || !dependsOn(task.id, task.repairs_task_id),
      'Repair cannot depend on the blocked original task');
  }
  unique(state.operations, 'operations');
  for (const operation of state.operations) {
    const final = operation.kind === 'final_acceptance';
    requireValue((final ? operation.task_id === null : tasks.has(operation.task_id))
      && ['implement', 'review', 'publish', 'merge', 'integrate', 'final_acceptance'].includes(operation.kind)
      && ['intended', 'running', 'completed'].includes(operation.status)
      && (operation.executor_ref === null || text(operation.executor_ref))
      && (operation.status !== 'running' || text(operation.executor_ref))
      && (operation.status !== 'completed' || operation.result !== null), 'Invalid operation');
    if (operation.result !== null) await evidence(root, operation.result);
    if (final) {
      requireValue(digest(operation.subject?.manifest_sha256)
        && digest(operation.subject?.contract_sha256), 'Invalid final acceptance subject');
      if (operation.status === 'completed') {
        const report = await readJson(await evidence(root, operation.result));
        requireValue(report.subject === operation.subject.manifest_sha256
          && report.contract_sha256 === operation.subject.contract_sha256
          && text(operation.executor_ref) && report.reviewer?.executor_ref === operation.executor_ref,
        'Final acceptance operation result mismatch');
      }
    }
  }
  requireValue(state.operations.filter(op => op.status !== 'completed').length <= 1,
    'More than one unresolved operation');
  const deliveries = new Map();
  if (deliveryMode !== 'local') {
    for (const task of state.tasks) {
      const old = previous?.state.tasks.find(item => item.id === task.id);
      // 已完成或已经合并的旧任务保持历史含义；升级不能为新的合并补造授权。
      const historical = old && ((old.status === 'done' && equal(old, task))
        || (previous.deliveries.get(task.id)?.merged
          && equal(old.delivery.review, task.delivery?.review)
          && equal(old.delivery.merge, task.delivery?.merge)
          && task.delivery?.kind === 'pr'
          && equal(deliveryIdentity(old.delivery), deliveryIdentity(task.delivery))));
      deliveries.set(task.id, await verifyDelivery(root, task, state, checkpoint, readingPrevious || historical));
    }
  }
  const prUsed = deliveryMode === 'local' ? 0 : new Set(state.tasks
    .filter(task => task.delivery?.kind === 'pr').map(task => prIdentity(task.delivery))).size;
  const prLimit = checkpoint.budget.pr_iteration_limit ?? null;
  const prAvailable = prLimit === null || prUsed < prLimit;
  let recovery = null;
  if (state.recovery_ref != null) {
    recovery = await readJson(await evidence(root, state.recovery_ref));
    requireValue(recovery.schema_version === 1 && recovery.run_id === state.run_id
      && Number.isSafeInteger(recovery.previous_revision) && recovery.previous_revision > 0
      && recovery.resolved === true && typeof recovery.checked_at === 'string'
      && /(?:Z|[+-]\d\d:\d\d)$/.test(recovery.checked_at)
      && Number.isFinite(Date.parse(recovery.checked_at))
      && Array.isArray(recovery.artifacts) && recovery.artifacts.length > 0, 'Invalid recovery evidence');
    for (const ref of recovery.artifacts) await evidence(root, ref);
  }
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
    const finalOperations = state.operations.filter(op => op.kind === 'final_acceptance');
    const operationMatches = finalOperations.length === 0 || finalOperations.some(op =>
      op.status === 'completed' && op.result?.sha256 === state.final_acceptance.sha256
      && op.subject.manifest_sha256 === report.subject
      && op.subject.contract_sha256 === report.contract_sha256
      && op.executor_ref === report.reviewer.executor_ref);
    finalPassed = finalCurrent && report.checks.every(check => check.status === 'PASS')
      && operationMatches && (state.verification_mode === 'deterministic' || independent);
  }
  return { root, state, checkpoint, gates, finalPassed, finalCurrent, deliveries, recovery,
    prUsed, prLimit, prAvailable };
}

/** 校验相邻快照的不可回退历史；预算/范围扩展须先核验单独的授权修订。 */
function transition(previous, current) {
  const a = previous.state;
  const b = current.state;
  const before = previous.checkpoint;
  const after = current.checkpoint;
  requireValue(a.run_id === b.run_id && b.revision === a.revision + 1
    && equal(a.contract_ref, b.contract_ref) && equal(a.goals, b.goals)
    && a.verification_mode === b.verification_mode
    && (a.delivery_mode ?? 'local') === (b.delivery_mode ?? 'local'), 'Workflow contract changed');
  // 基础设施恢复是单独的状态转换；人工/安全停止仍须明确授权修订。
  const recovered = a.stop_reason === 'infrastructure_blocked' && b.stop_reason === null
    && current.recovery?.previous_revision === a.revision
    && !equal(a.recovery_ref, b.recovery_ref)
    && Date.parse(current.recovery.checked_at) >= Date.parse(before.updated_at)
    && Date.parse(current.recovery.checked_at) <= Date.parse(after.updated_at)
    && after.activity.state === 'running' && current.gates.budget_available
    && b.operations.every(op => op.status === 'completed');
  requireValue(a.stop_reason === null || a.stop_reason === b.stop_reason || recovered, 'Stop barrier removed');
  requireValue(before.started_at === after.started_at && before.deadline_at === after.deadline_at
    && before.delivery_scope === after.delivery_scope
    && Date.parse(after.updated_at) >= Date.parse(before.updated_at), 'Budget identity changed');
  for (const key of ['candidate_limit', 'exploration_period_limit', 'pr_iteration_limit']) {
    requireValue((before.budget[key] ?? null) === (after.budget[key] ?? null), 'Budget limit changed');
  }
  requireValue(current.prUsed <= previous.prUsed || current.prLimit === null
    || current.prUsed <= current.prLimit, 'PR iteration limit exceeded');
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
  // 原站本来可能失败：首次保存基线不等于接受一个修复候选。
  const initialized = Object.values(before.versions).every(value => value === null)
    && digest(after.versions.original) && after.versions.best === after.versions.original
    && after.versions.trial === null && after.candidate_manifest?.sha256 === after.versions.original;
  requireValue(before.versions.original === after.versions.original || initialized, 'Original version changed');
  if (before.candidate_manifest?.sha256 === after.candidate_manifest?.sha256) {
    for (const old of before.required_gates) {
      const gate = after.required_gates.find(item => item.id === old.id);
      requireValue(gate && gate.kind === old.kind && old.checks.every(id => gate.checks.includes(id)),
        'Required gate coverage removed');
    }
  }
  if (before.versions.best !== after.versions.best && !initialized) {
    requireValue(after.versions.best !== null
      && after.candidate_manifest?.sha256 === after.versions.best
      && current.gates.delivery_verdict === 'ready', 'Unverified best promotion');
  }
  for (const old of a.tasks) {
    const task = b.tasks.find(item => item.id === old.id);
    requireValue(task && equal(old.goal_ids, task.goal_ids) && old.description === task.description
      && (old.repairs_task_id ?? null) === (task.repairs_task_id ?? null),
      'Task history removed or rewritten');
    requireValue(stages[old.status].includes(task.status), 'Invalid task transition');
    requireValue(old.status !== 'done' || equal(old, task), 'Completed task changed');
    requireValue(old.status === 'pending' || equal(old.depends_on, task.depends_on),
      'Active dependencies changed');
    requireValue(old.delivery?.kind !== 'pr' || task.delivery?.kind === 'pr',
      'PR task cannot become no-change work');
    requireValue(old.delivery?.kind !== 'pr'
      || prIdentity(old.delivery) === prIdentity(task.delivery), 'PR identity changed');
    if (task.status === 'done' && old.status !== 'done' && b.delivery_mode === 'auto_merge'
      && task.delivery?.kind === 'pr') {
      requireValue(old.status === 'integrating'
        && current.deliveries.get(task.id).integration.subject === after.versions.best
        && current.gates.delivery_verdict === 'ready'
        && b.tasks.filter(item => item.repairs_task_id === task.id).every(item => item.status === 'done'),
      'Task must finish on verified integration best with repairs complete');
    }
    if (old.delivery?.merge !== null && old.delivery?.merge !== undefined) {
      requireValue(task.delivery?.kind === 'pr'
        && equal(deliveryIdentity(old.delivery), deliveryIdentity(task.delivery))
        && equal(old.delivery.merge, task.delivery.merge), 'Merged PR identity changed');
    }
  }
  for (const task of b.tasks.filter(task => !a.tasks.some(old => old.id === task.id))) {
    requireValue(task.status === 'pending', 'New task must be pending');
  }
  requireValue(equal(a.operations.map(op => op.id), b.operations.slice(0, a.operations.length).map(op => op.id)),
    'Operation order changed');
  for (const old of a.operations) {
    const op = b.operations.find(item => item.id === old.id);
    requireValue(op && op.task_id === old.task_id && op.kind === old.kind
      && equal(op.subject, old.subject)
      && (old.executor_ref === null || old.executor_ref === op.executor_ref)
      && ['intended', 'running', 'completed'].indexOf(op.status)
        >= ['intended', 'running', 'completed'].indexOf(old.status), 'Operation history changed');
    requireValue(old.status !== 'completed' || equal(old, op), 'Completed operation changed');
  }
  if (b.final_acceptance !== null && !equal(a.final_acceptance, b.final_acceptance)) {
    requireValue(b.operations.some(op => op.kind === 'final_acceptance' && op.status === 'completed'
      && op.result?.sha256 === b.final_acceptance.sha256), 'Final acceptance requires a completed operation');
  }
  const added = b.operations.filter(op => !a.operations.some(old => old.id === op.id));
  requireValue(added.length <= 1, 'Multiple new operations');
  if (added.length) {
    requireValue(a.stop_reason === null && b.stop_reason === null, 'Cannot dispatch in stopped workflow');
    requireValue(!a.operations.some(op => op.status !== 'completed'), 'Recover unresolved operation first');
    const op = added[0];
    requireValue(after.activity.state === 'running' && current.gates.budget_available
      && (op.kind !== 'implement' || current.gates.candidate_available),
    'Dispatch budget or activity unavailable');
    const task = b.tasks.find(item => item.id === op.task_id);
    requireValue(!['implement', 'publish'].includes(op.kind) || withinPrBudget(current, task),
      'PR iteration limit prevents new work');
    const expectedStage = { implement: 'implementing', review: task?.status === 'pr_review'
      ? 'pr_review' : 'verifying', publish: task?.delivery?.kind === 'pr' && task.status === 'pr_review'
        ? 'pr_review' : 'verifying', merge: 'merging', integrate: 'integrating' };
    requireValue(op.status === 'intended' && op.executor_ref === null && op.result === null,
      'Persist dispatch intent before execution');
    if (op.kind === 'final_acceptance') {
      requireValue(b.tasks.every(item => item.status === 'done')
        && after.versions.trial === null && current.gates.delivery_verdict === 'ready'
        && after.candidate_manifest?.sha256 === after.versions.best
        && op.subject.manifest_sha256 === after.versions.best
        && op.subject.contract_sha256 === b.contract_ref.sha256,
      'Final acceptance is not ready for dispatch');
    } else {
      requireValue(task.pause_reason === null
        && task.status === expectedStage[op.kind]
        && task.depends_on.every(id => b.tasks.find(item => item.id === id).status === 'done'),
      'Task is not ready for dispatch');
    }
    if (op.kind === 'merge') {
      requireValue(b.delivery_mode === 'auto_merge' && current.deliveries.get(task.id)?.reviewed
        && equal(op.subject, deliveryIdentity(task.delivery)), 'Merge intent must bind reviewed PR');
      requireValue(current.gates.delivery_verdict === 'ready', 'Merge requires passing candidate gates');
      requireValue(after.candidate_manifest?.sha256 === current.deliveries.get(task.id).subject,
        'Gate candidate does not match reviewed PR candidate');
    }
    if (op.kind === 'publish') {
      requireValue(['auto_merge', 'pr_only'].includes(b.delivery_mode), 'Publish requires remote contract');
    }
  }
}

/** 只读判断目标证据及下一阶段；不启动任务、不查询宿主、不授予执行权限。 */
export async function verifyWorkflow(file, previousFile = null, now = Date.now()) {
  const previous = previousFile === null ? null : await load(previousFile, now, null, true);
  const current = await load(file, now, previous);
  if (previous !== null) transition(previous, current);
  const { state, checkpoint, gates, finalPassed, finalCurrent } = current;
  const unresolved = state.operations.some(op => op.status !== 'completed');
  const completed = state.tasks.every(task => task.status === 'done')
    && finalPassed && !unresolved && checkpoint.versions.trial === null
    && checkpoint.candidate_manifest?.sha256 === checkpoint.versions.best
    && gates.delivery_verdict === 'ready';
  requireValue(state.stop_reason !== 'completed' || completed, 'Missing completion evidence');
  const ready = state.tasks.filter(task => task.status !== 'done' && task.pause_reason === null
    && (gates.candidate_available || ['verifying', 'pr_review', 'merging', 'integrating'].includes(task.status))
    && withinPrBudget(current, task)
    && task.depends_on.every(id => state.tasks.find(item => item.id === id).status === 'done'));
  let next;
  let reason = state.stop_reason;
  // 停止屏障先于恢复/派发；停止后仅允许收回已有操作并保存真实终态。
  if (reason !== null) next = 'stop';
  else if (completed) { next = 'stop'; reason = 'completed'; }
  else if (gates.remaining_seconds === 0 || (gates.budget_known && !gates.budget_available)) {
    next = 'stop'; reason = 'budget_exhausted';
  }
  else if (unresolved) next = 'wait_or_recover';
  else if (!gates.budget_known) { next = 'estimate_budget'; reason = 'budget_unknown'; }
  else if (checkpoint.activity.state !== 'running') next = 'verify_executor';
  else if (ready.length) next = 'continue';
  else if (state.tasks.every(task => task.status === 'done') && !finalCurrent) next = 'final_acceptance';
  else if (!gates.candidate_available
    || ((state.delivery_mode ?? 'local') !== 'local' && !current.prAvailable)) {
    next = 'stop'; reason = 'iteration_limit';
  }
  else next = 'replan';
  return {
    schema_version: 1, run_id: state.run_id, revision: state.revision,
    goal_verdict: completed ? 'PASS' : 'INCOMPLETE', next, reason,
    ready_tasks: next === 'continue' ? ready.map(task => task.id) : [],
    pr_iterations_used: current.prUsed, pr_iteration_limit: current.prLimit,
    pr_iteration_available: current.prAvailable,
    legacy_delivery_tasks: [...current.deliveries].filter(([, value]) => value.legacy).map(([id]) => id),
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

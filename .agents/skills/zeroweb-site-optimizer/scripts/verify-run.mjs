import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { readFile, realpath, stat } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const statuses = ['PASS', 'FAIL', 'INCONCLUSIVE', 'SKIPPED'];
const digest = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
const nonempty = value => typeof value === 'string' && value.trim().length > 0;
function requireValue(condition, label) {
  if (!condition) throw new Error(label);
}
function timestamp(value) {
  requireValue(typeof value === 'string' && /(?:Z|[+-]\d\d:\d\d)$/.test(value)
    && Number.isFinite(Date.parse(value)), 'Invalid timestamp');
  return Date.parse(value);
}
function uniqueIds(items) {
  requireValue(Array.isArray(items) && items.every(item => item && nonempty(item.id))
    && new Set(items.map(item => item.id)).size === items.length, 'Invalid or duplicate IDs');
}
/** 有界读取结构化证据；供候选与目标检查器共用。 */
export async function readJson(file) {
  const info = await stat(file);
  requireValue(info.isFile() && info.size <= 2 * 1024 * 1024, 'Invalid JSON file size or type');
  return JSON.parse(await readFile(file, 'utf8'));
}
/** 校验运行目录内证据的真实路径、文件类型与内容身份。 */
export async function evidence(root, ref) {
  requireValue(ref && nonempty(ref.path) && digest(ref.sha256), 'Invalid evidence reference');
  requireValue(!path.isAbsolute(ref.path) && !ref.path.includes('\\')
    && !ref.path.split('/').includes('..'), 'Evidence path must stay inside the run directory');
  const file = await realpath(path.resolve(root, ref.path));
  const relative = path.relative(root, file);
  requireValue(relative && !relative.startsWith(`..${path.sep}`) && relative !== '..'
    && !path.isAbsolute(relative), 'Evidence symlink escapes the run directory');
  requireValue((await stat(file)).isFile(), 'Evidence must be a regular file');
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  requireValue(hash.digest('hex') === ref.sha256, 'Evidence hash mismatch');
  return file;
}

// 只读一致性检查：不执行记录里的命令，不推断进程存活，也不生成通过证据。
export async function verifyRun(checkpointPath, now = Date.now()) {
  const checkpointFile = await realpath(checkpointPath);
  const root = path.dirname(checkpointFile);
  const state = await readJson(checkpointFile);
  requireValue(state.schema_version === 1 && nonempty(state.run_id)
    && nonempty(state.delivery_scope) && nonempty(state.next_action), 'Invalid checkpoint identity');
  const started = timestamp(state.started_at);
  const deadline = timestamp(state.deadline_at);
  const updated = timestamp(state.updated_at);
  requireValue(started <= updated && started < deadline && updated <= now, 'Invalid checkpoint chronology');
  const activity = state.activity;
  requireValue(activity && ['running', 'awaiting_user', 'stopped', 'unknown'].includes(activity.state),
    'Invalid activity state');
  if (activity.executor !== null) {
    requireValue(activity.executor && ['host_task', 'owned_process'].includes(activity.executor.kind)
      && nonempty(activity.executor.ref), 'Invalid executor identity');
  }
  if (activity.checked_at !== null) {
    const checked = timestamp(activity.checked_at);
    requireValue(checked >= started && checked <= updated, 'Invalid activity check time');
  }
  requireValue(activity.state !== 'running'
    || (activity.executor !== null && activity.checked_at !== null), 'Running requires a checked executor');
  const budget = state.budget;
  requireValue(budget && ['handoff_seconds', 'candidates_used',
    'exploration_periods_used'].every(key =>
    Number.isSafeInteger(budget[key]) && budget[key] >= 0), 'Invalid budget counters');
  requireValue(['candidate_limit', 'exploration_period_limit'].every(key =>
    budget[key] === null || (Number.isSafeInteger(budget[key]) && budget[key] >= 0)),
  'Invalid budget limits');
  const estimate = budget.validation_estimate_seconds;
  requireValue(estimate === null || (Number.isSafeInteger(estimate) && estimate >= 0),
    'Invalid validation estimate');
  const stepEstimate = budget.next_step_estimate_seconds ?? null;
  requireValue(stepEstimate === null || (Number.isSafeInteger(stepEstimate) && stepEstimate >= 0),
    'Invalid next step estimate');
  const resources = budget.resources ?? [];
  requireValue(Array.isArray(resources) && new Set(resources.map(item => item?.unit)).size === resources.length,
    'Invalid resource budgets');
  for (const resource of resources) {
    requireValue(resource && nonempty(resource.unit)
      && ['limit', 'reserved', 'handoff_reserve'].every(key =>
        Number.isFinite(resource[key]) && resource[key] >= 0)
      && ['used', 'next_step_estimate'].every(key => resource[key] === null
        || (Number.isFinite(resource[key]) && resource[key] >= 0)), 'Invalid resource budget');
  }
  const reserve = estimate === null ? null : Math.max(1200, estimate + budget.handoff_seconds);
  const remaining = Math.max(0, Math.floor((deadline - now) / 1000));
  const budgetKnown = reserve !== null && stepEstimate !== null
    && resources.every(item => item.used !== null && item.next_step_estimate !== null);
  const iterationAvailable = (budget.candidate_limit === null || budget.candidates_used < budget.candidate_limit)
    && (budget.exploration_period_limit === null || budget.exploration_periods_used < budget.exploration_period_limit);
  const budgetAvailable = budgetKnown && remaining >= reserve + stepEstimate
    && resources.every(item =>
      item.used + item.reserved + item.next_step_estimate + item.handoff_reserve <= item.limit);
  requireValue(state.versions && ['original', 'best', 'trial'].every(key =>
    state.versions[key] === null || digest(state.versions[key])), 'Invalid version identity');
  const subject = state.candidate_manifest?.sha256 ?? null;
  if (state.candidate_manifest !== null) {
    await evidence(root, state.candidate_manifest);
    requireValue(subject === state.versions.trial || subject === state.versions.best,
      'Candidate manifest is not the recorded trial or best');
  }
  uniqueIds(state.required_gates);
  requireValue(Array.isArray(state.waivers), 'Invalid waivers');
  for (const waiver of state.waivers) {
    requireValue(waiver && ['gate_id', 'check_id', 'delivery_scope', 'reason', 'approval_ref']
      .every(key => nonempty(waiver[key])) && digest(waiver.subject), 'Invalid waiver');
  }
  const gates = [];
  for (const gate of state.required_gates) {
    requireValue(['target', 'engineering'].includes(gate.kind)
      && Array.isArray(gate.checks) && gate.checks.length > 0
      && gate.checks.every(nonempty) && new Set(gate.checks).size === gate.checks.length,
    'Invalid required gate');
    if (gate.result === null) {
      gates.push({ id: gate.id, kind: gate.kind, status: 'INCONCLUSIVE', waived: false });
      continue;
    }
    const report = await readJson(await evidence(root, gate.result));
    requireValue(report.schema_version === 1 && digest(subject) && report.subject === subject,
      'Gate report subject mismatch');
    requireValue((report.exit_code === null || Number.isSafeInteger(report.exit_code))
      && typeof report.completed === 'boolean' && typeof report.comparable === 'boolean',
    'Invalid gate execution metadata');
    uniqueIds(report.checks);
    requireValue(report.checks.every(check => statuses.includes(check.status)), 'Invalid check status');
    requireValue(report.checks.length === gate.checks.length
      && report.checks.every(check => gate.checks.includes(check.id)), 'Gate coverage mismatch');
    requireValue(Array.isArray(report.artifacts) && report.artifacts.length > 0,
      'Gate requires original evidence');
    for (const artifact of report.artifacts) await evidence(root, artifact);
    const complete = report.completed && report.comparable && report.exit_code !== null;
    const nonpass = report.checks.filter(check => check.status !== 'PASS');
    const rawStatus = !complete ? 'INCONCLUSIVE'
      : report.checks.some(check => check.status === 'FAIL') ? 'FAIL'
        : nonpass.length ? 'INCONCLUSIVE' : report.exit_code === 0 ? 'PASS' : 'FAIL';
    const waived = complete && nonpass.length > 0 && nonpass.every(check =>
      state.waivers.some(waiver => waiver.gate_id === gate.id && waiver.check_id === check.id
        && waiver.delivery_scope === state.delivery_scope && waiver.subject === subject));
    gates.push({ id: gate.id, kind: gate.kind, status: rawStatus, waived,
      checks: report.checks, exit_code: report.exit_code });
  }
  const targets = gates.filter(gate => gate.kind === 'target');
  const hasRequiredKinds = targets.length > 0 && gates.some(gate => gate.kind === 'engineering');
  const targetVerdict = targets.some(gate => gate.status === 'FAIL') ? 'FAIL'
    : targets.length && targets.every(gate => gate.status === 'PASS') ? 'PASS' : 'INCONCLUSIVE';
  const ready = hasRequiredKinds && gates.every(gate => gate.status === 'PASS' || gate.waived);
  const deliveryVerdict = !ready ? 'not_ready'
    : gates.some(gate => gate.waived) ? 'ready_with_waivers' : 'ready';
  return {
    schema_version: 1, run_id: state.run_id, activity_record: activity, live_verified: false,
    remaining_seconds: remaining, reserve_seconds: reserve,
    budget_known: budgetKnown, budget_available: budgetAvailable, iteration_available: iterationAvailable,
    can_start_candidate: activity.state === 'running' && budgetAvailable && iterationAvailable,
    target_verdict: targetVerdict, delivery_verdict: deliveryVerdict, gates,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  if (process.argv.length !== 3) {
    console.error('Usage: node verify-run.mjs <checkpoint.json>');
    process.exitCode = 2;
  } else {
    try {
      const result = await verifyRun(process.argv[2]);
      console.log(JSON.stringify(result, null, 2));
      process.exitCode = result.delivery_verdict === 'ready' ? 0 : 1;
    } catch {
      // 文件系统和 JSON 错误可能包含私有路径或正文；CLI 不回显输入。
      console.error('Invalid checkpoint or evidence; check schema, coverage, identity and file hashes.');
      process.exitCode = 2;
    }
  }
}

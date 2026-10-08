import { evidence, readJson } from './verify-run.mjs';

const keys = ['repo', 'pr', 'base_branch', 'base_sha', 'head_sha', 'subject'];
const text = value => typeof value === 'string' && value.trim().length > 0;
const same = (a, b) => keys.every(key => a?.[key] === b?.[key]);
function requireValue(condition, message) {
  if (!condition) throw new Error(message);
}

/** 从现有 operation 核对报告作者和终态；字符串不同不等于上下文独立。 */
function operationFor(state, task, ref, report) {
  const matches = state.operations.filter(op => op.task_id === task.id && op.kind === 'review'
    && op.status === 'completed' && op.result?.sha256 === ref.sha256
    && op.executor_ref === report.reviewer?.executor_ref && text(op.executor_ref));
  requireValue(matches.length === 1, 'Review requires one matching completed operation');
  return matches[0];
}

/** 检查双首轮、版本接续和每个返修补丁的非作者复核；保留原始报告身份。 */
export async function verifyReview(root, task, state, checkpoint, summary) {
  if (!summary) return { complete: false, reports: [] };
  requireValue(summary.stage === 'summary', 'Delivery review must be a summary');
  const summaryOp = operationFor(state, task, task.delivery.review, summary);
  const summaryIndex = state.operations.indexOf(summaryOp);
  const reports = [];
  for (const ref of summary.artifacts) {
    const file = await evidence(root, ref);
    // 日志、Markdown 等原始附件不按角色 JSON 解析；角色报告必须关联 review operation。
    if (!state.operations.some(op => op.task_id === task.id && op.kind === 'review'
      && op.result?.sha256 === ref.sha256)) continue;
    const report = await readJson(file);
    if (!['first', 'recheck'].includes(report.stage)) continue;
    requireValue(report.schema_version === 1 && report.task_id === task.id
      && /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(report.head_sha)
      && /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(report.base_sha)
      && ['test_validity', 'defects'].includes(report.role)
      && ['PASS', 'CHANGES_REQUIRED'].includes(report.verdict)
      && Array.isArray(report.open_findings)
      && report.reviewer?.independent === true
      && Array.isArray(report.artifacts) && report.artifacts.length > 0,
    'Invalid role review report');
    for (const artifact of report.artifacts) await evidence(root, artifact);
    const op = operationFor(state, task, ref, report);
    const index = state.operations.indexOf(op);
    requireValue(index < summaryIndex, 'Role review must precede summary');
    const candidate = await readJson(await evidence(root, report.candidate_manifest));
    requireValue(report.subject === report.candidate_manifest.sha256
      && candidate.source_sha === report.head_sha && candidate.base_sha === report.base_sha
      && candidate.dirty_patch === null, 'Role review candidate mismatch');
    requireValue(['repo', 'pr', 'base_branch'].every(key => report[key] === summary[key]),
      'Role review PR mismatch');
    requireValue(report.reviewer.executor_ref !== checkpoint.activity.executor?.ref,
      'Controller cannot supply role review');
    reports.push({ ref, report, op, index });
  }
  const first = reports.filter(item => item.report.stage === 'first');
  if (summary.review_scope === 'editorial') {
    requireValue(text(summary.scope_reason) && summary.scope_evidence != null,
      'Editorial review requires scope evidence');
    await evidence(root, summary.scope_evidence);
    return { complete: !state.operations.some((op, index) =>
      op.kind === 'implement' && op.task_id === task.id && index > summaryIndex), reports };
  }
  requireValue(summary.review_scope === 'dual', 'Missing review scope');
  requireValue(first.length === 2 && new Set(first.map(item => item.report.role)).size === 2,
    'Both first-round review roles required');
  requireValue(same(first[0].report, first[1].report), 'First-round candidates must match');
  requireValue(first[0].op.executor_ref !== first[1].op.executor_ref,
    'First-round reviewers must differ');
  const start = Math.min(...first.map(item => item.index));
  const end = Math.max(...first.map(item => item.index));
  const implementations = state.operations.map((op, index) => ({ op, index }))
    .filter(item => item.op.kind === 'implement');
  requireValue(first.every(item => !implementations.some(impl =>
    impl.index < item.index && impl.op.executor_ref === item.op.executor_ref)),
  'First-round reviewer participated in implementation');
  requireValue(!implementations.some(item => item.op.task_id === task.id
    && item.index > start && item.index < end), 'Implementation interrupted first rounds');
  const patches = implementations.filter(item => item.op.task_id === task.id && item.index > end);
  // 返修启动即撤销就绪，但允许先记录实施意图，不能要求实施前已存在新的复核。
  if (patches.some(item => item.index > summaryIndex || item.op.status !== 'completed')) {
    return { complete: false, reports };
  }
  const rechecks = reports.filter(item => item.report.stage === 'recheck');
  // 每份复核覆盖原首轮至当前版本的完整差异，允许不同角色分别复核不同作者的补丁。
  for (const item of rechecks) {
    requireValue(same(item.report.from, first[0].report) && same(item.report, summary)
      && Array.isArray(item.report.covers_operations)
      && item.report.covers_operations.every(id => patches.some(patch => patch.op.id === id)),
    'Recheck must bind original and current candidates');
    requireValue(item.report.covers_operations.every(id => {
      const patch = patches.find(value => value.op.id === id);
      return patch.index < item.index && patch.op.executor_ref !== item.op.executor_ref;
    }), 'Patch requires non-author recheck after implementation');
  }
  requireValue(same(first[0].report, summary) || rechecks.length > 0,
    'Changed candidate requires recheck');
  requireValue(patches.every(patch => rechecks.some(item =>
    item.report.covers_operations.includes(patch.op.id))), 'Missing patch recheck');
  // 有首轮发现时必须有可复查的处置证据，不能仅将汇总改成零发现。
  if (reports.some(item => item.report.open_findings.length > 0)) {
    requireValue(summary.resolution_ref != null, 'Findings require resolution evidence');
    await evidence(root, summary.resolution_ref);
  }
  return { complete: true, reports };
}

import { evidence, readJson } from './verify-run.mjs';

const text = value => typeof value === 'string' && value.trim().length > 0;
const sha = value => typeof value === 'string' && /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(value);
const identityKeys = ['repo', 'pr', 'base_branch', 'base_sha', 'head_sha'];
function requireValue(condition, message) {
  if (!condition) throw new Error(message);
}

/** 提取审查和合并意图绑定的精确 PR 身份，不使用可变分支名替代提交。 */
export function deliveryIdentity(delivery) {
  return Object.fromEntries(identityKeys.map(key => [key, delivery[key]]));
}

/** 核验阶段交付的不可变回执；不访问 GitHub，也不替代真实保护规则检查。 */
export async function verifyDelivery(root, task, state, checkpoint) {
  const delivery = task.delivery ?? null;
  if (delivery === null) {
    requireValue(!['pr_review', 'merging', 'integrating', 'done'].includes(task.status),
      'Task requires delivery evidence');
    return { reviewed: false, merged: false, integrated: false };
  }
  if (delivery.kind === 'no_change') {
    requireValue(text(delivery.reason) && !['pr_review', 'merging', 'integrating'].includes(task.status),
      'Invalid no-change delivery');
    return { reviewed: false, merged: false, integrated: false };
  }
  requireValue(delivery.kind === 'pr' && text(delivery.repo) && text(delivery.base_branch)
    && Number.isSafeInteger(delivery.pr) && delivery.pr > 0
    && sha(delivery.base_sha) && sha(delivery.head_sha), 'Invalid PR identity');
  const receipts = {};
  for (const key of ['review', 'merge', 'integration']) {
    requireValue(Object.hasOwn(delivery, key), 'Missing delivery receipt field');
    if (delivery[key] === null) continue;
    const report = await readJson(await evidence(root, delivery[key]));
    requireValue(report.schema_version === 1 && report.task_id === task.id
      && Array.isArray(report.artifacts) && report.artifacts.length > 0, 'Invalid delivery receipt');
    for (const ref of report.artifacts) await evidence(root, ref);
    receipts[key] = report;
  }
  const matches = report => report && identityKeys.every(key => report[key] === delivery[key]);
  // 历史任务保留自己的候选清单，不能随全局 best 推进而重绑旧审查。
  const subject = delivery.candidate_manifest?.sha256 ?? null;
  const candidate = subject === null ? null
    : await readJson(await evidence(root, delivery.candidate_manifest));
  const candidateMatches = candidate !== null && candidate.source_sha === delivery.head_sha
    && candidate.base_sha === delivery.base_sha && candidate.dirty_patch === null;
  const review = receipts.review;
  const implementers = state.operations.filter(op => op.kind === 'implement').map(op => op.executor_ref);
  const reviewed = matches(review) && candidateMatches && review.subject === subject && review.verdict === 'PASS'
    && Array.isArray(review.open_findings) && review.open_findings.length === 0
    && review.reviewer?.independent === true && text(review.reviewer.executor_ref)
    && review.reviewer.executor_ref !== checkpoint.activity.executor?.ref
    && !implementers.includes(review.reviewer.executor_ref);
  const merge = receipts.merge;
  const merged = reviewed && matches(merge) && merge.confirmed === true && sha(merge.commit)
    && merge.review_sha256 === delivery.review.sha256;
  const integration = receipts.integration;
  const integrated = merged && integration?.status === 'PASS' && sha(integration.commit)
    && integration.merge_commit === merge.commit && integration.contains_merge === true
    && typeof integration.subject === 'string' && /^[a-f0-9]{64}$/.test(integration.subject);
  requireValue(!['merging', 'integrating', 'done'].includes(task.status) || reviewed,
    'Current independent review required');
  requireValue(!(task.status === 'integrating' || (task.status === 'done'
    && state.delivery_mode === 'auto_merge')) || merged,
    'Confirmed merge required');
  requireValue(task.status !== 'done' || state.delivery_mode === 'pr_only' || integrated,
    'Passing integration required');
  return { reviewed, merged, integrated, integration, subject };
}

import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { realpath } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { evidence, readJson, verifyRun } from './verify-run.mjs';
import { verifyWorkflow } from './verify-workflow.mjs';
import { verifyDelivery } from './verify-delivery.mjs';
import { checkBody, readBody, reviewSection } from './pr-body.mjs';

/** 对比本次 GitHub 查询与已核验正文；正文修改不改变 head，必须单独检查摘要。 */
export function checkRemote(remote, delivery, bodyHash) {
  const hash = createHash('sha256').update(remote.body ?? '').digest('hex');
  if (remote.number !== delivery.pr || remote.headRefOid !== delivery.head_sha
    || remote.baseRefOid !== delivery.base_sha || remote.baseRefName !== delivery.base_branch
    || remote.state !== 'OPEN' || hash !== bodyHash) {
    throw new Error('Remote PR identity or body changed; renew evidence');
  }
  return hash;
}

/** 只读交付预检；使用参数数组调用 gh，绝不执行记录中提供的命令。 */
export async function deliveryCheck(file, taskId, previous = null, now = Date.now()) {
  const root = path.dirname(await realpath(file));
  const state = await readJson(file);
  if (state.revision > 1 && previous === null) throw new Error('Previous snapshot required');
  const workflow = await verifyWorkflow(file, previous, now);
  const task = state.tasks.find(item => item.id === taskId);
  if (!task || task.delivery?.kind !== 'pr') throw new Error('PR task required');
  const checkpointFile = await evidence(root, state.checkpoint_ref);
  const checkpoint = await readJson(checkpointFile);
  const gates = await verifyRun(checkpointFile, now);
  const delivery = await verifyDelivery(root, task, state, checkpoint);
  if (state.stop_reason !== null || task.pause_reason !== null
    || !task.depends_on.every(id => state.tasks.some(item => item.id === id && item.status === 'done'))
    || checkpoint.activity.state !== 'running' || !gates.budget_available
    || !['continue', 'wait_or_recover'].includes(workflow.next)
    || !['pr_review', 'merging'].includes(task.status)
    || state.operations.some(op => op.status !== 'completed'
      && !(op.kind === 'merge' && op.task_id === task.id && op.status === 'intended'))
    || !delivery.reviewed || !delivery.presented || gates.delivery_verdict !== 'ready'
    || checkpoint.candidate_manifest?.sha256 !== delivery.subject) {
    throw new Error('Delivery not ready; inspect workflow, review and presentation evidence');
  }
  const presentation = await readJson(await evidence(root, task.delivery.presentation));
  const remote = JSON.parse(execFileSync('gh', ['pr', 'view', String(task.delivery.pr),
    '--repo', task.delivery.repo, '--json', 'number,headRefOid,baseRefOid,baseRefName,state,isDraft,body'],
  { encoding: 'utf8', timeout: 30000, maxBuffer: 2 * 1024 * 1024 }));
  const bodyHash = checkRemote(remote, task.delivery, presentation.body_ref.sha256);
  return { ready: true, body_sha256: bodyHash, review_sha256: task.delivery.review.sha256,
    is_draft: remote.isDraft, checked_at: new Date(now).toISOString(),
    limitation: 'CI, branch protection, authorization and host isolation require direct verification.' };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const [command, file, taskId, extra] = process.argv.slice(2);
  try {
    if (command === 'draft' && file && !taskId) {
      checkBody(await readBody(file));
      console.log(JSON.stringify({ draft_body_valid: true, ready: false }));
    } else if (command === 'render' && file && taskId) {
      const root = path.dirname(await realpath(file));
      const state = await readJson(file);
      const task = state.tasks.find(item => item.id === taskId);
      if (!task) throw new Error('Unknown task');
      const body = await readBody(extra ?? new URL('../templates/github-pr.md', import.meta.url));
      const section = await reviewSection(root, task, state);
      if (!/^## 独立审查$/m.test(body)) throw new Error('Missing review section');
      console.log(body.replace(/^## 独立审查\n[\s\S]*?(?=^## |$(?![\s\S]))/m, `${section}\n`));
    } else if (command === 'ready' && file && taskId) {
      console.log(JSON.stringify(await deliveryCheck(file, taskId, extra ?? null), null, 2));
    } else throw new Error('Invalid arguments');
  } catch {
    console.error('Delivery check failed. Usage: delivery-check.mjs draft <body> | render <workflow> <task> [body] | ready <workflow> <task> [previous]');
    process.exitCode = 2;
  }
}

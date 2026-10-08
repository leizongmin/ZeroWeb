import { createHash } from 'node:crypto';
import { readFile, realpath } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { evidence, readJson } from './verify-run.mjs';
import { verifyWorkflow } from './verify-workflow.mjs';

/** 从唯一账本生成恢复导航；损坏的交付证据仍显示停止屏障，但不允许推进。 */
export async function recoveryPacket(file, previous = null, now = Date.now()) {
  const root = path.dirname(await realpath(file));
  const state = await readJson(file);
  await evidence(root, state.contract_ref);
  const checkpoint = await readJson(await evidence(root, state.checkpoint_ref));
  let verdict;
  try {
    if (state.revision > 1 && previous === null) throw new Error('Previous snapshot required');
    verdict = await verifyWorkflow(file, previous, now);
  } catch {
    verdict = { next: 'repair_records', live_verified: false,
      error: 'Workflow validation failed; inspect original evidence before proceeding.' };
  }
  const rules = ['SKILL.md', 'references/recovery.md', 'references/intake.md',
    'references/execution.md', 'references/checkpoint.md', 'references/workflow.md'];
  const active = state.tasks.filter(task => task.status !== 'done');
  if (active.some(task => task.delivery?.kind === 'pr'
    || ['verifying', 'pr_review', 'merging', 'integrating'].includes(task.status))) {
    rules.push('references/github-delivery.md', 'templates/github-pr.md', 'references/independent-review.md');
  }
  const skillRoot = fileURLToPath(new URL('..', import.meta.url));
  const requiredReads = await Promise.all(rules.map(async name => ({
    path: `.agents/skills/zeroweb-site-optimizer/${name}`,
    sha256: createHash('sha256').update(await readFile(path.join(skillRoot, name))).digest('hex'),
  })));
  return {
    schema_version: 1, run_id: state.run_id, revision: state.revision,
    contract_ref: state.contract_ref, checkpoint_ref: state.checkpoint_ref,
    started_at: checkpoint.started_at, deadline_at: checkpoint.deadline_at,
    stop_reason: state.stop_reason, activity: checkpoint.activity,
    versions: checkpoint.versions, next_action: checkpoint.next_action,
    tasks: active.map(({ id, status, pause_reason, delivery }) => ({ id, status, pause_reason, delivery })),
    unresolved_operations: state.operations.filter(op => op.status !== 'completed'),
    verdict, required_reads: [{ path: 'AGENTS.md' }, ...requiredReads],
    instruction: 'Reload these files in this context; verify host and remote state before any new operation.',
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    if (![3, 4].includes(process.argv.length)) throw new Error('Invalid arguments');
    const result = await recoveryPacket(process.argv[2], process.argv[3] ?? null);
    console.log(JSON.stringify(result, null, 2));
    process.exitCode = result.verdict.error ? 2 : 0;
  } catch {
    console.error('Recovery failed; use original workflow and checkpoint. Usage: recover.mjs <workflow> [previous]');
    process.exitCode = 2;
  }
}

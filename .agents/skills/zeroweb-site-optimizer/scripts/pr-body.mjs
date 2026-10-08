import { readFile, stat } from 'node:fs/promises';
import { evidence, readJson } from './verify-run.mjs';

const text = value => typeof value === 'string' && value.trim().length > 0;
const digest = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
function requireValue(condition, message) {
  if (!condition) throw new Error(message);
}
const headings = ['解决了什么', '怎么验证', '还有什么问题', '优化前后截图', '独立审查'];
const visibleBody = body => body.replace(/<!--[\s\S]*?-->/g, '').replace(/```[\s\S]*?```/g, '');

/** 有界读取 PR 正文，避免意外读取二进制或巨型日志。 */
export async function readBody(file) {
  requireValue((await stat(file)).size <= 1024 * 1024, 'PR body too large');
  return readFile(file, 'utf8');
}

/** 只接受 GitHub 持久附件 URL；不接受仓库图片、带凭据或临时签名的地址。 */
export function attachmentUrl(value) {
  try {
    const url = new URL(value);
    return url.protocol === 'https:' && !url.username && !url.password && !url.port
      && !url.search && !url.hash
      && ((url.hostname === 'github.com' && /^\/user-attachments\/assets\/[\w-]+$/.test(url.pathname))
        || (url.hostname === 'user-images.githubusercontent.com' && url.pathname.length > 1));
  } catch { return false; }
}

/** 从实际报告生成公开状态，不公开宿主 ID、私有路径或发现正文。 */
export async function reviewSection(root, task, state) {
  const rows = [];
  const completed = new Set();
  for (const op of state.operations.filter(op => op.task_id === task.id
    && op.kind === 'review' && op.status === 'completed')) {
    const report = await readJson(await evidence(root, op.result));
    if (!['first', 'recheck', 'summary'].includes(report.stage)) continue;
    requireValue(/^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(report.head_sha)
      && /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(report.base_sha),
      'Invalid visible review identity');
    const selected = report.stage !== 'summary' || op.result.sha256 === task.delivery?.review?.sha256;
    const label = report.stage === 'summary' ? '汇总'
      : `${report.role === 'test_validity' ? '测试有效性' : '缺陷'}${report.stage === 'first' ? '首轮' : '复核'}`;
    const current = selected && report.head_sha === task.delivery.head_sha
      && report.base_sha === task.delivery.base_sha;
    if (report.stage !== 'summary' || current) {
      completed.add(report.stage === 'summary' ? 'summary' : `${report.stage}:${report.role}`);
    }
    rows.push(`| ${label} | ${report.verdict === 'PASS' ? 'PASS' : 'CHANGES_REQUIRED'}${current ? '' : '（历史报告）'} | ${report.head_sha} / ${report.base_sha} | 本地证据 SHA-256：${op.result.sha256} |`);
  }
  const summary = task.delivery?.review
    ? await readJson(await evidence(root, task.delivery.review)) : null;
  const currentSummary = completed.has('summary') ? summary : null;
  for (const [key, label] of [['first:test_validity', '测试有效性首轮'],
    ['first:defects', '缺陷首轮'], ['summary', '汇总']]) {
    if (!completed.has(key)) rows.push(`| ${label} | ${currentSummary?.review_scope === 'editorial' && key !== 'summary' ? '不适用（纯文字）' : '待执行'} | — | — |`);
  }
  return [
    '## 独立审查', '',
    '| 环节 | 报告状态 | head / base | 证据 |', '|---|---|---|---|',
    ...rows,
    '', `未解决阻断项：${Array.isArray(currentSummary?.open_findings) ? currentSummary.open_findings.length : '未汇总'}。`,
    '报告保留在本地运行目录；以上为报告记录，独立性与交付就绪另由门禁核验。',
  ].join('\n');
}

/** 检查模板结构和占位符；Draft 允许如实写明缺口，但不能发布空模板。 */
export function checkBody(body) {
  requireValue(text(body) && !/\{\{[^}]*\}\}/.test(body), 'Unfilled PR template');
  // 隐藏注释或代码块中的标题不能代替读者可见的正文。
  const visible = visibleBody(body);
  for (const heading of headings) {
    const section = visible.match(new RegExp(`^## ${heading}\\s*\\n([\\s\\S]*?)(?=^## |$(?![\\s\\S]))`, 'm'));
    requireValue(section && text(section[1]), `Missing PR section: ${heading}`);
  }
}

/** 核验回读正文、当前汇总和截图映射；实际下载/呈现由所引用的人机证据证明。 */
export async function verifyPresentation(root, task, state) {
  if (!task.delivery.presentation) return false;
  const report = await readJson(await evidence(root, task.delivery.presentation));
  requireValue(report.schema_version === 1 && report.task_id === task.id,
    'Invalid PR presentation');
  // 新汇总先落盘、正文后发布；旧回执过期表示未就绪，不应阻塞补齐操作。
  if (!['repo', 'pr', 'base_branch', 'base_sha', 'head_sha'].every(key =>
    report[key] === task.delivery[key]) || report.review_sha256 !== task.delivery.review?.sha256) return false;
  const body = await readBody(await evidence(root, report.body_ref));
  checkBody(body);
  const visible = visibleBody(body);
  if (!visible.includes(await reviewSection(root, task, state))) return false;
  requireValue(Array.isArray(report.artifacts) && report.artifacts.length > 0,
    'PR presentation requires readback evidence');
  for (const ref of report.artifacts) await evidence(root, ref);
  const images = report.screenshots;
  requireValue(images && ['paired', 'not_applicable', 'prohibited', 'blocked'].includes(images.status),
    'Missing screenshot applicability');
  if (images.status === 'blocked') return false;
  if (images.status !== 'paired') {
    requireValue(text(images.reason) && visible.includes(images.reason), 'Screenshot exception needs visible reason');
    await evidence(root, images.evidence_ref);
    return true;
  }
  requireValue(Array.isArray(images.pairs) && images.pairs.length > 0, 'Missing screenshot pairs');
  for (const pair of images.pairs) {
    requireValue(text(pair.scene) && visible.includes(pair.scene), 'Missing screenshot scene');
    requireValue(pair.before?.url !== pair.after?.url, 'Before and after attachments must differ');
    requireValue(visible.split('\n').some(line => line.startsWith('|')
      && line.includes(`](${pair.before?.url})`) && line.includes(`](${pair.after?.url})`)),
    'Before and after must share a comparison table row');
    for (const role of ['before', 'after']) {
      const image = pair[role];
      requireValue(image && attachmentUrl(image.url) && digest(image.sha256)
        && /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(image.revision)
        && image.content_verified === true && image.render_verified === true,
      'Screenshot content or rendering unverified');
      requireValue(role !== 'after' || image.revision === task.delivery.head_sha,
        'After screenshot must match PR head');
      requireValue(new RegExp(`!\\[[^\\]]+\\]\\(${image.url.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\)`).test(visible),
        'Screenshot must appear in PR body');
      const capture = await readJson(await evidence(root, image.evidence_ref));
      requireValue(capture.url === image.url && capture.revision === image.revision
        && capture.download_ref?.sha256 === image.sha256 && capture.render_verified === true
        && typeof capture.content_type === 'string' && capture.content_type.startsWith('image/')
        && Number.isSafeInteger(capture.width) && capture.width > 0
        && Number.isSafeInteger(capture.height) && capture.height > 0,
      'Screenshot verification evidence mismatch');
      await evidence(root, capture.download_ref);
    }
  }
  return true;
}

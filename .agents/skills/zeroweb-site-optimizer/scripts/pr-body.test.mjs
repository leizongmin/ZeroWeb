import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, realpath, rm, writeFile, readFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { attachmentUrl, checkBody, reviewSection, verifyPresentation } from './pr-body.mjs';
import { checkRemote } from './delivery-check.mjs';

const hash = data => createHash('sha256').update(data).digest('hex');
const before = 'https://github.com/user-attachments/assets/before-test';
const after = 'https://github.com/user-attachments/assets/after-test';

/** 合成图文回执验证读取、引用和状态，不能证明真实上传或视觉语义。 */
async function fixture(t) {
  const root = await realpath(await mkdtemp(path.join(os.tmpdir(), 'zeroweb-pr-')));
  t.after(() => rm(root, { recursive: true, force: true }));
  let sequence = 0;
  async function save(data) {
    const content = typeof data === 'string' ? data : JSON.stringify(data);
    const name = `${sequence++}.txt`;
    await writeFile(path.join(root, name), content);
    return { path: name, sha256: hash(content) };
  }
  const raw = await save({ synthetic: true });
  const task = { id: 'fix', delivery: { repo: 'example/browser', pr: 12, base_branch: 'main',
    head_sha: 'b'.repeat(40), base_sha: 'a'.repeat(40), review: null } };
  const state = { operations: [] };
  let body = `## 解决了什么\n修复返回。\n\n## 怎么验证\n真实任务回归。\n\n## 还有什么问题\n无。\n\n## 优化前后截图\n返回列表\n\n| 前 | 后 |\n|---|---|\n| ![前](${before}) | ![后](${after}) |\n\n${await reviewSection(root, task, state)}\n`;
  const image = async (url, revision) => ({ url, revision, sha256: raw.sha256,
    content_verified: true, render_verified: true, evidence_ref: await save({
      url, revision, download_ref: raw, render_verified: true, content_type: 'image/png', width: 10, height: 10,
    }) });
  const report = { schema_version: 1, task_id: task.id, ...task.delivery,
    review_sha256: undefined, body_ref: null, artifacts: [raw],
    screenshots: { status: 'paired', pairs: [{ scene: '返回列表',
      before: await image(before, 'a'.repeat(40)), after: await image(after, 'b'.repeat(40)) }] } };
  async function check() {
    report.body_ref = await save(body);
    task.delivery.presentation = await save(report);
    return verifyPresentation(root, task, state);
  }
  return { root, state, task, report, save, raw, check, get body() { return body; },
    set body(value) { body = value; } };
}

test('draft accepts explicit gaps but rejects blank, hidden or unfilled sections', async t => {
  const f = await fixture(t);
  checkBody(f.body);
  assert.throws(() => checkBody(f.body.replace('修复返回。', '{{结果}}')), /Unfilled/);
  assert.throws(() => checkBody(f.body.replace('## 怎么验证\n真实任务回归。', '')), /Missing PR section/);
  assert.throws(() => checkBody(f.body.replace('## 怎么验证\n真实任务回归。', '<!-- ## 怎么验证\n真实任务回归。 -->')));
  assert.throws(() => checkBody(f.body.replace('真实任务回归。', '')));
  const template = await readFile(new URL('../templates/github-pr.md', import.meta.url), 'utf8');
  assert.throws(() => checkBody(template), /Unfilled/);
});

test('attachment URLs reject local, raw, signed and lookalike hosts', () => {
  assert.equal(attachmentUrl(before), true);
  for (const url of ['before.png', 'file:///tmp/a.png', 'https://github.com.evil.test/user-attachments/assets/a',
    'https://github.com/owner/repo/blob/main/a.png', `${before}?token=private`,
    'https://user:password@github.com/user-attachments/assets/a']) {
    assert.equal(attachmentUrl(url), false);
  }
});

test('paired presentation requires both visible current images and rendering evidence', async t => {
  const f = await fixture(t);
  assert.equal(await f.check(), true);
  const valid = f.body;
  f.body = valid.replace(`![后](${after})`, `[后](${after})`);
  await assert.rejects(f.check(), /must appear/);
  f.body = valid.replace(`![后](${after})`, `<!-- ![后](${after}) -->`);
  await assert.rejects(f.check(), /must appear|table row/);
  f.body = valid;
  f.report.screenshots.pairs[0].after.render_verified = false;
  await assert.rejects(f.check(), /unverified/);
  f.report.screenshots.pairs[0].after.render_verified = true;
  f.report.screenshots.pairs[0].after.revision = 'c'.repeat(40);
  await assert.rejects(f.check(), /match PR head/);
});

test('image digest must agree with verified downloaded bytes', async t => {
  const f = await fixture(t);
  f.report.screenshots.pairs[0].after.sha256 = 'e'.repeat(64);
  await assert.rejects(f.check(), /verification evidence mismatch/);
});

test('missing screenshots are blocked; inapplicability needs visible reason and evidence', async t => {
  const f = await fixture(t);
  f.report.screenshots = { status: 'blocked' };
  assert.equal(await f.check(), false);
  f.report.screenshots = { status: 'not_applicable' };
  await assert.rejects(f.check(), /visible reason/);
  f.report.screenshots = { status: 'prohibited', reason: '用户禁止公开截图', evidence_ref: f.raw };
  f.body += '\n用户禁止公开截图\n';
  assert.equal(await f.check(), true);
});

test('remote body edit invalidates an old readback even when head is unchanged', async t => {
  const f = await fixture(t);
  const remote = { number: 12, headRefOid: f.task.delivery.head_sha,
    baseRefOid: f.task.delivery.base_sha, baseRefName: 'main', state: 'OPEN', body: f.body };
  const bodyHash = hash(f.body);
  assert.equal(checkRemote(remote, f.task.delivery, bodyHash), bodyHash);
  remote.body = f.body.replace(`![后](${after})`, '');
  assert.throws(() => checkRemote(remote, f.task.delivery, bodyHash), /body changed/);
  remote.body = f.body;
  remote.baseRefOid = 'c'.repeat(40);
  assert.throws(() => checkRemote(remote, f.task.delivery, bodyHash), /identity/);
});

test('draft CLI has no remote dependency and rejects the raw template', async t => {
  const f = await fixture(t);
  const ref = await f.save(f.body);
  const cli = fileURLToPath(new URL('./delivery-check.mjs', import.meta.url));
  const invoke = file => spawnSync(process.execPath, [cli, 'draft', file],
    { encoding: 'utf8', timeout: 5000 });
  assert.equal(invoke(path.join(f.root, ref.path)).status, 0);
  assert.equal(invoke(fileURLToPath(new URL('../templates/github-pr.md', import.meta.url))).status, 2);
});

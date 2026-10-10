import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, realpath, rm, writeFile, readFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { attachmentAccessible, attachmentUrl, checkBody, reviewSection, verifyPresentation } from './pr-body.mjs';
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
  let body = `## 解决了什么\n修复返回。\n\n## 怎么验证\n真实任务回归。\n\n## 还有什么问题\n无。\n\n## 优化前后截图\n返回列表\n\n| 前 | 后 |\n|---|---|\n| 修复前返回后内容空白，列表无法回到详情 | 修复后返回正常显示详情内容 |\n| ![前](${before}) | ![后](${after}) |\n\n${await reviewSection(root, task, state)}\n`;
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
    return verifyPresentation(root, task, state, { probeAttachment: async () => true });
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

test('body image references must be uploaded attachments, not local paths', () => {
  // PR #117 实测回归：对比表残留运行目录相对路径，附件 URL 另起一行漏替换。
  const f = { body: `## 解决了什么\n修复返回。\n\n## 怎么验证\n真实任务回归。\n\n## 还有什么问题\n无。\n\n## 优化前后截图\n\n| 前 | 后 |\n|---|---|\n| ![前](evidence/t2l-zw-home-p6b.png) | ![后](evidence/t2l-zw-home-p6c.png) |\n\n## 独立审查\n\n| 环节 | 报告状态 |\n|---|---|\n` };
  assert.throws(() => checkBody(f.body), /not an uploaded attachment.*evidence\//);
  assert.throws(() => checkBody(f.body.replace(/evidence\/t2l-zw-home-p6[bc]\.png/g, './shot.png')), /not an uploaded attachment/);
  assert.doesNotThrow(() => checkBody(f.body
    .replace('](evidence/t2l-zw-home-p6b.png)', `](${before})`)
    .replace('](evidence/t2l-zw-home-p6c.png)', `](${after})`)));
  // 围栏代码块内的示例不拦截：表格用合法附件 URL，代码块里保留本地路径样例。
  const fenced = f.body
    .replace('](evidence/t2l-zw-home-p6b.png)', `](${before})`)
    .replace('](evidence/t2l-zw-home-p6c.png)', `](${after})`)
    .replace('## 优化前后截图\n', '## 优化前后截图\n\n```\n![示例](evidence/local.png)\n```\n');
  assert.doesNotThrow(() => checkBody(fenced));
});

test('ready gate probes each attachment URL and rejects unreachable ones', async t => {
  const f = await fixture(t);
  assert.equal(await f.check(), true);
  await assert.rejects(
    verifyPresentation(f.root, f.task, f.state, { probeAttachment: async () => false }),
    /Attachment not accessible/,
  );
});

test('image row requires an adjacent text description row above it', async t => {
  const f = await fixture(t);
  assert.equal(await f.check(), true);
  const valid = f.body;
  // 缺描述行（分隔行紧邻图片行）→ 拒绝。
  f.body = valid.replace('| 修复前返回后内容空白，列表无法回到详情 | 修复后返回正常显示详情内容 |\n', '');
  await assert.rejects(f.check(), /text description row/);
  // 表头行紧邻图片行（描述行错位到别处）→ 拒绝：表头不是描述。
  f.body = valid.replace('| 前 | 后 |\n|---|---|\n| 修复前返回后内容空白，列表无法回到详情 | 修复后返回正常显示详情内容 |\n',
    '| 前 | 后 |\n| 修复前返回后内容空白，列表无法回到详情 | 修复后返回正常显示详情内容 |\n|---|---|\n');
  await assert.rejects(f.check(), /text description row/);
  // 描述行内嵌图片引用 → 不算纯文字描述。
  f.body = valid.replace('修复前返回后内容空白，列表无法回到详情', `修复前见 ![x](${before})`);
  await assert.rejects(f.check(), /text description row/);
  // 描述行单格为空 → 拒绝。
  f.body = valid.replace('修复前返回后内容空白，列表无法回到详情', '');
  await assert.rejects(f.check(), /text description row/);
});

test('attachmentAccessible requires 2xx plus image magic or content type', async t => {
  const png = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0]);
  const html = new Uint8Array([0x3c, 0x68, 0x74, 0x6d, 0x6c, 0x3e]);
  const respond = (status, body, type) => async url => {
    assert.match(String(url), /^https:\/\/github\.com\//);
    return { ok: 200 <= status && status < 300, status,
      headers: { get: key => (key === 'content-type' ? type : null) },
      arrayBuffer: async () => body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength) };
  };
  assert.equal(await attachmentAccessible(before, respond(200, png, 'binary/octet-stream')), true);
  assert.equal(await attachmentAccessible(before, respond(206, png, null)), true);
  assert.equal(await attachmentAccessible(before, respond(200, html, 'text/html')), false);
  assert.equal(await attachmentAccessible(before, respond(200, html, 'image/png')), true);
  assert.equal(await attachmentAccessible(before, respond(404, png, 'image/png')), false);
  assert.equal(await attachmentAccessible(before, async () => { throw new Error('down'); }), false);
});

---
date: 2026-10-08
modules: build-support
---
# 符号链接路径调用 Node CLI 脚本导致入口守卫空转（rc=0 但未校验）

## 问题描述

`.claude/skills/zeroweb-site-optimizer` 是指向 `.agents/skills/zeroweb-site-optimizer` 的符号链接。经 `.claude` 路径调用其 `verify-workflow.mjs` CLI 时始终 rc=0 通过，但实际**未执行任何校验**——多轮「standalone rc=0 PASS」结论全部空转。

## 根因分析

脚本用 CLI 主入口守卫区分「被直接执行」与「被导入」：

```js
if (import.meta.url !== pathToFileURL(path.resolve(process.argv[1])).href) {
  // 被导入，导出 API 后退出
  return;
}
```

Node 会把 `import.meta.url` 解析到符号链接的 **realpath**（`.agents/...`），而 `process.argv[1]` 保持调用方写的字面路径（`.claude/...`）。两者经 `.href` 比对永不相等 → 脚本认为自己是被导入的，静默退出 rc=0。

## 解决方案

- 校验类脚本一律走真实路径或 JS API：
  - `node .agents/skills/<skill>/scripts/verify-workflow.mjs <args>`（.agents 直路径，argv[1] 与 realpath 一致）
  - 或写一个薄 wrapper `import { verifyWorkflow } from '<.agents 绝对路径>/verify-workflow.mjs'` 直接调函数（本项目 `tools/wf-check.mjs` 即此形态）
- 写带入口守卫的 Node 脚本时，用 `realpathSync(process.argv[1])` 先归一再比对，避免对调用方路径形态敏感。

## 如何避免

对任何「rc=0 即通过」的静默校验，首次接入时用**故意注入错误**的样例验证它能真报 INVALID（负样例自证），再信任其 PASS。本次靠两卷模式（prev 快照对照）首次触发真实报错才发现长期空转。

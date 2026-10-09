---
date: 2026-10-09
modules: tools
---

# symlink 路径调用 skill 脚本会静默空转（.claude/skills → .agents/skills）

## 问题描述

`node .claude/skills/zeroweb-site-optimizer/scripts/verify-workflow.mjs <wf> <prev>` 以 exit 0 静默退出、无任何输出，看起来像「校验通过」；同一文件经 `.agents/skills/...` 路径调用才真正执行校验（同一工作流实报 `Invalid activity state`，exit 2）。两份入口文件 SHA-256 完全一致，同参数同 CWD，结果却一「过」一败。

## 根因分析

`.claude/skills/zeroweb-site-optimizer` 是指向 `.agents/skills/zeroweb-site-optimizer` 的 symlink（目录项本身显示为普通目录，易误判为独立副本）。脚本主入口用自检 guard 判断是否被直接执行：

```js
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) { ... }
```

Node ESM 的 `import.meta.url` 是**realpath 解析后**的模块 URL（默认 symlink 跟随），恒为 `file:///.../.agents/...`；而 `path.resolve(argv[1])` 只做字面拼接，经 `.claude` 路径调用时是 `file:///.../.claude/...`。两者永不相等 → guard 不命中 → 脚本整体跳过 → exit 0。catch-all 错误处理（`catch { console.error('Invalid workflow...') }`）则把 `.agents` 路径下的真实校验异常（checkpoint.activity 被写成字符串导致 `Invalid activity state`）压成一行无细节消息。

## 解决方案

- 调用本仓 skill 下脚本一律用 `.agents/skills/...` 真身路径（`bump-checkpoint.py` 内部已如此引用）。
- 判断「exit 0 = 通过」前先确认脚本真的执行了（有输出）；静默 exit 0 + 零输出要怀疑 guard 未命中。
- 修数据问题（activity 结构）后，`bump-checkpoint.py` 无参重跑即可刷新 checkpoint_ref 摘要并复验（rev 单调递增无害）。

## 如何避免

给脚本写自执行 guard 时，比较前对 argv[1] 也做 `realpath`（`fs.realpathSync`），或改用 `process.argv[1]` 与 `import.meta.url` 双侧 realpath 归一；否则 symlink 布局下必现静默空转。

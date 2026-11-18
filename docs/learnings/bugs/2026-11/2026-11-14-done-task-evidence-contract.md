---
date: 2026-11-14
modules: site_optimizer
---

# done 任务须携带 task.evidence 终态证据——T66 隐式传承 T68 重撞

## 问题描述

T68 交付链 rev1139（done rev：task→done＋checkpoint_ref 换绑）落账后 validator
EXIT=2，CLI 只报「Invalid workflow, transition or evidence」。T66 走过同一迁移
且通过了，差异没有被抽象成契约，T68 的 done rev 只做了状态迁移与 CP 换绑，
漏填 `task.evidence`，校验在最后一环爆断。

## 根因分析

verify-workflow.mjs L74-76 的契约是三条件：

```js
requireValue(task.status !== 'done' || (task.evidence !== null && task.pause_reason === null),
  'Done task requires evidence');
if (task.evidence !== null) await evidence(root, task.evidence);
```

done 任务必须 `task.evidence` 非 null 且 `pause_reason === null`，且 evidence ref
（`{path, sha256}`）经 `evidence()` 校验文件存在与 sha256 恒一。T66 通过时恰好
带了 `evidence/t66r-task-evidence.md`，但「为什么带」没有被沉淀——隐式传承
（上一轮恰好带了）不是契约记忆。伴生坑：CLI 签名是
`verifyWorkflow(workflow.json, workflow-previous.json)`，收**文件路径**而非运行
目录；传目录会被 readJson 炸成 EXIT=2 假故障，且 CLI `catch {}` 吞掉真实错误，
须用 `node --input-type=module` 直调 `verifyWorkflow` 取 REAL_ERROR。

## 解决方案

1. done 迁移的落地脚本显式断言三条件（evidence 非 null＋pause null＋文件 sha
   实算恒一），不依赖上一轮形态。
2. 任务级终态证据文档结构化落盘（T66/T68 同构四节：变更面/门禁判据/交付集成
   终态/账本事故存证），以 `{path, sha256}` 绑定。
3. 确定性校验错误不重试洗白：存证（`workflow-v3-revNNNN-pre-evidence.json`）→
   rm 快照→修脚本→独立重放脚本只重建末 rev→live 同步→validator 裸跑判读
   （真 EXIT；输出接管道会让 `$?` 取到管道尾，白洗退出码）。

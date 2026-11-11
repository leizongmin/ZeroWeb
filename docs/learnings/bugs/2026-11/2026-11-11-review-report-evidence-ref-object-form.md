---
date: 2026-11-11
modules: site_optimizer
---

# 审查报告的 evidence 引用必须是 {path, sha256} 对象形态，字符串形态在账本绑定日爆断

## 问题描述

站点优化器角色审查报告（role review report）的 JSON 里，`artifacts` 与
`candidate_manifest` 若写成字符串路径（如 `"artifacts": ["evidence/x.md"]`），
派发时一切正常——这两个字段只有在账本绑定（review op 落账）那天才被
`verify-review.mjs` 消费。届时校验在
`Invalid evidence reference` 处失败，返修要重建整条报告链。

## 根因分析

`verify-run.mjs` 的 `evidence(root, ref)` 对引用做硬校验：
`ref && nonempty(ref.path) && digest(ref.sha256)`——只接受
`{path, sha256}` 对象形态，且 sha256 必须是 40/64 hex 实算值。
`verify-review.mjs` L42 遍历 `report.artifacts` 逐个过 `evidence()`，L46-48 把
`report.candidate_manifest` 先过 `evidence()` 再取其 `.sha256` 与
`report.subject` 比对。字符串形态在 `ref.path` 处即 undefined，直接判
Invalid。派发模板若只给「artifacts 列表 + 路径字符串」样例，子代理会按
字符串形态产出报告，错误延迟到绑定日才爆。

## 解决方案

- 派发模板明示对象形态并给样例：`{"path": "evidence/x.md",
  "sha256": "<实算>"}`；`candidate_manifest` 同形态且其 `sha256` 必须等于
  manifest 文件真实哈希、`source_sha` 等于报告 `head_sha`。
- sha 一律现场实算（`sha256sum`），不从上游字段转抄。
- 校验器排障用直调取真实错误：`node -e` 直接 `import` `evidence()` 传入引用，
  得到具体失败键，而不是只看顶层 `Invalid evidence reference`。

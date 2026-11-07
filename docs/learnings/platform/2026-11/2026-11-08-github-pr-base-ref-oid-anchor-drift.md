---
date: 2026-11-08
modules: crates/engine
---

# GitHub PR baseRefOid 粘滞锚：head push 不重锚，main 未前进时 PR 侧事件也不重锚

## 问题描述

验收交付把 PR 身份键（base_sha）绑到哪个提交，取决于 GitHub `baseRefOid` 的行为，而它并不总等于分支真基（merge-base）。T47/T48 期间形成的口径「head push 会重锚 baseRefOid」在 T49 被证伪；「PR 侧事件重锚」的边界也比记想的窄。

## 根因分析

实测（T49 全程，PR #284）：PR 创建时 baseRefOid 锚定当时 main tip（43ed4c25）；之后两次 head push、多次 body edit、ready 标记，baseRefOid 全程不动；main 期间前进到 4541beea 与 fc331f09e 均未触发重锚。重锚只在「main 已前进 + 新的 PR 侧事件（如 --base edit）」组合下发生（T47 实证）。锚与真基不一致不影响审阅面：三点 diff（`git diff A...head`，按 merge-base）在锚选择下逐字节恒等，两点 diff 才会混入 main 侧中间提交造成「身份变化」假警。

## 解决方案

- 身份绑定策略：manifest/base_sha 绑 PR baseRefOid 现值（粘滞锚），manifest note 披露真基 merge-base 与三点 diff 的 sha256 恒等证据；ready/合并前重探 baseRefOid，若重锚则按补齐路径换绑 manifest。
- 身份复核一律用三点 diff 字节恒等，不用两点。
- 判定某事件是否重锚，以当时 main 是否前进 + 事件类型两个变量实测为准，不沿用上一任务的口径（本条即是对 T47/T48 口径的修正）。

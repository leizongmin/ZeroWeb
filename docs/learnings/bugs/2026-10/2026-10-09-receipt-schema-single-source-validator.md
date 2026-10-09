---
date: 2026-10-09
modules: .agents/skills/zeroweb-site-optimizer
---

# 回执 schema 以校验器代码为单一事实源，不镜像既有文件的形状

## 问题描述

slice48 运行中连续踩中三个同类坑，共性都是「照既有文件的样子手抄，而既有文件本身不是契约」：

1. **merge 回执**：把 GitHub API 快照（head/mergeable/checks 原始返回）直接当作 `delivery.merge` 回执写入，校验器报 `Invalid delivery receipt`。事后核对 slice47 同名文件才发现：`pr115-merge-checks.json` 是 artifact（原始快照），真正的回执是 `op-s47-merge.json`，schema 为 `schema_version`/`task_id`/identity 五元组/`confirmed`/`commit`/`review_sha256`/`artifacts`。
2. **trial 字段**：晋升 trial 时写到 checkpoint 顶层 `trial` 字段，实际契约字段是 `versions.trial`；顶层字段 validator-inert（历史值恒 null），写错静默不生效。
3. **时间戳**：Python `strftime('%z')` 产出 `+0800`，校验器要求 `[+-]\d\d:\d\d`，`Invalid timestamp`；应使用 `datetime.now(ZoneInfo(...)).isoformat(timespec='seconds')`。

## 根因分析

schema 的事实源在 `.agents/skills/zeroweb-site-optimizer/scripts/verify-delivery.mjs`、`verify-run.mjs` 等校验器代码里；运行目录中的历史文件只是「曾经通过校验的实例」，artifact 与 receipt 又常常同目录同名前缀并存，按形状抄写等于把别人的实例误当契约，甚至把 artifact 抄成 receipt。

## 解决方案

写任何回执/检查点字段前，先读对应校验器源码里的判定条件（字段名、类型、交叉引用），再写：

- 交付回执 → `verify-delivery.mjs`（merge 需 `confirmed===true` + `review_sha256` 与 delivery.review 哈希相等；integration 需 `status==='PASS'` + `merge_commit===merge.commit` + `contains_merge===true` + 64 位 hex subject）
- 检查点 → `verify-run.mjs`（`versions.{original,best,trial}`，timestamp 带冒号时区）
- 原始快照类证据（GitHub API 返回等）单独存文件作 artifact，不冒充回执。

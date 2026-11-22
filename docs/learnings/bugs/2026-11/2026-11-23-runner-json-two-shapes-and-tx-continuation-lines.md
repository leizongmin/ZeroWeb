---
date: 2026-11-23
modules: testing
---

# 电池判读防真绿：runner JSON 两种形态与 transcript 续行消息

## 问题描述

同目录 `.acceptance/.../solo-r65/*-b0/css-values.json` 与旧代 solo 目录同名 JSON，判读脚本解析一个得 5080 键、另一个得 0 键——三门全 0 的「完美绿灯」其实是解析落空（定向腿 filter 空真绿同族的形态变体）。

## 根因分析

runner 产物有两种 JSON 形态，按调用方式不同而变：

1. **per-case 列表形**：`[[page, [{name, status, message, ...}], ...]]`——旧代电池形态，`isinstance(data, list)` 分支可解析；
2. **聚合摘要形**：`{"corpus": ..., "totals": {"pass":..., "fail":...}, "cases": [{"case", "pass", "fail"}]}`——r65 起形态，只有页级计数，无键级数据。

键级数据在 transcript（`/tmp/<batch>-tx-<page>-<arm>.log`，stdout 的 `transcript:` 行指向）。transcript 行格式 `Pass|Fail <page> :: <case name>`，**Fail 的 assert 消息在下一行缩进续行**（`  assert_equals: expected "X " but got "Y "`）——只解析单行会把 got 全部替换成无名兜底。另有旁支：聚合形 JSON 的 `totals` 可能缺 `fail` 键（全绿页），直接 `totals['fail']` 会 KeyError。

## 解决方案

- 判读脚本先探测 JSON 形态（dict→聚合、list→键级），聚合形改读 transcript 重建键级映射；发射前核对解析键数与页数，0 键=解析落空不是零回归。
- transcript 解析必须折叠缩进续行进前一行键的消息，再抽 `but got "..."`。
- 跨代比较（F2P-BREAK 等）先按页集合求交集：14 页 solo 与全语料 r5 账本直接对比会把「键不在本电池」错算成「键被打破」（7773 假 F2P）。

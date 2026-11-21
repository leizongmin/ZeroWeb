---
date: 2026-11-22
modules: css-parser, style-system, engine, wpt-runner
---

# 同一二进制跨进程翻转：random() 文档指针种子使单案 A/B 差分掷币

## 问题描述

wptfyi-css 电池的 A/B 对账里，`css/css-values/random-computed.tentative.html` 的
`Shared globally: random(--identifier, a, b)` 案多次表现为「b0 全绿、b1 红 1 例」，
页级差分形似 patch 因果的回归（patch 前通过、patch 后失败）。该案先后在
刀 H、KA2、G1a、MATHFN-LENPERC 等多轮电池中出现，每轮都要重新归因。

## 根因分析

两层机制叠加，第二层尚未完全定谳：

1. **采样种子含 Document 指针，进程间必变。** style-system 在 compute 期构造
   `RandomEvalContext`，`doc_seed` 取 Document 指针 `as u64`；`css-parser` 的
   random() 求值（`crates/css-parser/src/values/random.rs`）把它混入 sha256
   采样 base。Document 分配在堆上，地址每次进程启动都不同，所以同一个
   runner 二进制、同一页面，每个新进程都会重抽一次采样值。

2. **width/height 计算值串化舍入方向不一致（既存缺陷）。** 探针页实测：同一
   采样原始值 68266.7px，getComputedStyle 的 width 返回 68267px（进位）、
   height 返回 68266px（截尾）。该案断言两个元素的计算 width 与 height 相等
   （同名 identifier 应共享同一采样值），所以案的红绿取决于采样值的小数部分：
   小数超过 0.5 时两通道输出分歧，案红。实测普查（2026-11-22）：同一 b0
   二进制连开 4 个新进程全部红；合并电池腿计 b0 七抽四红、b1 七抽三红，合计
   十四抽七红，红率约 50%，双臂对称，与 patch 无关。

已知缺口：同一进程内，不同元素对的 height 也观察到 ±1 摆动（一对 278205px、
另一对 278206px），说明舍入链还有元素上下文参与，完整机制归数值精度域待
独立定谳。但「跨进程掷币、非 patch 因果」这个归因结论不受该缺口影响。

## 解决方案

**归因协议**：A/B 对账出现孤立单案翻转（±1）且读码找不到 patch 可达路径时，
先做「同 bin 掷币普查」再定账：

1. 用同一个 runner 二进制对目标页连跑至少 4 个新进程，统计该案翻转；双臂
   各做一轮建立基线红率。翻转出现即归环境形非决定性，不记 patch 账。
2. 需要机制实证时，写一次性探针页（自造 identifier，加一个无条件 dump
   计算值的失败测试）在 b0 直接复现同机制，排除「原页面恰好抽到好种子」
   的运气。探针页放进 `tests/wpt-runner/wpt-data` 会被 runner 语料枚举扫入，
   污染后续对账；该目录整体在 .gitignore 里，git status 不显示，删除没有
   提交风险提醒，用完必须手动删。
3. 账务处置双向剔除（不计 F2P 也不计 P2F）；run 内一致性不受影响，A/B
   结构本身仍有效。

**遗留缺陷挂账**：width/height 串化舍入分歧是真实的 pre-existing 缺陷，
属于数值精度域，需独立刀修复；修好后该案变确定性绿，掷币现象随之消失。

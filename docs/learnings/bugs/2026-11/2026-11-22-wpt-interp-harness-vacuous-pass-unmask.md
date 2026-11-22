---
date: 2026-11-22
modules: zero-engine, tests/wpt-runner
---

# WPT interpolation 页 b0「全过」可能是双空串空洞通过：加 serialize 臂触发 unmask 型 P2F

## 问题描述

KC-4 刀（为 `contain-intrinsic-size` 加 gCS 简写聚合臂）电池判读出现
interp 页 P2F=246，形态像「face 破坏了原本健康的插值通道」。b0（剥
face 基线）显示 interpolation 页 432/432 全过，易据此推出「KC-4 打断
了 b0 已工作的行为」——归因完全反向。

## 根因分析

`css/support/interpolation-testcommon.js` 的断言**双侧都读
getComputedStyle**：actual 读 target、expected 读独立 expected 元素
（:387/:406），且无 expected 非空守卫。当引擎 serializer 对该属性无
臂（`_ => String::new()`）时，双侧恒空串，`assert_equals('','')` 恒
真——整个插值页在 b0 从未测过任何行为。b1 贴上聚合臂后 gCS 首次产
出真值（typed 静态场），而引擎插值器/过渡注册/apply 写臂对 cis 全
缺位，registry 查询回落静态 → got 值非空且错 → 全线红。b0 的
Pass 案 `message=null`，断言名里的「真中点 41px 45px」只是期望串，
不是 got 值。

判别要点：b1 失败 message 的 got 值形态即通道诊断——CSSA/WAAPI 全
采样 got 静态 underlying、transitions 全采样 got 端值级联 = 「零插
值 + registry 回落」，而非「破坏了插值」。旁证探针：双臂各跑一条
`gCS(cis)` 读数，b0 全空串即定谳。

## 解决方案

- 基线验收不可信 harness 双读 gCS 页面的 b0 全过；先核 serializer
  在基线是否有该属性臂（`git show HEAD:<file> | grep`）。
- 给「无臂属性」新增 serialize 臂时，预期 unmask 型 P2F：插值臂、
  过渡注册门、apply 写臂（canonicalize 采纳门 ser1≠ser0 依赖它）须
  同 face 补齐，序列化聚合臂单独落库会引爆假回归（fail-stop 红线
  正确拦下了 KC-4）。
- 注册门与采样门是两处：登记走 strict 插值器、采样走 lenient
  （query_core），只补一处仍红。
- 案例全文见 `.acceptance/wptfyi-css/NOTES-kc5-cis-interp-vacuous
  -b0-unmask.md` 与 `PREP-kc5-cis-interp-slot-merge.md`（机制二：
  简写槽＋长手槽 HashMap 迭代序非确定，另一独立雷）。

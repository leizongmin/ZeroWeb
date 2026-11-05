---
date: 2026-11-05
modules: wpt-runner
---

# WPT 探针全局调用计数被 testharness 并发臂污染

## 问题描述

s291（html-semantics 插入 hook un-gate）探针页在单个 HTML 文件里放六个
`promise_test` 臂，用全局包裹 `__zwImgRevalidate5277`/`__zwImgSelectionQueue5277`
的计数器测「插入臂零重验」。首跑 A/B/D/F 四臂报 rev 增量 1–3（预期 0），
看似守卫失效。

## 根因分析

testharness.js 在同一 runner job 里**并行执行**多个测试臂：全局计数器的
「mark → 窗口 → 比对增量」测的是全文档的重验调用总量，窗口内其他臂的
attr 路径调用全部计入（arm A 的 300ms 窗内 rev=3，其中本臂贡献 0）。
计数器没有归属维度，无法区分「本臂插入臂调用」和「他臂属性路径调用」。

## 解决方案

两层修正后六臂全绿：

1. **身份过滤**：包裹器记录每次调用的 `(sel, handle)` 实参序列，
   各臂 mark 时快照日志下标，比对时只统计
   `sel === img.__zwSelector && handle === img.__zwHandle` 的条目——
   shim wrapper 世界里探针侧 `document.createElement` 产物与引擎内部
   共用同一身份对，可直接相等比较。
2. **臂串行化**：全局 promise 链（`afterChain()` 模式）把六臂改为顺序
   执行，窗口内只有本臂活跃，残余噪声（sweep/后台链）仍由身份过滤兜底。

附：断言「恰好一次 load」这类语料无钉面的计数断言，先经撤片实验
（stash 改动重建前二进制对照）归因后再决定是否保留——本例双 load 是
s289 账本既定决策（state-less 首飞同 URL 重入不早退）的后果，非本片
回归，撤 load 断言留真钉面。

## 避免方式

探针页多臂共文档时，凡测「某次调用是否发生」，计数器必须带身份维度
或全臂串行；不带维度的全局增量窗只能用于「单测试单页」形态
（zz-s272-probe2 先例）。

---
date: 2026-11-15
modules: engine, css-parser
---

# shim 写面刀须双世界同落：handle 世界才是 corpus 活跃写面

## 问题描述

g44（color-stops-parsing 60F，见
`docs/goal/wptfyi-css/evidence/2026-11-15-g44-color-stops-style-attr-write-gate.md`）
首刀把 style 内容属性写面拒收只落在 part03 `_zwMElSetAttribute`（物化/detached-doc
世界），corpus 重录 css-images 2881P/334F 恒一、净 ±0——刀口落错世界，语料零感知。

## 根因分析

shim 有两套元素世界，`setAttribute('style', …)` 各有独立实现：

- **handle 世界**：live document `createElement`（part06:4886）→
  `__zw_create_element` → `_wrapHandle` → part04 `_makeProxy`；其
  `setAttribute` 经 `__zw_set_attr_handle` 原文落库。**corpus/runner 页面脚本
  的活跃文档在这里**。
- **物化世界**：detached doc（part03:27348）与查询物化产物 → `_zwMEl` →
  part03 `_zwMElSetAttribute`。WPT 用例的 `document.createElement` 不会走到
  这里。

判 world 不能看「哪段代码有 setAttribute」就接线：part02（DOMParser
parse 世界）、part03（物化）、part04（handle 代理）各有实现，语义验证
（corpus 逐页 diff）之前 totals 恒同完全掩盖落点错误。

## 解决方案

- 写面刀先实证页面世界再落刀：corpus A/B 逐页 diff 是唯一判据（totals
  恒同 ≠ 变更生效）；首录净 ±0 即世界落点错误的强信号。
- 同型写面缺口按双世界同落收口：part04（corpus 实际写面）与 part03
  （物化世界同型回显缺口）各加同款臂，共享 part06 单实现
  （`_zwStyleAttrDeclFilter`）。
- setProperty/cssText 声明面（`_zwNormalizeDeclValue` 消费面）世界无关，
  与 setAttribute 内容属性面是两条通道——g43 修前者的刀不会覆盖后者。

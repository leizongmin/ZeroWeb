---
date: 2026-11-21
modules: css-parser, engine
---

# 通道活性基线页实证：已落刀同通道页全绿即免引擎接线

## 问题描述

新增值串化/折叠逻辑时，容易按"双引擎 store 路径"推理预先接线多处
（native CSD、mutation replay、quickjs shim 写面钩子），但 WPT 页实际
只走其中一条。按假说全接线会把 face 拉长、扩大与并行流的文件交集
（engine part03/part06 域常被其他刀占用）。

## 根因分析

通道是否承载某类页测是经验事实，不是结构推导能定的—— shim 与
mutation replay 两条路径都可能承接 `e.style` 同步读，取决于 runner 的
DOM 桥实现。但通道上已有别的刀落地过：同一 `fold_decl_specified` 通道
的 sin-cos-tan serialize 刀上线后 `sin-cos-tan-serialize.html` 全绿
（270P/0F，b1 基线在案）。

## 解决方案

动手接线前，先在最近一次电池 JSON 里找**同一通道上已落刀的基线页**：

1. 认出待修页走的 API 路径（如 `e.style[prop] = v` 串化断言）；
2. 找同通道先例页（sin-cos-tan-serialize 之于 fold_decl_specified
   重放折叠通道）；
3. 基线页全绿 ⇒ 通道活性实证 ⇒ 新逻辑扩同函数即可达，免接线；
4. 基线页本身红 ⇒ 通道假说被推翻 ⇒ 才需要补引擎站。

CALCSIZE-CANON 刀（2026-11-21）据此把原计划的 callbacks.rs 注册 +
part06.js 钩子收缩为 css-parser 单文件 face，face 域与并行 MOTION-3
线（engine/apply 域）零交集，电池一次通过 +6 全数兑现。

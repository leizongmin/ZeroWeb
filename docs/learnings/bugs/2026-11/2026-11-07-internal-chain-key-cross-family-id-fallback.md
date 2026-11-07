---
date: 2026-11-07
modules: crates/engine
---

# 内部按 _elKey 建链的 map 查找跨族落空——按元素 id 回落对齐

## 问题描述

svg-compat-v2 B6b（svgrect-1）排查中实证：SMIL 动画链注册表
（`_chainOf`）按 `_elKey(animEl)` 建键，但同一动画元素从不同入口取得
的 wrapper 键不同源——querySelector 产物是 sel 族（`'#animation\0
\0smil'`）、文档扫描产物是 handle 族（`'null\0__n0\0smil'`）。以
A 入口注册、以 B 入口查找（或反之）时链查落空，采样静默返回基线值，
断言失败无任何报错。svgrect-1 目标是根 svg 元素（viewBox 动画），
页面先 querySelector 根元素、SMIL 扫描另建 wrapper，正是该形态。
跨族 id 回落接入后转 P。

## 根因分析

与 [同一树节点存在多族稳定 JS 包装——跨族 identity 恒败](../bugs/2026-10/2026-10-14-js-dom-wrapper-family-identity.md)同根、但受害面不同：
10-14 条讲的是**调用方可见的 identity 比较**（`===`、端点落值）；本条
是**引擎内部 map 的键空间**——凡按 `_elKey`（或 handle 直拼）建内部
索引的机制（动画链、缓存表、监听器表），跨族写入/读取互不可见。内部
索引比 identity 更隐蔽：没有 `===` 可探，症状是「注册了但查不到」，
行为退化到缺省路径且无异常。

## 解决方案

`_chainOf` 查找 miss/dead 时按元素 id 回落：链记录已存元素 id 属性，
id 非空则遍历 map 按 `rec.id` 对齐。可行性前提：语料动画元素恒带 id
（WPT harness `runAnimationTest` 按 `getElementById(animationId)` 寻
动画元素），id 冲突面为零。数值路径与 token 路径同享该回落。

## 避免方式

- 新建按 `_elKey`/handle 建键的内部索引时，注册入口与消费入口必须同
  族；无法保证时，设计期就留 id（或其它族无关稳定键）回落，而不是等
  症状显形再补。
- 「内部表注册了但查不到、行为静默退化」的排查手段：临时导出 debug
  探针对比注册侧与查找侧的键值（本例 `key=#animation\0\0smil` vs 链
  键 `null\0__n0\0smil` 一定位）；注意探针 assert 顺序，先打值再断言，
  避免断言先失败阻断探针。

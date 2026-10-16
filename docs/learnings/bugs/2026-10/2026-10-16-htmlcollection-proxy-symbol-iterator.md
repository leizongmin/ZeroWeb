---
date: 2026-10-16
modules: crates/engine, apps/renderer
---

# HTMLCollection 的 Proxy 必须接受 Symbol 键并实现迭代

## 问题描述

现代页面调用 `Array.from(document.images)` 时，ZeroWeb 抛出 `TypeError: Cannot convert a Symbol value to a string`，后续页面脚本与分块资源不再执行。MDN 实站的生产 GPU 对照捕获到同一异常；Chrome 的同页冷加载正常。

## 根因分析

`document.images` 等属性由 `_liveQueryCollection` 返回 Proxy。`Array.from` 先读取 `Symbol.iterator`，但 Proxy `get` trap 将该 Symbol 直接传给 `parseInt`，于是抛错。该 Proxy 同时缺少 HTMLCollection 的迭代器。其他集合 Proxy 存在相同的 Symbol 键风险。

## 解决方案

在 `HTMLCollection` 和 `HTMLOptionsCollection` Proxy 中实现 `Symbol.iterator`，用当前集合快照逐项迭代；非字符串属性键不得进入 `parseInt`。NamedNodeMap 的集合 traps 同样先区分 Symbol 键。保留测试覆盖 `Array.from`、`for...of`、表单控件、option 和 Attr 集合。

验证：修复前回归因上述 TypeError 失败；修复后 `zero-engine` 相关集合测试 27/27 通过，随后使用同页生产 Chrome/ZeroWeb 采集核对脚本与资源行为。

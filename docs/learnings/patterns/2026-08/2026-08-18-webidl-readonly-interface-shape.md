---
date: 2026-08-18
modules: crates/engine/src/js_dom_shim/part02.js
---

# WebIDL readonly interface shape

## 问题描述

IndexedDB cursor 将 `key`、`primaryKey`、`source` 和 `direction` 直接写成实例 own properties。值读取正确，但属性描述符仍为 writable，key-only cursor 也因实例上存在值为 `undefined` 的 `value` 属性而违反接口形状。

## 根因分析

WebIDL readonly 不只约束赋值结果，还约束属性描述符和继承位置。不同接口成员不能用统一实例字段近似；`IDBCursorWithValue.value` 不属于基础 `IDBCursor`。

## 解决方案

可变状态保存在内部字段，公开 readonly 成员统一定义为 prototype getter 且不提供 setter。只属于派生接口的成员定义在派生 prototype。验证同时检查值、`Object.getOwnPropertyDescriptor()` 和 `property in object`，避免只验证读取结果。

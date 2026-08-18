---
date: 2026-08-18
modules: engine, page-runtime, storage
---

# IndexedDB keyPath own-property and sparse-array wire

## 问题描述

KeyPath 扩面暴露了 prototype getter 被错误调用、空 keyPath 被当作 out-of-line、File 元数据丢失，以及稀疏数组 wire 在 JSON 序列化时读取继承属性。

## 根因分析

Key extraction、structured clone 和 JSON wire 各自实现了不一致的属性访问。普通赋值创建数组元素还会被 Object.prototype 上的只读 accessor 阻止，留下 hole，随后 `JSON.stringify` 再次触发继承 getter。

## 解决方案

KeyPath 统一按 own property 提取，仅对 String/Array/Blob/File 的规范属性特殊处理。缺失路径注入使用 own data property；数组 wire 逐索引 `defineProperty`，不读取 prototype；File 使用独立 wire 保留 name、type、lastModified；Proxy 和 sparse array 在 key conversion 边界拒绝。

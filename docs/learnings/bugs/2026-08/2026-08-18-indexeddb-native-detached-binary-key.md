---
date: 2026-08-18
modules: crates/engine/src/js_dom_shim/part02.js
---

# IndexedDB native detached binary key

## 问题描述

Native `MessageChannel` transfer 后的 detached TypedArray 作为 IndexedDB query key 时泄漏 V8 `TypeError`，WPT 要求同步抛出 `DataError` DOMException。

## 根因分析

既有代码依赖 shim 自定义 `_detached` 标记。V8 原生 transfer 会真正 detach ArrayBuffer，不设置该属性；后续 `new Uint8Array(detachedBuffer, ...)` 在 key 解析内部抛出引擎 `TypeError`。

## 解决方案

Binary key 统一通过 try 边界提取 buffer、offset 和 length，并实际构造 `Uint8Array` 验证可访问性。构造失败统一视为无效 key，由公开 IndexedDB API 映射为 `DataError`。不要用私有标记作为 native transferable 状态的唯一证据。

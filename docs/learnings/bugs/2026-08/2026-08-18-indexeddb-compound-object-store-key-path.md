---
date: 2026-08-18
modules: engine, page-runtime, storage
---

# IndexedDB compound object-store key path schema drift

## 问题描述

JS shim 已能创建数组 key path 的 object store，但 upgrade 在同步到 Rust host 时失败。Index schema 已支持 string/sequence，object-store schema 仍固定为 `Option<String>`。

## 根因分析

同一 WebIDL `keyPath` 在 JS、host wire、storage metadata 和持久化层使用了不同类型。只修 JS key 提取会让行为在 upgrade commit 或跨会话恢复时再次失败。

## 解决方案

复用 typed key-path 表示贯通四层，并保留原字符串构造 API。持久化使用 untagged string/array 表示，使旧字符串数据无需格式迁移即可继续读取。跨层 schema 功能必须同时验证首次创建、host inspect 和 manager 重建。

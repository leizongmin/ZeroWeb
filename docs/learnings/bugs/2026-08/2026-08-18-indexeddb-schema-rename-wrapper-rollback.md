---
date: 2026-08-18
modules: crates/engine/src/js_dom_shim/part02.js
---

# IndexedDB schema rename wrapper rollback

## 问题描述

Object store/index rename 不仅要修改 schema 名称，还要立即更新 transaction scope、
DOMStringList 和同一 transaction 内已暴露对象的身份。Versionchange abort 后，旧名称
必须在 `abort()` 返回前恢复。

## 根因分析

只替换数据库的 schema map 可以恢复后续查询，却不会恢复已经返回给页面的
`IDBObjectStore`/`IDBIndex` wrapper。Wrapper 仍持有被修改的名称、metadata 和索引 map，
导致数据库状态已回滚但页面可观察对象仍显示已撤销的 rename。

## 解决方案

按 transaction 缓存 object store wrapper，并按 store 缓存 index wrapper。名称缓存与
全部已暴露实例列表必须分离：删除后同名重建需要新 wrapper，abort 又必须能找到已从名称
缓存移除的旧 wrapper。Rename setter 原子迁移 schema map、scope 与缓存键，同时记录既有
对象的名称变更。Abort 时先标记本次 upgrade 新建对象，再倒序恢复既有 wrapper 名称，
重新绑定 snapshot metadata、records 和 indexes，最后恢复数据库 schema map。回归必须
同时断言数据库列表、transaction scope、wrapper 名称、deleted 状态和 SameObject 身份。

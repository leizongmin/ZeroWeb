---
date: 2026-08-22
modules: storage, webview
---

# Storage root layout compatibility

## 问题描述

给已有持久化 owner 增加新的存储后端时，不能把原本属于单一后端的公开 `path`
参数静默改成新的总 storage root。否则旧版本已经写在该目录下的数据会被新版代码
当成子目录布局，导致已有数据不可见。

## 根因分析

`IndexedDbOwner::persistent(path)` 先前语义是直接把 `path` 作为 IndexedDB root。
新增 CacheStorage 持久化时，如果复用 `StorageManager::with_persistence(path)`，IndexedDB
会被移动到 `path/IndexedDB`，与既有磁盘布局不兼容。

## 解决方案

保留公开 API 的旧路径语义：`IndexedDbOwner::persistent(path)` 继续把 `path` 传给
IndexedDB 持久化，同时把 CacheStorage 放到 `path/CacheStorage`。新增兼容性测试先用旧
`StorageManager::with_indexed_db_persistence(path)` 写入，再通过 `IndexedDbOwner::persistent(path)`
读回，锁住旧布局可恢复。

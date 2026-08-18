# IndexedDB cursor transaction view

**日期**: 2026-08-18

**相关模块**: `crates/page-runtime/src/indexed_db_host/cursor.rs`

## 问题描述

Index cursor 打开后更新当前记录的 index key，再调用 `continue()` 时，静态 entries 快照无法反映 transaction buffered mutations。记录只被访问一次，而规范要求它按新 index 位置重新进入迭代。

## 根因分析

Cursor open 时预计算并永久保存全部 entries。Transaction 后续 `put/delete` 只更新 buffered view，不更新 cursor 快照，因此 cursor stepping 与同一 transaction 的可见数据分叉。

## 解决方案

Cursor registry 保存 store、index、query、direction 和 key-only 元数据。每次 continue/advance 都从 transaction 最新 view 重建并规范化 entries，再相对调用前的 `(key, primaryKey)` 选择下一项。Index cursor 使用 pair 顺序，object store cursor使用 primary key 顺序；unique/reverse 规则在每次重建后重新应用。

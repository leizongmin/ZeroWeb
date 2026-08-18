---
date: 2026-08-18
modules: crates/storage/src/indexed_db/types.rs, crates/page-runtime/src/indexed_db_host/cursor.rs
---

# IndexedDB transaction latest view

## 问题描述

同一 readwrite transaction 先删除已有主键再 `add()` 相同主键时，`tx_add` 仍只检查 live store 和正向 mutation 列表，错误报告 key 已存在。Cursor 每次 step 又无条件重建并排序全部 transaction view，大型迭代形成 O(n²)。

## 根因分析

事务操作存在两套可见性判断：`tx_get` 已按 mutation reverse order 实现 latest view，`tx_add` 却重复实现不完整的存在性检查。Cursor 则没有判断 transaction view 是否实际变化。

## 解决方案

`tx_add` 复用 `tx_get` 判断主键存在性。Transaction registry 维护 mutation generation，cursor 保存 observed generation；generation 未变化时复用排序 snapshot，发生 add/put/delete/clear 后才重建。回归同时覆盖 delete 后同 key add、index key 更新后重新进入迭代和 1000-record WPT。

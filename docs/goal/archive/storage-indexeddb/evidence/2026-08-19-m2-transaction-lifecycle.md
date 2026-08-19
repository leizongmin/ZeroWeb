# M2 transaction lifecycle

**日期**: 2026-08-19

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 explicit commit、transaction 基础反射、versionchange 内创建
transaction、durability、upgrade commit/user abort/backend abort 与 deactivation timing。

## 结果

- 修复前新增切片：21 Pass / 9 Fail / 0 Timeout
- 修复后新增切片：30 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：147 文件 / 868 Pass / 0 Fail / 0 Timeout /
  0 NotRun / 0 empty

## 实现

- inactive transaction 调用 commit() 同步抛 InvalidStateError
- IDBOpenDBRequest 使用独立继承类型与正确 WebIDL class tag
- IDBDatabase/IDBTransaction 补齐 class tag
- active versionchange transaction 期间禁止创建普通 transaction
- transaction 第三参数支持 default/strict/relaxed durability
- 非法 durability 同步抛 TypeError

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（147 文件 / 868 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- engine IndexedDB 定向回归：25 Pass
- fetch / runner / ledger 清单：147 / 147 / 147

# M2 metadata rollback

**日期**: 2026-08-19

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 versionchange abort 后的 object store/index metadata、
key generator、transaction SameObject、objectStore exception order 和 finished guard。

## 结果

- 修复前新增切片：14 Pass / 5 Fail / 0 Timeout
- 修复后新增切片：19 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：123 文件 / 716 Pass / 0 Fail / 0 Timeout /
  0 NotRun / 0 empty

## 实现

- deleteObjectStore 立即清空已暴露 wrapper 的 indexNames 并标记 index deleted
- abort 时用全部已暴露 index wrapper 列表恢复被删除的既有 index metadata
- 名称缓存与实例列表分离，允许同名重建同时保留旧 wrapper 的回滚能力
- transaction 已结束后 objectStore.index() 按规范抛 InvalidStateError
- transaction 仅暂时 inactive 时仍保持 TransactionInactiveError

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（123 文件 / 716 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- engine IndexedDB 定向回归：23 Pass
- fetch / runner / ledger 清单：123 / 123 / 123

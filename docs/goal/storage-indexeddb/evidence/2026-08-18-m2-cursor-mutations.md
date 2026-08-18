# M2 cursor mutations

**日期**: 2026-08-18

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 7 个上游文件，覆盖 `IDBObjectStore.getKey`、object store `openKeyCursor`、`IDBCursor.update/delete`、异常顺序，以及修改 index key 后继续迭代。

## 结果

+ 修复前：2 Pass / 50 Fail
+ 修复后：52 Pass / 0 Fail
+ 完整 imported 矩阵：59 文件 / 329 Pass / 0 Fail / 0 Timeout / 0 NotRun

## 实现

+ `getKey` 复用 transaction keys-only query，支持 key/range/no-match
+ cursor update/delete 复用 Rust transaction put/delete，并保持 request event model
+ mutation guard 对齐 inactive → deleted source → readonly → got-value/key-only 的异常顺序
+ host cursor 保存 store/index/query/direction，每步从 transaction 最新 view 重建 entries
+ index key 更新后的记录可按新位置再次进入 cursor iteration

## 门禁

+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make testharness-indexeddb`：Pass（59 文件 / 329 Pass）
+ `make test`：Pass
+ fetch / runner / ledger 清单：59 / 59 / 59

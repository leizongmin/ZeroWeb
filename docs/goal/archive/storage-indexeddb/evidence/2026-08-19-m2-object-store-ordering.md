# M2 object-store ordering

**日期**: 2026-08-19

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 database transaction 与 object-store
add/put/clear/delete/query 的异常顺序、request source、range delete 和 explicit commit guard。

## 结果

- 修复前新增切片：42 Pass / 1 Fail / 0 Timeout
- 修复后新增切片：43 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：139 文件 / 838 Pass / 0 Fail / 0 Timeout /
  0 NotRun / 0 empty

## 实现

- IDBDatabase.transaction 保持 closed connection 检查最高优先级
- scope 转换、空 scope 和 store existence 检查先于 mode 校验
- 无效 store 与无效 mode 同时存在时按规范抛 NotFoundError
- 已有 object-store CRUD guard、request source、range delete 和 commit guard 纳入固定 WPT

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（139 文件 / 838 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- engine IndexedDB 定向回归：25 Pass
- fetch / runner / ledger 清单：139 / 139 / 139

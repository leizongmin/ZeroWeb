# M2 index metadata

**日期**: 2026-08-18

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 create/deleteIndex、异常顺序、indexNames、
sequence keyPath、objectStore SameObject、request source 和 query guards。

## 结果

- 首轮定向运行：42 Pass / 6 Fail / 0 Timeout（48 个已报告 subtest）
- 修复后定向切片：49 Pass / 0 Fail / 0 Timeout
- 完整 imported 矩阵：107 文件 / 656 Pass / 0 Fail / 0 Timeout /
  0 NotRun / 0 empty

## 实现

- createIndex/deleteIndex 仅允许 active versionchange transaction，并按规范顺序抛错
- duplicate index name 在 keyPath 语法校验前抛 ConstraintError
- deleteIndex 对缺失 index 抛 NotFoundError，并保留 deleted metadata 状态
- sequence keyPath 在每个 IDBIndex 实例上独立复制，同一实例保持 SameObject
- versionchange abort 事件沿 transaction 到 database 冒泡
- schema host ConstraintError 保留为事务内部上下文，open request 对外报告 AbortError

## 门禁

- `cargo fmt --all -- --check`：Pass
- `cargo clippy --workspace --all-targets -- -D warnings`：Pass
- `make testharness-indexeddb`：Pass（107 文件 / 656 Pass / 0 empty）
- `make test`：Pass（V8 + GPU adapter + QuickJS）
- engine IndexedDB 定向回归：22 Pass
- fetch / runner / ledger 清单：107 / 107 / 107
- 固定 WPT 诊断资产与上游 checkout 一致

# M2 cursor iteration

**日期**: 2026-08-18

## 范围

固定 WPT revision：`315976933870b34d6ea30e3f6643403edae678ba`

新增 8 个上游文件，覆盖 object store/index 的四种 cursor direction、key range、cursor reuse，以及迭代期间 update/delete/add 的 transaction latest view。

## 结果

+ 修复前：20 Pass / 0 Fail / 1 Timeout
+ 修复后：21 Pass / 0 Fail / 0 Timeout
+ `idbcursor_iterating.any.js`：10.3s Timeout → 1.86s Pass
+ 完整 imported 矩阵：75 文件 / 381 Pass / 0 Fail / 0 Timeout / 0 NotRun / 0 empty

## 实现

+ cursor registry 记录 transaction mutation generation；view 未变化时复用 snapshot，mutation 后才重建
+ `tx_add` 使用 transaction latest view 判断主键存在性，支持同一事务 delete/clear 后重新 add
+ auto-commit 从 pending busy polling 改为事件驱动 completion check
+ versionchange transaction 在 `upgradeneeded` task 入口显式激活
+ testharness runner 对到期 timer task 使用 yield 快速推进，并拒绝已注册测试的空结果假绿

## 门禁

+ `cargo fmt --all -- --check`：Pass
+ `cargo clippy --workspace --all-targets -- -D warnings`：Pass
+ `make testharness-indexeddb`：Pass（75 文件 / 381 Pass / 0 empty）
+ `make test`：Pass
+ fetch / runner / ledger 清单：75 / 75 / 75

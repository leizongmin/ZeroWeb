# M2 cross-connection transaction scheduling core

**日期**: 2026-08-18

## 结果

同一 database 的 transaction 顺序表从单个 `IDBDatabase` connection 提升到共享 database state。存在 scope 冲突时，后创建的 transaction 延迟启动；其 `get` / `put` 操作在前序 transaction 完成后才进入 Rust host transaction。不同 database 与互不冲突的 readonly transaction 仍可并行。

## 上游 WPT

固定 revision：`315976933870b34d6ea30e3f6643403edae678ba`

+ 修复前：7 文件 / 3 Pass / 4 Fail
+ 修复后：7 文件 / 7 Pass / 0 Fail
+ 完整 imported 矩阵：45 文件 / 229 Pass / 0 Fail

新增用例覆盖 across-connections、across-databases、mixed-scopes、ordering、readonly waits for readwrite、readwrite scopes 和 readonly concurrency。

## 本地回归

`test_indexeddb_transactions_schedule_across_connections` 验证两个 connection 上先创建 `readwrite`、后创建 `readonly` 时，事件顺序为 `write → write-complete → read:new → read-complete`。

## 剩余

+ 将等待执行扩展到所有 object store、index 和 cursor operation
+ 将 scheduler ownership 提升到 browser owner，覆盖跨 renderer connection
+ 实现跨 connection `blocked` / `versionchange` 通知与 open request queue

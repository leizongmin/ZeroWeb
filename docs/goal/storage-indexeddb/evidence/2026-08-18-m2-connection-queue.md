# M2 connection request queue

**日期**: 2026-08-18

## 结果

同一 database name 的 `open()` / `deleteDatabase()` 请求进入共享 FIFO。版本升级或删除遇到存量 connection 时，先派 `versionchange`；事件处理后仍有 connection 才派一次 `blocked`；`IDBDatabase.close()` 唤醒队首请求。upgrade success、abort/error 和 delete success 都会释放队首并启动下一请求。

## 上游 WPT

固定 revision：`315976933870b34d6ea30e3f6643403edae678ba`

+ `idbdatabase_close.any.js`：0 / 2 → 2 / 2
+ `open-request-queue.any.js`：FIFO assertion Pass
+ 完整 imported 矩阵：47 文件 / 232 Pass / 0 Fail

## 本地回归

`test_indexeddb_blocked_upgrade_waits_for_connection_close` 固定事件顺序：

`versionchange → blocked → upgrade → success`

## 剩余

+ browser owner 级跨 renderer connection registry
+ browser→renderer `versionchange` / `blocked` 异步通知
+ 等待执行扩展到所有 object store、index 和 cursor operation

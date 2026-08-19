# M1 CRUD object store 生命周期 guard

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| Pass | 22 / 54 | 32 / 54 | +10 |
| Fail | 32 | 22 | -10 |
| 通过率 | 40.74% | 59.26% | +18.52pp |

`idbobjectstore_delete`、`clear`、`count` 三个文件已全绿。

## 修复

+ store handle 持有共享 metadata，deleteObjectStore 后旧 handle 立即失效
+ deleted store 操作统一抛 InvalidStateError
+ readonly transaction 写操作统一抛 ReadOnlyError
+ aborted/finished transaction 操作统一抛 TransactionInactiveError
+ add/put/get/delete/clear/count/index/cursor 共用同一 guard

## 下一步

实现 add/put/get 的 key 提取与 DataError 校验，共 11 个失败。

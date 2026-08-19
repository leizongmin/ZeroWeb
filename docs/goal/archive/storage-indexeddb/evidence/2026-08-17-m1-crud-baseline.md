# M1 object store CRUD 首批基线

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`
**范围**: add/put/get/delete/clear/count 6 文件

## 结果

| 指标 | 数值 |
|---|---:|
| 文件 | 6 |
| subtest | 51 |
| Pass | 15 |
| Fail | 36 |
| Timeout / Unsupported / NotRun | 0 |
| 通过率 | 29.41% |

## 失败聚类

| 数量 | 根因 |
|---:|---|
| 11 | key / DataError 校验 |
| 2 | 非法 index key 处理 |
| 5 | deleted store 的 InvalidStateError |
| 4 | readonly transaction 的 ReadOnlyError |
| 1 | aborted transaction 的 TransactionInactiveError |
| 3 | duplicate key / unique index ConstraintError |
| 6 | autoIncrement keyPath 与 cursor continuation |
| 4 | IDBKeyRange 与 count/get/delete query 语义 |

## 结论

CRUD 首批真实分母已建立。与 factory 50/50 合并后，当前 imported 范围为 65/101 Pass
（64.36%）。下一轻量修复选择 key range/query 4 项，根因集中且不依赖 Rust bridge。

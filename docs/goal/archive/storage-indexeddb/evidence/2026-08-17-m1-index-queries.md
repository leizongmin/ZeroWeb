# M1 index query 与生命周期

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| Pass | 8 / 20 | 20 / 20 | +12 |
| Fail | 12 | 0 | -12 |
| 通过率 | 40.00% | 100.00% | +60.00pp |

`idbindex_get`、`idbindex_getKey`、`idbindex_count` 三文件全部通过。imported 总计提升为
158 Pass / 166 subtest（95.18%）。

## 修复

+ index metadata 共享 deleted、multiEntry、createdInUpgrade 状态
+ deleteIndex 与 aborted versionchange 立即使对应 index handle 失效
+ get/getKey/count 按 index key 过滤，并按 index key、primary key 排序
+ key range 与非法 query 分别执行范围匹配和同步 DataError
+ runner 在 active step_timeout 完成前不接受 harness completion

## 下一步

实现 index openCursor 与 continue(key)，消除剩余 8 个失败。

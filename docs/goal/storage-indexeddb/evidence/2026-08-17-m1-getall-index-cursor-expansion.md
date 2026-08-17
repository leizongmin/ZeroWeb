# M1 getAll、index、cursor 扩面

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`

## 新增分母

| 分组 | 文件 | Pass | Fail | 通过率 |
|---|---:|---:|---:|---:|
| Object store getAll/getAllKeys | 2 | 34 | 0 | 100.00% |
| Index get/getKey/count | 3 | 8 | 12 | 40.00% |
| Index cursor continue | 1 | 0 | 8 | 0.00% |
| 新增合计 | 6 | 42 | 20 | 67.74% |

imported 总计扩展为 21 文件、166 subtest、146 Pass、20 Fail，通过率 87.95%。

## 修复

+ runner 支持按 META 顺序注入多个 support script
+ support 与 case 合并为同一 classic script，保持顶层 lexical 共享
+ IDBKeyRange 增加 lowerBound/upperBound 与单边界 includes
+ object store 增加 getAll/getAllKeys query、count、排序、克隆和 DataError
+ transaction 增加 commit 表面，并由 pending request 决定 complete 时点
+ MessagePort ArrayBuffer transfer 使用原生 transfer 真正 detach

## 下一步

先修 index get/getKey/count 的 query 与 deleted state，再实现 index cursor 和 continue(key)。

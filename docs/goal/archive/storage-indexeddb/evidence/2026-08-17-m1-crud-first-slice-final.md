# M1 CRUD 首批完成

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`

## 结果

| 指标 | 修复前 | 修复后 | 变化 |
|---|---:|---:|---:|
| Pass | 51 / 54 | 54 / 54 | +3 |
| Fail | 3 | 0 | -3 |
| 通过率 | 94.44% | 100.00% | +5.56pp |

首批 6 个 object store CRUD 文件全部通过。连同 factory/global/event 首批，当前 imported
15 文件共 104 个 subtest 全绿。该结果不代表上游 IndexedDB 目录整体通过率。

## 修复

+ add 重复主键返回异步 ConstraintError request
+ add/put 唯一索引冲突返回异步 ConstraintError request
+ request.error 在 request 进入 done 状态时才可见
+ error event 可取消；preventDefault 后 versionchange transaction 继续完成
+ 未取消的 request error 会 abort 所属 transaction

## 下一步

扩展 getAll、index 和 cursor 上游 WPT，建立下一批真实分母。

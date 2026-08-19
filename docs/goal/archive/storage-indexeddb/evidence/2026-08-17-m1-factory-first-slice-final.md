# M1 factory 首批 50/50

**日期**: 2026-08-17
**上游 revision**: `315976933870b34d6ea30e3f6643403edae678ba`
**范围**: factory/global/event 首批 9 文件

## 结果

| 指标 | 初始基线 | 最终 | 变化 |
|---|---:|---:|---:|
| Pass | 6 / 50 | 50 / 50 | +44 |
| Fail | 44 | 0 | -44 |
| 通过率 | 12.00% | 100.00% | +88.00pp |
| Timeout / Unsupported / NotRun | 0 | 0 | 0 |

## 最后一刀

+ request.transaction 在 upgradeneeded 期间暴露 versionchange transaction
+ transaction complete 先于 open success
+ transaction abort 先于 request error，错误为 `AbortError`
+ abort 原子恢复 version、object stores、records 和 indexes
+ unique index 冲突自动 abort
+ rollback 验证所需的 store reverse cursor 与 index.getKey

## 结论

导入的 factory 首批真实 WPT 已全部通过。该结果不扩大为整个 IndexedDB 目录结论：
页面仍走 in-memory 实现，Rust 引擎接线和持久化尚未开始；M1 下一步继续导入 object store
CRUD 用例并建立第二分类分母。

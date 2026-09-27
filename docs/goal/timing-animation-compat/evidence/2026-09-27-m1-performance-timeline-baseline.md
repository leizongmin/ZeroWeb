# M1 / DC-1 — performance-timeline window 子集通过率基线

**日期**: 2026-09-27
**套件**: `make testharness-performance-timeline`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-27-m1-performance-timeline-baseline.json](2026-09-27-m1-performance-timeline-baseline.json)

## 结果

| 指标 | 值 |
|---|---|
| 导入面 | 37 案（17 top-level html + 20 `.any.js` window 变体）|
| 执行 | 24 案（13 案内容级 skip：bfcache/dispatcher ×6、worker ×3、iframe ×4）|
| subtests | **19/44 = 43.2% Pass**（Fail × 16、Timeout × 9）|
| 全绿用例 | 7（po-observe.any、po-observe-type.any、po-observe-repeated-type.any、po-entries-sort.any、po-getentries.any、buffered-flag-observer.any、get-invalid-entries 外的 observer 生命周期面等）|

## 失败聚类（M2 修齐方向）

| 簇 | 案数 | 形态 | 归属 |
|---|---|---|---|
| completion 未达（Timeout）| 9 | 异步 observer 派发等待（po-callback-mutate / buffered-flag-after-timeout / droppedentriescount / po-disconnect-removes-observed-types / po-resource / webtiming-resolution / supportedEntryTypes / case-sensitivity / multiple-buffered-flag-observers）——microtask/任务队列派发时序 | M2 |
| entry buffer 重复计数 | 2 | `entries must match expected 1 but got 2`——buffered + 新 entry 双计 | M2 |
| measure 不抛（invalid 组合）| 2+ | `did not throw`——measure 参数校验面（SyntaxError/DOMException 语义，user-timing 同簇）| M2 |
| resource timing 依赖 | 2 | po-resource / resource entry 断言——resource-timing 为 goal 排除域（挂账：navigation-compat M2 后重入评估）| 记账 |
| entry 排序/状态断言 | 2+ | `sorted by startTime` 等 entry list 语义 | M2 |

已实现的真面（基线绿）：PerformanceObserver observe/disconnect/takeRecords 生命周期
主链、observe type/entryTypes 注册、entries 排序、getEntries 查询、buffered flag 主链。

## 导入面与排除

- 拉：top-level 17 html + helper（performanceobservers.js / navigation-id.helper.js）+
  `.any.js` 20 案（idlharness.any.js 不拉——WebIDLParser build 期资产，hr-time 同因）
- 运行面 skip ×13：
  - bfcache/navigationId 基建面 ×6（helper.sub.js 引用：back-forward-cache-restoration、
    navigation-id-{element-timing,long-task-task-attribution,mark-measure,reset,
    resource-timing}——跨窗会话历史机制，navigation-compat 流域记账）
  - worker 面 ×3（get-invalid-entries、navigation-id-worker-created-entries、
    worker-with-performance-observer——runner 无 worker 管道）
  - iframe 面 ×4（not-clonable、navigation-id-detached-frame、
    supportedEntryTypes-cross-realm-access、timing-removed-iframe）
- `/common/performance-timeline-utils.js`（user-timing 共用）已拉，经 runner
  inline_extras 绝对路径映射内联

## 下一步（M2）

1. Observer 异步派发时序（9 Timeout 簇——`_defer` microtask 近似 vs 任务队列语义）
2. entry buffer 去重/双计
3. measure 参数校验 DOMException 语义（与 user-timing 同簇合修）

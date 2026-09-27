# M1 / DC-1 — user-timing window 子集通过率基线

**日期**: 2026-09-27
**套件**: `make testharness-user-timing`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-27-m1-user-timing-baseline.json](2026-09-27-m1-user-timing-baseline.json)

## 结果

| 指标 | 值 |
|---|---|
| 导入面 | 35 案（15 html + 20 `.any.js` window 变体）|
| 执行 | 35 案（skip 0）|
| subtests | **241/354 = 68.1% Pass**（Fail × 108、Timeout × 5）|
| 全绿用例 | 15（mark/measure/clear 主链 html 大半 + user_timing_exists 等 any.js）|

四 corpus 中最高基线——R2768/R2821 shim 面（mark/measure/entry buffer/getEntries*/
clearMarks/clearMeasures + PerformanceObserver）已覆盖主链。

## 失败聚类（M2 修齐方向）

| 簇 | 案数 | 形态 | 归属 |
|---|---|---|---|
| 异常不抛（assert_throws_dom did not throw）| 42+ | mark/measure 非法参数应抛 SyntaxError/DataCloneError 等 DOMException——shim 静默或抛 TypeError（`property "code" is equal to undefined, expected 12` 形态同源）| M2 |
| `PerformanceMark is not defined` | 10+ | PerformanceMark/PerformanceMeasure 构造器接口缺位（mark-entry-constructor / structured-serialize-detail）| M2 |
| measure 语法错误类型错 | 12 | 抛 TypeError 而非 DOMException SyntaxError（measure_syntax_err.any 5 案全折此簇）| M2 |
| structured-clone detail 面 | 8 | mark/measure `detail` 参数结构化克隆 + `entry.detail` 缺省 null 语义 | M2 |
| mark 名校验面 | 4 | `The mark X does not exist`（should-throw 组合）| M2 |
| Timeout ×5 | 5 | completion 未达（supported-usertiming-types 等——observer 派发时序，performance-timeline 同簇）| M2 |

已实现的真面（基线绿）：mark/measure 创建与 entry 字段、getEntries{,ByName,ByType}、
clearMarks/clearMeasures（含点名/全清）、case-sensitivity、mark-measure-feature-detection、
supportedEntryTypes contains mark/measure。

## 导入面与排除

- 拉：top-level 15 html + `resources/` helper（user-timing-helper.js /
  webperftestharness.js / webperftestharnessextension.js——相对 src 经
  inline_local_scripts 自动内联）+ `.any.js` 20 案（idlharness.any.js 不拉，hr-time 同因）
- 运行面 skip ×0；`/common/performance-timeline-utils.js` 经 inline_extras 内联
- `.worker.js` 变体不拉（invoke_with_timing_attributes.worker.js、
  performance-measure-invalid.worker.js）

## 下一步（M2）

1. mark/measure 参数校验 DOMException 语义（最大簇 42+12+12，与 performance-timeline
   measure 簇同修）
2. PerformanceMark/PerformanceMeasure 接口构造器 + detail structured-clone
3. observer 派发时序（Timeout 尾簇）

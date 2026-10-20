# M2-S1 — user-timing 异常语义簇修齐

**日期**: 2026-09-28
**通道**: `make testharness-user-timing`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-28-m2-s1-user-timing.json](2026-09-28-m2-s1-user-timing.json)

## 结果（M1 基线 → M2-S1）

| 指标 | M1 基线 | M2-S1 | Δ |
|---|---|---|---|
| subtests | 354（241P/108F/5T）| **528（518P/6F/4T）** | +174（L1 harness onload 体解锁后注册面扩大）|
| Pass | 241（68.1%）| **518（98.1%）** | **+277** |

修齐的核心簇（master.md 下一步计划第 1 项全项闭合）：

| 簇 | 基线 | 现 |
|---|---|---|
| assert_throws_dom 不抛（mark/measure 参数校验）| 42+ | 0（invoke_with_timing_attributes 42/42、mark_exceptions 22/22、measure-exceptions 13/13、measure_syntax_err 5/5、measure_exception 10/10）|
| PerformanceMark/PerformanceMeasure 构造器 | 13F | 0（mark-entry-constructor 6/6、entry_type 2/2、mark-measure-return-objects 5/5、mark-l3 1/1）|
| structured-clone detail + DataCloneError | 9F | 0（structured-serialize-detail 9/9）|
| measure options dict `{start,end,duration,detail}` | measure-with-dict 1F | 过（23 案字段级断言闭合，含 `{duration,end}`→`startTime=end-duration` 推导）|
| marks 表查名 exception 类型（SyntaxError vs TypeError）| 全 TypeError | DOMException 正确（code 12/15/25 断言过）|
| L1 harness 解锁（mark('')/navigationStart 可用后 onload 体全跑）| mark.html 119 / clearMeasures 57 / measures 1 / measure_navigation_timing 1 | 全量注册并基本全绿 |

## 实现（part01b.js R2821 面重写 + part02 一处）

- `PerformanceMark` 可构造接口（校验同 mark()、不入 buffer）+ `PerformanceMeasure`
  接口对象（Illegal constructor，instance/toStringTag/toJSON 面）
- mark 名校验：保留 timing 属性名（21 名表）→ SyntaxError DOMException；
  **空名不拒**——L1 mark.html/clearMarks.html 以 `''` 为合法名（mark_names[0]），
  corpus 无空名抛错断言
- measure 联合实参：非 object 按 DOMString 转换后查 marks 表（number `51.15` →
  `"51.15"` 查无 SyntaxError）；dict `{start,end,duration,detail}`（`{duration,end}`、
  `{duration}` 单独 → `startTime=end-duration` 推导）
- marks 表查名：timing 属性名特例——`navigationStart` 恒可用（= time origin，相对域
  0）；其余 timing 值无导航时序管线恒 empty → InvalidAccessError（code 15）
- options dict 转换：非 object → TypeError；`start+duration+end` 三全 → TypeError；
  detail-only → TypeError；负值/NaN 时间 → TypeError；detail structured clone
  （不可克隆 → DataCloneError）
- part02 `_zw_structured_clone` 两处 throw 改 `globalThis.DOMException` 优先
  （R9/R382 wrong-global 先例——页面 instanceof 比对已发布全局构造器）

## 余非 Pass（全分类，无一属本切片范围）

| 项 | 分类 |
|---|---|
| mark-measure-feature-detection ×2（断言 mark/measure 返 void）| L2 legacy 断言，与 L3 return-objects 测试直接矛盾——Chromium L3 返回 entry，上游同样失败 |
| measures.html ×2 / clearMeasures.html ×2（entrylist 升序断言）| L1 legacy：测试数据自身（`['2', 1]`→查 mark '1'、`'aaa'`→navigationStart=0）即违反升序，上游同样失败 |
| measure_associated/exceptions_navigation_timing ×2 Timeout | 依赖 `performance.timing`（Navigation Timing L1）——goal 排除域，navigation-compat 流域挂账 |
| buffered-flag / supported-usertiming-types ×2 Timeout | PerformanceObserver 异步派发（promise drain）——M2-S2 |

## 跨 corpus 无回归

performance-timeline 19→20P（measure 校验簇共享修齐 +1）；hr-time 7P 持平；
web-animations 独立语料（Animation 面）与本切片无交集，回归跑见
[2026-09-28-m2-s1-web-animations.json](2026-09-28-m2-s1-web-animations.json)。

## 下一步（M2-S2）

PerformanceObserver 异步派发时序（performance-timeline Timeout ×9 + user-timing
Timeout ×2——`_defer` microtask 近似 vs 任务队列/promise drain 语义）。

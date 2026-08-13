# R17 — createEvent non-createable modern interface 抛 NotSupportedError（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R17
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**commit**: 见 `git log`（feat(js-dom): createEvent non-createable modern interface throws NotSupportedError）

## 背景

R14 createEvent alias 全覆盖后剩 15 失败。聚类：9 个 `Should throw NOT_SUPPORTED_ERR for non-legacy event interface`（AnimationEvent/TransitionEvent/PageTransitionEvent/PointerEvent/ClipboardEvent/ErrorEvent/ProgressEvent/PopStateEvent/WheelEvent）+ 6 个 TouchEvent assert_implements_optional。

根因：R14 把 modern event interface（non-createable）也加进了 createEvent map，但 spec createEvent 仅支持 **legacy** event interface；modern 接口应抛 NotSupportedError（modern 路径走 `new XxxEvent()` 构造器）。

## 改动

### 1. 移除 createEvent map 的 9 个 non-createable 接口（part06）

按 WPT someNonCreateableEvents 列表移除：wheelevent/pointerevent/popstateevent/progressevent/transitionevent/animationevent/pagetransitionevent/clipboardevent/errorevent。保留 createable legacy（DragEvent/InputEvent 等 + 用例 aliases 全集）。移除后 createEvent 对这些抛 NotSupportedError。

### 2. 更新 2 个受影响单测（part07）

- test_event_subclasses2_r2812：`createEvent('ProgressEvent') instanceof ProgressEvent`（旧 lenient）→ 改断言抛 NotSupportedError（ProgressEvent non-createable）。
- test_window_onerror_report_r2940：`createEvent('ErrorEvent')` → 改 `new ErrorEvent('error')`（modern 路径）。

### 3. event target null gap（核实：实际不存在）

R14 evidence 记录 createEvent event.target 默认 undefined（spec 期望 null）为 gap。R17 核实：_makeEvent（part03:1063-1064）已设 `target: null, currentTarget: null`，createEvent "initialized correctly" subtest（ev.target===null）已 Pass。**gap 不存在**（R14 误记）。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R16 | R17 | Δ |
|------|----|----|---|
| polyfill | 53.00% | **53.20%** | +0.20pp |
| native | 52.73% | **52.93%** | +0.20pp |

双路径对等差 0.27pp。**createEvent 用例**：264P/15F → **273P/6F**（+9 pass）。完整 JSON 快照入 evidence。

## 验证

engine v8 2086 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 剩余（createEvent 剩 6F）

TouchEvent assert_implements_optional（6）：polyfill testharness 对 OptionalFeatureUnsupportedError 未特殊处理（当作普通 fail 而非跳过）——testharness runner 行为 gap，非 createEvent 本身。

## 下一步

- createElementNS 保留原 tag 大小写（case.js prefix abc/Abc）/ classlist 剩 20F。
- iframe.contentDocument（深结构 html-compat 域，待评估）。
- testharness OptionalFeatureUnsupportedError 跳过语义（解 TouchEvent 6 + 其他 optional 用例）。

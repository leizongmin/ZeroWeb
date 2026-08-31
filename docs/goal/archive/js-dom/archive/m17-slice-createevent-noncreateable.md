# M4 R17 切片 — createEvent non-createable modern interface 抛 NotSupportedError

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**前置**: R16（classList toggle/replace）
**commit**: 见 `git log`（feat(js-dom): createEvent non-createable modern interface throws NotSupportedError）

## 背景

R14 后 createEvent 剩 15 失败：9 个 non-legacy interface 应抛（R14 误把 modern 加进 map）+ 6 个 TouchEvent assert_implements_optional。

## 改动

### 1. 移除 createEvent map 9 个 non-createable（part06）

WPT someNonCreateableEvents 列表：wheelevent/pointerevent/popstateevent/progressevent/transitionevent/animationevent/pagetransitionevent/clipboardevent/errorevent。spec createEvent 仅支持 legacy；modern 走 `new XxxEvent()`。

### 2. 更新 2 单测（part07）

test_event_subclasses2（ProgressEvent 抛）、test_window_onerror（ErrorEvent 改 new）。

### 3. event target null gap 核实不存在

_makeEvent 已设 target/currentTarget=null（part03:1063），createEvent 初始化测试已 Pass。R14 误记。

## 基线（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R16 | R17 | Δ |
|------|----|----|---|
| polyfill | 53.00% | 53.20% | +0.20pp |
| native | 52.73% | 52.93% | +0.20pp |

createEvent 用例 264P/15F → 273P/6F（+9）。双路径对等差 0.27pp。

## 验证

engine v8 2086 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 剩余

createEvent 剩 6F：TouchEvent assert_implements_optional（testharness OptionalFeatureUnsupportedError 跳过语义 gap）。

## 下一步

createElementNS 大小写 / classlist 剩 20F / iframe.contentDocument（深结构）/ testharness optional 跳过。

# M4 R14 切片 — createEvent alias 全覆盖 + 未知 type 抛 NotSupportedError

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**前置**: R13（classList 去重 + contains 空白不抛）
**commit**: 见 `git log`（feat(js-dom): createEvent alias coverage + NotSupportedError for unknown）

## 背景

Document-createEvent.https.html 第二大失败块（183 失败）。聚类：缺失 event 子类构造器（BeforeUnloadEvent 等 .prototype 崩）+ 未知 type 应抛 NOT_SUPPORTED_ERR（153）。

## 改动（3 文件）

### 1. 注册缺失 event 子类（part05）

BeforeUnloadEvent / DeviceMotionEvent / DeviceOrientationEvent / TextEvent / TouchEvent（_defineEventSubclass，空 props）。

### 2. createEvent map 全覆盖 + 未知抛（part06）

map 扩全集 alias（复数 Events/HTMLEvents/SVGEvents→Event、MouseEvents→MouseEvent、UIEvents→UIEvent、custom→CustomEvent + 缺失子类）。未知 type → NotSupportedError（spec，globalThis.DOMException）。返事件原型链 = 构造器 prototype。

### 3. 测试（part07）

更新 test_event_subclasses2_r2812（UnknownEvent 期望抛）。新增 test_create_event_aliases_and_not_supported_r14。

## 基线（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R13 | R14 | Δ |
|------|----|----|---|
| polyfill | 46.60% | 50.33% | +3.73pp |
| native | 46.33% | 50.07% | +3.74pp |

createEvent 用例 96P/183F → 264P/15F（+168）。**dom/nodes 突破 50%**。双路径对等差 0.26pp。

## 验证

engine v8 2085 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 已知 gap

createEvent 返事件 target 默认 undefined（spec 期望 null，Doc-createEvent.html:26）——独立 event 初始化 gap。createEvent 剩 15F（TouchEvent feature-detect 等）。

## 下一步

createDocumentType（81）/ classlist 剩 60F / createElementNS 大小写 / event target null。

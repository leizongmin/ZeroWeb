# R14 — createEvent alias 全覆盖 + 未知 type 抛 NotSupportedError（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**commit**: 见 `git log`（feat(js-dom): createEvent alias coverage + NotSupportedError for unknown）

## 背景

R13 后 Document-createEvent.https.html 第二大失败块（183 失败）。聚类：① `Cannot read properties of undefined (reading 'prototype')`（缺失 event 子类构造器，BeforeUnloadEvent/DeviceMotionEvent 等）② `Should throw NOT_SUPPORTED_ERR for pluralized`（153，createEvent 未知 type 应抛）。

## 改动

### 1. 注册缺失 event 子类构造器（part05）

BeforeUnloadEvent / DeviceMotionEvent / DeviceOrientationEvent / TextEvent / TouchEvent（`_defineEventSubclass`，prototype → Event/UIEvent，无特化字段）。使 `window[iface]` 存在 + createEvent map 能查到。

### 2. createEvent map 全覆盖 + 未知抛 NotSupportedError（part06）

- map 扩全集 alias：Events/HTMLEvents/SVGEvents→Event、MouseEvents→MouseEvent、UIEvents→UIEvent、custom→CustomEvent + CompositionEvent/MessageEvent/BeforeUnloadEvent/DeviceMotionEvent/DeviceOrientationEvent/TextEvent/TouchEvent。
- 未知 type（含复数 CustomEvents/KeyEvents）→ 抛 NotSupportedError（spec `dom-document-createevent`，DOMException code 9，globalThis.DOMException 保 identity）。原 lenient 回落 Event 改 spec 合规抛。
- createEvent 返事件原型链 = 对应构造器 prototype（Object.getPrototypeOf(ev)===window[iface].prototype）。

### 3. 测试更新 + 单测（part07）

- 更新 test_event_subclasses2_r2812：原断言 `createEvent('UnknownEvent') instanceof Event`（旧 lenient 行为）改为 spec 合规 `createEvent(未知) 抛 NotSupportedError`。
- 新增 test_create_event_aliases_and_not_supported_r14：alias 全集 prototype 链（21 组含复数/小写/缺失子类）+ 未知/复数抛 NotSupportedError + 返事件初始化（type 空/eventPhase 0/bubbles false）。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R13 | R14 | Δ |
|------|----|----|---|
| polyfill | 46.60% | **50.33%** | +3.73pp |
| native | 46.33% | **50.07%** | +3.74pp |

双路径对等差 0.26pp。**dom/nodes 突破 50%**。**createEvent 用例**：96P/183F → **264P/15F**（+168 pass）。完整 JSON 快照入 evidence。

## 验证

engine v8 2085 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 已知 gap（非本切片）

- createEvent 返事件 `target` 默认值 polyfill 为 undefined，spec 期望 null（Doc-createEvent.html:26）。独立 event 初始化 gap。
- createEvent 剩 15F（TouchEvent ontouchstart feature-detect + 个别初始化字段）。

## 下一步

- createDocumentType（81）/ classlist 剩 60F / createElementNS 大小写 / event target 默认 null。
- iframe.contentDocument（深结构 html-compat 域，待评估）。

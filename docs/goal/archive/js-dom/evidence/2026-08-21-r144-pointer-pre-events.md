# R144 — focus-event-document-move 1F→0F（指针激活序列 pre_events：mousedown/mouseup 先于 click）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/events/focus-event-document-move.html`（1 subtest，R142 Actions 放行后新可见）

## 根因

真实浏览器指针激活序列是 **mousedown → mouseup → click**（spec UI Events）。用例在
`onmousedown` 内把节点移入新 Document（`d2.appendChild(node)`），断言：
① 不崩 ② mousedown 后节点已从原文档消失（`getElementById('click') === null`）——
即 click 前的 DOM 变更语义可见。shim/R142 的 Generic 激活只派发 click，无
mousedown/mouseup，`onmousedown` handler 从不触发。

## 修复（page-runtime + webview）

- **`HtmlActionPlan` 新增 `pre_events` 字段**：cancelable event **前**派发的不可取消
  事件列表（区别于 click 后的 followup_events——事件序锚点在 cancelable event 两侧）。
  既有 11 个 plan 构造全 `vec![]` 零行为变化。
- **Generic 计划填充 `[mousedown, mouseup]`**（均 target 元素、cancelable——真实
  浏览器两者可取消）。
- **webview `dispatch_user_action_impl`**：cancelable_event 派发前循环派发
  pre_events（html_changed 并入 changed；canceled 分支忽略——mousedown 取消不抑制
  click 为简化语义）。

## click-on-absolute-pseudo 1F 维持（记录归因）

同簇另一件 `click-on-absolute-pseudo` 依赖 Chromium 专有 `Element.pseudo("::after")`
API 与 `event.pseudoTarget` 扩展（Firefox/Safari 均不支持，非标准面）。headless 无
hit-testing 无法给出真实 pseudoTarget——记 **Chromium 专有限制**不追（与真实浏览器
对齐即 Fail）。mousedown/mouseup 序列本身已由本修复就位（该用例的 click 派发
部分已过——剩余断言全在 pseudo API）。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| focus-event-document-move | **1P/0F 双路径 100%** |
| no-focus-events（R142 修复回归） | 2P 不回归 |
| dom/events 全量 | 416P/33F——fail 集 vs R143：focus-event-document-move 消失；+2 个 incumbent-global-subframe Timeout（已知调度 flake，R142 轮同款、隔离复跑 Pass）；fail 集零真新增 |
| Event-dispatch-click 隔离复跑 | 38P（全量跑中的 Timeout 为既知 flake 形态） |
| `make test` | 66 套件全绿（双矩阵） |
| fmt / clippy | 零 diff / 零警告（双矩阵） |

## 单元测试

`generic_activation_dispatches_click_without_default` 扩展：pre_events 恰
`[mousedown, mouseup]` 且先于 cancelable click。

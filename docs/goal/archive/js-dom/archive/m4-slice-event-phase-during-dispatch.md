# M4 R35 — eventPhase 反映 dispatch 阶段（Event-dispatch 系列最小子集）

**日期**: 2026-08-14
**轮次**: R35
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**前置**: R34（AT_TARGET stopPropagation + 双 stop-flag 兼容）
**状态**: ✅ 已 land（双路径对等，零回归）

---

## 背景

R34 后 Event-dispatch 系列是剩余聚类 ①（~30 个 0-pass 主力）。诊断 dom/events Event-dispatch-*.html 失败聚类，分两类：
- **纯 Event API / dispatch 语义**（可切片）：bubbles-false/true、order-at-target、multiple-stopPropagation 等
- **依赖 document/window 入 dispatch chain + cloneNode/new Document**（深结构，本轮不碰）：bubbles-true 的 5 subtest 全需 `[window, document, documentElement, body, ...]` path

诊断 probe（`target.dispatchEvent` 经 `_dispatchWithBubble`）实测 dispatch path：
```
html[]|html[]|body[]|#table|#table-body|#parent|#target|#target|#parent|#table-body|#table|body[]|html[]|html[]
PHASES=0,0,0,0,0,0,0,0,0,0,0,0,0,0  ← 全程 eventPhase=0(NONE)
```
发现两层独立缺口：
1. **eventPhase 全程 0**：`_dispatchToListeners` 收 phase 参数但没翻译成 eventPhase 数字
2. document/window 不在 dispatch chain（深结构，本轮跳过）

`Event-dispatch-order-at-target.html`（detached div，无 document/window 依赖）断言 target 阶段 capture/bubble listener 都 `eventPhase===AT_TARGET(2)`——纯 eventPhase 缺口，可独立 land。

## 实现

### `_dispatchToListeners`（part03.js）—— 设 eventPhase

设 currentTarget 后，按 phase 参数设 eventPhase：
```js
event.eventPhase = phase === 'capture' ? 1 : (phase === 'bubble' ? 3 : 2);
```
- `'capture'`（祖先 capture 阶段）→ CAPTURING_PHASE(1)
- `'all'`（target 阶段）→ AT_TARGET(2)：target 的 capture 与 non-capture listener 都 AT_TARGET（WPT order-at-target）
- `'bubble'`（祖先 bubble 阶段）→ BUBBLING_PHASE(3)

### `_dispatchWithBubble`（part03.js）—— dispatch 后复位

finally 块加 eventPhase + currentTarget 复位（spec `concept-event-dispatch` 末尾）：
```js
event.eventPhase = 0;       // NONE
event.currentTarget = null;
```

## 验证

| 门禁 | 命令 | 结果 |
|------|------|------|
| R35 polyfill 单测 | `cargo test -p zero-engine --features v8 --lib test_event_phase_during_dispatch_r35` | ✅ 1 passed（三阶段 + 复位 + currentTarget null） |
| 既有 event 测试 | `cargo test -p zero-engine --features v8 --lib test_event` | ✅ 15 passed（零回归） |
| engine v8 全量 | `cargo test -p zero-engine --features v8 --lib` | ✅ 2112 passed（R34 基线 2111 +1） |
| engine quickjs 全量 | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1411 passed（零回归） |
| clippy v8 | `cargo clippy -p zero-engine --features v8 --all-targets -- -D warnings` | ✅ 零警告 |
| clippy quickjs | `cargo clippy -p zero-engine -p zero-wpt-runner --no-default-features --features quickjs --all-targets -- -D warnings` | ✅ 零警告 |
| fmt | `cargo fmt --all -- --check` | ✅ 无 diff |
| WPT polyfill order-at-target | `make testharness-dom FILTER=Event-dispatch-order-at-target` | ✅ 0P→**全 Pass** |
| WPT native order-at-target | `make testharness-dom-native FILTER=Event-dispatch-order-at-target` | ✅ 0P→**全 Pass** |
| dom/events polyfill 全量 | `make testharness-dom FILTER=dom/events` | 177P/151F/6timeout（R34 175P → +2，53.52%→54.13%） |
| dom/events native 全量 | `make testharness-dom-native FILTER=dom/events` | 157P/171F/6timeout（R34 155P → +2，47.25%→47.87%） |

## 决策记录

- **为何本轮只做 eventPhase，不做 document/window 入 dispatch chain**：eventPhase 是纯 Event API 语义（`_dispatchToListeners` 已有 phase 参数，只缺翻译），改动面小、根因清楚、driving 用例（order-at-target）明确、零 document/window 依赖。document/window 入 chain 是深结构（dispatch 循环要遍历文档级节点 + window listener 独立存储 + cloneNode/new Document 基础设施），按轻量修复优先跳过，留下轮评估切片化。
- **currentTarget 复位 null 与既有测试**：既有 test_event_bubbling_to_ancestor 等在 listener 内读 currentTarget（dispatch 期间已设），复位在 finally/dispatch 后不影响。15 既有 event 测试零回归验证。

## 净影响

- DC-3（WPT dom 基线）：dom/events polyfill 53.52%→54.13%（+0.61pp）/ native 47.25%→47.87%（+0.62pp），双路径对等差 6.11pp
- DC-4（A/B 对照）：polyfill vs native 双路径行为等价（eventPhase 三阶段 + dispatch 后复位）
- Event-dispatch 系列最小子集入口解锁（order-at-target），为后续 document/window 入 chain 深结构铺路

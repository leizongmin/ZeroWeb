# M4 Slice R26 — Event.cancelBubble（stop propagation flag 公开镜像）

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R26
**前置**: R24（dom/events polyfill 42.58% / native 36.13%，双路径差 6.45pp）

## 切片选择（决策记录）

原计划 R26 = polyfill 三阶段分发（Event-dispatch 系列 ~44 个 0-pass 主力）。诊断后发现 Event-dispatch 系列的核心失败（`expected_targets[0]=window 但 actual="html"`）是**深结构**：polyfill capture/bubble 链不含 document/window（chain 止于 html），且 document/window/html 共享同一 listener key（html key），无法独立派发。正确支持需重构 document/window listener 独立存储 + document.cloneNode/Document/Text 构造器等基础设施，超出轻量切片。

转 **Event.cancelBubble**（独立、轻量、spec 明确）：WPT Event-cancelBubble.html 测 cancelBubble 作为 stop propagation flag 的公开镜像。polyfill event 对象缺 cancelBubble 属性。

## 修复

polyfill（`crates/engine/src/js_dom_shim/`）：

- **part03 `_makeEvent`**：加 `cancelBubble: false`（初始，spec）。`stopPropagation`/`stopImmediatePropagation` 设 `cancelBubble = true`（stop propagation flag 镜像）。与 defaultPrevented/_defaultPrevented 同款「公开镜像 + 私 flag」模式（cancelBubble 公开可读写，_propagationStopped 为 dispatch 内部读）。
- **part05 `initEvent`**：重置 `cancelBubble = false` + stop flags（spec `concept-event-initialize` 重置 dispatch flags；WPT "initEvent must set cancelBubble to false"）。

## 验证

- **单测** `test_event_cancel_bubble_mirror_r26`（part07.rs）：① 初始 false；② initEvent 设 false（重置）；③ stopPropagation 设 true；④ stopImmediatePropagation 设 true。v8 pass。
- **fmt + clippy 双矩阵**：zero-engine v8 + quickjs 零警告。
- **Event-cancelBubble.html 双路径**：0P→**4P/8**（cancelBubble 初始/initEvent/stopPropagation/stopImmediatePropagation 4 test pass；剩 4 是 setter 止上溯 dispatch 语义，需 dispatch 检查 cancelBubble，后续）。
- **dom/events 全量双路径**（完整 JSON 入 evidence）：

  | 路径 | R24 | R26 | Δ |
  |---|---|---|---|
  | Polyfill | 42.58%（132P） | **45.48%（141P）** | +2.90pp / +9P |
  | Native | 36.13%（112P） | **39.03%（121P）** | +2.90pp / +9P |
  | 双路径差 | 6.45pp | 6.45pp | 保持（同步提升） |

  双路径各 +9 pass（cancelBubble + 联动解锁），对等差不变（polyfill shim 双路径共享，同步受益）。

## 决策记录

- **为何 cancelBubble 而非完整三阶段分发**：Event-dispatch 系列是深结构（document/window listener 独立存储 + cloneNode/Document/Text 基础设施），单切片难全解。cancelBubble 是独立、spec 明确、轻量的缺口，net +9 pass。按「轻量修复优先、永不停」选 cancelBubble，三阶段分发深结构记入剩余聚类。
- **cancelBubble 作公开镜像而非 IDL setter**：spec cancelBubble 是 IDL attribute，setter 设 true 等同 stopPropagation。polyfill event 是普通对象，`ev.cancelBubble = true` 直接赋值不触发 setter。本切片用「公开镜像」模式（stopPropagation 设 cancelBubble=true，dispatch 读 _propagationStopped），覆盖 WPT cancelBubble 值断言。cancelBubble setter 副作用（设 true 止上溯）留后续（dispatch 检查 cancelBubble，R27 候选）。

## 残留（转 R27+）

- **Event-dispatch 系列**（深结构）：document/window listener 独立存储 + document.cloneNode / new Document / new Text / createHTMLDocument detached doc 基础设施 + capture/bubble 含 document/window 链。
- **cancelBubble setter 副作用**（dispatch 检查 cancelBubble 止上栖，Event-cancelBubble 剩 4 test）。
- EventListener handleEvent / Event-returnValue。
- 双路径差 6.45pp 收口（WheelEvent 子类链/SubclassedEvent，分散）。
- iframe.contentDocument / querySelector-mixed-case（dom/nodes 域）。

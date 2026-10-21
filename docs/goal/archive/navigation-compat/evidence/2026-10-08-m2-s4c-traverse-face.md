# M2-S4C — traverse 面 navigate 事件 + destination 动态面 + preemption（2026-10-08）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-08-m2-s4c-full-corpus.txt](2026-10-08-m2-s4c-full-corpus.txt)
**前序**: [2026-10-08-m2-s4b-navigate-event.md](2026-10-08-m2-s4b-navigate-event.md)

## 切片内容（master 下一步计划 ①——M2 ④-C 第一刀）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| traverse（back/forward/go）**派发 navigate 'traverse' 事件**（S3 队列任务内）：destination = 目标 record（**真实 key/id/index**）、hashChange 按有效 URL 对比、cancelable；preventDefault → 取消 traversal（不动 cursor/不派 popstate）；intercept → traversal 照常 + 跳默认滚锚 + handler 链后 committed/finished 结算 | `crates/engine/src/js_dom_shim/part02.js`（`_hist_applyTraversal` 重构） | WPT navigate-history-back-after-fragment / intercept-navigation-back |
| **destination 动态面**：`NavigationDestination` 增 bind 单元格（提交后绑 record——index 随 entry list 重算、dispose 后 -1）+ `getState()`（fragment/traverse 承继 entry navState；navigate() push 不承继——WPT navigate-destination-getState-*） | part05 + part02 | WPT navigate-destination-dynamic-index / -getState-* |
| **detach record 复活**：replace 语义让位的 record 入场外登记（≤16 环形）；traverse 回访该 session entry 时按 he 找回并在位顶替（key/id 保真），当位 record 对称入场外 | part02（`_navDetached`/`_navFindRecord`） | WPT navigate-history-back-after-fragment（destination.key = 原条目 key） |
| **preemption**：traverse 事件 dispatch 期间嵌套导航（navigate()/hash/pushState…）→ 本 traversal abort（committed/finished reject AbortError） | part02（`_navTraverseDispatching`/`_navPreempted` 旗标；traverse 自身派发带 `_zwSelf` 标记不自抢占） | WPT navigate-destination-dynamic-index forward 面 |
| `navigation.back()/forward()` 携 committed/finished：越界 reject InvalidStateError（`canGoBack/canGoForward` getter 同切片） | part02 | WPT intercept-navigation-back canGoBack |
| `navigation.transition`（NavigationTransition：finished + navigationType）——intercept 链进行中暴露、settle 即清 | part02 | spec nav-history-apis#navigation-transition |
| `navigation.addEventListener` **once 语义**（此前忽略 options——{once:true} listener 捕获到后续事件致 destination 断言错位） | part02 | WPT navigate-destination-dynamic-index |
| 初始 session entry 的**有效 URL** 口（url='' → 页面 URL fallback）供 traverse destination.url/hashChange 基面 | part02（`_histEntryUrl`） | WPT navigate-history-back-after-fragment `new URL(e.destination.url)` |

## 定位过程记录（两处非显然根因）

1. **once 缺失**：dynamic-index 的 `{once:true}` listener 未被移除 → 捕获后续 replace 事件的
   destination（index=-1）→ 首断言错位报 "expected 0 but got -1"。probe 日志链（t1 出现两次）
   定界。
2. **preemption 自抢占**：`_navPreempted` 局部 var 遮蔽 IIFE 旗标（改赋值后仍错）→ traverse
   自身派发时 `_navTraverseDispatching` 为真、自设旗标 → back() 被自己抢占。加 `_zwSelf`
   标记（traverse 自身派发不自抢占）。

## 数字

| corpus 域 | S4B 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api | 55/225 = 24.4% | 63/227 = **27.8%** | +8 Pass / -5 Timeout |
| history/the-history-interface | 45/49 = 91.8% | 45/49 = 91.8% | 0 |
| history/the-location-interface | 31/36 = 86.1% | 31/36 = 86.1% | 0 |
| traversal/history-traversal | 42/45 = 93.3% | 42/45 = 93.3% | 0 |
| 全量 | 184 P = 43.3% | **192 P = 45.0%** | +8 |

零回归：S4B 全部 Pass 案本轮全保持（逐案对照）。

## 质量门禁

- `make test`：全绿——20,200 P / 0 F。附带修复 R3254-K5 send_keys fixture 的 2s 硬编码
  案超时（连续四轮 workspace 并行负载下间歇超限、solo 恒过 1.05s——提至 10s 负载容限，
  案级超时非语义断言；本轮全绿实证修复生效）。
- `cargo fmt --all -- --check`：零 diff（.js + 单测 .rs 常量变更）

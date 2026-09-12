# M1 切片 2 — 事件循环时序差距清单（对照 HTML spec，2026-09-11）

对照基准：[HTML spec — event loop processing model](https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model)
（step 1–21）+ [microtask checkpoint](https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint)
+ [update the rendering](https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering)。

> 注：goal 契约「基线事实」中的 part01.js 行号为 2026-09-07 快照，本清单按 2026-09-11
> 现状重标（part01.js 现为 3662 行；MO polyfill 现位于 L2113+，IO L2866+，rAF L3388+）。

## 1. checkpoint 调用点精确盘点

### 1.1 唯一 checkpoint 位置

- `crates/script-sandbox/src/v8_runtime.rs:422`（`execute` 末）与 `:486`（`execute_json` 末）
  ——`try_catch.perform_microtask_checkpoint()`。**每 execute 恰一次，批尾排空**。
- V8 语义：drain-to-empty（含回调内新 enqueue 的微任务递归排空）——与 spec「checkpoint
  递归清空 microtask queue」一致。

### 1.2 execute 边界 = 现存的「准 task」边界

| 路径 | execute 粒度 | 与 spec task 边界的吻合度 |
|---|---|---|
| 页面多 `<script>` 执行（`webview.rs` `run_page_scripts_impl` 循环，每脚本一次 `sandbox.execute`） | 每脚本一 execute | ✅ 吻合（spec：每个 script element 是独立 task，尾后 checkpoint） |
| 异步回调 resolve（`v8_runtime.rs:304` `resolve_async_callback` → `self.execute`） | 每回调一 execute | ✅ 吻合（一回调一 task 一 checkpoint） |
| webview 批量排空（`drain_async_callbacks` / `drain_next_async_callback_if_pending`） | 逐个调 `resolve_async_callback` | ✅ 批内仍保每回调独立 execute 边界 |
| **runner timer 泵**（`testharness.rs` 注入 `__zw_fire_due_timers`，`take_probe` 每轮调用） | **全部到期 timer 一个 JS 调用**（单 execute） | ❌ 违反：N 个 task 合一批，checkpoint 只在批尾 |
| **renderer post-render tick**（`page_scripts.rs:353` `tick_observers` → 单 `execute_script_direct` 内连发 `__zw_observers_tick()` + `__zw_raf_tick()`） | **IO/RO/rAF 全部待派发回调一个 execute** | ❌ 违反：spec 每 observer 回调独立 invocation，栈空即 checkpoint |

### 1.3 无显式 task queue

宏任务 = 各 bridge 完成后 `resolve_async_callback` 的隐式 FIFO（`async_callback_rx`
channel 到达序）；无 spec 的多 task queue（timer/network/UI 分队列）与 oldest-first
语义。timer 顺序由 host 子线程 sleep 完成时序决定（runner 侧已用 `__zw_test_setTimeout`
记录式队列修 flake——生产路径无此保证）。

## 2. Spec 算法逐条对照

| Spec 步骤 | 现状实现 | 判定 | 差距 → 去向 |
|---|---|---|---|
| **step 1–2, 5**：从 task queues 取最老 task 执行后移除 | 无显式 task queue；`ResolveAsyncCallback` channel FIFO 隐式承担 | ❌ | 多队列 + oldest-first 缺失 → **M3 task queue** |
| **step 3–4**：运行 task（= 一次脚本/回调 invocation） | 每页面 `<script>`、每 `resolve_async_callback` 独立 execute | ✅ 常规路径吻合 | 批量派发点（§1.2 后两行）破坏边界 → **M3** |
| **step 6**：task 后 perform microtask checkpoint | execute 末 `perform_microtask_checkpoint`（v8_runtime.rs:422/486） | ⚠️ 单回调单 execute 路径 ≈ spec；批量路径违反 | per-task checkpoint（kill-switch）→ **M3** |
| **step 6 细则**：JS execution context stack 非空时**不** checkpoint（外层脚本运行中，内层回调返回不排空） | execute 末单点 checkpoint 天然满足（内层回调都在同一 execute 内） | ✅ | 无 |
| **setTimeout/setInterval task source**（timer 是 task，非 microtask） | host 路径 TimerBridge 真实延迟（js_worker.rs:584 注册）→ 独立 resolve | ⚠️ | 无 host fallback `_defer`（**宏任务压成微任务**——`part01.js:2043/2070`）→ M3 语义收口；生产路径 host 恒在，fallback 仅 reftest/polyfill 面 |
| **queueMicrotask / mutation observer microtask** | `_defer` = `Promise.resolve().then`（part01.js:1242）；MO polyfill 经 `_defer` 派发（L2627 注释） | ✅ 形态吻合 spec（MO 通知就是 microtask） | `_deferBudget = 256`/execute 截断后**静默丢回调**（记录；低优） |
| **update the rendering（step 7 判定 + step 9 rAF）** | rAF 默认同步 stub（预算 64/execute）；`ZW_RAF_FRAME_DRIVEN=1` 帧驱动（part01.js:3388，render 后 `__zw_raf_tick(ts)`） | ⚠️ 已落地 kill-switch 切片 | 不重做（goal 约束 1）；reftest 同步 stub 约束继续有效 |
| **update the rendering — notify intersection observers** | renderer：render 后 `tick_observers`（runtime.rs:708）→ `__zw_observers_tick()` 重算 threshold 越界派发；observe 时 initial notification 排队 | ⚠️ 时点 ≈（post-render vs spec rendering steps 内）；**runner/testharness 环境无此 tick**——IO 基线 65× `entries.length expected N but got 1` 失败簇的直接根因 | runner 侧 observer tick 接线 → **M1 切片 3**；初通知时点（`_defer` vs spec 首次渲染更新）记录待切片 3 评估 |
| **update the rendering — RO broadcast active resize observations** | 同上（size-diff 派发）；RO 基线 6/33 Timeout（notify/eventloop 等 delivery-timing 面） | ❌ runner 环境同缺 | 同上 → **M1 切片 3** |
| **idle（step 21+）requestIdleCallback** | 基础可用实现：无 host → `_defer` 微任务（part01.js:2083） | ❌ 非真实 idle 判定 | **待用户决策**（goal 范围外） |
| **host 侧 mutation → mutation observer microtask**（DOM spec「queue a mutation record」） | dom 层 `pending_mutations` 记录端可用；通知端死路（engine 全仓零调用 `process_mutations`/`take_mutation_records`，2026-09-11 复核仍为零） | ❌ host 驱动 mutation 对页面 MO 不可见 | 方案 C hybrid → **M2** |

## 3. 差距 → 里程碑映射（执行序）

1. **M1 切片 3**（IO/RO 语义）：runner 侧 observer tick 接线（对齐 renderer `tick_observers`
   语义，解锁基线最大失败簇）→ IO 构造器异常校验（threshold 范围 / rootMargin 语法
   throw——`observer-exceptions` 全簇失败，零几何依赖）。
2. **M2**：MO host 触发（方案 C：共享注册表 + host hook + NodeId↔handle 身份桥）——
   G10 死路是本目标三线中唯一「完全不可观测」的缺口。
3. **M3**：显式 task queue + per-task checkpoint（kill-switch 默认 OFF → 全量 A/B 零回归 →
   default-on）。批量派发点（runner timer 泵 / renderer observers tick）在 task queue
   落地后按「一回调一 task」重构。`_defer` fallback 语义收口随 M3 评估。
4. **待用户决策清单**：requestIdleCallback 真实 idle 时序（帧调度深化，goal 明确范围外）。

## 4. 已核对无差距项（避免后续重复排查）

- 微任务递归排空（V8 drain-to-empty）✅
- 多 `<script>` 元素间边界 ✅（每脚本独立 execute）
- 异步回调逐个 execute 边界 ✅（webview `drain_async_callbacks` 批内保边界）
- MO polyfill 微任务派发形态 ✅（符合 mutation observer microtask）
- 外层脚本运行中内层回调返回不 checkpoint ✅（单点 execute 末 checkpoint 天然满足）

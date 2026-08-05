# P1a 事件循环 Slice 1 — 帧驱动 requestAnimationFrame 设计

**日期**：2026-08-05
**关联轮次**：R2712（设计 doc；实施为后续切片）
**父目标**：`docs/goal/zero-web.md` P1 DOM/JS Bridge 原生化 → P1a slice 1（事件循环）
**状态**：设计就绪，待实施（kill-switch 默认 OFF，零默认行为变更）

---

## 0. 执行摘要

- **一句话目标**：把 `requestAnimationFrame`（rAF）从「同步立即执行 stub」升级为「render 后帧驱动」，让 SPA 动画 / 双 rAF 测量 / 帧同步逻辑在真实浏览器路径下行为正确；同时**不破坏 reftest**。
- **本期范围**：仅设计——产出可回退、默认零影响的实施契约（kill-switch `ZW_RAF_FRAME_DRIVEN`，默认 OFF 保留同步 stub）。实施为后续切片。
- **明确排除**：不改 reftest harness 的单渲染流程；不引入 spec 完整 task/microtask 调度框架；不做 IO/RO 持续跟踪（已有 `__zw_observers_tick` post-render tick，另议）。
- **核心约束**：① reftest 路径（`render_to_framebuffer` 单渲染、无 tick）必须保留同步 stub——双 rAF reftest 模式依赖立即执行；② 默认行为零变更（env 默认 OFF）；③ 复用既有 post-render `__zw_observers_tick` tick 入口，不新增第二条帧驱动链。
- **推荐方案**：shim 侧 rAF 队列 + 新增 `__zw_raf_tick(timestamp)` host 回调，env `ZW_RAF_FRAME_DRIVEN` 门控（默认 OFF 同步 stub / ON 帧驱动），renderer `tick_observers` 在 ON 时附带调用 `__zw_raf_tick`。
- **首个落地步骤**：shim 加 `_rafPending` 队列 + `__zw_raf_tick` 回调（default-OFF 时 rAF 仍走旧同步 stub，队列不填充、tick 空 no-op），加 driving test 验证 ON 路径。

---

## 1. 现状（R2712 recon 摘要）

经 Explore-agent 全量侦察（`tab_js_worker.rs` / `js_dom_shim.js` / `timer_bridge.rs` / `page_scripts.rs`）：

| 组件 | 状态 | 机制 | 关键位置 |
|------|------|------|----------|
| setTimeout/setInterval | ✅ EXISTS | TimerBridge host 线程 sleep + AsyncResolver → `ResolveAsyncCallback` 命令回 JS | `timer_bridge.rs:32-49`；`js_dom_shim.js:109-152` |
| requestAnimationFrame | ⚠️ PARTIAL=同步 stub | `fn(0)` 立即执行，无帧 tick；`cancelAnimationFrame` no-op；预算 64/execute | `js_dom_shim.js:559-568` |
| requestIdleCallback | ✅ EXISTS | 镜像 setTimeout（假 IdleDeadline timeRemaining=50） | `js_dom_shim.js:157-174` |
| microtask 队列 | ✅ EXISTS | V8 原生 queueMicrotask / Promise.then，execute 末 checkpoint drain | `js_dom_shim.js:25-35` |
| macro-task 队列 | ✅ EXISTS（隐式） | FIFO 命令通道（`ResolveAsyncCallback`），非显式 queue 结构 | `tab_js_worker.rs:40-71,293` |
| MutationObserver | ✅ EXISTS | JS Proxy trap + microtask flush（无 host 侧事件） | `js_dom_shim.js:176-252` |
| IntersectionObserver/ResizeObserver | ⚠️ PARTIAL | observe 时初始通知；post-render `__zw_observers_tick` 重算（持续跟踪或已部分工作） | `js_dom_shim.js:254-506`；`page_scripts.rs:127-143` |
| worker 命令枚举 | ✅ EXISTS | 6 变体（`Execute`/`ExecuteModule`/`SetDomSnapshot`/`ResolveAsyncCallback`/`SetFetchHandler`/`Shutdown`） | `tab_js_worker.rs:40-71` |
| render-loop 帧通知 | ✅ EXISTS（IO/RO 用） | `publish_webview` 后 `tick_observers` → 执行 `__zw_observers_tick()`；**未**接 rAF | `page_scripts.rs:127-143`；`renderer/main.rs:161-164` |

**结论**：master.md 旧「P1a 4 切片」框架**已部分过时**——setTimeout/ric/microtask/MO/macro-task FIFO 均已存在；**rAF 同步 stub 是真缺口**，且 post-render tick 基建已就位（只需把 rAF 接上）。

### 1.1 reftest 兼容性约束（关键）

reftest harness（`tests/wpt-runner/src/reftest.rs`）经 `apply_scripted_dom_mutations` 执行页面 JS 后**单次渲染**（`render_to_framebuffer`），**不**走 renderer 的 `tick_observers`，**不**泵帧。当前 rAF 同步 stub 在 execute() 内立即 `fn(0)`，使双 rAF reftest 模式（`requestAnimationFrame(() => requestAnimationFrame(() => { setup; }))`）在一次 execute 内完成、DOM 变更进入最终 HTML 被单渲染捕获。**若 rAF 改为帧驱动（延后到 `__zw_raf_tick`），reftest 路径永不触发 tick → rAF 回调不 fire → reftest 断**。故 reftest 路径必须保留同步 stub。

---

## 2. 目标状态

`requestAnimationFrame(cb)` 在**浏览器/renderer 路径**下：注册 `cb` 到 `_rafPending` 队列、返 id；render 后 `tick_observers` 调 `__zw_raf_tick(ts)` → 按队列序 `cb(ts)`、清空。`cancelAnimationFrame(id)` 移除。reftest 路径（env OFF）保留同步 stub。

---

## 3. 设计

### 3.1 kill-switch（核心安全机制）

env `ZW_RAF_FRAME_DRIVEN`：`0`/unset = 同步 stub（reftest + 现状）；`1` = 帧驱动。**默认 OFF = 零默认行为变更**，reftest / product-smoke / 全量 `make test` 不受影响（同 font-metric `ZW_PERFONT_LINEHEIGHT` dormant 模式）。

### 3.2 env→shim plumbing

现有 kill-switch（`ZW_CONTENT_VISIBILITY` / `ZW_PERFONT_LINEHEIGHT` / `ZW_REAL_RECT`）均在 **Rust 侧** `std::env::var` 读取、门控 Rust 行为。rAF 逻辑在 **JS shim**，需把 env 注入 shim。两种方案：

- **方案 A（推荐）**：worker 初始化（Rust）读 `std::env::var("ZW_RAF_FRAME_DRIVEN")`，execute shim 前 inject `globalThis.__ZW_RAF_FRAME_DRIVEN = true/false`。shim 据 `globalThis.__ZW_RAF_FRAME_DRIVEN` 分支。改动：`tab_js_worker.rs` 初始化 + shim 3 处（rAF/cancelAnimationFrame/`__zw_raf_tick`）。
- 方案 B：注册 `__zw_env_flag(name)` host 回调，shim 调用查询。多一次 host 往返、且 reftest harness 未必注册该回调。拒绝。

**选定方案 A**：注入一次全局布尔，shim 分支零额外往返；reftest harness 用同一 shim 但 env unset → `false` → 同步 stub。

### 3.3 shim 侧改动（伪代码）

```js
// requestAnimationFrame
var _rafPending = {};            // id -> cb
var _rafSeq = 0;
var __ZW_RAF_FRAME_DRIVEN = false; // host init 覆盖
globalThis.requestAnimationFrame = function(fn) {
  var id = ++_rafSeq;
  if (globalThis.__ZW_RAF_FRAME_DRIVEN) {
    _rafPending[id] = fn;        // 延后到 __zw_raf_tick
  } else {
    // 旧同步 stub（reftest 兼容）：预算内立即 fn(0)
    if (_rafBudget-- > 0) { try { fn(0); } catch(e){} }
  }
  return id;
};
globalThis.cancelAnimationFrame = function(id) {
  if (globalThis.__ZW_RAF_FRAME_DRIVEN) delete _rafPending[id];
  // OFF 路径 no-op（旧行为）
};
// host 在 render 后调用（renderer tick_observers）
globalThis.__zw_raf_tick = function(ts) {
  if (!globalThis.__ZW_RAF_FRAME_DRIVEN) return;
  var cbs = _rafPending; _rafPending = {}; // 本帧快照、清空（rAF 内重注册入下一帧）
  for (var id in cbs) { try { cbs[id](ts); } catch(e){} }
};
```

**timestamp**：`__zw_raf_tick(ts)` 的 `ts` 由 host 传（`performance.now()`-类单调时钟 ms；复用既有 timestamp 源，若无可传 0 并记 follow-up）。

### 3.4 renderer 侧改动

`apps/renderer/src/page_scripts.rs:tick_observers` 在执行 `__zw_observers_tick()` 后，附带执行 `if(globalThis.__zw_raf_tick)globalThis.__zw_raf_tick(<ts>);`。shim 在 OFF 时 `__zw_raf_tick` 早返 → 零开销。

**注**：reftest harness 用 `render_to_framebuffer`（不经 `tick_observers`），故 reftest 永不调 `__zw_raf_tick` → OFF 路径下 rAF 仍走同步 stub，兼容。

### 3.5 browser（in-process tab_worker）路径

in-process 回退路径（`ZERO_BROWSER_MULTIPROCESS=0`）若也走类似 post-render 通知，同样附带 `__zw_raf_tick`（实施时核查 `tab_worker.rs` / browser backend 的 render 后回调点）。多进程 renderer 路径已在 §3.4 覆盖。

---

## 4. 验收场景（实施后）

```
场景: 帧驱动 rAF ON 路径 fire on tick
  假设 env ZW_RAF_FRAME_DRIVEN=1，shim 已注入 globalThis.__ZW_RAF_FRAME_DRIVEN=true
  当 requestAnimationFrame(cb); __zw_raf_tick(16.7)
  那么 cb 被 16.7 调用一次；_rafPending 清空
  验证: test_raf_frame_driven_fires_on_tick（shim 单测，直接调 globalThis.__zw_raf_tick）

场景: cancelAnimationFrame ON 路径移除
  假设 ON 路径
  当 var id=requestAnimationFrame(cb); cancelAnimationFrame(id); __zw_raf_tick(0)
  那么 cb 不被调用
  验证: test_raf_frame_driven_cancel

场景: OFF 路径保留同步 stub（reftest 兼容）
  假设 env unset / __ZW_RAF_FRAME_DRIVEN=false
  当 requestAnimationFrame(cb)
  那么 cb 立即被 0 调用（预算内）；__zw_raf_tick no-op
  验证: test_raf_sync_stub_preserved_off（回归守护）

场景: 默认零行为变更
  假设 env unset
  当 make test + make product-smoke（welcome）
  那么 13334 全绿、welcome 17.03% 持平（无回归）
  验证: make test / make product-smoke
```

---

## 5. 约束与假设

### 必须约束
- env `ZW_RAF_FRAME_DRIVEN` 默认 unset/OFF = 同步 stub，零默认行为变更。
- reftest harness（`render_to_framebuffer` 单渲染）不经 `tick_observers`，OFF 路径下 rAF 行为与现状逐字节一致。

### 禁止约束
- 不改 reftest harness 的单渲染流程（不泵帧）。
- 不删除同步 stub 分支（reftest 依赖）。

### 假设
- renderer `tick_observers` 在每次 publish_webview（render）后被调用一次——已 recon 确认（`page_scripts.rs:127-143`）。状态：已验证。
- in-process browser 路径有等价 post-render 回调点——待实施时核查 `tab_worker.rs`。状态：待验证。

---

## 6. 实施交接

### 文件/模块清单

| 路径 | 动作 | 目的 |
|------|------|------|
| `crates/engine/src/js_dom_shim.js` | 修改 | rAF/cancelAnimationFrame 按 `__ZW_RAF_FRAME_DRIVEN` 分支；加 `_rafPending` 队列 + `__zw_raf_tick` |
| `apps/browser/src/tab_js_worker.rs` | 修改 | worker init 读 `ZW_RAF_FRAME_DRIVEN` env、execute shim 前 inject `globalThis.__ZW_RAF_FRAME_DRIVEN` |
| `apps/renderer/src/page_scripts.rs` | 修改 | `tick_observers` 附带执行 `__zw_raf_tick(ts)` |
| `apps/browser/src/tab_worker.rs`（in-process） | 核查±改 | post-render 回调点附带 `__zw_raf_tick`（若存在） |
| `crates/engine/src/js_dom_bridge_tests.rs`（或 shim 单测） | 新增 | 3 验收场景测试 |

### 推荐修改顺序
1. shim 加 `_rafPending` + `__zw_raf_tick` + `__ZW_RAF_FRAME_DRIVEN` 分支（OFF 时行为不变）+ shim 单测（ON/OFF/cancel）。
2. renderer `tick_observers` 附带 `__zw_raf_tick`（OFF 时 shim 早返，零影响）。
3. worker init inject `globalThis.__ZW_RAF_FRAME_DRIVEN`（读 env）。
4. 核查 in-process browser 路径。
5. `make test`（默认 OFF 零回归）+ `make product-smoke`（welcome 持平）+ ON 路径 driving test。

### 首批提交建议
| 提交 | 范围 | 预期结果 | 验证 |
|------|------|----------|------|
| R2713a | shim rAF 分支 + `__zw_raf_tick` + 单测（不接 renderer） | OFF 零变更；ON 由单测验证 | shim 单测 + make test |
| R2713b | renderer tick + worker env inject | ON 路径端到端 | ON driving test + product-smoke |

---

## 7. 回滚

`ZW_RAF_FRAME_DRIVEN` unset/OFF → 同步 stub（逐字节现状）。任意切片净负即可关 env 回退，无代码 revert 风险。

---

## 8. 后续（不在本期）

- ON 路径 A/B 量化后决定是否 default-on（需评估 reftest 改造代价：reftest harness 泵帧 vs 保留 OFF）。
- ~~IO/RO 持续跟踪核查（`__zw_observers_tick` 是否已覆盖，或仍需 Slice 2b）~~。**R2714 已 land**：post-render `__zw_observers_tick` → IO `_schedule` → `_crossed` threshold 越界再派发（renderer worker 测试 `renderer_js_worker_intersection_observer_refires_on_threshold_cross` 覆盖）。
- ~~`performance.now()` 真实单调时钟接入（rAF ts 精度）~~。**R2768 已 land** performance.now()（`__zw_performance_now` host 回调 + shim `globalThis.performance.now`），**R2769 接入 rAF ts**：`tick_observers`（`page_scripts.rs:145`）改 `__zw_raf_tick(performance.now())`（旧传 0），ON 路径 rAF 回调收真 DOMHighResTimeStamp；OFF 路径 shim 早返零影响。

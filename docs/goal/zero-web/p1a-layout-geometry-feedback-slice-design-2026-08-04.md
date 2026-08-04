# P1a 切片设计：布局几何反馈（getBoundingClientRect 真实化 → 解锁 IntersectionObserver/ResizeObserver）

> **状态**：实施前设计（implementation-ready），2026-08-04。承接 P1a 架构侦察（`p1a-architecture-recon-2026-08-04.md`）。目标：把生产 worker 路径的 `getBoundingClientRect` 从零 DOMRect 桩升级为真实布局 rect，并建立「host 布局结果 → JS」反馈 plumbing，作为 IO/RO 的共同基建。

## 问题

`js_dom_shim.js:776-786` 的 `getBoundingClientRect` 返回零 DOMRect（注释：动态 reftest 多用作「强制 reflow」触发器，返回值多不使用；零值不抛、对纯 reflow 触发语义正确）。**真缺口** = 实际使用 rect 值的 JS（布局测量、动画判定、视口检测）被阻断。`IntersectionObserver`/`ResizeObserver` 亦无真实实现——三者共同依赖「host 把布局 rect 喂回 JS」的反馈通路，当前不存在。

## 既有可复用基建（侦察确认）

1. **`HitTestCache`**（`crates/engine/src/hit_test.rs`）：`from_document + layout_root` 构造，存布局快照（`node_id → 位置/尺寸`），`collect_hit_test_nodes` 遍历。**host 已能 node_id → rect**。
2. **`FetchBridge` handler-cell 模式**（`crates/engine/src/fetch_bridge.rs`）：`handler_cell: Arc<Mutex<Option<Handler>>>` + `register(sandbox)` 注 `__zw_*` 回调 + `set_handler` spawn 后注入生产 handler（chicken-and-egg 解）。`TimerBridge` 同模式。
3. **`register_callback`**（`Sandbox` trait）：闭包 `&[String] -> String` 同步返回——**rect 查询是同步的**（无网络/wait），可 inline 返回，不需 `AsyncResolver`/子线程（比 fetch 更简单）。
4. **shim `__zwResolveCallback` / `__zw_fetch` 模式**：JS 侧调 host 回调拿值已建立。

## 设计：RectBridge（镜像 FetchBridge，同步版）

### Host 侧（`crates/engine/src/rect_bridge.rs`，新文件 ~50 行）

```text
RectBridge {
    rect_cell: Arc<Mutex<Option<RectLookupHandler>>>,   // RectLookupHandler = Arc<dyn Fn(u64) -> Option<Rect4> + Send + Sync>
}
// Rect4 = (x:f32, y:f32, w:f32, h:f32) → 序列化 "x,y,w,h" 字符串
register(sandbox): sandbox.register_callback("__zw_getBoundingClientRect", |args| {
    let id = args[0].parse::<u64>();
    let cell = rect_cell.lock();
    match (cell.handler)(id) {
        Some((x,y,w,h)) => format!("{x},{y},{w},{h}"),
        None => ""                                    // 空 → shim 回落零 rect（零回归）
    }
})                                                    // 同步，无 spawn
set_handler(handler): 注入生产 rect 查询闭包（spawn 后，layout 就绪后）
```

### Layout-rect snapshot 通路（浏览器/渲染进程侧）

- 浏览器/渲染进程在**每次 render 后**已有 layout 结果（`HitTestCache` / `layout_root`，见 `apps/browser/src/paint_ipc.rs:291 ipc_layout_to_snapshot(&cache.layout_root)`）。
- 增一个**共享 `Arc<Mutex<HashMap<u64, Rect4>>>` layout-rect snapshot**：render 后写入（遍历 layout tree 按 node_id 填 rect），`RectBridge` 的 handler 闭包读它。
- `RectBridge::set_handler` 在 worker spawn 后注入「读 snapshot 按 node_id 查 rect」的闭包。
- **实施起始须核验**：浏览器 render 后写 snapshot 的精确 hook（`tab_scripts.rs apply_recorded_mutations → re-render → ?`；renderer 侧 `js_worker` 对应点）。候选：render 完成回调 / `HitTestCache` 更新点。

### JS shim 侧（`js_dom_shim.js`）

```js
if (prop === 'getBoundingClientRect') {
  return function() {
    var id = this.__nodeId;                            // 元素 nodeId
    if (id != null && typeof __zw_getBoundingClientRect === 'function') {
      try {
        var s = __zw_getBoundingClientRect(String(id));
        if (s && s.indexOf(',') >= 0) {
          var p = s.split(',');                        // x,y,w,h
          var x=+p[0], y=+p[1], w=+p[2], h=+p[3];
          return { x:x, y:y, top:y, left:x, right:x+w, bottom:y+h, width:w, height:h, toJSON:function(){return this;} };
        }
      } catch(_e) {}
    }
    return { x:0, y:0, top:0, left:0, right:0, bottom:0, width:0, height:0, toJSON:function(){return this;} };  // 回落（零回归）
  };
}
```

## 切片分解

- **Slice 1（本设计核心）**：`RectBridge` + `__zw_getBoundingClientRect` + layout-rect snapshot 通路（浏览器+渲染进程）+ shim 接线 + kill-switch + driving test。解锁 `getBoundingClientRect` 真实值。
- **Slice 2（续）**：`IntersectionObserver`——host 按 snapshot 计算 target 与 root viewport 的 intersection ratio，越阈值时经 `AsyncResolver`/`__zwResolveCallback` 触发 `obs._callback(entries, observer)`（复用 S1 异步通路）。
- **Slice 3（续）**：`ResizeObserver`——host 跟踪 node 尺寸变化（snapshot diff），变化时触发回调。

## Kill-switch / 可回退

- env `ZW_REAL_RECT`：默认 on；`=0` 时 `RectBridge::register` 不注册回调（或 handler 永远返 None）→ shim 回落零 rect（=当前行为，零回归）。
- 三态 A/B：self-source reftest + product-smoke + 相关 dir oracle，net≥0 才 land（参考 rendering-compat A/B 门禁；本切片属 zero-web 但同样适用防回归）。

## 测试计划

1. **单元**：`RectBridge` handler cell（set_handler 前后行为；未注入→空串；注入→"x,y,w,h"）。
2. **集成**：shim `getBoundingClientRect` 经 `__zw_getBoundingClientRect` 返回真实 DOMRect（无 handler → 零回落）。
3. **Driving**：动态 reftest——JS 设 `el.style.width='100px'`，`getBoundingClientRect().width` 断言≈100（当前因零 rect 不可实现，本切片使其通过）。

## 实施起始 checklist（下轮）

1. 核验浏览器/渲染进程 render 后 layout-rect 可写 snapshot 的精确 hook（`tab_scripts.rs` render 流 + `paint_ipc.rs` layout_root）。
2. 定位 `js_dom_shim.js` 元素 proxy 的 `__nodeId` 访问路径（getBoundingClientRect 闭包内取 id）。
3. 起 `rect_bridge.rs`（镜像 `timer_bridge.rs` ~50 行结构）+ lib.rs 导出。
4. 接 worker（`tab_js_worker.rs` + renderer `js_worker`）set_handler + snapshot 写入。
5. shim 接线 + kill-switch + 三态 A/B + driving test。

## R2644 plumbing 核验细化（实施前验证结果）

本节补全 checklist 项 1/2/4 的具体机制（核验后确认 renderer 路径可先行）：

1. **rect 数据源 = `WebView::build_hit_test_cache()`**（`apps/renderer/src/main.rs:283`，render 后调）。`HitTestCache::snapshot()` → `HitTestLayoutSnapshot`（每节点 `node_id, x, y, width, height`，见 `paint_export.rs:395 IpcHitTestLayoutNode` / `hit_test_layout_to_ipc`）。**NodeId → rect 数据已存在**，仅需 retain 一份查询用 snapshot。
2. **共享 snapshot 机制**：`Arc<Mutex<HashMap<u64 /*NodeId*/, Rect4>>>`——renderer 主循环 render 后从 `build_hit_test_cache().snapshot().layout_root` 遍历填入；js_worker 的 RectBridge handler 读它。renderer **同进程**（main loop 与 js_worker 异线程，经 Arc<Mutex> 共享）；browser **跨进程**（须 IPC rect 查询，本切片先 defer，renderer 先行）。
3. **identity → NodeId 解析在 js_worker 线程可做**：`register_dom_callbacks`（`js_dom_bridge.rs:573`）已在 js_worker 线程跑 `__zw_query_match`/`__zw_get_attr` 等（持 Document 快照解析 selector）。RectBridge handler 同线程，可复用同一解析（identity=handle `__n{n}`/selector → NodeId）→ 查 snapshot rect。
4. **元素 identity 取法**：shim 元素 proxy 的 compound key（`js_dom_shim.js:137` `_mo_id(handle, sel)`）——getBoundingClientRect 闭包内取该 key 传 `__zw_getBoundingClientRect`（非简单 `__nodeId`，设计原文 `u64` 须改为 `&str` identity；RectBridge 已按 `&str` 实装 @ R2643）。

### 首切片已知限制（接受，follow-up）

- **rect 反映「上次 render」**（stale-but-non-zero）：JS 改样式后同脚本内立即读 `getBoundingClientRect` 见 pre-change rect（因 reflow 未触发）。**force-reflow-on-demand**（gBCR 触发/等待同步 reflow）为 follow-up 深改。首切片对「读已渲染元素 rect」的 JS 净正向，对「改后即读」场景仍 stale——driving test 须选「读已渲染元素」类（非改后即读）。
- browser 跨进程路径 defer（需 IPC rect query RPC）。

### 修正后实施步骤（renderer 先行）

1. `crates/engine`：加共享 snapshot 类型（`LayoutRectSnapshot = Arc<Mutex<HashMap<u64, Rect4>>>`）+ 从 `HitTestCache::snapshot().layout_root` 填充的 helper。
2. `apps/renderer/src/main.rs`：render 后（`build_hit_test_cache` @ :283 附近）填 snapshot；把 snapshot clone 传 js_worker。
3. `apps/renderer/src/js_worker.rs`：构造 `RectBridge` + `register`（镜像 `TimerBridge` @ :214）+ `set_handler`（闭包：identity→NodeId〔复用 register_dom_callbacks 的 Document 解析〕→ snapshot rect）。
4. `js_dom_shim.js`：`getBoundingClientRect` 取元素 compound key 调 `__zw_getBoundingClientRect`，解析 `x,y,w,h`→DOMRect，无 handler/空→零回落。
5. kill-switch `ZW_REAL_RECT` + driving reftest（读已渲染元素 rect）+ 三态 A/B。

## R2646 scope 深化：identity→NodeId 是真架构缺口（非「低风险快速见效」）

实施 step 2 前核验发现 **R2644 假设「identity→NodeId 解析在 js_worker 可复用」不成立**，slice 实际比 P1a「低风险快速见效」框定大得多：

1. **`apply_dom_mutations`（`js_dom_bridge.rs:177`）的 `handles: HashMap<String, NodeId>` 是 ephemeral**——每次 apply 调用重建、用后即弃，**不持久化**。RectBridge handler 无法查历史 handle→NodeId。
2. **`find_by_selector` 需 `Document`（已解析）**，而 js_worker 查询回调（`__zw_query_match` 等）走 HTML 字符串（`query_match_selector(&html, &sel)` 返字符串值），**不暴露 NodeId**。
3. **selector-keyed snapshot 亦不成立**：shim 元素身份 = 任意 selector 或 handle `__n{n}`，与 `stable_selector_for_node` 生成的规范选择器**不保证一致**；handle 更非 selector。

→ **不存在现成 persistent identity→NodeId（或 identity→rect）映射**供 handler 用。须**新建**该映射基建。

### 可行重架构路径（多 session，须选一）

- **(A) 持久化 handles map**：`apply_dom_mutations` 的 `handles` 改为 `Arc<Mutex<HashMap<String,NodeId>>>` 持久跨调用、共享给 RectBridge handler；selector 身份经 `Document`（parse dom_html 或共享 Document）`find_by_selector`。改 `apply_dom_mutations`（browser/renderer/reftest 三处共用，须 A/B）。
- **(B) identity→rect 直映**：renderer render 后遍历 layout + 为每元素算 stable_selector→rect 入 snapshot；shim 侧 `getBoundingClientRect` 传 stable selector（须 shim 能产出与 host 一致的 stable selector——双向对齐复杂）。
- **(C) parse-on-query**：handler 每 query 解析 dom_html→Document→find_by_selector→NodeId→rect。自包含但每 query 一次 HTML 解析（贵），且 handle 身份仍须持久 map。

### 结论与建议

- gBCR 真实化是**多 session 架构 slice**（建 persistent identity→rect 映射基建），**非** P1a「低风险快速见效」原框定。RectBridge（R2643）+ layout-rect snapshot（R2645）building blocks 已 land 且无论哪条路径都复用。
- **建议**：先做 **(A) 持久化 handles map** 作为 identity 基建首个 sub-slice（最小、对 handle 身份直接生效，selector 经 Document 补），kill-switch 守护、A/B 防回归；稳定后再接 renderer 接线 + shim + driving test。或**pivot** 到更高 ROI 的 P1a 项（见 recon），gBCR 作为长期项排期。
- 此 finding 不阻塞——按 rally 规则记入控制面，CONTINUE 推进（先 (A) sub-slice 或 pivot 由下轮定）。

## R2647：path (C) 对 selector-identity 可行——renderer gBCR 已 land 并验证

**R2646 结论的纠偏**：R2646 把 identity→NodeId 当作「真架构缺口」，隐含假设是「worker 的 `parse_html(dom_html)` 与渲染管线的 Document 的 NodeId 不一致」。**核验 `pipeline_budget.rs:106/197` 后该假设不成立**：渲染管线每次 render 都 **fresh `parse_html(session.html)`**（不持有持久增量 Document），而 js_worker 持的 `dom_html` 是同一字符串。

→ **slotmap fresh-map + 相同插入顺序 → 确定性 NodeId**（守护测试 `rect_bridge::tests::test_node_id_determinism_across_fresh_parses` 验证同一 HTML 两次 fresh-parse 对 `#t`/`span.c`/`span`/`div` 返回相同 `node_id_to_u64`）。故 **path (C) parse-on-query 对 selector-identity 直接成立**，无需 path (A) 的持久化 handles map。

**已 land（renderer 路径，commit 本轮）**：

| 模块 | 变更 |
|------|------|
| `crates/engine/src/rect_bridge.rs` | 新增 `make_dom_html_rect_handler(dom_html, snapshot)`：handler 每次 query fresh-parse `dom_html`→`find_by_selector(identity)`→`NodeId`→查 snapshot。注：`Document` 非 `Send` → 不能跨调用缓存于 `Send+Sync` handler，**每 query 一次 parse**（path C 已接受；perf follow-up 可在 js_worker 线程做 thread-local 缓存）。+ 确定性守护测试 + handler 单测。 |
| `crates/engine/src/hit_test.rs` | `HitTestCache::fill_layout_rect_snapshot(&snapshot)`：直接遍历内部 `layout_root`（`LayoutBox`）填 NodeId→rect，**避免 `snapshot()` 整树 clone**。 |
| `crates/engine/src/js_dom_shim.js` | `getBoundingClientRect`：selector-identity 元素（`querySelector`/`getElementById`，`sel`=stable_selector）→ `__zw_getBoundingClientRect(sel)` 解析 `"x,y,w,h"`→真实 DOMRect。未注册/未命中/handle-identity → 零 rect（= 旧行为零回归）。 |
| `apps/renderer/src/js_worker.rs` | `RendererJsWorker` 加 `rect_snapshot: LayoutRectSnapshot` 字段 + `rect_snapshot()` accessor；`js_worker_main` 构造 `RectBridge` + register + `set_handler(make_dom_html_rect_handler(...))`，kill-switch `ZW_REAL_RECT=0` 不注册。+ 2 driving test（real rect / 空 snapshot 零回落）。 |
| `apps/renderer/src/main.rs` | `publish_webview`：render 后 `hit_test.fill_layout_rect_snapshot(&self.js_worker.rect_snapshot())`。 |

**验证**：`make test` 全绿（renderer 19 worker 测试含 2 新 gBCR；engine rect_bridge 8 测试）；workspace clippy `-D warnings` 零警告；`cargo fmt` clean。

**已知限制（接受，follow-up）**：

1. **handle-identity 不支持**：`createElement` 节点（`sel` 为空）`find_by_selector("__n{n}")` 不匹配 → 零 rect。需 path (A) 持久 handle→selector/NodeId 或 shim 在 append 后更新身份。**本切片覆盖 selector-identity（querySelector/getElementById——测量既有渲染元素的常见情形）**。
2. **stale-but-non-zero**：rect 反映「上次 render」；JS 改样式后同脚本内即读见 pre-change rect。force-reflow-on-demand 为深改 follow-up。
3. **每 query 一次 HTML parse**：`Document` 非 `Send` 不能跨调用缓存。perf follow-up：thread-local 缓存或改 selector-keyed snapshot 免 parse。
4. **browser 路径未接**：browser 有 in-process headless（`headless.rs` WebView）+ cross-process（`process_backend.rs`）两后端；shim 共享但 browser `tab_js_worker` 未注册 RectBridge → browser/reftest/WebView 路径 gBCR 仍零 rect（= 旧行为，零回归）。browser 接线为下一切片。

**为何 selector-identity 净正向且零回归**：renderer worker 路径新增真实 gBCR；browser/reftest/WebView 路径 `__zw_getBoundingClientRect` 未注册 → shim 回落零 rect（= 旧行为）；空 snapshot / 未命中同样回落。三项 driving + 全量 `make test` 守护。

## R2648：browser in-process 路径 gBCR 接线——覆盖剩余 browser 后端

R2647 限制 4「browser 路径未接」的收尾。核验 browser 后端：**cross-process `process_backend.rs` 不在 browser 进程跑 JS**（grep 无 js_worker/JsExecutor/run_page_scripts）——它委托 renderer 进程执行 JS，而 renderer js_worker 已在 R2647 接 RectBridge → **cross-process browser gBCR 随 R2647 已工作**。剩余缺口仅 **in-process `tab_worker` 回退路径**（`ZERO_BROWSER_MULTIPROCESS=0` 或 renderer binary 不可用时）。

**已 land（browser in-process，commit 本轮）**——镜像 R2647 renderer 接线：

| 模块 | 变更 |
|------|------|
| `apps/browser/src/tab_js_worker.rs` | `TabJsWorkerHandle` 加 `rect_snapshot` 字段 + `rect_snapshot()` accessor + `real_rect_enabled()` kill-switch；`js_worker_main` 构造 RectBridge + register + `set_handler(make_dom_html_rect_handler(...))`。+ 2 driving test（real rect / 空 snapshot 零回落），镜像 renderer。 |
| `apps/browser/src/tab_worker.rs` | `push_snapshot` 闭包加 `js_worker: Option<&TabJsWorkerHandle>` 参数，render 后从 `snapshot.hit_test`（`from_webview` 已建，**复用避免二次 `build_hit_test_cache`**）填 `fill_layout_rect_snapshot`；9 个 call site 传 `_js_worker.as_ref()`。 |
| `apps/browser/Cargo.toml` | `[dev-dependencies]` 加 `zero-dom`（driving test 解析 html 取 NodeId 填 snapshot）。 |

**验证**：`make test` 全绿（browser tab_js_worker 2 新 gBCR 测试）；workspace clippy `-D warnings` 零警告；fmt clean。

**覆盖现状**：renderer 进程（R2647）+ browser in-process tab_worker（R2648）gBCR 均真实化；cross-process browser 经 renderer 进程已覆盖；reftest/WebView 嵌入路径仍零 rect（未注册回调，= 旧行为，零回归——这些路径无 layout-rect 反馈需求，reftest 用 gBCR 仅作 reflow 触发器，零值正确）。

**剩余 follow-up（未变）**：① handle-identity（createElement）需 path A 持久身份映射；② stale-but-non-zero；③ 每 query 一次 parse（perf）；④ IntersectionObserver/ResizeObserver（Slice 2/3，复用 gBCR 基建）。

## R2649：gBCR perf 硬化——thread-local Document 缓存（消除每 query HTML parse）

收尾 R2647 限制 3「每 query 一次 HTML parse」。`make_dom_html_rect_handler` 原每次 gBCR 调用都 `parse_html(dom_html)`（`Document` 非 `Send` 不能跨 `Send+Sync` 闭包缓存）——循环调用 gBCR（如测量 N 元素）= N 次全 HTML parse，生产陡坡。

**已 land**：改用 `thread_local! { RECT_DOC_CACHE: RefCell<Option<(String, Document)>> }`——per-worker-thread 独立槽（无 `Send` 约束），键 = html 字符串；html 变化（render 后 dom_html 更新）才重 parse，同 render 帧多次 gBCR 复用同一 `Document`。`const { RefCell::new(None) }` 初始化（clippy `missing_const_for_thread_local`）。+ 失效正确性测试（html1→html2 切换后旧 selector 不存在、新 selector 命中）。

**验证**：`make test` 全绿（rect_bridge 9 测试含新缓存失效测试）；workspace clippy `-D warnings` 零警告；fmt clean。行为零变化（缓存透明，html 变化即重 parse）。

**为何 thread_local 安全**：每个 js_worker 是独立 OS 线程 → 独立 thread_local 槽，无跨 worker 串扰；html 字符串作键保证 render 后 dom_html 更新触发失效；线程退出随 thread_local 释放（无泄漏，缓存 = 单页 DOM 大小有界）。

## R2650：Slice 2a — IntersectionObserver 真实化（JS 侧，复用 gBCR）

承接 gBCR 基建（Slice 1，R2645-R2649）落地后的 follow-up ④「IntersectionObserver/ResizeObserver（Slice 2/3，复用 gBCR 基建）」。核验 shim：生产 worker 路径（`js_dom_shim.js`）**完全无 IntersectionObserver**——`new IntersectionObserver(...)` 抛 ReferenceError 中断整个脚本（区别于 MutationObserver/fetch/setTimeout 已真实化）。旧 polyfill（`dom_bridge.rs:1159`，仅 WebView 路径）有 observe/unobserve/disconnect/takeRecords + Entry 桩但**永不触发回调**。

**关键决策**：IO **纯 JS 侧实现**（镜像 MutationObserver 的 JS 侧拦截 + microtask 派发模式），**复用已落地的 `__zw_getBoundingClientRect` host 回调**（gBCR path C）+ `innerWidth/innerHeight` 算 target 与 root 的 intersection——**无需新 host Rust 基建**（设计原文 Slice 2 的 host-side tick / AsyncResolver 方案 deferred，JS 侧更简且零 host 风险）。

**已 land（纯 shim `js_dom_shim.js`，commit 本轮）**——MutationObserver 之后插入 IO block：

| 组件 | 行为 |
|------|------|
| `IntersectionObserver(callback, options)` | 存 callback；解析 options.threshold（number\|number[] → 升序去重 clamp [0,1]，空→[0]）+ options.root（null=viewport，元素取 `__zwSelector` rect）。`_thresholds`/`_rootSel`/`_targets`/`_lastRatio`/`_scheduled` 状态。 |
| `_compute(id)` | `__zw_getBoundingClientRect(sel)` 取 target rect（复用 gBCR；未注册/sel 空/未命中→零 rect），与 root rect（viewport `innerWidth×innerHeight` 或 root 元素 rect）算 `_io_intersect` 重叠；ratio = inter.area / target.area；isIntersecting = inter 非零面积。 |
| `_crossed(id, ratio)` | 越阈值检测：`_lastRatio[id]==null`（首次=initial notification）→ 派发；否则 ratio 与上次跨过任一 threshold 边界才派发。 |
| `_schedule()` | `_defer`（microtask）派发：遍历 `_targets`，对越阈值者构造 IntersectionObserverEntry（time/boundingClientRect/intersectionRect/intersectionRatio/isIntersecting/rootBounds/target），`obs._callback(entries, obs)`。 |
| `observe/unobserve/disconnect/takeRecords` | 镜像 MutationObserver；observe 排队 initial notification，disconnect 清 `_targets` 使排队中的派发成空。 |
| `IntersectionObserverEntry` | 兼容构造（部分脚本 `new IntersectionObserverEntry(init)`）。 |

**spec 对齐**：observe 即排队一次 initial notification（spec §3.2 保证行为）——视口内 target 派发 isIntersecting=true + ratio；视口外派发 isIntersecting=false + ratio=0（仍派发，非丢弃）。

**验证**：`make test` **13217 passed / 0 failed / 74 ignored**（exit 0，零回归）+ workspace clippy `-D warnings` 零警告 + fmt clean + `make product-smoke` 全 struct PASS（welcome desktop **17.03%** 精确持平 held baseline + wintertc/morning desktop + welcome/morning/wintertc 窄屏 375/320 全 PASS）。3 driving test @ renderer worker（intersecting→`true:true:full` / not-intersecting→`false:0` initial notification / disconnect→不派发）；browser worker 经共享 shim 覆盖（无需重复测试，IO 逻辑全在共享 shim，区别于 gBCR 的 per-worker host 接线）。

**为何零回归且净正向**：旧 shim 无 IO → `new IntersectionObserver` 抛 ReferenceError **中断脚本后续全部 JS**；本切片消除之（IO 常驻，不抛）。gBCR 未注册（reftest/polyfill/WebView 路径）→ target rect 为零 → isIntersecting=false，仍派发 initial notification（no-throw）。self-source reftest test/ref 同经 shim → 净中性；product smoke welcome 17.03% 精确持平证真实页面 JS 执行链零回归。

**已知限制（接受，follow-up）**：
1. **仅 observe 时计算**（非持续 host tick）：scroll/resize/async-layout 变化触发的后续通知为 **Slice 2b**（须 host render-loop tick 或 `__zwResolveCallback` 重算钩子）。首切片覆盖 spec 保证的 initial notification——视口检测/懒加载初始化/feature-detect 的主流模式。
2. **handle-identity（createElement）元素** sel 为空 → 零 rect（同 gBCR 限制，path A 持久身份映射 follow-up）。
3. **rootMargin 暂按 0** 处理（defer 像素/% 展开）；root 为元素时取其 selector rect。
4. **ResizeObserver（Slice 3）** 仍未实现（shim 无；旧 polyfill 有桩不触发）——下一切片，复用同一 gBCR rect 反馈 + size-diff 检测。

## R2651：Slice 3 — ResizeObserver 真实化（JS 侧，复用 gBCR）

承接 Slice 2a（IO）落地后的 follow-up「ResizeObserver（Slice 3），复用同一 gBCR rect 反馈 + size-diff 检测」。核验 shim：生产 worker 路径（`js_dom_shim.js`）**完全无 ResizeObserver**——`new ResizeObserver(...)` 抛 ReferenceError 中断整个脚本（与 IO 同）；旧 polyfill（`dom_bridge.rs`，仅 WebView 路径）有 observe/unobserve/disconnect/takeRecords + Entry 桩但**永不触发回调**。

**关键决策**：RO **纯 JS 侧实现**（镜像 IO 的 JS 侧拦截 + microtask 派发模式），**复用已落地的 `__zw_getBoundingClientRect` host 回调**（gBCR path C）+ size-diff 检测——**无需新 host Rust 基建**（同 IO 决策：host-side tick / AsyncResolver 方案 deferred，JS 侧更简且零 host 风险）。直接复用 IO 已落地的 `_io_rectFromSel`（gBCR-via-selector）/ `_io_domRect`（DOMRect 构造）/ `_io_id`（observe 身份）rect 辅助，无重复实现。

**已 land（纯 shim `js_dom_shim.js`，commit 本轮）**——IntersectionObserver 之后插入 RO block：

| 组件 | 行为 |
|------|------|
| `ResizeObserver(callback)` | 存 callback；`_targets`（id→{proxy}）/ `_lastSize`（id→{w,h}，undefined=未派发过=initial）/ `_scheduled` 状态。 |
| `_schedule()` | `_defer`（microtask）派发：遍历 `_targets`，对每个 target 经 `_io_rectFromSel(sel)` 取当前 size（复用 gBCR）；`_lastSize[id]==null`（首次=initial）**或**宽高变化 → 构造 ResizeObserverEntry（target/contentRect/borderBoxSize/contentBoxSize/devicePixelContentBoxSize）投递 `obs._callback(entries, obs)`，更新 `_lastSize[id]`。 |
| `observe/unobserve/disconnect/takeRecords` | 镜像 IO；observe 排队 initial notification（重复 observe 已观察 target 不重置 last，size-diff 自然处理 layout 变化），disconnect 清 `_targets` 使排队中的派发成空。 |
| `ResizeObserverEntry` | 兼容构造（部分脚本 `new ResizeObserverEntry(init)`）。 |

**spec 对齐**：observe 即排队一次 initial notification（spec §4 保证行为）——contentRect 匹配 snapshot 尺寸；后续仅在尺寸变化时派发（同 spec）。entry 形态兼容主流库（contentRect + borderBoxSize/contentBoxSize 数组）。

**验证**：`make test` 全绿（exit 0，零回归）+ workspace clippy `-D warnings` 零警告 + fmt clean + `make product-smoke` 全 struct PASS（welcome/morning/wintertc desktop + 窄屏 375/320 全 PASS，desktop diff≤20%）。3 driving test @ renderer worker（initial→`true:100x50:100` / 零回落→`0x0` / disconnect→不派发）；browser worker 经共享 shim 覆盖（无需重复测试，RO 逻辑全在共享 shim，复用 gBCR host 接线）。

**为何零回归且净正向**：旧 shim 无 RO → `new ResizeObserver` 抛 ReferenceError **中断脚本后续全部 JS**；本切片消除之（RO 常驻，不抛）。gBCR 未注册（reftest/polyfill/WebView 路径）→ contentRect 为零 → 仍派发 initial notification（no-throw）。self-source reftest test/ref 同经 shim → 净中性；product smoke struct PASS 证真实页面 JS 执行链零回归。

**已知限制（接受，follow-up）**：
1. **仅 observe 时计算**（非持续 host tick）：resize/async-layout 变化触发的后续通知为 **Slice 2b**（须 host render-loop tick 或 `__zwResolveCallback` 重算钩子，与 IO 同）。首切片覆盖 spec 保证的 initial notification——组件挂载测量/虚拟列表 item 量测的主流模式。
2. **contentRect 取 gBCR rect**（≈border-box）：真实浏览器报 content-box（padding/border 扣除），本切片近似为 border-box。padding/border 扣除为 follow-up（须 host 暴露 content-box rect）。
3. **handle-identity（createElement）元素** sel 为空 → 零 rect（同 gBCR/IO 限制，path A 持久身份映射 follow-up）。
4. **borderBoxSize/contentBoxSize 近似为单元素数组**：inlineSize=width、blockSize=height（无 writing-mode 方向区分，horizontal-tb 假设）。

**P1a layout-geometry-feedback 切片进度小结**：Slice 1（gBCR 真实化 R2645-R2649，含 renderer/browser in-process 接线 + thread-local cache）+ Slice 2a（IO R2650）+ Slice 3（RO R2651）均已 land。剩余共同 follow-up = **Slice 2b**（持续 host tick：scroll/resize/async-layout 变化触发的后续 IO/RO 通知，须 host render-loop tick 或 `__zwResolveCallback` 重算钩子）+ **path A**（handle-identity createElement 元素持久身份映射）+ content-box rect（padding/border 扣除）。

## R2652：Slice 2b — observer host render-loop tick（JS 侧 registry + host 单点注入）

承接 Slice 2a/3 限制 ①「仅 observe 时计算，非持续 host tick」。observe 仅派发 initial notification；后续真实 layout 变化（render 后 snapshot 填了真实 rect）不再触发回调——IO/RO 退化成「measure-on-mount-only」。本切片补 host render-loop tick。

**关键决策**：IO/RO 的 `_schedule()` 已复算所有 target，并在 cross-threshold（IO `_crossed`）/ size-change（RO size-diff）时派发——故 Slice 2b 不需 recon 设想的「新增 host `FrameTick` 命令变体」，而是 **「render 后对每个活跃 observer 调一次 `_schedule()`」**：纯 JS 侧 registry + host 在 `publish_webview` 末尾单点注入 tick 脚本。`_defer`（queueMicrotask）在 execute 末尾 V8 checkpoint drain，回调同步触发。

**已 land（commit 本轮）**：

| 模块 | 变更 |
|------|------|
| `crates/engine/src/js_dom_shim.js` | `_zwObservers` registry（IO/RO 构造时 push）+ `globalThis.__zw_observers_tick`：遍历 registry，跳过无活跃 target 者（disconnect/unobserve-all 后 no-op），对活跃者调 `_schedule()`（复用既有 cross/size-change gate）。 |
| `apps/renderer/src/page_scripts.rs` | `pub fn tick_observers(ctx) -> bool`：镜像 `dispatch_dom_event` 的 `set_dom_snapshot→clear mutations→execute_script_direct(tick)→apply_recorded_mutations`；返回回调是否改 DOM。 |
| `apps/renderer/src/main.rs` | `publish_webview` 末尾（`fill_layout_rect_snapshot` + `publish_render_with_layout` 后）触发 tick——覆盖所有 render（初始 load / 事件派发 / rerender）；`observer_tick_depth: u32` 重入守卫防 tick→rerender→publish→tick 链（单次外部触发最多 2 次 publish）；kill-switch `ZW_REAL_RECT=0`（兼 gBCR）或 JS 关时跳过；`tick_observers_inner`：tick → apply mutation → 若改 DOM 单次 `rerender_publish_webview`（再入 publish_webview，depth>0 跳过 tick）。 |
| `apps/renderer/src/js_worker.rs` | `real_rect_enabled()` 提为 `pub(crate)`（兼作 tick kill-switch）+ driving test + `wait_eq` 轮询辅助（probe 本身触发 microtask drain）。 |

**为何收敛/零反馈环**：observer 的 `_schedule()` 仅在 cross-threshold/size-change 时派发——target 几何稳定则不派发；callback 设置固定尺寸 1-2 tick 即收敛。`observer_tick_depth` 守卫为兜底：即便 callback 每次改 layout，也限制单次外部触发最多 2 次 publish（tick 的 rerender 再入 publish_webview 时 depth>0 跳过 tick）。

**验证**：`make test` 全绿（exit 0，零回归——含 renderer runtime 单测、integration 全量）+ workspace clippy `-D warnings` 零警告 + fmt clean + `make product-smoke` 全 struct PASS（welcome/morning/wintertc desktop diff≤20% + 窄屏 375/320 全 PASS，per-render tick 无无限渲染）。driving test @ renderer worker：observe（initial，__calls=1，__last=100x50）→ 更新 snapshot 200x80 → `__zw_observers_tick` → RO size-diff 再次派发（__calls=2，__last=200x80）→ size 未变再 tick → 不派发（__calls 仍 2）。

**已知限制（follow-up）**：
1. **browser in-process `tab_worker` 路径未接 tick**：mirror follow-up——shim 共享（`__zw_observers_tick` 全 worker 可用），仅需 `apps/browser/src/tab_scripts.rs` 加 `tick_observers` + `tab_worker.rs` 的 `push_snapshot`/render 路径接线。cross-process browser 经 renderer 进程已覆盖（renderer `publish_webview` 已 tick）。
2. **observer 注册表 leak = observer 创建总数**：每页有界（构造即 push，不随 disconnect 移除）。WeakRef 注册表（V8 支持）为生产硬化 follow-up；当前 disconnect 的 observer tick 时跳过（无 target），仅占数组槽。
3. **tick 回调 DOM mutation 仅单次 rerender**：不递归 tick，故 callback 链式改 layout（A 改 → 影响 B 的 target）的完全收敛依赖下一外部触发。可接受（主流测量类 callback 一次即收敛）。

**P1a layout-geometry-feedback 切片进度小结（更新）**：Slice 1 + 2a + 3 + **2b（renderer 路径 host tick，本轮）** 均 land。剩余 follow-up：① browser in-process tick 接线（mirror）；② path A handle-identity（createElement 元素持久身份映射）；③ content-box rect（padding/border 扣除）；④ WeakRef 注册表硬化。

## R2661：path A handle-identity——createElement 元素 gBCR/IO/RO 返真实 rect

承接 gBCR/IO/RO 各切片共同 follow-up「handle-identity（createElement 元素，sel 为空）→ 零 rect」。R2647 path C 解决了 selector-identity（querySelector/getElementById 元素）；但 JS 持原 `createElement` 返回的 handle 引用（组件式 hold-ref 测量）走 handle-identity（`__n{n}`），`find_by_selector("__n{n}")` 不匹配 → 零 rect。本切片补 path A 持久 handle→身份基建。

### 关键设计决策（recon 后确认）

1. **必须 selector 解析，不能直映 handle→NodeId**：worker 的 `apply_dom_mutations` 在「mutated doc D'」上为 createElement 分配 NodeId（slotmap 追加在已有节点之后），而渲染管线/handler 都 fresh-parse 序列化后的 html（文档序分配）。**D' 的 NodeId ≠ fresh-parse 的 NodeId**（如 `insertBefore` 把新元素插中间时）。故唯一稳健映射是 handle→**selector**，再 `find_by_selector`（fresh-parse）→ NodeId（与 snapshot 键一致，R2647 确定性）。
2. **唯一性闸门（避免错值）**：`stable_selector_for_node` 对无 id/class 元素只返 tag（如 `"div"`），多同 tag 文档里有歧义——`find_by_selector` 返首个匹配可能是**别的元素**（静默错值，比零值更坏）。故仅当选择器在文档中**唯一匹配**（`query_selector_all.len()==1`）才入 map；歧义者跳过 → 该 handle 回落零 rect（宁可零值不错值）。新增 `unique_selector_for_node` helper。
3. **反映变更后状态**：在 `apply_dom_mutations` **末尾**遍历 ephemeral handles 算选择器（同 batch 内后置 `SetAttrOnHandle` 设的 id/class 已生效）——覆盖 `createElement; el.id='x'; appendChild` 主流模式。
4. **持久 map + 跨线程共享**：apply 在 renderer 主线程 / browser 主线程跑，handler 在 js_worker 线程读 → 须 `Arc<Mutex<HashMap<String,String>>>`（`HandleSelectorMap`，镜像 `LayoutRectSnapshot` 模式）。worker 持有，clone 给 handler；apply 路径 merge（upsert）。
5. **导航失效**：`SetDomSnapshot` url 变化（导航）→ worker 清空 map（旧页 handle 在新页无效）。同页跨 batch upsert，handle 持久（JS 持 ref 跨事件/定时器复测）。

### 已 land（commit 本轮）

| 模块 | 变更 |
|------|------|
| `crates/engine/src/js_dom_bridge.rs` | `unique_selector_for_node`（stable_selector + 唯一性闸门）；`apply_dom_mutations` 末尾遍历 handles 建 handle→唯一 selector map 并返回（`Result<HashMap<String,String>, String>`）；`apply_mutations_to_html` 丢弃 map（返 String，**零测试改动**）；新增 `apply_mutations_to_html_with_handles` 返 `(String, map)` 供生产路径。+ 3 单测（唯一/歧义/同 batch 后置 id）。 |
| `crates/engine/src/rect_bridge.rs` | `HandleSelectorMap` 类型 + `new_handle_selector_map()`；`make_dom_html_rect_handler` 加 `handle_map` 第 3 参 + handle-identity 分支（`is_handle_identity`：`__n` 前缀 → 查 map→selector；否则当 selector）。+ 2 单测（handle 命中返真实 rect / 空 map 回落 None）。 |
| `crates/engine/src/js_dom_shim.js` | gBCR 闭包 `var id = sel \|\| handle`（sel 空→handle）；IO `_compute` 与 RO `_schedule` 的 `_io_rectFromSel` 传 `sel \|\| __zwHandle`。覆盖 createElement 元素的 gBCR + IO + RO。 |
| `apps/renderer/src/js_worker.rs` | `RendererJsWorker` 加 `handle_selector_map` 字段 + 构造/clone 给 handler + `SetDomSnapshot` url 变化清空 + `pub handle_selector_map()` accessor。+ driving test（createElement+setId+append→apply 产 map→merge→set_dom_snapshot 新 html→填 snapshot→经 handle 测量返真实 rect）。 |
| `apps/renderer/src/page_scripts.rs` | `apply_recorded_mutations` 改用 `apply_mutations_to_html_with_handles`，merge map 进 `ctx.js_worker.handle_selector_map()`。 |
| `apps/browser/src/tab_js_worker.rs` | 镜像 renderer：`handle_selector_map` 字段 + 构造 + clone 给 handler + `SetDomSnapshot` url 变化清空 + accessor。 |
| `apps/browser/src/tab_scripts.rs` | `apply_recorded_mutations` 改用 `apply_mutations_to_html_with_handles`，merge map。 |

**验证**：`make test` 全绿（exit 0，零回归）+ workspace clippy `-D warnings` 零警告 + fmt clean + `make product-smoke` 全 struct PASS（welcome desktop **17.03%** 精确持平 held baseline + 窄屏 375/320 全 PASS）。engine 单测（apply_dom_mutations map 3 + rect_bridge handle 2）+ renderer worker driving test 验证全链路。

**为何零回归且净正向**：旧路径 handle-identity sel 空 → 零 rect；本切片 sel 空→传 handle→查 map→真实 rect（净正向，解锁 SPA 动态元素测量）。map 未命中（歧义跳过 / 未 merge / browser reftest 路径未注册）→ 回落零 rect（= 旧行为）。self-source reftest test/ref 同经 shim → 净中性；product smoke welcome 17.03% 持平证真实页面 JS 执行链零回归。唯一性闸门保证不返错值。

**已知限制（接受，follow-up）**：
1. **tag-only 歧义元素回落零 rect**：无 id/class 且文档有多个同 tag 的 createElement 元素 → 选择器歧义 → 不入 map → 零 rect（宁可零值）。覆盖有 id/class 的主流模式（`el.id=`/`el.className=` 设后测）。`:nth-child` 结构选择器（dom 选择器引擎暂不支持）可解锁无 id 元素，为 follow-up。
2. **跨 batch 改 id 不更新 map**：batch 内 self-contained（create+setup+append）；跨 batch `el.id=` 经 SetAttrOnHandle 在 apply 报「unknown handle」（ephemeral handles 仅本 batch）不应用 → dom_html 不变 → 原 selector 仍有效（无 stale）。即跨 batch handle 操作本就受限（既有行为，非本切片引入）。
3. **stale-but-non-zero（同 gBCR）**：rect 反映「上次 render」；createElement 同脚本内即读见零（元素未 render）。下次 render 后复测见真实 rect（driving test 即此模式）。
4. **root 为 createElement 元素的 IO**：`opts.root.__zwSelector` 为 null → 回落 viewport（root=create-element 罕见，follow-up）。
5. **browser in-process observer host-tick（R2652 follow-up ①）仍未接**：与 path A 独立。

**P1a layout-geometry-feedback 切片进度小结（再更新）**：Slice 1 + 2a + 3 + 2b + **path A（handle-identity，本轮）** 均 land。gBCR/IO/RO 现覆盖 selector-identity + handle-identity 两种元素身份。剩余 follow-up：① browser in-process observer tick 接线（R2652 mirror）；② `:nth-child` 结构选择器（解锁 tag-only 歧义元素）；③ content-box rect（padding/border 扣除）；④ WeakRef observer 注册表硬化；⑤ force-reflow-on-demand（改后即读见真实 rect）。



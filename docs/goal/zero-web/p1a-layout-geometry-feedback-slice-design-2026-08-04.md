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



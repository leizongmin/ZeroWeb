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

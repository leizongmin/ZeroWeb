# 快照、live DOM、mutation 读回与原生路由

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查 createElement 后查询身份丢失、apply 后属性读空、快照重解析和 native/shim 路由。对象生命周期另读 [句柄与代际](handles-and-generations.md)，布局/发布的宿主事务见 [运行时](../../../runtime/index.md)。

## Gotchas
- 进程内 WebView 可发布共享 live Document；renderer/tab JS 线程从 `dom_html` 快照重解析，不能把主线程的 `Rc<RefCell<Document>>` 跨线程捕获进 Send + Sync 回调。 (`crates/webview/src/webview.rs`, `apps/renderer/src/js_worker.rs`, `apps/browser/src/tab_js_worker.rs`)
- worker 的 DOM 快照变化只刷新数据源，不重新安装 Event/CustomEvent；重复安装会覆盖 shim 的私有取消字段，导致监听器已 preventDefault 而宿主仍执行默认动作。 (`apps/renderer/src/js_worker.rs`, `apps/browser/src/tab_js_worker.rs`, `crates/engine/src/quickjs_dom_bindings.rs`)
- drain mutation 后必须 bump 查询视图的 drain generation；仅以 snapshot generation 和 mutation 数量缓存，会在没有新快照且新旧批数量相等时命中旧查询视图。 (`apps/renderer/src/page_scripts.rs`, `crates/engine/src/js_dom_bridge.rs`)
- apply 成功还要通知 shim apply_generation 并更新 handle→selector 映射；Remove 失效不能删掉同批后续重建的新绑定，否则 query 回来的 proxy 身份与新节点断开。 (`apps/renderer/src/page_scripts.rs`, `crates/engine/src/js_dom_bridge.rs`, `crates/webview/src/webview.rs`)
- renderer 按 handle→selector 表的 value 反查是 O(n)，多 handle 指向同 selector 时命中不确定；WebView 的 selector→handle 正向表是最后写者胜，不能宣称两条反查路径同样确定。 (`apps/renderer/src/js_worker.rs`, `crates/webview/src/webview.rs`)
- text/tag/attr 的 mutation history 回落机制已存在，但当前 WebView/renderer apply 链未调用 `append_mutation_history`；批边界 harness 可装配该契约，不证明生产跨 apply 读回已接通。 (`crates/engine/src/js_dom_bridge/callbacks.rs`, `crates/engine/src/js_dom_bridge.rs`, `apps/renderer/src/page_scripts.rs`, `crates/webview/src/webview.rs`, `git:b487eeac3a812a47fbb0e1e169aebdfe3252bbcf`)
- attr 的空字符串值不能区分 set-empty 与 absent/remove；读回要同时核对 get 和 has，两回调批内 miss 才回落历史，不能用空字符串代替是否命中的信号。 (`crates/engine/src/js_dom_bridge/callbacks.rs`, `crates/engine/src/js_dom_bridge.rs`, `git:dad3f9969`)

## Architecture
- native DOM 已是唯一生产安装路径，但页面对象仍需 shim 的事件、集合及 Web API；删除 `ZW_NATIVE_DOM` 开关并不意味着可以删掉字符串回调或 shim。 (`crates/webview/src/webview.rs`, `crates/engine/src/dom_bindings/mod.rs`, `git:222b2d6c9a58c6d46e00dfe46359f0c4709990a9`)
- HTML 刷新 helper 留在 engine，worker 调 `refresh_*_dom_source_from_html`；在 browser 直接用 zero_dom 解析曾因仅 dev-dependency 而破坏生产库构建。 (`crates/engine/src/quickjs_dom_bindings.rs`, `apps/browser/src/tab_js_worker.rs`, `git:a3b2d7459e0698b31d76b8ff32db9e5fb26713e8`)

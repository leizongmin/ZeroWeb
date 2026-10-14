# 身份桥、活文档与引擎对象生命周期

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

区分三个有效期：Document 的 NodeId、引擎 wrapper/模板的 Context/Isolate，以及 shim 查询视图的快照/drain/apply 代际。底层身份失效与缓存视图失效不是同一事件。

## Gotchas
- 同一个 NodeId 可仍存在于 detached arena，而其 wrapper 可已被 V8 GC 回收；树连接态、节点存在性和 JS 对象可达性不能用一个 stale 布尔值替代。 (`crates/engine/src/dom_bindings/gc.rs`, `crates/engine/src/dom_bindings/node.rs`)
- Context reset、DOM 快照更新、mutation drain 和 host apply 各自改变不同缓存层，不能只刷新 HTML 文本就认为模板、监听器和查询缓存都已同步。 (`apps/renderer/src/js_worker.rs`, `apps/renderer/src/page_scripts.rs`, `crates/engine/src/js_dom_bridge.rs`)

## Child Knowledge Nodes
- [NodeId、对象缓存、代际与 GC](handles-and-generations.md) — weak/Persistent、internal slot、跨引擎销毁与集合身份。
- [快照、live DOM、mutation 读回与原生路由](snapshot-and-live-dom.md) — 线程数据源、查询缓存、handle/selector 表与历史回落接线。

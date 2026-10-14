# 原生 DOM 与 shim 协作

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

原生模板、对象缓存与 shim 共同组成页面 DOM 面。身份/生命周期问题先读 identity；具体方法、named handler 和 customElements hooks 再读 API。DOM 树算法不在这里重复。

## Architecture
- 同一个节点可能经 native wrapper 或 shim selector/handle proxy 访问，原生安装已固定不等于所有查询、事件与写入都绕过 shim；跨面身份及状态同步仍是宿主责任。 (`crates/engine/src/dom_bindings/mod.rs`, `crates/engine/src/js_dom_bridge.rs`, `crates/webview/src/webview.rs`)
- bindings 的 raw native 测试与页面生产装配分开，前者直接建 Context，后者还安装 shim、宿主 callback 及 apply；排查须按实际消费者判断哪层未接。 (`crates/engine/src/dom_bindings/tests.rs`, `crates/engine/src/dom_bindings/tests_ce.rs`, `apps/renderer/src/js_worker.rs`)

## Child Knowledge Nodes
- [身份桥、活文档与引擎对象生命周期](identity/index.md) — 跨 Context 句柄、GC、snapshot/live 与 mutation 读回。
- [节点、元素、集合、CSSOM 与自定义元素](api-surface.md) — classList/style/dataset、选择器回落与 CE realm/lifecycle。

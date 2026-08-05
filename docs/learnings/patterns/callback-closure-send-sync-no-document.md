# 回调闭包 Send+Sync 约束：不能缓存 Document

- 日期：2026-08-05
- 相关模块：`zero-engine`（`js_dom_bridge.rs`）、`zero-script-sandbox`（`register_callback`）、`zero-dom`（`Document`）
- 关联轮次：R2706（getComputedStyle per-snapshot 缓存）

## 问题描述

想给 `__zw_get_computed_style`（及其它 `__zw_*` host 回调）加 per-snapshot 缓存，缓存解析后的
`Document` + 计算结果，避免每次属性查询重跑 `parse_html` + `compute_styles`。直觉写法：

```rust
let cache: Arc<Mutex<Option<(String, Document, HashMap<NodeId, ComputedStyle>)>>> =
    Arc::new(Mutex::new(None));
sandbox.register_callback("__zw_get_computed_style", Box::new(move |args| { ... }));
```

编译报错：`Document cannot be sent between threads safely` / `Sync not implemented`。

## 根因分析

`V8Sandbox::register_callback` 的签名要求 `Box<dyn Fn(&[String]) -> String + Send + Sync>`。
闭包捕获的 `Arc<Mutex<T>>` 要 `Send + Sync`，归结为 `T: Send`。

`zero_dom::Document` **不是 `Send`**，三处来源：

1. `observers: Vec<MutationObserver>` —— 持 `dyn Fn(&[MutationRecord])` 回调（非 Send）。
2. `event_listeners: HashMap<(NodeId, String), Vec<ListenerEntry>>` —— `ListenerEntry` 持
   `dyn Fn(&mut Event)`（非 Send）。
3. `nodes: SlotMap<NodeId, NodeData>` —— `NodeData` 经 html5ever 含 `tendril::Tendril`，
   其 `NonAtomic` variant 用 `Cell<usize>`（非 Sync）。

任一处都足以使整个 `Document` 非 Send，故**任何**把 `Document` 放进 `Send+Sync` 闭包捕获的
缓存都不可能编译通过。

## 解决方案

不缓存 `Document`，只缓存**纯值类型**的结果。`ComputedStyle` 全是 enum/`f64`/`Vec<String>`/
`Option<String>`，是 `Send`。所以缓存结构改为：

```rust
// html_key → (selector → ComputedStyle)
let cache: Arc<Mutex<Option<(String, HashMap<String, ComputedStyle>)>>> = Arc::new(...);
```

- html 变（snapshot key 变）→ 清空 per-selector 缓存。
- 同 selector 命中 → 仅 `serialize_computed_property`（O(1)）。
- 新 selector → `compute_document_styles`（parse+cascade，产临时 `Document`）一次，clone 该
  selector 的 `ComputedStyle` 入缓存。

`Document` 仅作临时局部变量存在（`compute_document_styles` 返回后随栈帧释放），不入缓存，绕开
Send 约束。代价：查 N 个不同元素 = N 次 cascade（无法跨元素共享一次 parse+cascade 的 Document）。

## 如何避免 / 复用提示

- **任何 `register_callback` 闭包想跨调用缓存的状态，都必须是 `Send` 的纯值类型**（String/
  HashMap<纯值>/Vec<纯值>/数字）。不要试图缓存 `Document`、`StyleSystem` 等含闭包/`Cell`/`Rc`
  的复合类型。
- 若确需跨元素共享一次 cascade（缓存整个 Document + styles），只能用 `thread_local!`（不要求
  Send，因为不进闭包捕获）——代价是引入 per-thread 全局、跨线程迁移时缓存失效，仅在确有性能
  必要且确认 V8 isolate 单线程使用时采用。
- 排查「某类型是否 Send」的快速法：`grep` 该类型及其字段类型有无 `Rc<`/`RefCell<`/`Cell<`/
  `*const `/`*mut `/`dyn Fn`；html5ever/tendril 系类型默认带 `Cell`，须警惕。

# NodeId、对象缓存、代际与 GC

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查同节点是否返回同对象、跨 WebView panic、detached 节点监听器与原生 internal slot。底层 arena/tree 语义见 [文档身份](../../../page-core/document/tree-and-parsing.md)。

### User Task Navigation

`gc.rs` / `quickjs_dom_bindings.rs` 管缓存；`webview.rs` 管安装及销毁。`tests_collections.rs` 验证集合身份，`tests_events.rs` 包含跨 Isolate 重建及焦点镜像判据；本轮未运行。

## Gotchas
- V8 wrapper 的 internal slot 存 NodeId FFI 编码，不是可解引用的节点指针；读操作必须 decode 并在当前 Document 校验节点仍存在。 (`crates/engine/src/dom_bindings/mod.rs`, `crates/engine/src/dom_bindings/gc.rs`, `crates/engine/src/dom_bindings/node.rs`)
- `removeChild` 只断开树连接，arena 中节点仍可存在；不能在移除时清 listener，否则 detached 引用重新挂树后监听器丢失，V8 wrapper GC finalizer 才是对应清理点。 (`crates/engine/src/dom_bindings/gc.rs`, `crates/engine/src/dom_bindings/node.rs`)
- V8 节点及集合身份缓存持 Weak，JS 丢最后强引用可回收；QuickJS 的 NODE_OBJECTS、LISTENERS 与 registry 使用强 Persistent，不能把两者描述为相同的弱回收机制。 (`crates/engine/src/dom_bindings/gc.rs`, `crates/engine/src/quickjs_dom_bindings.rs`)
- QuickJS 的 `__zwNodeFfi` 不可枚举、不可配置、不可写，但其 u64 经 f64 存储，只能保证安全整数范围内精确；属性保护不等于完整 64 位身份表示。 (`crates/engine/src/quickjs_dom_bindings.rs`)
- TLS 模板/Global 不能跨 Isolate 使用；WebView 记自己安装后的 state_generation，异代先 reset 再重装，同代只刷新 DOM Rc，保持对象身份并避免每 execute 重建模板。 (`crates/webview/src/webview.rs`, `crates/engine/src/dom_bindings/mod.rs`, `git:3d41e6d11bbeaa6ac5ce5d59757b36444ed63e7f`)
- WebView Drop 清原生 TLS 必须发生在引擎销毁边界；只释放 Context 而保留旧 Global/Persistent，会使同线程下一 WebView 命中已销毁引擎的句柄。 (`crates/webview/src/webview.rs`, `crates/engine/src/dom_bindings/gc.rs`, `crates/engine/src/quickjs_dom_bindings.rs`)

## Architecture
- Attr 身份键是 `(owner NodeId, attribute name)`，NamedNodeMap、classList、style、dataset 身份键是 owner；这些对象读当前属性而不是缓存属性值，不能用创建时值替代 live getter。 (`crates/engine/src/dom_bindings/namednodemap.rs`, `crates/engine/src/dom_bindings/dom_token_list.rs`, `crates/engine/src/dom_bindings/dataset.rs`)

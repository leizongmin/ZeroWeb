# 节点、元素、集合、CSSOM 与自定义元素

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

按 API 查 `node.rs`、`element.rs`、`document.rs`、named-property-handler 集合与 CE hooks。这里覆盖绑定层，树算法归 [文档模型](../../page-core/document/index.md)，CSS 计算归 [样式](../../page-core/styles/index.md)。

### User Task Navigation

`dom_bindings/tests_dom_api.rs` / `tests_collections.rs` / `tests_ce.rs` 直接装 native，`tests_ab_compare.rs` 比较双路径。低层测试不经完整页面 shim，不把它们当生产对象路由已验收。

## Gotchas
- 原生 classList 是同 owner 的 live 对象，外部 setAttribute 会反映到后续读取；删除最后 token 写 `class=""` 而不是移除属性，hasAttribute 仍为 true。 (`crates/engine/src/dom_bindings/dom_token_list.rs`, `crates/engine/src/dom_bindings/tests_collections.rs`)
- classList 多 token 写入先校验全部参数再 mutation，后续非法 token 不得留下前面合法 token；当前 native 校验用 Unicode `is_whitespace`，比规范 ASCII whitespace 宽，不能把测试的 ASCII 样本扩大成完全合规结论。 (`crates/engine/src/dom_bindings/dom_token_list.rs`, `crates/engine/src/dom_bindings/tests_collections.rs`)
- dataset 缺失键返回 undefined，style 缺失属性返回空串；二者都用 named handler，但不能共享一个无差别的缺省值策略。 (`crates/engine/src/dom_bindings/dataset.rs`, `crates/engine/src/dom_bindings/css_style_declaration.rs`)
- dataset/style 对 constructor、toString、then 等协议名让原型处理，不把它们当任意 DOM 字段拦截；否则 Promise 及对象协议会读到错误值。 (`crates/engine/src/dom_bindings/dataset.rs`, `crates/engine/src/dom_bindings/css_style_declaration.rs`)
- 页面 getElementById 的 selector 快路径可能遇到 `%`、`[` 等特殊 id，不可把选择器解析错误等同元素不存在；当前 shim 捕获后枚举 `[id]` 精确比较回落。 (`crates/engine/src/js_dom_shim/part06.js`, `git:d51a357c9a46ad3454c0f7ccf4e66922215df4cc`)

## Architecture
- CE 构造/registry 由 shim 管，native 树写负责 connect/disconnect/attribute hooks；非连字符标签快速跳过，hook 不存在时静默，不应从低层 native 安装推断 registry 已就位。 (`crates/engine/src/dom_bindings/custom_elements.rs`, `crates/engine/src/dom_bindings/tests_ce.rs`)
- CE hook 携带元素 owner_document 的 root 身份，lookup/notify 按 ownerId 路由 realm；移动子树的 lifecycle 应按真实连接态变化派发，不能只按 insert/remove 操作次数派发。 (`crates/engine/src/dom_bindings/custom_elements.rs`, `crates/engine/src/dom_bindings/tests_ce.rs`)
- 原生 accessor 应从持有 internal slot 的实例读取 NodeId；PoC 的 JS `class extends` 证明 super() 可继承 native slot 布局，但把 accessor 移到无 slot 的 prototype holder 会读不到身份。 (`crates/script-sandbox/src/dom_bindings.rs`, `crates/engine/src/dom_bindings/mod.rs`)

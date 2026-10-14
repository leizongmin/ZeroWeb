# 解析、序列化、树身份与查询

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 `Document`、html5ever `TreeSink`、片段解析、序列化与原生查询。动态 JS 身份与快照同步见 [绑定身份](../../scripting/dom-binding/identity/index.md)，不由本节点定义。

### User Task Navigation
修复 innerHTML 的表格/foreign context 时从 `parse_html_fragment_with_scripting` 入手；序列化往返查 `serialize_node` 与 template 侧表。验证源码入口为 `crates/dom/src/tests/tests_7_parser.rs`、`crates/dom/src/serializer.rs` 内测试，本次未运行。

## Gotchas
- `NodeId::is_valid()` 恒为 true，不能证明节点仍存在；必须用所属 `Document::get/contains` 查 slotmap，不能跨 Document 解释同一个 key。 (`crates/dom/src/node.rs`, `crates/dom/src/document/mod.rs`)
- `remove_child` 只解除父子关系并移除子树 id 索引，不销毁 slotmap 节点；脱离文档后的节点仍可按原身份访问。 (`crates/dom/src/document/mod.rs`)
- id 索引重建与插入都用 HashMap `insert`，重复 id 会覆盖旧映射；不能根据 `:target` 注释推断 `get_element_by_id` 总能取树序首个。 (`crates/dom/src/document/mod.rs`, `crates/dom/src/document/target.rs`)
- `parse_html` 默认 scripting enabled，noscript 按 raw text；inert 文档必须明确传 false，否则解析形态与序列化转义都不同。 (`crates/dom/src/parser.rs`, `crates/dom/src/serializer.rs`)
- 片段解析必须传真实命名空间和 local name；table context 会生成合成 html 包裹及隐式 tbody，拿完整文档解析代替会触发 foster-parent 丢内容。 (`crates/dom/src/parser.rs`, `crates/dom/src/tests/tests_7_parser.rs`)
- foreign context 的 CDATA 通过预转义桥接为 character data；进入 integration point 的 HTML 子元素后的逐点命名空间切换尚不在该桥接范围，不能称完整 XML fragment parser。 (`crates/dom/src/parser.rs`)
- `template` 内容存于独立 inert DocumentFragment，元素 children 不代表 contents；查询、克隆、序列化必须分别核对侧表路径。 (`crates/dom/src/parser.rs`, `crates/dom/src/document/mod.rs`, `crates/dom/src/serializer.rs`)
- style/script 等 raw-text 子节点不能通用 HTML 转义，否则 CDATA、比较符与 CSS/JS 源码在再次解析时被破坏；noscript 还要读取文档 scripting 旗标。 (`crates/dom/src/serializer.rs`)
- XML/XHTML 判定来自 DOCTYPE public_id 含 XHTML 的检测，底层仍是 html5ever；它影响属性值大小写匹配，不等于换用了 XML 解析器。 (`crates/dom/src/parser.rs`, `crates/dom/src/query.rs`)

## Architecture
- DomBuilder 完成时直接搬移 slotmap、root 与 template 侧表，保持构建期 NodeId 有效；id 索引另行重建，不应恢复逐节点克隆的双树路径。 (`crates/dom/src/parser.rs`, `crates/dom/src/document/mod.rs`)
- DOM 查询使用自己的 SimpleSelector/SelectorChain 解析，而非 style-system 的 CSS AST；列表查询通过单次 DFS 保持文档序与去重，两套选择器支持面不能互相推断。 (`crates/dom/src/query.rs`, `crates/dom/src/document/mod.rs`)
- 命名空间属性序列化保留 xml、xmlns、xlink 等前缀；只输出 local name 会改变 SVG 引用和命名空间 round-trip。 (`crates/dom/src/serializer.rs`)

## Decisions
- template 选择侧表而非给每个 ElementData 增加 Option 字段，源码记录过节点尺寸变化造成 DOM 微基准劣化；这是布局敏感的存储取舍，不是当前重新测量的性能结论。 (`crates/dom/src/document/mod.rs`)

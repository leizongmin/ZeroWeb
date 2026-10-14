# 文档模型与 HTML 语义

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

`zero-dom` 持有原生文档、节点、属性和运行时状态判定；html5ever 提供 HTML 构树，DOM 查询自有解析器。页面样式消费者依赖 Document 的共享状态，JS 绑定与事件循环不在本域实现。

### User Task Navigation
动态状态同步继续读 [脚本绑定身份](../../scripting/dom-binding/identity/index.md) 与 [事件默认动作](../../scripting/event-loop/events-and-default-actions.md)。静态 DOM 验证判据在 `crates/dom/src/tests/mod.rs` 及各源码内测试；本次没有实跑。

## Gotchas
- 原生 DOM、JS live DOM 与渲染快照不能仅凭同名 API 判定等价；重新解析生成新 Document 后，运行时焦点和 URL 等状态需要调用者再次注入。 (`crates/dom/src/document/mod.rs`, `crates/dom/src/document/focus.rs`, `crates/dom/src/document/target.rs`)

## Architecture
- Document 的 slotmap 负责身份，树关系、id、template、shadow 和 mutation 各有独立存储面；变更应沿 Document 方法维护关系，不能只改 NodeData.children 就视为完成。 (`crates/dom/src/document/mod.rs`, `crates/dom/src/document/shadow.rs`)

## Child Knowledge Nodes
- [解析、序列化、树身份与查询](tree-and-parsing.md) — 排查片段上下文、template、raw text、id 索引和查询支持差异。
- [Mutation、Range 与 Shadow DOM](mutations-and-shadow.md) — 排查变更递送、范围偏移、封装边界和 slot fallback。
- [焦点、表单状态、校验与事件模型](forms-and-focus.md) — 排查 Tab 顺序、状态伪类、静态校验和传播控制。

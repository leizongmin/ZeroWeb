# Spec: M2 — HTML 解析 + DOM 树

**版本**: v1.0
**日期**: 2026-05-30
**作者**: AI Assistant
**状态**: Confirmed

---

## 1. 背景与目标

### 1.1 背景

M1 里程碑已完成项目骨架和渲染基础设施迁移。M2 是构建页面内核的第一步：需要能够解析 HTML 文档并构建 DOM 树，为后续的 CSS 样式计算（M3）、布局（M4）和渲染（M5）提供基础数据结构。

html5ever 是 Servo 项目中的 HTML 解析器，采用 Apache-2.0/MIT 双许可，完全符合本项目的许可证要求。它实现了 WHATWG HTML 规范的解析算法，包括错误恢复机制。

### 1.2 目标

- 基于 html5ever 构建完整的 DOM 树数据结构
- 实现符合 DOM Level 2+ 规范的核心 API
- 提供高效的节点存储和查询能力
- 为后续里程碑（CSS、布局、渲染）提供稳定的 DOM 接口

### 1.3 范围边界

**包含**:
- 完整的 DOM 节点类型（Element、Text、Document、Comment、DocumentType、DocumentFragment）
- html5ever TreeSink 集成，支持 HTML 解析和错误恢复
- DOM 树操作 API（appendChild、removeChild、insertBefore、replaceChild 等）
- 属性操作 API（getAttribute、setAttribute、removeAttribute 等）
- 查询 API（getElementById、querySelector/querySelectorAll 基础版）
- textContent 和 innerHTML 序列化
- MutationObserver 基础框架
- ≥50 个单元测试，覆盖率 ≥ 70%
- ≥3 个 criterion 基准测试

**明确排除**:
- Shadow DOM（基础框架预留接口，完整实现在 M12）
- 事件系统（M6）
- DOM Level 3+ 扩展 API
- Range 和 Selection API
- 跨文档节点操作（adoptNode、importNode）

---

## 2. 需求类型概览

| 类型 | 适用 | 来源 |
|------|------|------|
| 业务需求 | 是 | 目标文档 M2 |
| 功能需求 | 是 | 第 3 节 |
| 非功能需求 | 是 | 第 4 节 |
| 接口需求 | 是 | 第 5 节 |

---

## 3. 功能需求

### FR-M2-001: DOM 节点类型
- **描述**: `dom` crate **必须**实现完整的 DOM 节点类型
- **验收标准**:
  - [ ] 支持 Element（含标签名、属性列表、命名空间）
  - [ ] 支持 Text（含文本内容）
  - [ ] 支持 Comment（含注释内容）
  - [ ] 支持 DocumentType（含 name、public_id、system_id）
  - [ ] 支持 DocumentFragment
  - [ ] 支持 Document（文档根节点，含 quirks_mode）
  - [ ] 每个节点有稳定的 NodeId（在删除前保持不变）
  - [ ] 节点存储支持 O(1) 按 ID 查找
- **优先级**: Must

### FR-M2-002: HTML 解析器集成
- **描述**: `dom` crate **必须**集成 html5ever 的 TreeSink trait，支持 HTML 文档解析
- **验收标准**:
  - [ ] 实现 html5ever 的 TreeSink trait（17 个必需方法）
  - [ ] 提供 `parse_html(html: &str) -> Document` 入口函数
  - [ ] 支持 UTF-8 输入解析
  - [ ] 支持错误恢复（malformed HTML 不 panic）
  - [ ] 支持 quirks mode 检测
  - [ ] 正确处理文档类型声明（DOCTYPE）
  - [ ] 正确处理模板元素（`<template>`）
- **优先级**: Must

### FR-M2-003: DOM 树操作 API
- **描述**: `dom` crate **必须**提供完整的 DOM 树修改和遍历 API
- **验收标准**:
  - [ ] `create_element(tag: &str) -> NodeId`
  - [ ] `create_text_node(text: &str) -> NodeId`
  - [ ] `create_comment(text: &str) -> NodeId`
  - [ ] `create_document_fragment() -> NodeId`
  - [ ] `append_child(parent: NodeId, child: NodeId) -> Result<()>`
  - [ ] `remove_child(parent: NodeId, child: NodeId) -> Result<NodeId>`
  - [ ] `insert_before(parent: NodeId, new: NodeId, ref: NodeId) -> Result<()>`
  - [ ] `replace_child(parent: NodeId, new: NodeId, old: NodeId) -> Result<NodeId>`
  - [ ] `clone_node(node: NodeId, deep: bool) -> NodeId`
  - [ ] 遍历: `parent_node`, `first_child`, `last_child`, `next_sibling`, `previous_sibling`, `child_nodes`
  - [ ] `has_child_nodes(node: NodeId) -> bool`
  - [ ] `text_content(node: NodeId) -> Option<String>`
  - [ ] `set_text_content(node: NodeId, text: &str)`
  - [ ] `inner_html(node: NodeId) -> String`（序列化为 HTML 字符串）
- **优先级**: Must

### FR-M2-004: 属性操作 API
- **描述**: `dom` crate **必须**提供元素属性操作 API
- **验收标准**:
  - [ ] `get_attribute(node: NodeId, name: &str) -> Option<String>`
  - [ ] `set_attribute(node: NodeId, name: &str, value: &str)`
  - [ ] `remove_attribute(node: NodeId, name: &str)`
  - [ ] `has_attribute(node: NodeId, name: &str) -> bool`
  - [ ] `attribute_names(node: NodeId) -> Vec<String>`
  - [ ] 支持 id 和 class 特殊属性的快速访问
- **优先级**: Must

### FR-M2-005: 查询 API
- **描述**: `dom` crate **必须**提供基础的 DOM 查询 API
- **验收标准**:
  - [ ] `get_element_by_id(doc: &Document, id: &str) -> Option<NodeId>`
  - [ ] `get_elements_by_tag_name(doc: &Document, tag: &str) -> Vec<NodeId>`
  - [ ] `get_elements_by_class_name(doc: &Document, class: &str) -> Vec<NodeId>`
  - [ ] `query_selector(node: NodeId, selector: &str) -> Option<NodeId>`（基础选择器：标签、#id、.class、[attr]）
  - [ ] `query_selector_all(node: NodeId, selector: &str) -> Vec<NodeId>`
- **优先级**: Must

### FR-M2-006: MutationObserver 框架
- **描述**: `dom` crate **必须**提供 MutationObserver 基础框架
- **验收标准**:
  - [ ] `MutationRecord` 结构体（type、target、added_nodes、removed_nodes、attribute_name、old_value）
  - [ ] `MutationObserver` 结构体（observe、disconnect、takeRecords）
  - [ ] DOM 修改操作自动记录 mutation 记录
  - [ ] 支持 childList 和 attributes 变更类型
- **优先级**: Should

---

## 4. 非功能需求

### NFR-M2-001: 性能 — DOM 树构建
- **描述**: DOM 树构建（10k 节点）**应当**在合理时间内完成
- **测量**: criterion 基准测试
- **优先级**: Should

### NFR-M2-002: 性能 — 查询效率
- **描述**: querySelector 在 1000 元素树中查询**应当**高效
- **测量**: criterion 基准测试
- **优先级**: Should

### NFR-M2-003: 代码质量
- **描述**: `dom` crate **必须**满足项目质量门禁
- **测量**: `cargo build` + `cargo clippy` 零警告，`cargo test` 全通过
- **优先级**: Must

### NFR-M2-004: 测试覆盖率
- **描述**: `dom` crate 行覆盖率**必须** ≥ 70%
- **测量**: `cargo-llvm-cov` 通过 `scripts/check-coverage.sh`
- **优先级**: Must

---

## 5. 接口需求

### IF-M2-001: html5ever TreeSink 实现
- **类型**: Trait 实现
- **规范**: `Document` 实现 `html5ever::TreeSink`
  - `Handle = NodeId`
  - `Output = Document`
  - `ElemName<'a> = &'a QualName`

### IF-M2-002: 公共 API
- **类型**: Rust lib crate API
- **规范**: 所有 DOM 操作通过 `Document` 方法提供，`NodeId` 作为节点引用

---

## 6. 约束与假设

### 6.1 技术约束
- **C-M2-001**: 使用 `slotmap` crate 实现 NodeId 到 NodeData 的映射（稳定 ID，O(1) 查找）
- **C-M2-002**: html5ever 版本 0.29（已锁定在 workspace Cargo.toml）
- **C-M2-003**: 不引入 MPL 许可证依赖

### 6.2 假设
- **A-M2-001**: html5ever 0.29 的 TreeSink trait 稳定可用 — 待验证
- **A-M2-002**: slotmap 的性能满足 DOM 树操作需求 — 待验证

---

## 7. 技术设计（RFC）

### 7.1 目标架构

```
┌─────────────────────────────────────────────┐
│              Document (入口)                  │
│  ┌─────────────────────────────────────┐     │
│  │         SlotMap<NodeId, NodeData>   │     │
│  │  ┌──────┐ ┌──────┐ ┌──────┐        │     │
│  │  │Elem  │ │Text  │ │Doc   │ ...    │     │
│  │  └──────┘ └──────┘ └──────┘        │     │
│  └─────────────────────────────────────┘     │
│  ┌─────────────────────────────────────┐     │
│  │    IdMap (HashMap<String, NodeId>)  │     │
│  └─────────────────────────────────────┘     │
│  ┌─────────────────────────────────────┐     │
│  │   MutationObserver 注册表            │     │
│  └─────────────────────────────────────┘     │
└─────────────────────────────────────────────┘
```

### 7.2 核心数据结构

```rust
/// 节点 ID（slotmap 键，稳定且唯一）
pub struct NodeId(slotmap::DefaultKey);

/// DOM 文档
pub struct Document {
    nodes: SlotMap<NodeId, NodeData>,
    root: NodeId,
    quirks_mode: QuirksMode,
    id_map: HashMap<String, NodeId>,
    observers: Vec<Box<dyn MutationCallback>>,
    pending_mutations: Vec<MutationRecord>,
}

/// 节点数据
pub struct NodeData {
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
}

/// 节点类型枚举
pub enum NodeKind {
    Document(DocumentData),
    Element(ElementData),
    Text(TextData),
    Comment(CommentData),
    DocumentType(DocumentTypeData),
    DocumentFragment,
}

pub struct ElementData {
    pub name: QualName,
    pub attributes: Vec<Attribute>,
    pub id: Option<String>,
    pub class_list: Vec<String>,
}

pub struct TextData {
    pub content: String,
}

pub struct CommentData {
    pub content: String,
}

pub struct DocumentTypeData {
    pub name: String,
    pub public_id: Option<String>,
    pub system_id: Option<String>,
}
```

### 7.3 实现方案比较

| 方案 | 描述 | 优势 | 劣势 | 决策 |
|------|------|------|------|------|
| A: slotmap + 索引 | 使用 slotmap 存储，NodeId 为 key | 稳定 ID、O(1) 查找、内存紧凑 | 需要额外维护 sibling 指针 | ✅ 选定 |
| B: Arena 分配 | 使用 typed_arena 或 bumpalo | 分配快、缓存友好 | 无稳定 ID、删除复杂 | ❌ 排除 |
| C: Rc<RefCell<Node>> | 使用引用计数智能指针 | 实现简单、树遍历直观 | 运行时开销、循环引用风险 | ❌ 排除 |

**选择理由**: slotmap 提供稳定的 NodeId（删除后不会复用于不同节点），O(1) 查找性能，且与 html5ever TreeSink 的 Handle = NodeId 设计完美契合。

### 7.4 实施计划

1. 添加 slotmap 依赖到 workspace
2. 实现核心数据结构（NodeId、NodeData、Document）
3. 实现 DOM 树操作 API
4. 实现 html5ever TreeSink trait
5. 实现属性操作和查询 API
6. 实现 MutationObserver 框架
7. 编写单元测试（≥50 个）
8. 编写基准测试（≥3 个）
9. 验证编译、测试、覆盖率

### 7.5 测试策略

- **单元测试**: 每个模块独立测试，覆盖正常路径、边界条件、错误恢复
- **基准测试**: DOM 树构建、查询、批量操作
- **集成测试**: html5ever 解析完整 HTML 文档

### 7.6 回滚计划

dom crate 是新增模块，不影响其他已有 crate。如有问题可直接回退到占位状态。

---

## 8. TBD 清单

| ID | 项目 | 优先级 | 缺失信息 | 后续步骤 |
|----|------|--------|----------|----------|
| TBD-M2-1 | querySelector 完整选择器语法支持范围 | 重要 | 首期仅支持基础选择器，复杂选择器在 M3 CSS 解析器就绪后扩展 | M3 时确定 |

---

## 9. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-05-30 | 初始版本 — M2 里程碑 Spec + RFC |

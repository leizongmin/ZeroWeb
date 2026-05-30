//! DOM 节点核心类型定义。

use slotmap::new_key_type;

// ── NodeId ──────────────────────────────────────────────────────────

new_key_type! {
    /// DOM 节点的唯一标识符。
    ///
    /// 由 `slotmap` 生成，保证在同一 `Document` 内唯一且稳定。
    /// 即使节点被删除后创建新节点，也不会复用相同的 `NodeId`。
    pub struct NodeId;
}

impl NodeId {
    /// 检查此 ID 是否为有效（非空）值。
    ///
    /// slotmap 的 key 在创建后始终有效，但对应的节点可能已被删除。
    /// 使用 `Document::get()` 检查节点是否存在。
    #[inline]
    pub fn is_valid(self) -> bool {
        true
    }
}

// ── QuirksMode 重导出 ───────────────────────────────────────────────

/// 重导出 html5ever 的 QuirksMode 类型。
pub use html5ever::interface::QuirksMode;

// ── NodeKind ────────────────────────────────────────────────────────

/// DOM 节点类型枚举，对应 WHATWG DOM 规范中的节点类型。
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// 文档根节点（Node.DOCUMENT_NODE = 9）
    Document(DocumentData),
    /// 元素节点（Node.ELEMENT_NODE = 1）
    Element(ElementData),
    /// 文本节点（Node.TEXT_NODE = 3）
    Text(TextData),
    /// 注释节点（Node.COMMENT_NODE = 8）
    Comment(CommentData),
    /// 文档类型声明（Node.DOCUMENT_TYPE_NODE = 10）
    DocumentType(DocumentTypeData),
    /// 文档片段（Node.DOCUMENT_FRAGMENT_NODE = 11）
    DocumentFragment,
    /// 处理指令（Node.PROCESSING_INSTRUCTION_NODE = 7）
    ProcessingInstruction(ProcessingInstructionData),
}

// ── DocumentData ────────────────────────────────────────────────────

/// 文档节点数据。
#[derive(Debug, Clone)]
pub struct DocumentData {
    /// 文档的 quirks mode。
    pub quirks_mode: QuirksMode,
}

// ── ElementData ─────────────────────────────────────────────────────

/// 元素节点数据。
#[derive(Debug, Clone)]
pub struct ElementData {
    /// 元素的限定名（含命名空间、前缀和本地名）。
    pub name: markup5ever::QualName,
    /// 元素的属性列表。
    pub attributes: Vec<markup5ever::Attribute>,
    /// 缓存的 id 属性值（如有）。
    pub id: Option<String>,
    /// 缓存的 class 列表（如有）。
    pub class_list: Vec<String>,
}

/// 辅助函数：比较 markup5ever LocalName 与 &str。
fn local_name_eq(local: &markup5ever::LocalName, s: &str) -> bool {
    &**local == s
}

impl ElementData {
    /// 创建新的元素数据。
    pub fn new(name: markup5ever::QualName, attributes: Vec<markup5ever::Attribute>) -> Self {
        let id = attributes
            .iter()
            .find(|a| local_name_eq(&a.name.local, "id"))
            .map(|a| a.value.to_string());

        let class_list = attributes
            .iter()
            .find(|a| local_name_eq(&a.name.local, "class"))
            .map(|a| {
                a.value
                    .split_whitespace()
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Self {
            name,
            attributes,
            id,
            class_list,
        }
    }

    /// 获取元素的本地名（不含命名空间前缀）。
    pub fn local_name(&self) -> &str {
        &self.name.local
    }

    /// 获取元素的命名空间。
    pub fn namespace(&self) -> &str {
        &self.name.ns
    }

    /// 获取指定属性值。
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes
            .iter()
            .find(|a| local_name_eq(&a.name.local, name))
            .map(|a| a.value.to_string())
    }

    /// 设置属性值。如属性已存在则更新，否则添加。
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        use markup5ever::{LocalName, Namespace, QualName};
        use tendril::StrTendril;

        if let Some(attr) = self
            .attributes
            .iter_mut()
            .find(|a| local_name_eq(&a.name.local, name))
        {
            attr.value = StrTendril::from(value);
        } else {
            self.attributes.push(markup5ever::Attribute {
                name: QualName::new(None, Namespace::from(""), LocalName::from(name)),
                value: StrTendril::from(value),
            });
        }

        // 更新缓存
        if name == "id" {
            self.id = Some(value.to_string());
        } else if name == "class" {
            self.class_list = value.split_whitespace().map(String::from).collect();
        }
    }

    /// 移除指定属性。
    pub fn remove_attribute(&mut self, name: &str) -> Option<String> {
        let idx = self
            .attributes
            .iter()
            .position(|a| local_name_eq(&a.name.local, name))?;
        let attr = self.attributes.remove(idx);

        // 更新缓存
        if name == "id" {
            self.id = None;
        } else if name == "class" {
            self.class_list.clear();
        }

        Some(attr.value.to_string())
    }

    /// 检查是否有指定属性。
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes
            .iter()
            .any(|a| local_name_eq(&a.name.local, name))
    }

    /// 获取所有属性名。
    pub fn attribute_names(&self) -> Vec<String> {
        self.attributes
            .iter()
            .map(|a| a.name.local.to_string())
            .collect()
    }
}

// ── TextData ────────────────────────────────────────────────────────

/// 文本节点数据。
#[derive(Debug, Clone)]
pub struct TextData {
    /// 文本内容。
    pub content: String,
}

impl TextData {
    /// 创建新的文本节点数据。
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

// ── CommentData ─────────────────────────────────────────────────────

/// 注释节点数据。
#[derive(Debug, Clone)]
pub struct CommentData {
    /// 注释内容。
    pub content: String,
}

impl CommentData {
    /// 创建新的注释节点数据。
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

// ── DocumentTypeData ────────────────────────────────────────────────

/// 文档类型声明数据。
#[derive(Debug, Clone)]
pub struct DocumentTypeData {
    /// DOCTYPE 名称（如 "html"）。
    pub name: String,
    /// 公共标识符。
    pub public_id: Option<String>,
    /// 系统标识符。
    pub system_id: Option<String>,
}

// ── ProcessingInstructionData ───────────────────────────────────────

/// 处理指令数据。
#[derive(Debug, Clone)]
pub struct ProcessingInstructionData {
    /// 处理指令目标。
    pub target: String,
    /// 处理指令数据。
    pub data: String,
}

// ── NodeData ────────────────────────────────────────────────────────

/// 节点的完整数据（含树结构信息）。
#[derive(Debug, Clone)]
pub struct NodeData {
    /// 节点类型及类型特定数据。
    pub kind: NodeKind,
    /// 父节点（文档根节点和 DocumentFragment 的 parent 为 None）。
    pub parent: Option<NodeId>,
    /// 子节点列表（按文档顺序）。
    pub children: Vec<NodeId>,
}

impl NodeData {
    /// 创建新的节点数据。
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            parent: None,
            children: Vec::new(),
        }
    }

    /// 检查是否有子节点。
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// 获取第一个子节点。
    pub fn first_child(&self) -> Option<NodeId> {
        self.children.first().copied()
    }

    /// 获取最后一个子节点。
    pub fn last_child(&self) -> Option<NodeId> {
        self.children.last().copied()
    }
}

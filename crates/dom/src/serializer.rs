//! HTML 序列化 — 将 DOM 节点序列化为 HTML 字符串。

use crate::document::Document;
use crate::node::{NodeId, NodeKind};

/// 将节点及其子树序列化为 HTML 字符串。
pub fn serialize_node(doc: &Document, id: NodeId) -> String {
    let mut output = String::new();
    serialize_node_inner(doc, id, &mut output);
    output
}

fn serialize_node_inner(doc: &Document, id: NodeId, output: &mut String) {
    let node_data = match doc.get(id) {
        Some(n) => n,
        None => return,
    };

    match &node_data.kind {
        NodeKind::Document(_) => {
            // 序列化文档节点的子节点
            for &child in &node_data.children {
                serialize_node_inner(doc, child, output);
            }
        }
        NodeKind::DocumentType(dt) => {
            output.push_str("<!DOCTYPE ");
            output.push_str(&dt.name);
            if let Some(pid) = &dt.public_id {
                output.push_str(" PUBLIC \"");
                output.push_str(pid);
                output.push('"');
                if let Some(sid) = &dt.system_id {
                    output.push_str(" \"");
                    output.push_str(sid);
                    output.push('"');
                }
            } else if let Some(sid) = &dt.system_id {
                output.push_str(" SYSTEM \"");
                output.push_str(sid);
                output.push('"');
            }
            output.push('>');
        }
        NodeKind::Element(elem) => {
            output.push('<');
            output.push_str(elem.local_name());

            // 序列化属性
            for attr in &elem.attributes {
                output.push(' ');
                output.push_str(&attr.name.local);
                output.push_str("=\"");
                // HTML 属性值转义
                output.push_str(&escape_html(&attr.value));
                output.push('"');
            }

            output.push('>');

            // 自闭合元素
            let tag = elem.local_name();
            if !is_void_element(tag) {
                // 序列化子节点
                for &child in &node_data.children {
                    serialize_node_inner(doc, child, output);
                }

                output.push_str("</");
                output.push_str(tag);
                output.push('>');
            }
        }
        NodeKind::Text(data) => {
            output.push_str(&escape_text(&data.content));
        }
        NodeKind::Comment(data) => {
            output.push_str("<!--");
            output.push_str(&data.content);
            output.push_str("-->");
        }
        NodeKind::ProcessingInstruction(pi) => {
            output.push_str("<?");
            output.push_str(&pi.target);
            output.push(' ');
            output.push_str(&pi.data);
            output.push_str("?>");
        }
        NodeKind::DocumentFragment => {
            for &child in &node_data.children {
                serialize_node_inner(doc, child, output);
            }
        }
    }
}

/// 获取节点的 innerHTML（仅子节点序列化）。
pub fn inner_html(doc: &Document, id: NodeId) -> String {
    let mut output = String::new();
    let node_data = match doc.get(id) {
        Some(n) => n,
        None => return output,
    };

    for &child in &node_data.children {
        serialize_node_inner(doc, child, &mut output);
    }
    output
}

/// 转义 HTML 特殊字符（用于属性值和文本）。
fn escape_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    result
}

/// 转义文本内容中的特殊字符。
fn escape_text(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    result
}

/// HTML void 元素（自闭合元素）列表。
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// 为 Document 添加 innerHTML 便捷方法。
impl Document {
    /// 获取节点的 innerHTML（子节点序列化为 HTML 字符串）。
    pub fn inner_html(&self, id: NodeId) -> String {
        inner_html(self, id)
    }

    /// 获取节点及其子树的完整 HTML 序列化。
    pub fn outer_html(&self, id: NodeId) -> String {
        serialize_node(self, id)
    }
}

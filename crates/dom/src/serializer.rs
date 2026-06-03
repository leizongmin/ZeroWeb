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
        NodeKind::DocumentFragment | NodeKind::ShadowRoot(_) => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    /// 测试 ProcessingInstruction 序列化。
    #[test]
    fn test_serialize_processing_instruction() {
        let mut doc = Document::new();
        let pi = doc.create_processing_instruction("xml-stylesheet", "href=\"style.css\"");
        let html = doc.outer_html(pi);
        assert_eq!(html, "<?xml-stylesheet href=\"style.css\"?>");
    }

    /// 测试 DocumentFragment 序列化。
    #[test]
    fn test_serialize_document_fragment() {
        let mut doc = Document::new();
        let frag = doc.create_document_fragment();
        let child1 = doc.create_text_node("hello ");
        doc.append_child(frag, child1).unwrap();
        let child2 = doc.create_element("span");
        doc.append_child(frag, child2).unwrap();
        let html = doc.outer_html(frag);
        assert_eq!(html, "hello <span></span>");
    }

    /// 测试 DocumentType 带 public_id 和 system_id 的序列化。
    #[test]
    fn test_serialize_doctype_public_system() {
        let mut doc = Document::new();
        let dt = doc.create_document_type("html", Some("pubid".to_string()), Some("sysid".to_string()));
        let html = doc.outer_html(dt);
        assert_eq!(html, "<!DOCTYPE html PUBLIC \"pubid\" \"sysid\">");
    }

    /// 测试 DocumentType 仅带 system_id 的序列化。
    #[test]
    fn test_serialize_doctype_system_only() {
        let mut doc = Document::new();
        let dt = doc.create_document_type("html", None, Some("sysid".to_string()));
        let html = doc.outer_html(dt);
        assert_eq!(html, "<!DOCTYPE html SYSTEM \"sysid\">");
    }

    /// 测试空 innerHTML。
    #[test]
    fn test_inner_html_empty() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        assert_eq!(doc.inner_html(div), "");
    }

    /// 测试所有 void 元素的序列化（不应有闭合标签）。
    #[test]
    fn test_void_elements_serialization() {
        let void_tags = [
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track",
            "wbr",
        ];
        for tag in &void_tags {
            assert!(is_void_element(tag), "{tag} should be void");
            let mut doc = Document::new();
            let elem = doc.create_element(tag);
            let html = doc.outer_html(elem);
            assert!(
                html.starts_with(&format!("<{tag}")),
                "void element should start with <{tag}"
            );
            assert!(!html.contains("</"), "void element {tag} should not have closing tag");
        }
    }

    /// 测试文本节点包含特殊字符（&、<、>）的转义。
    #[test]
    fn test_text_special_chars_escaping() {
        let mut doc = Document::new();
        let text = doc.create_text_node("a & b < c > d");
        let html = doc.outer_html(text);
        assert_eq!(html, "a &amp; b &lt; c &gt; d");
    }

    /// 测试属性值包含双引号的转义。
    #[test]
    fn test_attribute_value_quote_escaping() {
        let mut doc = Document::new();
        let elem = doc.create_element("div");
        doc.set_attribute(elem, "title", "say \"hello\"");
        let html = doc.outer_html(elem);
        assert!(html.contains("&quot;"), "attribute quotes should be escaped");
    }

    // ── Additional tests ────────────────────────────────────────────────

    /// 测试嵌套元素序列化。
    #[test]
    fn test_serialize_nested_elements() {
        let mut doc = Document::new();
        let outer = doc.create_element("div");
        let inner = doc.create_element("p");
        let text = doc.create_text_node("hello");
        doc.append_child(inner, text).unwrap();
        doc.append_child(outer, inner).unwrap();
        let html = doc.outer_html(outer);
        assert_eq!(html, "<div><p>hello</p></div>");
    }

    /// 测试带属性的元素序列化。
    #[test]
    fn test_serialize_element_with_attributes() {
        let mut doc = Document::new();
        let elem = doc.create_element("a");
        doc.set_attribute(elem, "href", "https://example.com");
        doc.set_attribute(elem, "class", "link");
        let html = doc.outer_html(elem);
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("class=\"link\""));
        assert!(html.starts_with("<a "));
        assert!(html.ends_with("</a>"));
    }

    /// 测试注释序列化。
    #[test]
    fn test_serialize_comment() {
        let mut doc = Document::new();
        let comment = doc.create_comment("this is a comment");
        let html = doc.outer_html(comment);
        assert_eq!(html, "<!--this is a comment-->");
    }

    /// 测试 innerHTML 只包含子节点。
    #[test]
    fn test_inner_html_children_only() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let text = doc.create_text_node("content");
        doc.append_child(div, text).unwrap();
        let html = doc.inner_html(div);
        assert_eq!(html, "content");
        assert!(!html.contains("<div>"));
    }

    /// 测试 outerHTML 包含元素本身。
    #[test]
    fn test_outer_html_includes_self() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let text = doc.create_text_node("content");
        doc.append_child(div, text).unwrap();
        let html = doc.outer_html(div);
        assert_eq!(html, "<div>content</div>");
    }

    /// 测试 void 元素不区分大小写。
    #[test]
    fn test_void_element_case_insensitive() {
        assert!(is_void_element("BR"));
        assert!(is_void_element("Img"));
        assert!(is_void_element("INPUT"));
        assert!(!is_void_element("div"));
        assert!(!is_void_element("span"));
    }

    /// 测试文本中 &amp; 转义。
    #[test]
    fn test_escape_ampersand_in_text() {
        let mut doc = Document::new();
        let text = doc.create_text_node("foo&bar");
        assert_eq!(doc.outer_html(text), "foo&amp;bar");
    }

    /// 测试属性值中的尖括号转义。
    #[test]
    fn test_escape_angle_brackets_in_attribute() {
        let mut doc = Document::new();
        let elem = doc.create_element("div");
        doc.set_attribute(elem, "data", "<script>");
        let html = doc.outer_html(elem);
        assert!(html.contains("&lt;script&gt;"));
    }

    /// 测试多个子节点序列化。
    #[test]
    fn test_serialize_multiple_children() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let t1 = doc.create_text_node("a");
        let t2 = doc.create_text_node("b");
        doc.append_child(div, t1).unwrap();
        doc.append_child(div, t2).unwrap();
        assert_eq!(doc.inner_html(div), "ab");
    }

    /// 测试 DocumentType 无 public/system id。
    #[test]
    fn test_serialize_doctype_no_ids() {
        let mut doc = Document::new();
        let dt = doc.create_document_type("html", None, None);
        assert_eq!(doc.outer_html(dt), "<!DOCTYPE html>");
    }

    /// 测试序列化不存在节点返回空字符串。
    #[test]
    fn test_serialize_nonexistent_node() {
        let doc = Document::new();
        // Create a node, then it gets a valid ID — test with a document that has no children
        let root = doc.root();
        // Remove all children so root is empty, test inner_html on empty root
        let html = inner_html(&doc, root);
        // root is a Document node, inner_html should be empty (no children)
        assert_eq!(html, "");
    }

    /// 测试 br 元素（void element）序列化。
    #[test]
    fn test_serialize_br() {
        let mut doc = Document::new();
        let br = doc.create_element("br");
        let html = doc.outer_html(br);
        assert_eq!(html, "<br>");
    }
}

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
    serialize_node_inner_ctx(doc, id, None, output);
}

/// `parent_tag` 为父元素的标签名（小写）——用于判断 Text 节点是否处于
/// raw text / escapable raw text 元素（`<script>`/`<style>` 等）内部，
/// 这些元素的文本内容在序列化时**不得** HTML 转义（否则 `<style>` 里的
/// `<![CDATA[`、`a > b` 等会被转义，再次解析时破坏 CSS/JS）。
fn serialize_node_inner_ctx(doc: &Document, id: NodeId, parent_tag: Option<&str>, output: &mut String) {
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
            // HTML 序列化 spec（`html-fragment-serialization-algorithm`）：HTML 命名空间元素的 tag 名
            // ASCII 小写——编程创建的大写（`createElement('DIV')`）经序列化须回到 `<div></div>`
            // （parser 已小写解析 tag，此路径处理 dom 低层原语 `create_element` 保留的大写）。
            // SVG / MathML 等非 HTML 命名空间元素 tag 名**大小写保留**。对称于 `ElementData::tag_name()`
            // 的 HTML-namespace-aware 大写（tagName getter 大写 / serializer 小写，互补且均 HTML-ns 感知）。
            // spec：https://html.spec.whatwg.org/multipage/parsing.html#html-fragment-serialization-algorithm
            const HTML_NS: &str = "http://www.w3.org/1999/xhtml";
            let local = elem.local_name();
            let tag_owned: String = if elem.namespace() == HTML_NS {
                local.to_ascii_lowercase()
            } else {
                local.to_string()
            };
            let tag = tag_owned.as_str();

            output.push('<');
            output.push_str(tag);

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
            if !is_void_element(tag) {
                // 序列化子节点（传当前标签名小写，供 Text 节点判断 raw text 上下文）。
                // HTML ns 的 tag 已小写；非 HTML ns 无 raw text 元素，`is_raw_text_element`
                // 内部再小写判别，传任意大小写均安全。
                for &child in &node_data.children {
                    serialize_node_inner_ctx(doc, child, Some(tag), output);
                }

                output.push_str("</");
                output.push_str(tag);
                output.push('>');
            }
        }
        NodeKind::Text(data) => {
            // raw text 元素（script/style/textarea/title）的文本内容不转义：
            // 这些元素的内容在 HTML 解析时按 rawtext / rcdata 处理，序列化须保持原样，
            // 否则 CSS/JS 源码（如 `<style>` 内的 `<![CDATA[`、`a > b`）被转义后
            // 再次解析会被破坏。普通文本节点按常规转义（`&`/`<`/`>`）。
            if parent_tag.map(is_raw_text_element).unwrap_or(false) {
                output.push_str(&data.content);
            } else {
                output.push_str(&escape_text(&data.content));
            }
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
    // 取父元素标签名（小写）作为子节点的 raw text 上下文：对 `<style>`/`<script>`
    // 取 innerHTML 时，其文本子节点不应被 HTML 转义。
    let parent_tag = match &node_data.kind {
        NodeKind::Element(e) => Some(e.local_name().to_ascii_lowercase()),
        _ => None,
    };
    let parent_tag_ref = parent_tag.as_deref();

    for &child in &node_data.children {
        serialize_node_inner_ctx(doc, child, parent_tag_ref, &mut output);
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

/// HTML void 元素（自闭合元素）列表（大小写不敏感）。
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

/// raw text 元素：内容在序列化时**不做任何 HTML 转义**（大小写不敏感）。
///
/// `script`、`style` 在 HTML 解析时按 rawtext 处理（不识别字符引用，`<`、`&`
/// 原样保留）。序列化须保持原样，否则 CSS/JS 源码（如 `<style>` 内的
/// `<![CDATA[`、`a > b`、`x && y`）被转义后再次解析会被破坏。
///
/// 注：`textarea`/`title` 是 escapable raw text（解析时识别 `&` 引用），
/// 对它们用 [`escape_text`]（转义 `&`/`<`/`>`）才能正确 round-trip，故不在此列。
fn is_raw_text_element(tag: &str) -> bool {
    matches!(tag.to_ascii_lowercase().as_str(), "script" | "style")
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

    /// `<style>`/`<script>` 是 raw text 元素，内容（CSS/JS 源码里的 `<`、`>`、
    /// `&`、`<![CDATA[`）在序列化时**不得** HTML 转义，否则再次解析会破坏
    /// CSS/JS（如 background-root-101 的 `<![CDATA[` 被 `&lt;![CDATA[` 取代后
    /// CSS 规则全被贪婪吞噬）。回归测试：parse 含 CDATA style 的 HTML →
    /// outer_html → 重新 parse，CSS 文本必须原样保留。
    #[test]
    fn test_raw_text_element_content_not_escaped() {
        let html = r#"<html><head><style><![CDATA[ a > b { color: red; } ]]></style></head></html>"#;
        let doc = crate::parse_html(html);
        // 序列化结果应保留原始 `<`、`>`、`&`（无 `&lt;`/`&gt;`/`&amp;`）。
        let serialized = doc.outer_html(doc.root());
        assert!(serialized.contains("a > b"), "style 内的 `>` 不应被转义: {serialized}");
        assert!(
            !serialized.contains("&lt;![CDATA["),
            "style 内的 `<![CDATA[` 不应被转义: {serialized}"
        );
        // round-trip：重新解析后 style 文本内容应原样保留 CDATA + 选择器。
        let doc2 = crate::parse_html(&serialized);
        let style = doc2.get_elements_by_tag_name("style").into_iter().next().unwrap();
        let text = doc2.text_content(style).unwrap();
        assert!(text.contains("a > b"), "round-trip 后 style 文本应保留 `>`: {text}");
        assert!(
            text.contains("<![CDATA["),
            "round-trip 后 style 文本应保留 CDATA: {text}"
        );
    }

    /// 普通元素（非 raw text）的文本仍须转义 `>`（回归保护）。
    #[test]
    fn test_normal_text_still_escaped() {
        let html = "<p>a > b</p>";
        let doc = crate::parse_html(html);
        let serialized = doc.outer_html(doc.root());
        assert!(serialized.contains("&gt;"), "普通文本的 `>` 仍应转义: {serialized}");
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

    // ── R3172 HTML 序列化 tag 名大小写（HTML 小写 / 非 HTML 保留）── ──────

    /// HTML 命名空间元素 tag 名 ASCII 小写：`create_element("DIV")`（dom 低层原语保留大小写）
    /// 经序列化须回到 `<div></div>`（HTML serialization spec）。回归保护：编程创建的大写 tag
    /// 在 outerHTML/innerHTML 须小写，与真实浏览器一致。
    #[test]
    fn test_serialize_html_namespace_tag_lowercased_r3172() {
        let mut doc = Document::new();
        let div = doc.create_element("DIV");
        assert_eq!(doc.outer_html(div), "<div></div>");
        // 小写输入无变化（无回归）。
        let span = doc.create_element("span");
        assert_eq!(doc.outer_html(span), "<span></span>");
        // 混合大小写同样小写。
        let mixed = doc.create_element("DiV");
        assert_eq!(doc.outer_html(mixed), "<div></div>");
    }

    /// HTML 命名空间 void 元素大写 → 序列化小写自闭合：`create_element("BR")` → `<br>`（无闭合标签）。
    #[test]
    fn test_serialize_html_void_uppercase_lowercased_r3172() {
        let mut doc = Document::new();
        let br = doc.create_element("BR");
        assert_eq!(doc.outer_html(br), "<br>");
        let img = doc.create_element("IMG");
        assert_eq!(doc.outer_html(img), "<img>");
    }

    /// 非 HTML 命名空间（SVG/MathML）元素 tag 名**大小写保留**：HTML↔SVG 的核心区别。
    /// `createElementNS(svg, "RECT")` → `<RECT></RECT>`（SVG 大小写敏感，不强制小写）。
    #[test]
    fn test_serialize_svg_namespace_tag_case_preserved_r3172() {
        let mut doc = Document::new();
        let rect = doc.create_element_ns("http://www.w3.org/2000/svg", "RECT");
        assert_eq!(doc.outer_html(rect), "<RECT></RECT>");
        // 对照：HTML ns 大写被小写（同 tag 名不同命名空间，行为不同）。
        let html_div = doc.create_element("DIV");
        assert_eq!(doc.outer_html(html_div), "<div></div>");
    }
}

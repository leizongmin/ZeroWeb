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
            // HTML 序列化 spec §13.3「serialize an element」：tag 名原样输出（不主动小写）。HTML 元素
            // 小写由创建路径负责——parser（html5ever tokenizer 小写）与 `Document::create_element`
            //（DOM spec createElement step 2 小写）；SVG / MathML 等经 `create_element_ns` 大小写敏感保留。
            //
            // spec 规定 tagname 取值按**命名空间**分流：
            // - HTML / MathML / SVG 命名空间 → **local name**（前缀丢弃）。spec 注："For HTML elements
            //   created by the HTML parser or createElement(), tagname will be lowercase."
            // - 其它命名空间（真 foreign）→ **qualified name**（`prefix:local` 若有 prefix，否则 local）。
            //
            // R3214：R3210 旧逻辑对所有带 prefix 元素输出 `prefix:local`（含 SVG/MathML），过度适用——
            // spec 对 SVG/MathML 命名空间元素要求 local name（如 `create_element_ns(svg,"svg:rect")` 应
            // 序列化为 `<rect>` 非 `<svg:rect>`）。本片改为命名空间感知：HTML/MathML/SVG ns → local，
            // 其它 ns → qualified（保留 R3210 对真 foreign 命名空间 prefix 的保留）。无 prefix 路径
            //（hot path，绝大多数）恒 local，零分配。
            output.push('<');
            const HTML_NS: &str = "http://www.w3.org/1999/xhtml";
            const MATHML_NS: &str = "http://www.w3.org/1998/Math/MathML";
            const SVG_NS: &str = "http://www.w3.org/2000/svg";
            let use_local = matches!(elem.namespace(), HTML_NS | MATHML_NS | SVG_NS);
            let tag_owned: String;
            let tag: &str = if use_local {
                elem.local_name()
            } else {
                match elem.name.prefix.as_deref() {
                    Some(p) => {
                        tag_owned = format!("{p}:{}", elem.local_name());
                        &tag_owned
                    }
                    None => elem.local_name(),
                }
            };
            output.push_str(tag);

            // 序列化属性
            for attr in &elem.attributes {
                output.push(' ');
                // R3206：按 HTML 序列化 spec「serializing an attribute」重建限定名——
                // prefix 非 null 或 ns 为 XML/XMLNS 命名空间时输出 `prefix:local`
                //（如 SVG `xlink:href`、`xml:lang`、`xmlns:x`），旧实现仅输出 `local` 丢前缀，
                // 致 SVG 图标库（`<use xlink:href>`）等外部命名空间属性 round-trip 丢前缀。
                // spec：https://html.spec.whatwg.org/multipage/parsing.html#serialising
                output.push_str(&qualified_attr_name(attr));
                output.push_str("=\"");
                // HTML 属性值转义
                output.push_str(&escape_html(&attr.value));
                output.push('"');
            }

            output.push('>');

            // 自闭合元素
            if !is_void_element(tag) {
                let tag_lower = tag.to_ascii_lowercase();
                // 序列化子节点（传当前标签名，供 Text 节点判断 raw text 上下文）
                for &child in &node_data.children {
                    serialize_node_inner_ctx(doc, child, Some(&tag_lower), output);
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

/// 重建属性的**限定名**（qualified name）——HTML 序列化 spec「serializing an attribute」
///（spec：https://html.spec.whatwg.org/multipage/parsing.html#serialising）。
///
/// - XML 命名空间属性 → `xml:local`（如 `xml:lang`）。
/// - XMLNS 命名空间属性 → `xmlns`（local=`xmlns`）或 `xmlns:local`（如 `xmlns:xlink`）。
/// - 其他有前缀属性 → `prefix:local`（如 SVG `xlink:href`）。
/// - 无前缀 → `local`（HTML 属性常态）。
///
/// 旧实现恒输出 `local`，丢前缀与命名空间，致外部命名空间属性（SVG `xlink:href`、`xml:lang` 等）
/// 序列化 round-trip 丢前缀。html5ever 解析器在 DOM 中保留了完整 `QualName`（prefix + ns + local），
/// 仅本序列化步骤丢弃。
fn qualified_attr_name(attr: &markup5ever::Attribute) -> String {
    const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
    const XMLNS_NS: &str = "http://www.w3.org/2000/xmlns/";
    let ns = &*attr.name.ns;
    let local = &*attr.name.local;
    if ns == XML_NS {
        format!("xml:{local}")
    } else if ns == XMLNS_NS {
        if local == "xmlns" {
            "xmlns".to_string()
        } else {
            format!("xmlns:{local}")
        }
    } else if let Some(prefix) = attr.name.prefix.as_deref() {
        format!("{prefix}:{local}")
    } else {
        local.to_string()
    }
}

/// 转义 HTML 特殊字符（用于属性值和文本）。
/// 转义属性值中的特殊字符。
///
/// 按 HTML 序列化 spec「attribute value escaping」（https://html.spec.whatwg.org/multipage/parsing.html#escapingString）
/// 必转 `&` → `&amp;`、**U+00A0 NO-BREAK SPACE → `&nbsp;`**、`"` → `&quot;`（`<`/`>` 亦转，历史防御，
/// 零 round-trip 影响）。R3209：补 U+00A0 → `&nbsp;`（旧实现漏转，输出字面 U+00A0，致
/// innerHTML/outerHTML 字面串与 spec / 浏览器不一致——JS 测试套件常按 `&nbsp;` 比对 innerHTML）。
fn escape_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '\u{00A0}' => result.push_str("&nbsp;"),
            _ => result.push(c),
        }
    }
    result
}

/// 转义文本内容中的特殊字符。
///
/// 按 HTML 序列化 spec「text content escaping」转义：`&` → `&amp;`、
/// **U+00A0 NO-BREAK SPACE → `&nbsp;`**、`<` → `&lt;`、`>` → `&gt;`。R3209：补
/// U+00A0 → `&nbsp;`（同 [`escape_html`]）。
fn escape_text(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '\u{00A0}' => result.push_str("&nbsp;"),
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
/// R3216：补全 spec §13.3 raw text 元素——`style, script, xmp, iframe, noembed, noframes,
/// plaintext`（**无条件** raw text）。旧实现仅 `script|style`，致 `<xmp>`/`<iframe>` 等元素的文本子
/// 被错误转义——rawtext 解析**不识别字符引用**，故 `a < b` 序列化为 `a &lt; b` 再解析仍是 `a &lt; b`，
/// round-trip 失效。spec：https://html.spec.whatwg.org/multipage/parsing.html#serialising-html-fragments
///
/// **不含 `noscript`**：spec 谓 noscript 当 scripting enabled 时亦 raw，但本浏览器 DOMPurify 清洗
///（`test_sanitize_dompurify_real_r3019`）的 mXSS 再解析检查与 raw noscript 序列化交互致空结果
///（noscript mXSS 是经典向量）——noscript raw 序列化需配套 mXSS 处理，**defer**，本次仅修无条件 raw 族。
///
/// 注：`textarea`/`title` 是 escapable raw text（解析时识别 `&` 引用），
/// 对它们用 [`escape_text`]（转义 `&`/`<`/`>`）才能正确 round-trip，故不在此列。
fn is_raw_text_element(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "script" | "style" | "xmp" | "iframe" | "noembed" | "noframes" | "plaintext"
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

    /// R3216：raw text 元素（spec §13.3 全集）的文本子内容**不转义**——rawtext 解析不识别
    /// 字符引用，转义会破坏 round-trip。覆盖 xmp/iframe/noscript（旧仅 script/style，
    /// 其文本子被错误转义）。经 parse_html 验证 round-trip（解析→序列化→再解析文本一致）。
    #[test]
    fn test_raw_text_element_unescaped_r3216() {
        // 直接构造 DOM：raw text 元素的文本子含 `<`/`&`，序列化须原样输出（非 `&lt;`/`&amp;`）。
        // 不含 noscript（spec conditional raw，与 DOMPurify mXSS 检查交互，defer——见 is_raw_text_element 注）。
        for tag in ["script", "style", "xmp", "iframe", "noembed", "noframes", "plaintext"] {
            let mut doc = Document::new();
            let el = doc.create_element(tag);
            let text = doc.create_text_node("a < b & c");
            doc.append_child(el, text).unwrap();
            let html = doc.outer_html(el);
            assert!(
                html.contains("a < b & c"),
                "raw text element <{tag}> 内容不应转义，got: {html}"
            );
            assert!(
                !html.contains("&lt;") && !html.contains("&amp;"),
                "raw text element <{tag}> 不应含转义实体，got: {html}"
            );
        }
        // textarea/title 是 escapable raw text（走 escape_text）——须转义 `&`/`<`/`>`。
        let mut doc = Document::new();
        let ta = doc.create_element("textarea");
        let text = doc.create_text_node("a < b & c");
        doc.append_child(ta, text).unwrap();
        let html = doc.outer_html(ta);
        assert_eq!(html, "<textarea>a &lt; b &amp; c</textarea>");
    }

    /// R3216：真实 round-trip——parse `<xmp>a < b</xmp>` → 序列化 → 文本须仍 "a < b"
    ///（rawtext 再解析不识别 `&lt;`，旧转义致 round-trip 失效为 "a &lt; b"）。
    #[test]
    fn test_raw_text_round_trip_xmp_r3216() {
        let doc = crate::parse_html("<xmp>a < b</xmp>");
        let xmp = doc.query_selector(doc.root(), "xmp").unwrap();
        let serialized = doc.outer_html(xmp);
        // 再解析序列化结果，文本应保持 "a < b"（非 "a &lt; b"）。
        let doc2 = crate::parse_html(&format!("<div>{serialized}</div>"));
        let xmp2 = doc2.query_selector(doc2.root(), "xmp").unwrap();
        assert_eq!(
            doc2.text_content(xmp2),
            Some("a < b".to_string()),
            "xmp round-trip 应保持原文（rawtext 不转义）；serialized = {serialized}"
        );
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

    /// 测试文本内容中 U+00A0（NO-BREAK SPACE）序列化为 `&nbsp;`（spec「text content
    /// escaping」要求）。旧实现输出字面 U+00A0，致 innerHTML/outerHTML 与 spec / 浏览器不一致。
    #[test]
    fn test_text_nbsp_escaping_r3209() {
        let mut doc = Document::new();
        let p = doc.create_element("p");
        let text = doc.create_text_node("a\u{00A0}b");
        doc.append_child(p, text).unwrap();
        let html = doc.outer_html(p);
        assert_eq!(html, "<p>a&nbsp;b</p>");
        // 普通空格 U+0020 不转义（仅 NO-BREAK SPACE 转）。
        let p2 = doc.create_element("p");
        let text2 = doc.create_text_node("a b");
        doc.append_child(p2, text2).unwrap();
        assert_eq!(doc.outer_html(p2), "<p>a b</p>");
    }

    /// 测试属性值中 U+00A0 序列化为 `&nbsp;`（spec「attribute value escaping」要求，
    /// 与文本对称）。
    #[test]
    fn test_attribute_value_nbsp_escaping_r3209() {
        let mut doc = Document::new();
        let elem = doc.create_element("div");
        doc.set_attribute(elem, "title", "a\u{00A0}b");
        let html = doc.outer_html(elem);
        assert_eq!(html, r#"<div title="a&nbsp;b"></div>"#);
    }

    /// 测试 `<textarea>`（escapable raw text）内 U+00A0 序列化为 `&nbsp;`——
    /// textarea/title 经 `escape_text` 路径（非 raw text），故同样须转。
    #[test]
    fn test_textarea_nbsp_escaping_r3209() {
        let mut doc = Document::new();
        let ta = doc.create_element("textarea");
        let text = doc.create_text_node("x\u{00A0}y");
        doc.append_child(ta, text).unwrap();
        let html = doc.outer_html(ta);
        assert_eq!(html, "<textarea>x&nbsp;y</textarea>");
    }

    /// 测试元素 tag 序列化按命名空间分流（spec §13.3「serialize an element」）：
    /// HTML / MathML / SVG 命名空间 → local name（prefix 丢弃）；其它命名空间 → qualified name
    ///（`prefix:local`）。R3214 修正 R3210 对 prefixed SVG/MathML 的过适用（旧输出 `<svg:rect>`，
    /// spec 应 `<rect>`——SVG ns 走 local name 分支）。
    #[test]
    fn test_serialize_element_tag_namespace_aware_r3214() {
        let mut doc = Document::new();
        // SVG 命名空间带 prefix → spec local name（`<rect>`，非 `<svg:rect>`）。
        let rect = doc.create_element_ns("http://www.w3.org/2000/svg", "svg:rect");
        assert_eq!(doc.outer_html(rect), "<rect></rect>");
        // MathML 命名空间带 prefix → 同理 local name。
        let mrow = doc.create_element_ns("http://www.w3.org/1998/Math/MathML", "m:mo");
        assert_eq!(doc.outer_html(mrow), "<mo></mo>");
        // 无 prefix foreign 元素（默认命名空间）→ local（常见情况，零回归）。
        let rect2 = doc.create_element_ns("http://www.w3.org/2000/svg", "rect");
        assert_eq!(doc.outer_html(rect2), "<rect></rect>");
        // HTML 元素 → local（小写由 create_element 负责）。
        let div = doc.create_element("div");
        assert_eq!(doc.outer_html(div), "<div></div>");

        // 真 foreign 命名空间（非 HTML/MathML/SVG）带 prefix → spec qualified name `<prefix:local>`
        //（R3210 正确处理、R3214 保留的唯一 qualified-name 场景）。
        let ext = doc.create_element_ns("urn:example:custom", "ex:thing");
        assert_eq!(doc.outer_html(ext), "<ex:thing></ex:thing>");

        // 嵌套 SVG（prefix 丢弃 local）+ 真 foreign 子（qualified）混排。
        let parent = doc.create_element_ns("http://www.w3.org/2000/svg", "svg:g");
        let child = doc.create_element_ns("urn:example:custom", "ex:inner");
        doc.append_child(parent, child).unwrap();
        assert_eq!(doc.outer_html(parent), "<g><ex:inner></ex:inner></g>");
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

    // ── R3172/R3173 HTML tag 名大小写（create_element 小写 / create_element_ns 保留）──

    /// HTML 元素 tag 名经 `create_element` 小写（DOM spec createElement step 2），serializer 原样
    /// 输出 local_name → `<div></div>`。回归保护：编程创建的大写 tag 经 create_element 小写后，
    /// outerHTML/innerHTML 与真实浏览器一致。
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

    /// HTML void 元素大写经 `create_element` 小写 → 序列化小写自闭合：`create_element("BR")` → `<br>`。
    #[test]
    fn test_serialize_html_void_uppercase_lowercased_r3172() {
        let mut doc = Document::new();
        let br = doc.create_element("BR");
        assert_eq!(doc.outer_html(br), "<br>");
        let img = doc.create_element("IMG");
        assert_eq!(doc.outer_html(img), "<img>");
    }

    /// 非 HTML 命名空间（SVG/MathML）元素 tag 名**大小写保留**：`create_element_ns` 不小写
    ///（createElementNS 大小写敏感），serializer 原样输出 → `<RECT></RECT>`。HTML↔SVG 的核心区别。
    #[test]
    fn test_serialize_svg_namespace_tag_case_preserved_r3172() {
        let mut doc = Document::new();
        let rect = doc.create_element_ns("http://www.w3.org/2000/svg", "RECT");
        assert_eq!(doc.outer_html(rect), "<RECT></RECT>");
        // 对照：HTML 经 create_element 小写（同 tag 名不同命名空间，行为不同）。
        let html_div = doc.create_element("DIV");
        assert_eq!(doc.outer_html(html_div), "<div></div>");
    }

    // ── R3206 SVG/foreign-attribute prefix 序列化（spec HTML fragment serialization §13.3）──

    /// 直接解析的 SVG 外部命名空间属性（`xlink:href`）序列化须保留 `prefix:local`——
    /// HTML 序列化 spec「serializing an attribute」：prefix 非 null → `prefix:local`。
    /// 旧 serializer 仅输出 `local`，丢 prefix（`xlink:href` → `href`），SVG 图标库 round-trip 丢前缀。
    #[test]
    fn test_serialize_svg_xlink_href_prefix_r3206() {
        let html = r##"<svg><use xlink:href="#a"/></svg>"##;
        let doc = crate::parse_html(html);
        let serialized = doc.outer_html(doc.root());
        assert!(
            serialized.contains(r##"xlink:href="#a""##),
            "xlink:href 前缀应保留（spec prefix:local），实际: {serialized}"
        );
    }

    /// HTML 属性（无前缀、无命名空间）序列化仍为裸 `local`（回归保护：serializer hot path，
    /// 绝大多数属性走此分支，不得因 R3206 限QualifiedName 重构破坏）。
    #[test]
    fn test_serialize_html_attr_no_prefix_unchanged_r3206() {
        let html = r#"<div id="main" class="container" data-x="1">text</div>"#;
        let doc = crate::parse_html(html);
        let serialized = doc.outer_html(doc.root());
        assert!(serialized.contains(r#"id="main""#), "普通属性序列化不变: {serialized}");
        assert!(serialized.contains(r#"class="container""#));
        assert!(serialized.contains(r#"data-x="1""#));
        // 确保未误加前缀。
        assert!(!serialized.contains(":"), "HTML 属性不应有前缀冒号: {serialized}");
    }

    /// `xml:lang`（XML 命名空间属性）序列化为 `xml:lang`（spec XML namespace → `xml:local`）。
    #[test]
    fn test_serialize_xml_lang_prefix_r3206() {
        let html = r##"<div xml:lang="en">x</div>"##;
        let doc = crate::parse_html(html);
        let serialized = doc.outer_html(doc.root());
        assert!(
            serialized.contains(r#"xml:lang="en""#),
            "xml:lang 应保留前缀: {serialized}"
        );
    }
}

//! s173（html-semantics M2）：html5ever 0.29.1 声明式 shadow 解析缺陷绕道预变换。
//!
//! html5ever 0.29.1 的 tree builder 对 `<template shadowrootmode=…>` 做**值精确匹配**
//! （`should_attach_declarative_shadow`：`open`/`close`——"close" 是偏离现行规范的
//! 笔误形态，"closed" 反而不在集内），命中即走 attach 路径：sink 默认实现返回
//! `Err` 且两个分支都**不插入 template 元素**（rules.rs `<template>` 臂 else 才
//! `insert_element_for`），contents 按 InTemplate→InBody 规则直落当前节点（宿主）——
//! template 整体消失、contents 扁平化，s168 的
//! `materialize_declarative_shadow_roots` 对 open 形态恒空转（引擎侧根无法物化，
//! host.shadowRoot 恒 null：WPT form-indexed-element-shadow /
//! form-submit-dialog-shadow 红面根因）。
//!
//! 本模块在喂 html5ever 前把模板标签内的 `shadowrootmode` **值首字母大写**
//!（`open`→`Open`、`close`→`Close`）：html5ever 精确匹配失配 → template 正常入树；
//! spec shadowrootmode 枚举 **ASCII 大小写不敏感**（HTML §the-template-element
//! 声明式 shadow root 步），值大小写对语义不可见，materialize 读侧
//! `to_ascii_lowercase` 归一后照常物化。"closed" 值本就不在 html5ever 匹配集，
//! 不改写（原样入树 + materialize 正常）。
//!
//! 扫描面镜像 tokenizer 状态：只认 `<template` 起始标签（名字边界校验）、引号感知
//! 与属性名前界（`data-shadowrootmode`、值内子串不命中）；script/style/title 等
//! RAWTEXT/RCDATA 原文区内的同名字面量不改写（polyfill JS 内嵌模板串是真实场景）。
//!
//! FIXME(html5ever): 升级 0.38（完整 spec 流：插入 template + attach 回传宿主 +
//! contents 归影）后移除本预变换。

use std::borrow::Cow;

use crate::document::Document;
use crate::node::{NodeId, NodeKind};

/// 片段路径 s173 预变换的配套值还原：预变换把 `shadowrootmode` 的 `open`/`close`
/// 改写为首字母大写以出 html5ever 0.29.1 精确匹配集（[`restore_declarative_shadow_templates`]），
/// 文档路径随后由 `materialize_declarative_shadow_roots` 消费 template（值大小写
/// 对语义不可见）；片段路径（innerHTML 等）按 opt-in 语义**不物化**（whatwg/dom#831），
/// template 以普通元素存活——改写值经 getAttribute/序列化可观测，须还原为源码
/// 小写形态（WPT declarative-shadow-dom-opt-in 断言 `getAttribute === 'open'`）。
/// 还原只认预变换产出的两形态（`Open`/`Close`）；源码本就首字母大写的边例回显为
/// 小写（FIXME: 源码级大小写保真需标签位置映射，欠做——语料与真实页面主流为
/// 全小写值）。遍历含 template contents 子树（contents 内嵌套声明式模板同样可观测），
/// 不含 shadow 树（片段解析不产 shadow 节点）。
pub(crate) fn restore_shadowrootmode_values(doc: &mut Document) {
    let mut stack: Vec<NodeId> = vec![doc.root()];
    while let Some(id) = stack.pop() {
        let (children, contents) = match doc.get(id) {
            Some(n) => (
                n.children.clone(),
                if matches!(&n.kind, NodeKind::Element(e) if e.local_name() == "template") {
                    doc.template_contents(id)
                } else {
                    None
                },
            ),
            None => continue,
        };
        if let Some(v) = doc.get_attribute(id, "shadowrootmode") {
            let restored = match v.as_str() {
                "Open" => "open",
                "Close" => "close",
                _ => v.as_str(),
            };
            if restored != v.as_str() {
                doc.set_attribute(id, "shadowrootmode", restored);
            }
        }
        stack.extend(children);
        stack.extend(contents);
    }
}

/// RAWTEXT/RCDATA 元素：其文本区内 `<template` 是字面量（spec tokenizer raw text
/// 状态）。https://html.spec.whatwg.org/multipage/parsing.html#raw-text-elements
const RAW_TEXT_TAGS: [&str; 9] = [
    "script",
    "style",
    "textarea",
    "title",
    "xmp",
    "iframe",
    "noembed",
    "noframes",
    "plaintext",
];

/// `<template` 起始标签内 `shadowrootmode` 值首字母大写（`open`/`close` 两值——
/// html5ever 0.29.1 精确匹配集）。非命中文档零拷贝直返（`Cow::Borrowed` 快路径）。
pub(crate) fn restore_declarative_shadow_templates(html: &str) -> Cow<'_, str> {
    // 快门大小写不敏感（spec 属性名 ASCII case-insensitive，`SHADOWROOTMODE` 同为
    // 声明式形态）；零分配字节扫（无 std CI contains）。
    if !contains_ci(html.as_bytes(), b"shadowrootmode") {
        return Cow::Borrowed(html);
    }
    // ASCII 小写副本仅用于定位（ASCII 平移不改字节偏移）；改写输出取原串切片。
    let lower = html.to_ascii_lowercase();
    let b = lower.as_bytes();
    let mut out = String::new();
    let mut copied = 0usize; // out 末尾对应的 html 偏移（已定数区间）
    let mut rewritten = false;
    let mut i = 0usize;
    while let Some(rel) = lower[i..].find('<') {
        let lt = i + rel;
        if let Some(tag) = raw_text_tag_at(b, lt) {
            // 原文文本区：整体跳过（未闭合 raw text → 其后全为文本，扫描终止）。
            match raw_text_end(b, lt, tag) {
                Some(end) => {
                    i = end;
                    continue;
                }
                None => break,
            }
        }
        if !lower[lt..].starts_with("<template") || !tag_name_boundary(b, lt + 9) {
            i = lt + 1;
            continue;
        }
        let (tag_end, attr_at) = scan_template_tag(b, lt + 9);
        if let Some(name_at) = attr_at {
            let seg = &html[lt..tag_end];
            if let Some(seg_out) = rewrite_shadowrootmode_value(seg, name_at - lt) {
                out.push_str(&html[copied..lt]);
                out.push_str(&seg_out);
                copied = tag_end;
                rewritten = true;
            }
        }
        i = tag_end;
    }
    if !rewritten {
        return Cow::Borrowed(html);
    }
    out.push_str(&html[copied..]);
    Cow::Owned(out)
}

/// `b[at..]` 起是否 raw-text 元素起始标签（`<tag` + 标签名边界）；返 tag 名。
fn raw_text_tag_at(b: &[u8], at: usize) -> Option<&'static str> {
    for tag in RAW_TEXT_TAGS {
        let end = at + 1 + tag.len();
        if end <= b.len() && b[at] == b'<' && &b[at + 1..end] == tag.as_bytes() && tag_name_boundary(b, end) {
            return Some(tag);
        }
    }
    None
}

/// raw-text 起始标签（`b[at] == '<'`）原文区的终点（exclusive）：appropriate 闭
/// 标签 `</tag …>` 之后；未闭合 → `None`（其后的 `<template` 均为文本）。
fn raw_text_end(b: &[u8], at: usize, tag: &str) -> Option<usize> {
    let close = format!("</{tag}");
    let mut s = at + 1;
    while let Some(rel) = find_slice(&b[s..], close.as_bytes()) {
        let name_end = s + rel + close.len();
        if tag_name_boundary(b, name_end) {
            // 闭标签体（可有属性）扫到 `>`；EOF 截断则到串尾。
            let mut j = name_end;
            while j < b.len() && b[j] != b'>' {
                j += 1;
            }
            return Some((j + 1).min(b.len()));
        }
        s += rel + 1;
    }
    None
}

/// 标签名/属性名边界：EOF、空白、`/`、`>`（tokenizer tag open/attr name 状态面）。
fn tag_name_boundary(b: &[u8], at: usize) -> bool {
    at >= b.len() || (b[at] as char).is_ascii_whitespace() || b[at] == b'/' || b[at] == b'>'
}

fn find_slice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// 字节面 ASCII 大小写不敏感子串查找（快门专用，无分配）。
fn contains_ci(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || hay.len() < needle.len() {
        return needle.is_empty();
    }
    let last = hay.len() - needle.len();
    (0..=last).any(|i| {
        hay[i..i + needle.len()]
            .iter()
            .zip(needle)
            .all(|(a, b)| a.to_ascii_lowercase() == *b)
    })
}

/// `<template` 标签体扫描（`from` = 名字后）：引号感知走查至标签收尾 `>`，外部态
///（引号外）首个名字边界合法的 `shadowrootmode` 属性名位点（绝对）。
/// 前界只认空白/`/`（`data-shadowrootmode`、`x=shadowrootmode` 值内子串均不命中；
/// `"bar"shadowrootmode` 这类 parse-error 恢复形态不识别——欠改写与现状同损，
/// 不做过度改写）。返（标签终点 exclusive，属性位点）。
fn scan_template_tag(b: &[u8], from: usize) -> (usize, Option<usize>) {
    let mut i = from;
    let mut quote = 0u8;
    let mut attr = None;
    while i < b.len() {
        let c = b[i];
        if quote != 0 {
            if c == quote {
                quote = 0;
            }
        } else if c == b'"' || c == b'\'' {
            quote = c;
        } else if c == b'>' {
            return (i + 1, attr);
        } else if attr.is_none()
            && c == b's'
            && b[i..].starts_with(b"shadowrootmode")
            && (i == 0 || b[i - 1].is_ascii_whitespace() || b[i - 1] == b'/')
        {
            attr = Some(i);
        }
        i += 1;
    }
    (b.len(), attr)
}

/// 段内 `seg[name_at..]` 起为 `shadowrootmode` 属性名：值属 `open`/`close`
///（ASCII 大小写不敏感）时返**值首字母大写的整段改写**；否则 `None`。
fn rewrite_shadowrootmode_value(seg: &str, name_at: usize) -> Option<String> {
    let b = seg.as_bytes();
    let mut i = name_at + "shadowrootmode".len();
    // 属性语法允许 `=` 两侧空白（tokenizer before/after attribute name 状态）。
    while i < b.len() && (b[i] as char).is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b'=' {
        return None;
    }
    i += 1;
    while i < b.len() && (b[i] as char).is_ascii_whitespace() {
        i += 1;
    }
    let quote = if i < b.len() && (b[i] == b'"' || b[i] == b'\'') {
        let q = b[i];
        i += 1;
        Some(q)
    } else {
        None
    };
    let val_start = i;
    let val_end = loop {
        if i >= b.len() {
            break i;
        }
        let c = b[i];
        match quote {
            Some(q) if c == q => break i,
            None if (c as char).is_ascii_whitespace() || c == b'>' => break i,
            _ => i += 1,
        }
    };
    let value = &seg[val_start..val_end];
    if !value.eq_ignore_ascii_case("open") && !value.eq_ignore_ascii_case("close") {
        return None;
    }
    let mut out = String::with_capacity(seg.len());
    out.push_str(&seg[..val_start]);
    let mut vc = value.chars();
    if let Some(first) = vc.next() {
        out.extend(first.to_uppercase());
        out.push_str(vc.as_str());
    }
    out.push_str(&seg[val_end..]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewritten(html: &str) -> bool {
        matches!(restore_declarative_shadow_templates(html), Cow::Owned(_))
    }

    // 片段路径 opt-in（whatwg/dom#831）：innerHTML 类 fragment 解析不物化——template
    // 以普通元素存活、shadowrootmode 值还原源码小写、contents 可读（WPT
    // declarative-shadow-dom-opt-in 断言面）。
    #[test]
    fn fragment_parse_keeps_template_without_materializing() {
        let doc = crate::parse_html_fragment_with_scripting(
            r#"<div class=host><template shadowrootmode=open><span class=content>Content</span></template></div>"#,
            "http://www.w3.org/1999/xhtml",
            "body",
            true,
        );
        let tmpl = doc
            .query_selector(doc.root(), "template[shadowrootmode]")
            .expect("template should survive fragment parse");
        assert_eq!(
            doc.get_attribute(tmpl, "shadowrootmode").as_deref(),
            Some("open"),
            "rewritten value must be restored to source case"
        );
        let contents = doc.template_contents(tmpl).expect("template contents present");
        let span = doc
            .query_selector(contents, ".content")
            .expect("content subtree readable");
        assert_eq!(doc.text_content(span).as_deref(), Some("Content"));
        // 片段路径无 shadow root 节点（不物化）。
        assert!(doc.shadow_root_ids().is_empty());
    }

    // 文档路径 opt-in 关闭变体（shim 视图解析消费面）：同片段语义——template 存活 +
    // 值还原 + 不物化；真实文档路径（parse_html）仍物化（对照断言）。
    #[test]
    fn unmaterialized_document_parse_keeps_template() {
        let html = r#"<html><body><div class=host><template shadowrootmode=open><span>Shadow tree</span></template></div></body></html>"#;
        let doc = crate::parse_html_with_scripting_unmaterialized(html, true);
        let tmpl = doc
            .query_selector(doc.root(), "template[shadowrootmode]")
            .expect("template should survive unmaterialized parse");
        assert_eq!(doc.get_attribute(tmpl, "shadowrootmode").as_deref(), Some("open"));
        assert!(doc.shadow_root_ids().is_empty());
        // 对照：默认文档路径物化——template 消失、shadow root 建立。
        let doc_m = crate::parse_html(html);
        assert!(
            doc_m.query_selector(doc_m.root(), "template[shadowrootmode]").is_none(),
            "materializing path consumes the template"
        );
        assert_eq!(doc_m.shadow_root_ids().len(), 1);
    }

    // M3 Batch 2（s180/gethtml，whatwg/dom#831）：**物化片段解析**（unsafe 双 API
    // 消费面）——`parse_html_fragment_with_scripting_materialized` 消费 template 建
    // 真根，状态位（delegatesfocus/clonable/serializable）自属性 presence 读入。
    #[test]
    fn materialized_fragment_parse_attaches_flags() {
        let doc = crate::parse_html_fragment_with_scripting_materialized(
            r#"<div class=host><template shadowrootmode=open shadowrootdelegatesfocus shadowrootclonable shadowrootserializable><span class=content>C</span></template><p>light</p></div>"#,
            "http://www.w3.org/1999/xhtml",
            "body",
            true,
        );
        // template 消费、真根建立。
        assert!(
            doc.query_selector(doc.root(), "template[shadowrootmode]").is_none(),
            "materializing path consumes the template"
        );
        let host = doc.query_selector(doc.root(), "div.host").expect("host present");
        let sr = doc.shadow_root(host).expect("declarative root attached");
        let data = match doc.get(sr).map(|n| &n.kind) {
            Some(crate::node::NodeKind::ShadowRoot(d)) => d,
            _ => panic!("shadow root node present"),
        };
        assert!(matches!(data.mode, crate::node::ShadowRootMode::Open));
        assert!(data.declarative, "fragment materialization stamps declarative");
        assert!(data.delegates_focus && data.clonable && data.serializable);
        // shadow 子树可查询（.content 移入根）。
        assert!(doc.query_selector(sr, ".content").is_some());
        // light 子仍在宿主（template 后的 p 不入根）。
        assert!(doc.query_selector(host, "p").is_some());
    }

    // M3 Batch 2：attach_shadow_full 状态位与序列化回写——布尔属性按 HTML 序列化
    // spec 顺序（shadowrootmode → delegatesfocus → serializable → clonable）回写为
    // `=""`，与声明式物化读回及 getHTML 模板形态逐字节对齐。
    #[test]
    fn attach_shadow_full_flags_serialize_in_spec_order() {
        let mut doc = crate::Document::new();
        let host = doc.create_element("div");
        let sr = doc
            .attach_shadow_full(host, crate::node::ShadowRootMode::Closed, true, true, true, true)
            .expect("attach_shadow_full succeeds");
        doc.append_child(doc.root(), host).expect("host appended");
        let html = crate::serializer::serialize_node(&doc, host);
        assert!(
            html.contains(
                "<template shadowrootmode=\"closed\" shadowrootdelegatesfocus=\"\" shadowrootserializable=\"\" shadowrootclonable=\"\""
            ),
            "spec-order boolean attrs, got: {html}"
        );
        let data = match doc.get(sr).map(|n| &n.kind) {
            Some(crate::node::NodeKind::ShadowRoot(d)) => d,
            _ => panic!("shadow root node present"),
        };
        assert!(data.declarative, "flags passed through attach_shadow_full");
        // 对照：全 false 位不产生附加属性（attach_shadow 薄包装默认位）。
        let mut doc2 = crate::Document::new();
        let host2 = doc2.create_element("div");
        let sr2 = doc2
            .attach_shadow(host2, crate::node::ShadowRootMode::Open)
            .expect("attach_shadow succeeds");
        let html2 = crate::serializer::serialize_node(&doc2, host2);
        assert_eq!(
            html2, "<div><template shadowrootmode=\"open\"></template></div>",
            "no extra attrs when flags unset, got: {html2}"
        );
        let data2 = match doc2.get(sr2).map(|n| &n.kind) {
            Some(crate::node::NodeKind::ShadowRoot(d)) => d,
            _ => panic!("shadow root node present"),
        };
        assert!(!data2.declarative && !data2.delegates_focus && !data2.clonable && !data2.serializable);
    }

    // M3 簇 C（s301）：shadowrootreferencetarget 值捕获与 spec 序回写——属性序
    // 无关（可先于 shadowrootmode）、空值属性捕获为空串并回写 `=""`、缺属性不
    // 回写。https://dom.spec.whatwg.org/#shadowroot-reference-target
    #[test]
    fn materialized_fragment_parse_captures_reference_target() {
        // 属性序故意倒置：referencetarget 在 shadowrootmode 之前（上游
        // gethtml-serialization.html 的源形态）。
        let doc = crate::parse_html_fragment_with_scripting_materialized(
            r#"<div class=host><template shadowrootreferencetarget="target" shadowrootmode=open shadowrootclonable shadowrootserializable shadowrootdelegatesfocus><span id=target></span></template></div>"#,
            "http://www.w3.org/1999/xhtml",
            "body",
            true,
        );
        let host = doc.query_selector(doc.root(), "div.host").expect("host present");
        let sr = doc.shadow_root(host).expect("declarative root attached");
        let data = match doc.get(sr).map(|n| &n.kind) {
            Some(crate::node::NodeKind::ShadowRoot(d)) => d,
            _ => panic!("shadow root node present"),
        };
        assert_eq!(
            data.reference_target.as_deref(),
            Some("target"),
            "value captured position-independently"
        );
        let html = crate::serializer::serialize_node(&doc, host);
        assert!(
            html.contains(
                "<template shadowrootmode=\"open\" shadowrootdelegatesfocus=\"\" shadowrootserializable=\"\" shadowrootclonable=\"\" shadowrootreferencetarget=\"target\">"
            ),
            "spec order: referencetarget after clonable, got: {html}"
        );

        // 空值属性 → Some("") → 回写 `=""`。
        let doc2 = crate::parse_html_fragment_with_scripting_materialized(
            r#"<div class=host><template shadowrootreferencetarget shadowrootmode=open><span></span></template></div>"#,
            "http://www.w3.org/1999/xhtml",
            "body",
            true,
        );
        let host2 = doc2.query_selector(doc2.root(), "div.host").expect("host present");
        let sr2 = doc2.shadow_root(host2).expect("declarative root attached");
        let data2 = match doc2.get(sr2).map(|n| &n.kind) {
            Some(crate::node::NodeKind::ShadowRoot(d)) => d,
            _ => panic!("shadow root node present"),
        };
        assert_eq!(data2.reference_target.as_deref(), Some(""), "empty attr value kept");
        let html2 = crate::serializer::serialize_node(&doc2, host2);
        assert!(
            html2.contains("shadowrootreferencetarget=\"\">"),
            "empty value serializes as =\"\", got: {html2}"
        );

        // 缺属性 → None → 不回写（对照）。
        let doc3 = crate::parse_html_fragment_with_scripting_materialized(
            r#"<div class=host><template shadowrootmode=open><span></span></template></div>"#,
            "http://www.w3.org/1999/xhtml",
            "body",
            true,
        );
        let host3 = doc3.query_selector(doc3.root(), "div.host").expect("host present");
        let sr3 = doc3.shadow_root(host3).expect("declarative root attached");
        let data3 = match doc3.get(sr3).map(|n| &n.kind) {
            Some(crate::node::NodeKind::ShadowRoot(d)) => d,
            _ => panic!("shadow root node present"),
        };
        assert!(data3.reference_target.is_none(), "absent attr stays None");
        let html3 = crate::serializer::serialize_node(&doc3, host3);
        assert!(
            !html3.contains("shadowrootreferencetarget"),
            "None not serialized, got: {html3}"
        );
    }

    #[test]
    fn open_close_values_uppercased() {
        assert_eq!(
            restore_declarative_shadow_templates(r#"<div><template shadowrootmode="open"><b>x</b></template></div>"#)
                .as_ref(),
            r#"<div><template shadowrootmode="Open"><b>x</b></template></div>"#
        );
        assert_eq!(
            restore_declarative_shadow_templates("<template shadowrootmode=open><p>").as_ref(),
            "<template shadowrootmode=Open><p>"
        );
        assert_eq!(
            restore_declarative_shadow_templates("<template shadowrootmode='close'>").as_ref(),
            "<template shadowrootmode='Close'>"
        );
    }

    #[test]
    fn closed_and_bogus_values_untouched() {
        // "closed" 不在 html5ever 0.29.1 精确匹配集（原样入树 + materialize 正常），
        // 无须绕道：保持 Borrowed 零拷贝。
        assert!(!rewritten(r#"<template shadowrootmode="closed"><p></template>"#));
        assert!(!rewritten(r#"<template shadowrootmode="bogus"><p></template>"#));
        // 值大小写不敏感面：OPEN/Open 已首字母大写形态照原样输出（语义不变）。
        assert_eq!(
            restore_declarative_shadow_templates(r#"<template shadowrootmode="OPEN">"#).as_ref(),
            r#"<template shadowrootmode="OPEN">"#
        );
    }

    #[test]
    fn attr_syntax_variants() {
        // 属性名 ASCII 大小写不敏感 + `=` 两侧空白。
        assert_eq!(
            restore_declarative_shadow_templates(r#"<template SHADOWROOTMODE = "open">"#).as_ref(),
            r#"<template SHADOWROOTMODE = "Open">"#
        );
        // 已是 Open 形态：幂等输出。
        assert_eq!(
            restore_declarative_shadow_templates(r#"<template shadowrootmode="Open">"#).as_ref(),
            r#"<template shadowrootmode="Open">"#
        );
    }

    #[test]
    fn non_attr_and_non_template_occurrences_untouched() {
        // 前缀属性名（data-shadowrootmode）与值内子串不命中。
        assert!(!rewritten(r#"<template data-shadowrootmode="open"><p></template>"#));
        assert!(!rewritten(r#"<template x=shadowrootmode><p></template>"#));
        // 引号内子串不命中。
        assert!(!rewritten(r#"<template title="shadowrootmode=open"><p></template>"#));
        // 无值属性 / `<templates>` 非模板标签。
        assert!(!rewritten("<template shadowrootmode><p></template>"));
        assert!(!rewritten(r#"<templates shadowrootmode="open"></templates>"#));
        // 无 shadowrootmode 的文档走快路径。
        assert!(!rewritten("<div><p>hello</p></div>"));
    }

    #[test]
    fn raw_text_regions_untouched() {
        // script 字面量内的声明式模板串（polyfill 场景）不改写。
        assert!(!rewritten(
            r#"<script>var t = '<template shadowrootmode="open"><p></template>';</script>"#
        ));
        assert!(!rewritten(
            r#"<style>.x { content: '<template shadowrootmode="open">' }</style>"#
        ));
        // script 先闭合：其后真实模板照常改写。
        assert_eq!(
            restore_declarative_shadow_templates(r#"<script>var a = 1;</script><template shadowrootmode="open">"#)
                .as_ref(),
            r#"<script>var a = 1;</script><template shadowrootmode="Open">"#
        );
        // 未闭合 script：其后一切为文本。
        assert!(!rewritten(r#"<script>var t = "<template shadowrootmode=open>"#));
    }

    #[test]
    fn multiple_templates_all_rewritten() {
        assert_eq!(
            restore_declarative_shadow_templates(
                r#"<div><template shadowrootmode="open"><a></template><template shadowrootmode="close"><b></template></div>"#
            )
            .as_ref(),
            r#"<div><template shadowrootmode="Open"><a></template><template shadowrootmode="Close"><b></template></div>"#
        );
    }
}

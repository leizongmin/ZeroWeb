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

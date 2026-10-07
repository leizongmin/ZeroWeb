// part39（t8b）：structural path 快速通道语义钉——纯 Rust 无沙箱（不走 V8）。
// 快速通道（`try_structural_path_fastpath`）与通用 CSS 引擎（`doc.query_selector`）
// 对 [`structural_path_selector`] 生成形态必须逐一结果一致；非该形态一律回落。
// https://drafts.csswg.org/selectors-4/#nth-child-pseudo

use super::*;

/// t8b 一致性主钉：全元素 structural path 逐一 fastpath == 通用引擎。
/// 文档含 doctype、注释、文本节点干扰——`:nth-child` 元素序口径必须跳过它们。
#[test]
fn structural_path_fastpath_matches_css_engine_all_elements() {
    let html = "<!DOCTYPE html><!-- c --><html><head><title>t</title></head>\
<body><div id=\"a\"><p>one</p>text<span></span><!-- x --><p>two</p></div>\
<section><article class=\"b\"><ul><li>1</li><li>2</li><li>3</li></ul></article></section>\
<div data-x=\"y\"><em></em><strong><i>deep</i></strong></div></body></html>";
    let doc = parse_html(html);
    let root = doc.root();
    let mut checked = 0usize;
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        let children = doc.get(id).map(|n| n.children.clone()).unwrap_or_default();
        for c in children {
            stack.push(c);
            let is_elem = doc
                .get(c)
                .is_some_and(|n| matches!(n.kind, NodeKind::Element(_)));
            if !is_elem {
                continue;
            }
            let path = structural_path_selector(&doc, c)
                .expect("element must have structural path");
            let via_engine = doc.query_selector(root, &path);
            let via_fast = try_structural_path_fastpath(&doc, &path);
            assert_eq!(
                via_engine,
                Some(c),
                "engine mismatch for path {path}"
            );
            assert_eq!(
                via_fast,
                Some(c),
                "fastpath mismatch for path {path}"
            );
            checked += 1;
        }
    }
    assert!(checked >= 15, "expected ≥15 elements, got {checked}");
}

/// t8b 回落钉：非 structural 形态（#id / tag.class / 后代复合 / :nth-child 公式 /
/// 独立 :nth-child 段）fastpath 恒 None，`find_by_selector` 行为与通用引擎不变。
#[test]
fn structural_path_fastpath_falls_back_for_non_structural_forms() {
    let html = "<html><body><div id=\"a\"><p class=\"x\">one</p><p>two</p></div>\
<ul><li>1</li><li>2</li></ul></body></html>";
    let doc = parse_html(html);
    let root = doc.root();
    // 形态表：fastpath 必须识别失败（回落），find_by_selector 结果与引擎一致。
    let non_structural = [
        "#a",
        "p.x",
        "div p",
        "ul > li",
        "li:nth-child(2n)",
        "li:nth-child(odd)",
        ":nth-child(2)",
        "li:nth-child(1):last-child",
        "div > p:nth-child(2) em",
        "",
    ];
    for sel in non_structural {
        assert!(
            try_structural_path_fastpath(&doc, sel).is_none(),
            "fastpath must not claim non-structural form: {sel:?}"
        );
        assert_eq!(
            find_by_selector(&doc, sel),
            doc.query_selector(root, sel),
            "find_by_selector fallback diverged for {sel:?}"
        );
    }
    // 语义结果抽查：回落路径仍正确命中。
    assert_eq!(
        find_by_selector(&doc, "#a").map(|n| doc.get_attribute(n, "id")),
        Some(Some("a".to_string()))
    );
    assert!(find_by_selector(&doc, "p.x").is_some());
}

/// t8b 边界钉：structural 形态但失配（pos 越界 / tag 不对）——fastpath None，
/// `find_by_selector` 与引擎一致（None），不误命中兄弟/祖先。
#[test]
fn structural_path_fastpath_rejects_mismatched_segments() {
    let html = "<html><body><div><span>s1</span><span>s2</span></div>\
<div><span>s3</span></div></body></html>";
    let doc = parse_html(html);
    let root = doc.root();
    let mismatches = [
        "html:nth-child(1) > body:nth-child(2) > div:nth-child(3) > span:nth-child(4)", // pos 越界（span 只有 2）
        "html:nth-child(1) > body:nth-child(2) > section:nth-child(3)",                 // tag 不对（无 section）
        "html:nth-child(1) > body:nth-child(2) > div:nth-child(9)",                     // 顶层 pos 越界
    ];
    for sel in mismatches {
        assert_eq!(
            try_structural_path_fastpath(&doc, sel),
            doc.query_selector(root, sel),
            "mismatch path diverged: {sel:?}"
        );
        assert!(find_by_selector(&doc, sel).is_none(), "must not hit: {sel:?}");
    }
    // 同族命中锚：形态正确时仍命中。
    let good = "html:nth-child(1) > body:nth-child(2) > div:nth-child(2) > span:nth-child(1)";
    let hit = find_by_selector(&doc, good);
    assert_eq!(
        hit.map(|n| doc.get_attribute(n, "id")),
        Some(None)
    );
    assert_eq!(
        hit.and_then(|n| structural_path_selector(&doc, n)),
        Some(good.to_string())
    );
}

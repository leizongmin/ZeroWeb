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

/// t8b 边界钉：structural 形态但失配（pos 越界 / 位次命中但 tag 不对）——
/// fastpath None，`find_by_selector` 与引擎一致（None），不误命中兄弟/祖先。
#[test]
fn structural_path_fastpath_rejects_mismatched_segments() {
    let html = "<html><body><div><span>s1</span><span>s2</span></div>\
<div><span>s3</span></div></body></html>";
    let doc = parse_html(html);
    let root = doc.root();
    let mismatches = [
        // pos 越界（该层元素子不足）：链中途与顶层各一。
        "html:nth-child(1) > body:nth-child(2) > div:nth-child(3) > span:nth-child(4)",
        "html:nth-child(1) > body:nth-child(2) > div:nth-child(9)",
        // 位次命中但 tag 不对（tag 比较分支）：中段与末段各一。
        "html:nth-child(1) > section:nth-child(2)", // 位次 2 命中 body，tag ≠ body
        "html:nth-child(1) > body:nth-child(2) > em:nth-child(1)", // 位次 1 命中 div，tag ≠ div
        "html:nth-child(1) > body:nth-child(2) > div:nth-child(1) > em:nth-child(1)", // 位次 1 命中 span，tag ≠ span
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

/// t8b（D1 返修）：字面怪名标签（`<div#x>`）——引擎把选择器段 `div#x` 复合解析为
/// tag `div` + id `x`（无匹配，与真实浏览器一致），fastpath 不得把 `div#x` 当字面
/// tag 认领（位次命中 + 字面 local_name 恰为 `div#x` 时会误命中）。
#[test]
fn structural_path_fastpath_rejects_non_ident_tag_chars() {
    // HTML 分词器允许字面 `<div#x>` 产生含 `#` 的 local_name。
    let html = "<html><body><span></span><div#x>t</div#x></body></html>";
    let doc = parse_html(html);
    let root = doc.root();
    let sel = "html:nth-child(1) > body:nth-child(2) > div#x:nth-child(2)";
    assert!(
        try_structural_path_fastpath(&doc, sel).is_none(),
        "tag segment `div#x` is not a CSS ident; fastpath must fall back"
    );
    assert_eq!(
        find_by_selector(&doc, sel),
        doc.query_selector(root, sel),
        "fallback must match engine"
    );
    assert!(find_by_selector(&doc, sel).is_none(), "engine finds nothing");
    // 同文档常规 path 不受白名单收窄影响，仍走快速通道命中。
    let good = "html:nth-child(1) > body:nth-child(2) > span:nth-child(1)";
    assert!(find_by_selector(&doc, good).is_some());
    // 形态变体：大写 tag 双侧同为 ASCII 不敏感比较，认领面结果一致；
    // 段间多余空格（tag 含空白）白名单拒绝回落，由通用引擎解析（`>` 语义）。
    let upper = "html:nth-child(1) > BODY:nth-child(2) > span:nth-child(1)";
    assert_eq!(find_by_selector(&doc, upper), doc.query_selector(root, upper));
    assert!(find_by_selector(&doc, upper).is_some());
    let spaces = "html:nth-child(1) >  body:nth-child(2)";
    assert_eq!(find_by_selector(&doc, spaces), doc.query_selector(root, spaces));
    assert!(find_by_selector(&doc, spaces).is_some());
}

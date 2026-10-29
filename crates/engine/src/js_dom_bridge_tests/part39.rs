// part39（t8b）：structural path 快速通道语义钉——纯 Rust 无沙箱（不走 V8）。
// 快速通道（`try_structural_path_fastpath`）与通用 CSS 引擎（`doc.query_selector`）
// 对 [`structural_path_selector`] 生成形态必须逐一结果一致；非该形态一律回落。
// https://drafts.csswg.org/selectors-4/#nth-child-pseudo

use super::*;

/// t8b 一致性主钉：全元素 structural path 逐一引擎命中；fastpath 对**其认领形态**
///（纯 `tag:nth-child` 段链，s145 刀 D 起锚定形态 `anchor > …` 首段即不匹配，
/// 设计内回落通用引擎——见 [`structural_path_selector`] 文档）逐一 == 引擎。
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
    let mut fastpath_checked = 0usize;
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
            if let Some(fast) = via_fast {
                assert_eq!(
                    fast, c,
                    "fastpath mismatch for claimed path {path}"
                );
                fastpath_checked += 1;
            }
            checked += 1;
        }
    }
    assert!(checked >= 15, "expected ≥15 elements, got {checked}");
    // fastpath 认领覆盖不得萎缩：无锚链（html/head/body 直系与 data-x 子树——
    // data-* 非身份锚）仍生成 root 形态，fastpath 必须认领。
    assert!(
        fastpath_checked >= 6,
        "expected ≥6 fastpath-claimed paths, got {fastpath_checked}"
    );
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

/// s145 刀 D 回归钉：body afterbegin 连环插入（WPT targetted-form populateForm
/// 形态）下，锚祖先（form[target]）子树内歧义元素（input）的结构路径必须
/// **逐代互异**且各自恒解析回本代节点——旧 root 形态两代字节相同（`html > body >
/// form:nth-child(2) > input:nth-child(1)`），`_proxyCache`/`_inputValues` 按
/// `_elKey` 跨代碰撞（元素身份共享、脏值/listener 泄漏）。
#[test]
fn structural_path_anchored_distinct_across_afterbegin_generations() {
    // gen1：iframe(1) form(2)；gen2 afterbegin：iframe(3) form(4)（s142 后
    // form 有 target 锚，input 无属性可锚——旧形态下两代 input 路径串相同）。
    let mut doc = parse_html(
        "<html><body></body></html>",
    );
    // 无锚中间层（div 不带 id/class/name）不得隔断锚定：input 的最近身份锚
    // 祖先是 form[target]，路径须落根于锚而非 div。
    let gen1 = "<iframe name=\"form-test-target-0\"></iframe>\
<form target=\"form-test-target-0\"><div><input name=\"n1\" value=\"v1\"></div></form>";
    apply_insert_afterbegin(&mut doc, gen1);
    let gen2 = "<iframe name=\"form-test-target-1\"></iframe>\
<form target=\"form-test-target-1\"><div><input name=\"n1\" value=\"v1\"></div></form>";
    apply_insert_afterbegin(&mut doc, gen2);

    let sel_of_input = |doc: &Document, target: &str| -> Option<String> {
        let form = find_by_selector(doc, &format!("form[target=\"{target}\"]"))
            .expect("anchor form must resolve");
        let div = doc
            .get(form)
            .map(|n| n.children.clone())
            .unwrap_or_default()
            .into_iter()
            .find(|c| doc.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Element(ref e) if e.local_name() == "div")))
            .expect("form must contain div");
        let input = doc
            .get(div)
            .map(|n| n.children.clone())
            .unwrap_or_default()
            .into_iter()
            .find(|c| doc.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Element(ref e) if e.local_name() == "input")))
            .expect("div must contain input");
        unique_selector_for_node(doc, input)
    };

    let sel1 = sel_of_input(&doc, "form-test-target-0").expect("gen1 input sel");
    let sel2 = sel_of_input(&doc, "form-test-target-1").expect("gen2 input sel");
    // 逐代互异（碰撞根消除）。
    assert_ne!(sel1, sel2, "cross-generation selector collision: {sel1:?}");
    // 锚定落根（非 root 形态）+ 锚内相对段。
    assert!(
        sel1.starts_with("form[target=\"form-test-target-0\"] > "),
        "gen1 input must anchor at its form, got {sel1:?}"
    );
    // 各自恒解析回本代节点（含 gen1 在 gen2 插入后的位移态）。
    let back1 = find_by_selector(&doc, &sel1).expect("gen1 sel must resolve");
    assert_eq!(
        doc.get_attribute(back1, "name"),
        Some("n1".to_string()),
    );
    let form1 = find_by_selector(&doc, "form[target=\"form-test-target-0\"]")
        .expect("gen1 form");
    assert_eq!(
        element_parent(&doc, back1)
            .and_then(|p| element_parent(&doc, p)),
        Some(form1),
        "gen1 sel must resolve inside gen1 form"
    );
    let back2 = find_by_selector(&doc, &sel2).expect("gen2 sel must resolve");
    let form2 = find_by_selector(&doc, "form[target=\"form-test-target-1\"]")
        .expect("gen2 form");
    assert_eq!(
        element_parent(&doc, back2)
            .and_then(|p| element_parent(&doc, p)),
        Some(form2),
        "gen2 sel must resolve inside gen2 form"
    );
}

/// 测试辅助：body afterbegin 插入 HTML 片段（populateForm 同位语义，单条
/// InsertAdjacentHtml——片段内相对序保留：iframe(1)、form(2)）。
fn apply_insert_afterbegin(doc: &mut Document, html: &str) {
    apply_dom_mutations(
        doc,
        &[DomMutation::InsertAdjacentHtml {
            selector: "body".to_string(),
            position: "afterbegin".to_string(),
            html: html.to_string(),
        }],
    )
    .expect("afterbegin insert must apply");
}

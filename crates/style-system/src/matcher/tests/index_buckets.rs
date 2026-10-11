//! P11 索引差分 + 规模统计（2026-10-11）。
//!
//! 两件事：
//! 1. 差分正确性：索引路径（collect_matching_declarations_with_media）与无索引
//!    全量参考（collect_from_rules 传 None）在对抗性样式表 + 多元素 × 伪元素 ×
//!    媒体/容器上下文组合下逐声明相等（桶是必要条件过滤，不改变匹配与级联序）。
//! 2. 规模统计（仅 stderr 输出，供 P11 诊断读取；无断言）：
//!    ZW_P11_FIXTURE_CSS 指向真实 CSS 时打印桶大小分布与候选数样本。

use crate::matcher::{
    ContainerContext, MatchingDecl, StylesheetIndex, build_stylesheet_index, collect_from_rules,
    collect_matching_declarations_with_media, collect_pseudo_declarations_with_media,
};
use zero_css_parser::media_query::{
    MediaContext, MediaType, PointerValue, PrefersColorSchemeValue, ReducedMotionValue,
};
use zero_css_parser::{Parser, Stylesheet};
use zero_dom::{Document, NodeId, parse_html};

// ── 差分 ──

fn media_ctx() -> MediaContext {
    MediaContext {
        viewport_width: 800.0,
        viewport_height: 600.0,
        media_type: MediaType::Screen,
        prefers_color_scheme: PrefersColorSchemeValue::Light,
        prefers_reduced_motion: ReducedMotionValue::NoPreference,
        pointer_type: PointerValue::Fine,
        resolution_dpi: 96.0,
    }
}

/// 无索引全量参考：直接走 collect_from_rules(None)（self_at_rule_dispatch 全量路径）。
fn full_scan(
    doc: &Document,
    element: NodeId,
    sheets: &[Stylesheet],
    media_ctx: Option<&MediaContext>,
    container_ctx: Option<&ContainerContext>,
    pseudo: Option<&str>,
) -> Vec<MatchingDecl> {
    let mut results = Vec::new();
    let mut layer_counter: usize = 0;
    let mut layer_map = std::collections::HashMap::new();
    for sheet in sheets {
        collect_from_rules(
            doc,
            element,
            &sheet.rules,
            None,
            &mut results,
            media_ctx,
            container_ctx,
            None,
            &mut layer_counter,
            &mut layer_map,
            pseudo,
        );
    }
    results
}

const ADVERSARIAL_CSS: &str = r#"
/* tag 桶 */
div { color: black; }
li, p { margin: 1px; }
/* class 桶（含链、同规则多选择器跨界桶） */
.card { color: red; }
.card.featured { color: blue; }
#hero.card { color: green; }
#hero { border: 1px solid; }
.special, .card { padding: 2px; }
/* :not/:is 参数不挖（元素可能匹配也可能不匹配） */
.card:not(.y) { color: purple; }
:is(.list, .menu) .item { color: teal; }
div:not(#hero) { outline: 1px; }
/* attr-only / 裸伪类 → universal */
[data-x] { color: olive; }
[data-x="1"] { color: navy; }
*:hover { color: pink; }
/* universal */
* { line-height: normal; }
/* 伪元素（tag / class 键） */
.card::before { content: "B"; }
em::after { content: "A"; }
#hero::after { content: "H"; }
/* @media 嵌套（匹配 / 不匹配 / 深嵌套） */
@media screen {
  .card { color: maroon; }
  div span { font-weight: bold; }
  @media (min-width: 500px) {
    .featured { outline: 2px; }
  }
  @supports (display: grid) {
    .card { display: grid; }
    @container (min-width: 200px) {
      .item { color: khaki; }
    }
  }
}
@media print and (min-width: 2000px) {
  .card { color: silver; }
}
@media (min-width: 99999px) {
  .card { color: coral; }
}
/* @layer：语句 + 命名块 + 匿名块，与顶层规则交错 */
@layer base, top;
@layer base {
  .card { color: gray; }
}
@layer top {
  .card { color: lightgreen; }
}
@layer {
  .item { color: gold; }
}
/* @supports 真/假 */
@supports (display: block) {
  .text { letter-spacing: 1px; }
}
@supports (foo-bar-xyz: nope) {
  .text { letter-spacing: 99px; }
}
/* @container 匹配/不匹配 */
@container (min-width: 200px) {
  .card { color: crimson; }
}
@container (min-width: 5000px) {
  .card { color: whitesmoke; }
}
/* 长后代链（评估成本形态） */
ul li p span em { vertical-align: top; }
/* 靠后的顶层规则（源码序 vs @media 块） */
.card { color: final; }
"#;

const ADVERSARIAL_HTML: &str = r##"<html><body>
<div id="hero" class="card featured">h</div>
<span class="card">s</span>
<p class="text muted" data-x="1">p</p>
<ul class="list"><li class="item">one</li><li>two</li></ul>
<section id="main">m</section>
<a class="link external" href="#">a</a>
<div class="y">y</div>
<em class="card">e</em>
<div>plain</div>
<div class="menu"><li class="item">mi</li></div>
</body></html>"##;

fn elements_by_tags(doc: &Document, tags: &[&str]) -> Vec<NodeId> {
    let mut out = Vec::new();
    for t in tags {
        out.extend(doc.get_elements_by_tag_name(t));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// 差分：索引路径 == 无索引全量，覆盖 常规/伪元素 × 媒体上下文 × 容器上下文。
#[test]
fn p11_indexed_equals_full_scan() {
    let sheet = Parser::parse_stylesheet(ADVERSARIAL_CSS);
    let sheets = vec![sheet];
    let index = build_stylesheet_index(&sheets);
    let doc = parse_html(ADVERSARIAL_HTML);
    let els = elements_by_tags(&doc, &["div", "span", "p", "li", "section", "a", "em", "ul"]);
    assert!(els.len() >= 10, "样本元素过少: {}", els.len());

    let mctx = media_ctx();
    let cctx_match = ContainerContext::with_size(300.0, 600.0);
    let cctx_miss = ContainerContext::with_size(100.0, 600.0);
    let ctx_variants: [(&str, Option<&MediaContext>, Option<&ContainerContext>); 5] = [
        ("media+container", Some(&mctx), Some(&cctx_match)),
        ("media only", Some(&mctx), None),
        ("container only", None, Some(&cctx_match)),
        ("no ctx", None, None),
        ("container miss", Some(&mctx), Some(&cctx_miss)),
    ];

    for el in els {
        let desc = format!("{:?}", el);
        for (cname, mc, cc) in &ctx_variants {
            let indexed = collect_matching_declarations_with_media(&doc, el, &sheets, &index, mc.as_deref(), *cc);
            let full = full_scan(&doc, el, &sheets, mc.as_deref(), *cc, None);
            assert_eq!(
                indexed, full,
                "常规路径差分失败: el={desc} ctx={cname}\nindexed={:?}\nfull={:?}",
                indexed, full
            );
            for pseudo in ["before", "after"] {
                let ip = collect_pseudo_declarations_with_media(&doc, el, &sheets, &index, mc.as_deref(), *cc, pseudo);
                let fp = full_scan(&doc, el, &sheets, mc.as_deref(), *cc, Some(pseudo));
                assert_eq!(
                    ip, fp,
                    "伪元素 ::{pseudo} 差分失败: el={desc} ctx={cname}\nindexed={:?}\nfull={:?}",
                    ip, fp
                );
            }
        }
    }
}

/// 多样式表（引擎把每个 <link> CSS 作为独立样式表传入）下的同性质差分。
#[test]
fn p11_indexed_equals_full_scan_multi_sheet() {
    let sheet_a = Parser::parse_stylesheet("@media all { .card { background: a; } } .card { background: b; }");
    let sheet_b = Parser::parse_stylesheet(
        "@layer top { .card { color: l1; } } #hero { color: h1; } @media (min-width: 500px) { .featured { color: f1; } }",
    );
    let sheets = vec![sheet_a, sheet_b];
    let index = build_stylesheet_index(&sheets);
    let doc = parse_html(r#"<div id="hero" class="card featured">x</div><span class="card">y</span>"#);
    let els = elements_by_tags(&doc, &["div", "span"]);
    let mctx = media_ctx();
    for el in els {
        let indexed = collect_matching_declarations_with_media(&doc, el, &sheets, &index, Some(&mctx), None);
        let full = full_scan(&doc, el, &sheets, Some(&mctx), None, None);
        assert_eq!(indexed, full, "多样式表差分失败 el={el:?}");
    }
}

// ── 规模统计（P11 诊断输出，无断言；ZW_P11_FIXTURE_CSS 未设时跳过） ──

#[test]
fn p11_index_stats() {
    let Ok(path) = std::env::var("ZW_P11_FIXTURE_CSS") else {
        eprintln!("ZW_P11_FIXTURE_CSS not set; skip");
        return;
    };
    let Ok(css) = std::fs::read_to_string(&path) else {
        eprintln!("read failed: {path}");
        return;
    };
    let sheet = Parser::parse_stylesheet(&css);
    let index = build_stylesheet_index(std::slice::from_ref(&sheet));

    // 桶大小：借用内部字段的统计（同 crate 内可访问私有字段——经由 pub(crate) 访问器）。
    let (n_selector_total, n_universal, n_buckets_class, n_buckets_id, n_buckets_tag, n_nested) = index.stats();
    eprintln!(
        "selectors={n_selector_total} universal={n_universal} \
         buckets: class={n_buckets_class} id={n_buckets_id} tag={n_buckets_tag} nested={n_nested}"
    );

    // 候选数样本：flat CSS 全顶层，元素样本直接从 HTML 取。
    let html_path = std::env::var("ZW_P11_FIXTURE_HTML").unwrap_or_default();
    let Ok(html) = std::fs::read_to_string(&html_path) else {
        eprintln!("ZW_P11_FIXTURE_HTML not set; skip candidate sampling");
        return;
    };
    let doc = parse_html(&html);
    // 抽样 200 个元素：每 50 个取 1 个
    let mut sample = Vec::new();
    for tag in ["div", "span", "a", "p", "li", "button"] {
        for (i, el) in doc.get_elements_by_tag_name(tag).into_iter().enumerate() {
            if i % 50 == 0 {
                sample.push(el);
            }
        }
    }
    for el in sample.iter().take(20) {
        let n = index.candidate_count(&doc, *el);
        eprintln!("el={el:?} candidates={n}");
    }
}

//! P124（t-p124 github 渲染器 styled render 爆炸修复）：@media 命中预计算与
//! pseudo pass 候选门两处优化必须与旧全量走查**逐点等价**。
//!
//! - `build_stylesheet_index_with_media` 预计算的命中表 vs `build_stylesheet_index`
//!   的现场求值回退：同一 collect 调用结果全等（嵌套 @media、@media×@layer、
//!   命中/未命中组合）。
//! - pseudo pass 候选门（`pseudo_rule_set` + `has_pseudo` 递归门）：删除的块
//!   不贡献候选，层号仅相对序参与级联 → 与未门禁结果全等。
//!
//! 值断言仅用于证明结果非空、判据有牙（媒体命中/未命中确实翻转），等价性以
//! 全 Vec 比较为准。

use crate::matcher::{
    build_stylesheet_index, build_stylesheet_index_with_media, collect_matching_declarations_with_media,
    collect_pseudo_declarations_with_media,
};
use zero_css_parser::Parser;
use zero_css_parser::media_query::MediaContext;
use zero_dom::parse_html;

/// 覆盖：顶层普通规则、命中/未命中 @media、嵌套 @media、@media×@layer 伪元素、
/// 顶层伪元素、跨元素伪元素（span::before 不应漏进 div 的候选门）、不可解析
/// prelude 的 @media（未知媒体特征——P124 二轮热点：逐元素回退重解析）。
const CSS: &str = "\
.u { color: navy; } \
@media (min-width: 500px) { \
  .a { color: red; } \
  @media (max-width: 600px) { .a { background: green; } .a::before { content: \"inner-only\"; } } \
  @layer l1 { .a::before { content: \"in-layer-media\"; } } \
} \
@media (min-width: 800px) { .a { color: blue; } } \
@media (hover:hover) { .a { color: olive; } } \
@media (forced-colors:active) { .a { color: maroon; } } \
@layer l3 { @media (min-width: 100px) { .a { color: purple; } } } \
.a::before { content: \"top\"; } \
.a::after { content: \"tail\"; } \
@layer l2 { .a::marker { content: \"m\"; } } \
span::before { content: \"span-only\"; } \
";

const HTML: &str = r#"<html><body><div class="a" id="t"></div><span class="u"></span></body></html>"#;

fn collect_all(
    sheet: &zero_css_parser::Stylesheet,
    doc: &zero_dom::Document,
    el: zero_dom::NodeId,
    media_ctx: &MediaContext,
    pseudo: Option<&str>,
    precomputed: bool,
) -> Vec<(String, String, bool, (u32, u32, u32), Option<usize>)> {
    let sheets = std::slice::from_ref(sheet);
    let index = if precomputed {
        build_stylesheet_index_with_media(sheets, Some(media_ctx))
    } else {
        build_stylesheet_index(sheets)
    };
    match pseudo {
        Some(name) => collect_pseudo_declarations_with_media(doc, el, sheets, &index, Some(media_ctx), None, name),
        None => collect_matching_declarations_with_media(doc, el, sheets, &index, Some(media_ctx), None),
    }
}

/// 普通级联与伪元素级联：预计算索引 == 回退索引（两个视口 × 两个元素 × 4 种 pass）。
#[test]
fn p124_precomputed_media_hits_equal_fallback() {
    let sheet = Parser::parse_stylesheet(CSS);
    let doc = parse_html(HTML);
    let div = *doc.get_elements_by_tag_name("div").last().expect("div");
    let span = *doc.get_elements_by_tag_name("span").last().expect("span");
    for (label, ctx) in [
        ("inner-hit", MediaContext::new(550.0, 600.0)),
        ("all-media-miss", MediaContext::new(100.0, 600.0)),
        ("outer-hit-inner-miss", MediaContext::new(900.0, 600.0)),
    ] {
        for (elem_name, el) in [("div", div), ("span", span)] {
            for pseudo in [None, Some("before"), Some("after"), Some("marker")] {
                let fallback = collect_all(&sheet, &doc, el, &ctx, pseudo, false);
                let precomputed = collect_all(&sheet, &doc, el, &ctx, pseudo, true);
                assert_eq!(
                    fallback, precomputed,
                    "P124: {label} {elem_name} pseudo={pseudo:?} 预计算与回退结果须全等"
                );
            }
        }
    }
}

/// 判据有牙：内容断言证明媒体翻转真实生效、伪元素候选门没有误杀。
#[test]
fn p124_gate_teeth_media_flip_and_pseudo_present() {
    let sheet = Parser::parse_stylesheet(CSS);
    let doc = parse_html(HTML);
    let div = *doc.get_elements_by_tag_name("div").last().expect("div");

    // 550px：嵌套内层命中 → red + green；::before = top + in-layer-media + inner-only
    //（@layer l1 在外层 media 内，随外层命中）。
    let ctx_hit = MediaContext::new(550.0, 600.0);
    let normal = collect_all(&sheet, &doc, div, &ctx_hit, None, true);
    assert!(
        normal.iter().any(|(p, v, ..)| p == "color" && v == "red")
            && normal.iter().any(|(p, v, ..)| p == "background" && v == "green"),
        "P124: 550px 应命中嵌套 @media（red/green），实际 {normal:?}"
    );
    let before = collect_all(&sheet, &doc, div, &ctx_hit, Some("before"), true);
    assert!(
        before.iter().any(|(_, v, ..)| v == "\"in-layer-media\"")
            && before.iter().any(|(_, v, ..)| v == "\"top\"")
            && before.iter().any(|(_, v, ..)| v == "\"inner-only\""),
        "P124: 550px ::before 应含 top/in-layer-media/inner-only，实际 {before:?}"
    );

    // 900px：外层 (min-500) 命中 → in-layer-media 在；内层 (max-600) 未命中 →
    // 无 green、无 inner-only；min-width:800 命中 → blue。
    let ctx_mid = MediaContext::new(900.0, 600.0);
    let normal_mid = collect_all(&sheet, &doc, div, &ctx_mid, None, true);
    assert!(
        normal_mid.iter().any(|(p, v, ..)| p == "color" && v == "blue")
            && !normal_mid.iter().any(|(p, v, ..)| p == "background"),
        "P124: 900px 应命中 800px 块（blue）且错过 600px 内层，实际 {normal_mid:?}"
    );
    let before_mid = collect_all(&sheet, &doc, div, &ctx_mid, Some("before"), true);
    assert!(
        before_mid.iter().any(|(_, v, ..)| v == "\"top\"")
            && before_mid.iter().any(|(_, v, ..)| v == "\"in-layer-media\"")
            && !before_mid.iter().any(|(_, v, ..)| v == "\"inner-only\""),
        "P124: 900px ::before = top + in-layer-media、无 inner-only，实际 {before_mid:?}"
    );

    // 跨元素隔离：div 的 ::before 门禁不吞掉 span::before。
    let span = *doc.get_elements_by_tag_name("span").last().expect("span");
    let span_before = collect_all(&sheet, &doc, span, &ctx_hit, Some("before"), true);
    assert!(
        span_before.iter().any(|(_, v, ..)| v == "\"span-only\""),
        "P124: span::before 不得被 pseudo 候选门误杀，实际 {span_before:?}"
    );

    // 不可解析 prelude 的 @media（未知媒体特征）：两索引下都不应用（parse 失败 →
    // 回退 parse→None→空表→false；预计算缓存同值），且预计算后不再走回退重解析。
    let normal_all = collect_all(&sheet, &doc, div, &ctx_hit, None, true);
    assert!(
        !normal_all
            .iter()
            .any(|(p, v, ..)| p == "color" && (v == "olive" || v == "maroon")),
        "P124: 未知媒体特征 @media 不得应用（olive/maroon），实际 {normal_all:?}"
    );
    // @layer 包 @media：预计算下钻覆盖嵌套形态（min-width:100px 恒命中 → purple 在）。
    assert!(
        normal_all.iter().any(|(p, v, ..)| p == "color" && v == "purple"),
        "P124: @layer 内嵌 @media 命中块须应用（purple），实际 {normal_all:?}"
    );
}

/// 无伪元素规则 + 全命中 @media 的退化页：门禁/预计算双双零影响。
#[test]
fn p124_no_pseudo_rules_page_unchanged() {
    let css = "@media (min-width: 10px) { .a { color: teal; } } .a { opacity: 0.5; }";
    let sheet = Parser::parse_stylesheet(css);
    let doc = parse_html(HTML);
    let div = *doc.get_elements_by_tag_name("div").last().expect("div");
    let ctx = MediaContext::new(550.0, 600.0);
    let fallback = collect_all(&sheet, &doc, div, &ctx, None, false);
    let precomputed = collect_all(&sheet, &doc, div, &ctx, None, true);
    assert_eq!(fallback, precomputed, "P124: 无伪元素页两索引须全等");
    let before = collect_all(&sheet, &doc, div, &ctx, Some("before"), true);
    assert!(
        before.is_empty(),
        "P124: 无伪元素规则页 ::before 应为空，实际 {before:?}"
    );
}

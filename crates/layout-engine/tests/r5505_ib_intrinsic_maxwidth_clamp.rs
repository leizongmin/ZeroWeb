//! R5505 主判例：`box_content_max_width` 子树递归值经子自身 definite max-width
//! 贡献钳制（css-sizing-3 §5.2 intrinsic contribution）。
//!
//! 缺陷机制（上游 inline-block-non-replaced-width-003/004 ✗2.25%，shrink-to-fit
//! preferred 宽域）：`box_content_max_width_inner` 三处子递归臂（R4032 destretch 臂、
//! R54xx 纯文本原子臂、auto 宽块级子臂）裸取递归值，子自身 `max-width` 不参与
//! 贡献——父 inline-block `width:auto` 的 shrink-to-fit
//!（`shrink_inline_blocks_to_content` horizontal 臂消费本测量）取未钳文本 advance
//! 写宽，红背景暴露；姊妹实现 `block_intrinsic_width_ex` 同位臂（R5310 塌 0 递归 /
//! R5368 块级子）此前已钳，双路径漏同步。钳后父 preferred 宽 = 子钳后 used 宽
//!（CSS2 §10.3.9）。
//!
//! 锚形字体无关化：锚环境无 Ahem（fallback per-char advance 非确定值），基准臂
//! 只断言相对关系（父贴子 ∧ 子>121 保钳余量）；钳臂断言绝对值 120（显式钳目标
//! 与 fallback advance 无关）。48px 字号保证 fallback advance 下 8 字符内容宽
//! 稳定 >121。
//!
//! 谱系：R5310（原子 inline 塌 0 域递归 + max-width 钳）、R5368（块级子贡献钳）、
//! R4568（definite-width 短路）、R5348（shrink grow 补腿）——本钉收口
//! `box_content_max_width` 递归路径同族漏钳。

use zero_layout_engine::{LayoutBox, LayoutEngine};

fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let zero_dom::NodeKind::Element(e) = &n.kind
        && e.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find(id, doc, c))
}

fn layout(
    html: &str,
    css: &str,
) -> (
    zero_dom::Document,
    std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
    zero_layout_engine::LayoutResult,
) {
    let doc = zero_dom::parse_html(html);
    let ss = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[ss]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, styles, result)
}

const BASE_CSS: &str = r#"
    body { margin: 0; }
    div#outer { display: inline-block; width: auto; font-size: 48px; }
"#;

/// 基准臂：子无 max-width——父贴子内容真值（>121 保钳臂余量；本臂不依赖
/// fallback advance 的绝对值）。
#[test]
fn r5505_unconstrained_baseline_gt_clamp_target() {
    let (doc, _styles, result) = layout(
        r#"<html><body><div id="outer"><span id="inner">12345678</span></div></body></html>"#,
        &format!("{BASE_CSS} span#inner {{ display: inline-block; }}"),
    );
    let outer = find("outer", &doc, &result.root).expect("outer 盒在树");
    let inner = find("inner", &doc, &result.root).expect("inner 盒在树");
    assert!(
        (outer.width - inner.width).abs() < 1.0,
        "无约束时父贴子宽（outer={} inner={}）",
        outer.width,
        inner.width
    );
    assert!(
        inner.width > 121.0,
        "基准内容宽须 >121（钳臂余量），got {}",
        inner.width
    );
}

/// 主判例（width-003 形）：inline-block 父 + inline-block 子 max-width:120px
///（内容 8 字符 48px，真值 >121）——父 shrink 宽取钳后贡献 120。
#[test]
fn r5505_inline_block_child_maxwidth_clamps_parent_stf() {
    let (doc, _styles, result) = layout(
        r#"<html><body><div id="outer"><span id="inner">12345678</span></div></body></html>"#,
        &format!("{BASE_CSS} span#inner {{ display: inline-block; max-width: 120px; }}"),
    );
    let outer = find("outer", &doc, &result.root).expect("outer 盒在树");
    let inner = find("inner", &doc, &result.root).expect("inner 盒在树");
    assert!(
        (inner.width - 120.0).abs() < 1.0,
        "子自身 max-width 120 生效，got {}",
        inner.width
    );
    assert!(
        (outer.width - 120.0).abs() < 1.0,
        "父 shrink 宽 = 子钳后贡献 120，got {}",
        outer.width
    );
}

/// 主判例（width-004 形）：inline-block 父 + **block** div 子 max-width:120px——
/// 块级子递归臂同款钳制。
#[test]
fn r5505_block_child_maxwidth_clamps_parent_stf() {
    let (doc, _styles, result) = layout(
        r#"<html><body><div id="outer"><div id="inner">12345678</div></div></body></html>"#,
        &format!("{BASE_CSS} div#inner {{ max-width: 120px; }}"),
    );
    let outer = find("outer", &doc, &result.root).expect("outer 盒在树");
    let inner = find("inner", &doc, &result.root).expect("inner 盒在树");
    assert!(
        (inner.width - 120.0).abs() < 1.0,
        "子自身 max-width 120 生效，got {}",
        inner.width
    );
    assert!(
        (outer.width - 120.0).abs() < 1.0,
        "父 shrink 宽 = 子钳后贡献 120，got {}",
        outer.width
    );
}

//! R5404（css-pseudo-4 #first-line-styling 切片 3a）：::first-line 伪样式 line-height
//! 位布局期应用——首行域（首个强制断前）容器直接文本 run 行高取伪样式解析值。
//!
//! 驱动案 css-pseudo first-line-line-height-003（✗1.89%）：div line-height 40px +
//! `div:first-of-type::first-line { line-height: 0px; color: purple }`，ref 页把首行
//! 包进 `<span>`（同款声明直接命中 inline 盒）——ZW 对 span 变体行盒装配已自洽，
//! 修复让 ::first-line 变体的 run 与 span 变体同槽同值（line_height 覆盖），行盒
//! 装配/strut 交互/glyph 垂直度量逐臂复用既有机制。端到端判定由该案 reftest 承载。
//!
//! 域边界（collect 层可断言的部分）：强制断（br）终止首行域；color-only 页
//! （伪样式行高 = 容器行高）零操作；伪样式 font-size 差异域（001/002 的
//! font-size: 2.5em/16px）不激活——仅施行高会全段错置，属 FIXME 切片 3b。
//!
//! kill-switch `ZW_FIRST_LINE_LH`（default-on；进程内 set_var 不可靠，R3858 教训
//! ——OFF 轮由 corpus 覆盖，不设单测；OFF 态 003 回 ✗1.89%）。

use super::super::*;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_style_system::{ComputedStyle, LineHeightValue};

/// 003 案同构：div（lh 40px）内「首行文本 + br + 次行文本」，::first-line 伪样式
/// 行高 0px。返回收集到的文本 run 行高序列（按文档序）。
fn collect_lh_sequence(first_line_lh_px: f64, pseudo_font_size_px: Option<f64>) -> Vec<f32> {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();
    let t1 = doc.create_text_node("This is a first line.");
    doc.append_child(div, t1).unwrap();
    let br = doc.create_element("br");
    doc.append_child(div, br).unwrap();
    let t2 = doc.create_text_node("This is a second line.");
    doc.append_child(div, t2).unwrap();

    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::Block;
    cs.line_height = LineHeightValue::Length(LengthValue::Px(40.0));
    let mut fl = ComputedStyle::default();
    fl.line_height = LineHeightValue::Length(LengthValue::Px(first_line_lh_px));
    if let Some(fs) = pseudo_font_size_px {
        fl.font_size = LengthValue::Px(fs);
    }
    cs.first_line_pseudo = Some(Box::new(fl));

    let mut styles = std::collections::HashMap::new();
    styles.insert(div, cs);

    let ctx = InlineFormattingContext::new(800.0);
    ctx.collect_inline_items(&doc, div, &styles)
        .into_iter()
        .filter_map(|item| match item {
            InlineItem::Text(run) => Some(run.line_height),
            _ => None,
        })
        .collect()
}

/// ★load-bearing：首行域 run 行高 = 伪样式行高（0），br 后次行 run 回容器行高
///（40）——与 ref 页 span 包裹变体（span 行高 0 直接命中）同槽同值。
#[test]
fn r5404_first_line_lh_applies_until_forced_break() {
    let lhs = collect_lh_sequence(0.0, None);
    assert_eq!(
        lhs,
        vec![0.0, 40.0],
        "首行 run 行高应为伪样式 0px，br 后次行回容器 40px，got {lhs:?}"
    );
}

/// color-only 页（伪样式行高 = 容器行高，c25-pseudo-elmnt 族同构）零操作——
/// 解析行高相同不激活，run 行高保持容器值。
#[test]
fn r5404_first_line_lh_equal_value_is_noop() {
    let lhs = collect_lh_sequence(40.0, None);
    assert_eq!(lhs, vec![40.0, 40.0], "伪样式行高与容器相同时不应激活覆盖，got {lhs:?}");
}

/// 伪样式 font-size 与容器不一致（001/002 字号域）不激活——切片 3a 仅覆盖
/// 行高位差异族，字号域留 FIXME 切片 3b（全段仅施行高会错置）。
#[test]
fn r5404_first_line_lh_font_size_domain_excluded() {
    let lhs = collect_lh_sequence(0.0, Some(40.0));
    assert_eq!(
        lhs,
        vec![40.0, 40.0],
        "伪样式 font-size 与容器不一致时不应激活覆盖，got {lhs:?}"
    );
}

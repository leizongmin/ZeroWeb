//! R5317（CSS2 §10.8.1 + §10.3.2）：va:text-* 原子盒行盒围合 = margin box 并集 +
//! 基线锚定（text-bottom 底 margin 边 = B+D_parent / text-top 顶 margin 边 = B−A_parent）+
//! 后续行 restack + img 原子 border-box 推进口径 + 原子盒中断可折叠空白序列。
//!
//! 驱动案 css/CSS2/css1/c44-ln-box-000/001（chromium 对端模型已验证）：
//! - c44-001（fs 10/20，strut 盒 [3,23] 基线 13，A_parent 8 / D_parent 2）：img1
//!   text-bottom（60×60 + margin 15）margin box 底锚 15 → [−75,15]；img2 text-top
//!   margin box 顶锚 5 → [5,95]；strut 并集 → 行盒 170、扩展后基线 88；行 1 y=20、
//!   行 2 restack y=190（20+170）。旧缺 Arms：行盒 110、基线 58（reftest ✗21.12%）。
//! - c44-000（40px img 无 margin）：margin box [−25,15]/[5,45] 并集 strut → 行盒 70、
//!   基线 38；final 系 img 盒 y0/y30。
//! - img 原子参排尺寸 = border box（content + padding + border）：存面 build（水平
//!   Path B 空 inline_block_sizes 表）旧用 content 尺寸 → 与 layout 树（taffy border
//!   box）分裂（30 vs 60 → 行盒 110 vs 170）。
//! - 「a <img> b」双空格分居原子两侧 = 两个独立可折叠序列（css-text-3 §4.1），均存活。

use crate::types::InlineLayoutLine;

/// 收集（首个）存树 IFC 与宿主盒高。
fn find_ifc(node: &crate::LayoutBox) -> Option<(&Vec<InlineLayoutLine>, f32)> {
    if let Some(lines) = &node.inline_layout {
        return Some((lines, node.content_height));
    }
    node.children.iter().find_map(find_ifc)
}

fn layout(html: &str, css: &str) -> crate::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 主判例（c44-ln-box-001 语义）：text-bottom/text-top 原子 margin box 围合行盒 170、
/// 基线锚 88、行 restack y=20/190；旧实现行盒 110、基线 58、行 2 y=130。
#[test]
fn r5317_text_bottom_top_atom_margin_box_union_line_box() {
    let css = "div { font: 10px/20px Ahem; width: 39em; padding: 0.5em 1em; border: 1em solid green; } \
               img { width: 3em; height: 3em; padding: 0.5em; border-style: solid; border-width: 1em; margin: 1.5em; } \
               .a { vertical-align: text-bottom; } .b { vertical-align: text-top; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body><div>xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx<img class="a" />xxxx xxxx<img class="b" />xxxx xxxx xxxx</div></body></html>"#
    );
    let result = layout(&html, css);
    let (lines, div_h) = find_ifc(&result.root).expect("div 应有存树 IFC");

    assert_eq!(lines.len(), 3, "应 3 行（文本行/原子行/文本行），实得 {lines:?}");
    let (l0, l1, l2) = (&lines[0], &lines[1], &lines[2]);
    // 行 0：纯 strut 行（fs10/lh20）。
    assert!(
        (l0.y - 0.0).abs() < 0.01 && (l0.height - 20.0).abs() < 0.01,
        "行0 y/h 应 0/20，实得 {}/{}",
        l0.y,
        l0.height
    );
    assert!(
        (l0.baseline_y - 13.0).abs() < 0.01,
        "行0 基线应 13，实得 {}",
        l0.baseline_y
    );
    // 行 1：原子行 = strut [3,23] ∪ img1 mb [−75,15] ∪ img2 mb [5,95] → y=20、h=170、基线 88。
    assert!(
        (l1.y - 20.0).abs() < 0.01,
        "行1 y 应 20（行0 高 restack），实得 {}",
        l1.y
    );
    assert!(
        (l1.height - 170.0).abs() < 0.01,
        "行1 高应 170（margin box 并集），实得 {}",
        l1.height
    );
    assert!(
        (l1.baseline_y - 88.0).abs() < 0.01,
        "行1 基线应 88（13+top_extend 75），实得 {}",
        l1.baseline_y
    );
    // 行 2：restack = 行0 + 行1 final 高。
    assert!(
        (l2.y - (l0.height + l1.height)).abs() < 0.01 && (l2.y - 190.0).abs() < 0.01,
        "行2 y 应 190（20+170 restack），实得 {}",
        l2.y
    );
    // 原子盒 run.y 基线锚定：img1 border-box 顶 = B+D_parent−mb_h+mt = 88+2−90+15 = 15；
    // img2 = B−A_parent+mt = 88−8+15 = 95（空 text 片段即原子 run）。
    let atom_ys: Vec<f32> = l1.fragments.iter().filter(|f| f.text.is_empty()).map(|f| f.y).collect();
    assert!(
        atom_ys.iter().any(|y| (y - 15.0).abs() < 0.01) && atom_ys.iter().any(|y| (y - 95.0).abs() < 0.01),
        "原子 run y 应含 15/95（基线锚定），实得 {atom_ys:?}"
    );
    assert!(
        (div_h - 210.0).abs() < 0.01,
        "div 内容高应 210（3 行 20+170+20），实得 {div_h}"
    );
}

/// 对照判例（c44-ln-box-000 语义）：40px 无 margin img，text-bottom mb [−25,15] /
/// text-top mb [5,45] 并集 strut → 行盒 70、基线 38、final 系原子盒 y0/y30。
#[test]
fn r5317_unmargined_atom_union_line_box_70() {
    let css = "div { font: 10px/20px Ahem; width: 500px; } \
               img { width: 40px; height: 40px; } \
               .a { vertical-align: text-bottom; } .b { vertical-align: text-top; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body><div><img class="a" /><img class="b" /></div></body></html>"#
    );
    let result = layout(&html, css);
    let (lines, div_h) = find_ifc(&result.root).expect("div 应有存树 IFC");

    assert_eq!(lines.len(), 1, "应单行，实得 {lines:?}");
    let l = &lines[0];
    assert!((l.height - 70.0).abs() < 0.01, "行盒高应 70，实得 {}", l.height);
    assert!(
        (l.baseline_y - 38.0).abs() < 0.01,
        "基线应 38（13+25），实得 {}",
        l.baseline_y
    );
    let atom_ys: Vec<f32> = l.fragments.iter().filter(|f| f.text.is_empty()).map(|f| f.y).collect();
    assert!(
        atom_ys.iter().any(|y| (y - 0.0).abs() < 0.01) && atom_ys.iter().any(|y| (y - 30.0).abs() < 0.01),
        "原子盒 y 应 0/30（B+D−h / B−A），实得 {atom_ys:?}"
    );
    assert!((div_h - 70.0).abs() < 0.01, "div 高应 70，实得 {div_h}");
}

/// img 原子参排推进 = border box：content 30 + padding 5×2 + border 10×2 = 60。
/// 「a<img>b」的 b 字形 x = 10+60 = 70（旧 content 口径 40 → b x=50）。
#[test]
fn r5317_img_atom_advance_is_border_box() {
    let css = "div { font: 10px/20px Ahem; width: 500px; } \
               img { width: 30px; height: 30px; padding: 5px; border-style: solid; border-width: 10px; }";
    let html = format!(r#"<html><head><style>{css}</style></head><body><div>a<img />b</div></body></html>"#);
    let result = layout(&html, css);
    let (lines, _) = find_ifc(&result.root).expect("div 应有存树 IFC");
    let b = lines[0]
        .fragments
        .iter()
        .find(|f| f.text.contains('b'))
        .expect("b 片段");
    assert!(
        (b.x - 70.0).abs() < 0.01,
        "b 片段 x 应 70（a 10 + img border box 60），实得 {}",
        b.x
    );
}

/// 原子盒中断可折叠空白序列（css-text-3 §4.1）：「a <img> b」两空格分居原子两侧，
/// 均存活 → b 片段起点 ≥ a+空格+原子+空格 = 90（旧实现第二空格被并入跨原子序列吞掉 → 80）。
#[test]
fn r5317_atomic_inline_breaks_collapsible_ws_sequence() {
    let css = "div { font: 10px/20px Ahem; width: 500px; } \
               img { width: 30px; height: 30px; padding: 5px; border-style: solid; border-width: 10px; }";
    let html = format!(r#"<html><head><style>{css}</style></head><body><div>a <img /> b</div></body></html>"#);
    let result = layout(&html, css);
    let (lines, _) = find_ifc(&result.root).expect("div 应有存树 IFC");
    let b = lines[0]
        .fragments
        .iter()
        .find(|f| f.text.contains('b'))
        .expect("b 片段");
    // b 片段含前导空格时 x=80 宽 20；不含时 x=90 宽 10——两形态 b 字形起点均须 90。
    let b_glyph_x = if b.text.starts_with(' ') { b.x + 10.0 } else { b.x };
    assert!(
        (b_glyph_x - 90.0).abs() < 0.01,
        "b 字形 x 应 90（两空格均存活），实得片段 x={} w={} text={:?}",
        b.x,
        b.width,
        b.text
    );
}

/// img_atom_border_box_extra 单元判例：content-box 声明补 padding+border；
/// border-box 声明豁免（taffy 已含）；border-style none/hidden 侧宽不计。
#[test]
fn r5317_img_atom_border_box_extra_unit() {
    use zero_css_parser::values::BoxSizingValue;
    use zero_style_system::ComputedStyle;
    use zero_style_system::property::types::BorderStyleValue;

    let mut s = ComputedStyle::default();
    s.padding_left = zero_css_parser::values::LengthValue::Px(5.0);
    s.padding_right = zero_css_parser::values::LengthValue::Px(5.0);
    s.padding_top = zero_css_parser::values::LengthValue::Px(5.0);
    s.padding_bottom = zero_css_parser::values::LengthValue::Px(5.0);
    s.border_left_width = zero_css_parser::values::LengthValue::Px(10.0);
    s.border_right_width = zero_css_parser::values::LengthValue::Px(10.0);
    s.border_top_width = zero_css_parser::values::LengthValue::Px(10.0);
    s.border_bottom_width = zero_css_parser::values::LengthValue::Px(10.0);
    s.border_left_style = BorderStyleValue::Solid;
    s.border_right_style = BorderStyleValue::Solid;
    s.border_top_style = BorderStyleValue::Solid;
    s.border_bottom_style = BorderStyleValue::Solid;
    assert_eq!(
        crate::inline::img_atom_border_box_extra(&s),
        (30.0, 30.0),
        "content-box：extra = padding+border"
    );
    // border-box 声明豁免。
    s.box_sizing = BoxSizingValue::BorderBox;
    assert_eq!(
        crate::inline::img_atom_border_box_extra(&s),
        (0.0, 0.0),
        "border-box 声明尺寸已含 padding+border，不重补"
    );
    // none 侧宽不计（CSS2 §8.5：border-style none → border-width 计算值 0）。
    s.box_sizing = BoxSizingValue::ContentBox;
    s.border_top_style = BorderStyleValue::None;
    assert_eq!(
        crate::inline::img_atom_border_box_extra(&s),
        (30.0, 20.0),
        "border-style none 侧不计宽"
    );
}

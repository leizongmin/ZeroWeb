//! R5342（css-overflow-4 §block-ellipsis）：line-clamp 截断后**整行内容被 ellipsis
//! 置换**的行盒只含 strut——行盒高度取容器 used line-height（R5298 捕获），不依赖
//! 被置换内容高度。
//!
//! 驱动案 block-ellipsis-018：容器 `font: 25px Ahem` 宽 100px、`line-clamp: 1`，
//! span 1.5em（37.5px）的 9 字长词独占首行（ZW 词内不可断 → 单 min-content 单元
//! 溢出行）。ellipsis 宽 25px，保留首单元仍放不下 → 整行置换 → 行盒 37.5→25、
//! 基线 20（half-leading 0 + Ahem ascent 0.8×25）。kill-switch `ZW_CLAMP_STRUT`
//!（进程内 set_var 不可靠，R3858 教训——OFF 轮由 corpus 覆盖，不设单测）。

use crate::types::InlineLayoutLine;

/// 收集（首个）存树 IFC 与宿主块盒高。
///
/// R1576 嵌套存储形态：clamp 容器（div.clamp）自身的 IFC 存于其递归 inline 子盒
///（span），宿主盒高须取**携带 IFC 子盒的父块**（block-ellipsis-018 的背景高度
/// 语义在 div 盒上）；r5298 直存形态（IFC 存块容器自身）走自身分支。
fn find_ifc(node: &crate::LayoutBox) -> Option<(&Vec<InlineLayoutLine>, f32)> {
    if let Some(lines) = &node.inline_layout {
        return Some((lines, node.content_height));
    }
    for child in &node.children {
        if let Some(lines) = &child.inline_layout {
            return Some((lines, node.content_height));
        }
        if let Some(found) = find_ifc(child) {
            return Some(found);
        }
    }
    None
}

fn compute(html: &str, css: &str) -> crate::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 主判例（block-ellipsis-018 同构）：溢出长词整行被置换，行盒 25/基线 20，
/// 容器高 25。修复前行盒 37.5（被置换 span 的 1.5em 行高）。
#[test]
fn r5342_fully_displaced_line_takes_strut_height() {
    let css = ".clamp { width: 100px; font: 25px Ahem; line-clamp: 1; } \
               .large { font-size: 1.5em; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div class="clamp"><span class="large">XXXXXXXXX XXX</span></div>
</body></html>"#
    );
    let result = compute(&html, css);
    let (lines, div_h) = find_ifc(&result.root).expect("clamp div 应有存树 IFC");

    assert_eq!(lines.len(), 1, "clamp:1 应保留 1 行，实得 {lines:?}");
    let l = &lines[0];
    assert!(
        (l.height - 25.0).abs() < 0.01,
        "整行置换行盒高应 25（strut = 容器 used line-height），实得 {}",
        l.height
    );
    assert!(
        (l.baseline_y - 20.0).abs() < 0.01,
        "基线应 20（Ahem strut ascent 0.8×25），实得 {}",
        l.baseline_y
    );
    assert!((div_h - 25.0).abs() < 0.01, "容器高应 25，实得 {div_h}");
}

/// 边界守卫：首单元 + ellipsis 恰好放得下（75 + 25 == 100）→ 不整行置换，
/// 行盒保持内容高度 37.5（判据为严格大于，边界不触发）。
#[test]
fn r5342_first_unit_fitting_ellipsis_keeps_content_height() {
    let css = ".clamp { width: 100px; font: 25px Ahem; line-clamp: 1; } \
               .large { font-size: 1.5em; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div class="clamp"><span class="large">OK XXXXXXXXX</span></div>
</body></html>"#
    );
    let result = compute(&html, css);
    let (lines, _) = find_ifc(&result.root).expect("clamp div 应有存树 IFC");

    assert_eq!(lines.len(), 1, "clamp:1 应保留 1 行，实得 {lines:?}");
    // 行盒保持内容高度（span 1.5em normal 行高 37.5 经 R5298 同源 normal 整数化
    // 为 37；容差 1.0 只锚「未触发置换」量级，不绑整数化方向）。
    assert!(
        (lines[0].height - 37.0).abs() < 1.0,
        "首单元 OK(75px) + ellipsis(25px) 恰满 100 → 非整行置换，行盒应保持内容高 37，实得 {}",
        lines[0].height
    );
}

/// 负向守卫：ellipsis 放得下的常规 clamp（无溢出长词）行盒逐行不变——
/// 保留行高度仍由内容决定（corpus 零扰动锚，r5298 同款三行文案）。
#[test]
fn r5342_fitting_clamp_lines_unchanged() {
    let css = "p { width: 300px; font: 16px Ahem; line-clamp: 2; margin: 0 }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body><p>hello world enough text to wrap two lines here</p></body></html>"#
    );
    let result = compute(&html, css);
    let (lines, p_h) = find_ifc(&result.root).expect("p 应有存树 IFC");

    assert_eq!(lines.len(), 2, "clamp:2 应保留 2 行，实得 {lines:?}");
    for (i, l) in lines.iter().enumerate() {
        assert!(
            (l.height - 16.0).abs() < 0.01,
            "L{i} 行盒高应 16（ellipsis 放得下，不置换），实得 {}",
            l.height
        );
    }
    assert!((p_h - 32.0).abs() < 0.01, "p 高应 32（2×16），实得 {p_h}");
}

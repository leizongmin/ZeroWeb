//! R1280：[inline 内容 + float] 容器的 inline 内容绕 float 流动（CSS §9.5）。
//!
//! floats-006 谱系：非 BFC 容器 `#div1` 含 inline `<span>X</span>` + 2 个 float:left 子。
//! 期望：inline 文本 X 绕到 float 右侧（x≈200，2×100px 左 float 占 [0,200]）。
//!
//! 旧实现（R1280 前）：`has_direct_paintable_text` 把 blockified float（display:Block +
//! float≠none）误计为 block-level → 容器 paint_text 早退 → X 经 span 自身 Path B 在非
//! float-excluded 位（x≈0）渲染；且 `compute_final_inline_layouts` 的 `is_pure_ahem`
//! 守卫使非纯 Ahem 容器（div1 用默认字体）不存 IFC。R1280 协调修复：① float 子不计为
//! block（容器 paint_text 跑）+ ② 含 float 排除的容器存 IFC（Path A 真实 styles → 折叠
//! inline 子 is_ahem_font=true + render v_offset is_ahem 分支正确）。详见 master.md R1280。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::FloatValue;
use zero_style_system::StyleSystem;

/// 找到第一个含 float 子元素的、指定宽度的块级容器（模拟 floats-006 的 `#div1`）。
fn find_float_container(root: &LayoutBox, width: f32) -> Option<&LayoutBox> {
    if (root.width - width).abs() < 0.5 && root.children.iter().any(|c| !matches!(c.float, FloatValue::None)) {
        return Some(root);
    }
    for child in &root.children {
        if let Some(f) = find_float_container(child, width) {
            return Some(f);
        }
    }
    None
}

/// R1280 ②（Path A 存储）：含 float 排除的容器（默认字体 div1 + Ahem span）须存 IFC，
/// 且折叠的 inline 文本片段落 float-excluded 位（x≈200）+ is_ahem_font=true。
///
/// 旧实现：div1.inline_layout = None（is_pure_ahem 守卫跳过非纯 Ahem 容器）。
#[test]
fn test_float_container_stores_ifc_with_excluded_inline() {
    let html = "<html><body style=\"margin:0\"><div style=\"height:200px;width:300px\">\n   <span style=\"font:100px/1 Ahem\">X</span>\n   <div style=\"float:left;width:100px;height:100px\"></div>\n   <div style=\"float:left;width:100px;height:100px\"></div>\n</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div1 = find_float_container(&result.root, 300.0).expect("should find #div1 (w=300 with float children)");

    // ②：含 float 排除的容器须存 IFC（旧实现 is_pure_ahem 守卫跳过 → None）。
    let inline_layout = div1
        .inline_layout
        .as_ref()
        .expect("float-container must store IFC (R1280 Path A for [inline+float])");

    // 找到文本 "X" 的片段（折叠自 span）。
    let x_frag = inline_layout
        .iter()
        .flat_map(|line| line.fragments.iter())
        .find(|f| f.text.contains('X'))
        .expect("should find folded 'X' fragment in div1 IFC");

    // X 须绕到 2×100px 左 float 右侧（x≈200，CSS §9.5 inline 内容绕 float 流动）。
    // 旧实现（无 R1280）：X 在 x≈0（与 float 重叠）。
    assert!(
        x_frag.x >= 190.0,
        "folded inline text must be float-excluded (x≈200); got x={}",
        x_frag.x
    );
    // 真实 styles → 折叠 inline 子 is_ahem_font=true（Path B override maps 不传播 is_ahem）。
    assert!(
        x_frag.is_ahem_font,
        "folded inline text must use real font metrics (is_ahem_font=true); got false"
    );
}

//! R5245：float 宿主 R109 匿名块 float 旗标清除 + 匿名块片段 intrinsic 片段级测量锚点。
//!
//! 修前两处叠加损坏 bidi-breaking-001 set 4（float 内裸文本 + 空块子 + 裸文本）：
//! ① R109 匿名块 taffy→dom 映射宿主（tree.rs `taffy_to_dom.insert(anon, host)`），
//!    converter 按宿主样式解析出 float → LayoutBox 旗标误带 float=Left，
//!    adjust_float_positions 把匿名块当真 float 并排放置并按 Σ 收缩宿主宽、
//!    remeasure_text_with_float_exclusions 误建伪排除带重测宿主文本
//!    （float 盒 133×55 塌陷 250×15、文本逸出盒外）。
//! ② R4489 片段测量只接 Definite 宽，taffy intrinsic 查询（float shrink-to-fit 的
//!    MaxContent）落回 measure_text_content 文本节点臂——整串 shaped 测量对混排
//!    文本系统性偏窄（「א + - × ÷」59.4 应 ~107）且多节点片段只测首个文本子，
//!    float 宿主收缩过窄、片段多折行（74 宽 3 行/段，应 132 宽单行）。
//! 修后（CSS2 §9.2.1.1：匿名块为流内盒不浮动；§9.5 shrink-to-fit 以内容固有宽为准）：
//! 匿名块 float=None、宿主宽 = max(段 max-content)、匿名块堆叠不并排。
//!
//! kill-switch：`ZW_R109_ANON_NOFLOAT=0` 回退①；`ZW_FRAGMENT_MEASURE=0` 回退整个
//! R4489 片段测量（含②）。OFF 反证以 reftest bidi-breaking-001 实证
//!（分别 ✗1.65% / ✗0.96%，双开 ✓0.00%）；单测进程内不设 env。

use super::*;

/// 收集 float 盒的直接匿名块子（R109 片段 node_id 映射宿主 → node_id 与宿主相同）。
fn anon_fragment_children(box_node: &LayoutBox) -> Vec<&LayoutBox> {
    box_node
        .children
        .iter()
        .filter(|c| c.node_id == box_node.node_id)
        .collect()
}

/// R5245 锚①：float 宿主 + 裸文本 + 空块子 + 裸文本——匿名块不继承宿主 float 旗标，
/// 宿主宽 = max(段 max-content)（Ahem 1em/字：3 字段 = 300px），匿名块堆叠不并排。
#[test]
fn r5245_float_host_anon_blocks_inflow_not_refloated() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font:100px/1 Ahem; float:left\">\
  aaa\
  <div></div>\
  bbb\
</div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找 float 盒（body 直下首块）
    let float_box = result
        .root
        .children
        .iter()
        .flat_map(|c| &c.children)
        .find(|b| !matches!(b.float, zero_css_parser::values::FloatValue::None))
        .expect("float 盒存在");
    assert!(
        matches!(float_box.float, zero_css_parser::values::FloatValue::Left),
        "宿主自身须保留 float=Left"
    );

    // 修复①：shrink-to-fit = max(段 max-content) = 300（非修复②前文本节点臂偏窄值）。
    assert!(
        (float_box.width - 300.0).abs() < 1.0,
        "float 宽应 = 段 max-content 300，实 {}",
        float_box.width
    );
    // 两段各单行：盒高 = 2 行 × 100。
    assert!(
        (float_box.height - 200.0).abs() < 1.0,
        "float 高应 = 两单行段 200，实 {}",
        float_box.height
    );

    // 修复②：匿名块子不携带宿主 float 旗标（流内盒，CSS2 §9.2.1.1）。
    let anons = anon_fragment_children(float_box);
    assert_eq!(anons.len(), 2, "两个匿名块片段子");
    for anon in &anons {
        assert!(
            matches!(anon.float, zero_css_parser::values::FloatValue::None),
            "匿名块须 float=None（不继承宿主旗标）"
        );
    }
    // 堆叠不并排：x 同列、y 递增（0 / 100），各单行高 100。
    assert!(
        (anons[0].x - anons[1].x).abs() < 0.5 && anons[0].x.abs() < 0.5,
        "匿名块应同列堆叠，实 x=({},{})",
        anons[0].x,
        anons[1].x
    );
    assert!(
        anons[1].y > anons[0].y + 50.0,
        "匿名块应纵向堆叠，实 y=({},{})",
        anons[0].y,
        anons[1].y
    );
    for anon in &anons {
        assert!(
            (anon.height - 100.0).abs() < 1.0,
            "匿名块应单行高 100，实 {}",
            anon.height
        );
    }
}

/// R5245 锚①b（修复②专属）：多节点片段（文本+inline 元素+文本）的 intrinsic 测量
/// 须按片段 IFC 全节点累计——文本节点臂回退只测首个文本子（300），float 收缩过窄。
#[test]
fn r5245_multi_node_fragment_intrinsic_covers_all_items() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font:100px/1 Ahem; float:left\">\
  aaa<span>bb</span>ccc\
  <div></div>\
  ddd\
</div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_box = result
        .root
        .children
        .iter()
        .flat_map(|c| &c.children)
        .find(|b| !matches!(b.float, zero_css_parser::values::FloatValue::None))
        .expect("float 盒存在");
    assert!(
        (float_box.width - 800.0).abs() < 1.0,
        "float 宽应 = 多节点段 max-content 800（3+2+3 字），实 {}（文本节点臂只测首文本子会得 300）",
        float_box.width
    );
}

/// R5245 锚②（非 float 宿主回归守）：普通块容器混排的匿名块本就 float=None，
/// 修复不得改变既有 R109 装配（几何 = 常规块流）。
#[test]
fn r5245_non_float_host_anon_blocks_unchanged() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font:100px/1 Ahem; width:800px\">\
  aaa\
  <div style=\"height:50px\"></div>\
  bbb\
</div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let host = result
        .root
        .children
        .iter()
        .flat_map(|c| &c.children)
        .find(|b| b.is_block_level)
        .expect("块宿主存在");
    let anons = anon_fragment_children(host);
    assert_eq!(anons.len(), 2, "两个匿名块片段子");
    for anon in &anons {
        assert!(
            matches!(anon.float, zero_css_parser::values::FloatValue::None),
            "非 float 宿主的匿名块 float=None"
        );
    }
    // 常规块流：两匿名块分列空块子前后（y=0 / y=150），各单行高 100。
    assert!(
        (anons[0].y - 0.0).abs() < 1.0 && (anons[1].y - 150.0).abs() < 1.0,
        "匿名块 y 应 0/150，实 ({},{})",
        anons[0].y,
        anons[1].y
    );
    for anon in &anons {
        assert!(
            (anon.height - 100.0).abs() < 1.0,
            "匿名块应单行高 100，实 {}",
            anon.height
        );
    }
}

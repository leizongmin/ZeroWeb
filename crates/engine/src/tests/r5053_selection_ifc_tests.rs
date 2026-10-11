//! R5053：IFC 文本路径 ::selection 行级高亮（css-pseudo-4 §highlight-selectors）。
//!
//! textarea 无 retained 值的可见文本走 paint_text 的 IFC 路径（R5052 只接了
//! paint_input_value 的 attr/retained 路径）。驱动：selection-textarea-011
//! （select() 全选 + ::selection 黄底绿字 vs 静态黄底绿字 ref）。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// 全选 + ::selection 规则 → padding box 高亮 fill + 行字形伪样式 color。
#[test]
fn textarea_selection_highlights_ifc_text() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#ta".to_string(), 0, 18)]);
    let result = pipeline.render_html(
        r#"<body><textarea id="ta" style="background-color: red; color: yellow; width: 20ch;">Selected Selected </textarea></body>"#,
        "textarea::selection { background-color: yellow; color: green }",
    );
    let prims = result.primitives();
    // 高亮 fill：伪样式黄底一枚（盖红底）。
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(255, 255, 0, 255)),
        "全选 textarea 应有一枚 ::selection 高亮 fill"
    );
    // 字形全部绿色（::selection color 覆盖元素黄）；页面仅 textarea 文本。
    assert!(
        prims.glyphs.len() >= 16,
        "value 18 字符应出字形，实际 {}",
        prims.glyphs.len()
    );
    assert!(
        prims.glyphs.iter().all(|g| g.color == Color::rgba(0, 128, 0, 255)),
        "选中行字形应全绿，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}

/// 无选区注入 → 零高亮、字形保持元素色（零新红面）。
#[test]
fn textarea_without_selection_unchanged() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(
        r#"<body><textarea id="ta" style="background-color: red; color: yellow; width: 20ch;">Selected Selected </textarea></body>"#,
        "textarea::selection { background-color: yellow; color: green }",
    );
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color == Color::rgba(255, 255, 0, 255)),
        "无选区 → 无高亮 fill"
    );
    assert!(
        prims.glyphs.iter().all(|g| g.color == Color::rgba(255, 255, 0, 255)),
        "无选区 → 字形保持元素色"
    );
}

/// R5054：普通元素（div）::selection —— getSelection().addRange() 回流路径（shim
/// `__zw_dump_dom_selections` 同形记录），高亮 = 逐行片段横向范围（非全宽）。
#[test]
fn div_selection_highlights_fragment_extent() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#d".to_string(), 0, 5)]);
    let result = pipeline.render_html(
        r#"<body><div id="d" style="color: yellow; font: 50px Ahem;">Hello</div></body>"#,
        "div::selection { background-color: green; color: white }",
    );
    let prims = result.primitives();
    // 高亮 fill 绿一枚（片段横向范围）。
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "div 选中应有片段范围高亮 fill"
    );
    // 字形白色（::selection color）。
    assert!(
        prims.glyphs.iter().all(|g| g.color == Color::rgba(255, 255, 255, 255)),
        "选中行字形应为伪样式色"
    );
    // 高亮宽度 < 内容宽（片段范围非全宽条——div 宽默认撑满 400px）。
    if let Some(f) = prims.fills.iter().find(|f| f.color == Color::rgba(0, 128, 0, 255)) {
        assert!(
            f.rect.size.width < 390.0,
            "高亮应为片段范围而非全宽，实际 {}",
            f.rect.size.width
        );
    }
}

// ── R5054：inline-block relative 双重计数回归（converter 清空 taffy inset + 后处理臂
// 唯一施加；highlight-z-index-001 top:1.25em 位移 2×60px 实证）──────────────

/// inline-block position:relative top:60px 应单次偏移 +60（双重计数时 +120）。
#[test]
fn inline_block_relative_offset_applied_once() {
    let html =
        |extra: &str| format!(r#"<body><p>x</p><div id="rel" style="display: inline-block;{extra}">R</div></body>"#);
    let rel_node_id = |h: &str| {
        let doc = zero_dom::parse_html(h);
        doc.get_elements_by_tag_name("div")
            .iter()
            .copied()
            .find(|&e| doc.get_attribute(e, "id").as_deref() == Some("rel"))
            .expect("rel div")
    };
    let box_y = |h: &str, nid: zero_dom::NodeId| -> f32 {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(h, "");
        fn walk(box_node: &zero_layout_engine::LayoutBox, nid: zero_dom::NodeId) -> Option<f32> {
            if box_node.node_id == Some(nid) {
                return Some(box_node.y);
            }
            for c in &box_node.children {
                if let Some(y) = walk(c, nid) {
                    return Some(y);
                }
            }
            None
        }
        walk(&result.layout.root, nid).expect("rel box in layout tree")
    };
    let base_html = html("");
    let offset_html = html(" position: relative; top: 60px;");
    let static_y = box_y(&base_html, rel_node_id(&base_html));
    let y60 = box_y(&offset_html, rel_node_id(&offset_html));
    assert!(
        ((y60 - static_y) - 60.0).abs() < 0.5,
        "inline-block relative top:60px 应单次偏移 +60（双重计数 +120）：static {static_y} vs {y60}"
    );
}

// ── R5055：nbsp-only 文本子可绘制（CSS Text §3 保留内容；trim() 按 Unicode
// White_Space 剔除 U+00A0 曾致元素零渲染）──────────────

/// `<div>&nbsp;</div>` 应产出 nbsp 字形（div::selection 高亮同帧可用）。
#[test]
fn nbsp_only_text_child_renders() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#d".to_string(), 0, 1)]);
    let result = pipeline.render_html(
        r#"<body><div id="d" style="color: green; font: 100px/1 Ahem;">&nbsp;</div></body>"#,
        "div::selection { background-color: green; }",
    );
    let prims = result.primitives();
    // Ahem nbsp = 1em 方块字形（选择高亮与字形同为绿，fills 不强断言）。
    assert!(
        prims.glyphs.iter().any(|g| g.glyph_id == 0x00A0_u32),
        "nbsp 字形应入图元，实际 glyphs={}",
        prims.glyphs.len()
    );
}

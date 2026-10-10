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

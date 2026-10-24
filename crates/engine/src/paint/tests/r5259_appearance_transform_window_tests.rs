//! R5259（appearance widget 纳入 per-node 效果窗口——CSS Transforms 1
//! §transform-rendering 变换作用于元素全部渲染输出）：paint_appearance 旧在
//! paint_node_inner 效果窗口之后发射，translate 路径（R3901 图元级平移）只平移
//! 窗口内图元——transform-input-017：UA 背景随 translateX/Y 移动而边框环/accent
//! glyph 留在原位。移入窗口后 widget 全部图元随变换整体移动。

use crate::pipeline::RenderPipeline;

/// transform 作用于 input widget 全部图元（背景 + 边框环 + accent）。
#[test]
fn transform_translates_appearance_widget_primitives_whole() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        r#"<body><input type="checkbox" checked style="transform: translateX(40px); margin: 10px"></body>"#,
        "",
    );
    let prims = result.primitives();
    assert!(!prims.fills.is_empty(), "checkbox widget 应发射图元");
    let unshifted: Vec<_> = prims
        .fills
        .iter()
        .filter(|f| f.rect.origin.x < 40.0)
        .map(|f| (f.rect.origin.x, f.rect.origin.y, f.rect.size.width, f.rect.size.height))
        .collect();
    assert!(
        unshifted.is_empty(),
        "margin 10px + translateX(40px) 下全部图元 x 应 ≥ 40（随变换整体移动），\
         留在原位的图元 = 效果窗口外发射（回归形态），实际 {unshifted:?}"
    );
}

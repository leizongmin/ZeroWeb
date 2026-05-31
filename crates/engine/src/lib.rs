//! # zero-engine
//!
//! 页面内核 — HTML/DOM/CSSOM/样式/布局/绘制/脚本协调。
//!
//! 整合各子模块，实现完整的页面加载和渲染管线。
//!
//! ## 核心模块
//!
//! - [`paint`] — 绘制命令生成，将布局盒树转换为渲染图元
//! - [`dirty`] — 脏区域追踪，管理需要重绘的屏幕区域
//! - [`composite`] — 合成层逻辑，决定元素图层分配
//! - [`pipeline`] — 端到端渲染管线，编排 HTML→CSS→Layout→Paint

#![warn(missing_docs)]

pub mod composite;
pub mod dirty;
pub mod paint;
pub mod pipeline;

pub use composite::*;
pub use dirty::*;
pub use paint::*;
pub use pipeline::*;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use zero_css_parser::values::ColorValue;
    use zero_layout_engine::LayoutBox;
    use zero_layout_engine::types::OverflowClip;
    use zero_render_foundation::color::Color;
    use zero_style_system::ComputedStyle;

    use crate::composite::promote_compositing_layers;
    use crate::paint::{Painter, color_value_to_render, hsla_to_rgba};
    use crate::pipeline::RenderPipeline;

    /// 测试空文档调用 paint 不 panic 且返回空图元。
    #[test]
    fn test_paint_empty_document() {
        let doc = zero_dom::Document::new();
        let layout = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 800.0,
            content_height: 600.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };
        let styles = HashMap::new();

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        assert!(
            painter.primitives().is_empty(),
            "empty document should produce no render ops"
        );
    }

    /// 测试单个 div 元素经样式和布局计算后，合成层至少返回一个图层。
    #[test]
    fn test_composite_single_box() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");

        let child_box = LayoutBox {
            node_id: Some(elem),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 200.0,
            content_height: 100.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        styles.insert(elem, ComputedStyle::default());

        let layers = promote_compositing_layers(&child_box, &styles);
        assert!(!layers.is_empty(), "composite should return at least one layer");
        assert!(layers[0].is_root, "first layer should be root");
    }

    /// 测试渲染管线在样式变化后重新计算样式，脏标记触发重新计算。
    #[test]
    fn test_pipeline_recompute_style() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><div class=\"box\">Content</div></body></html>";

        // 首次渲染：无 CSS
        let first = pipeline.render_html(html, "");
        assert!(pipeline.layout().is_some());

        // 修改样式：添加背景色
        let doc = zero_dom::parse_html(html);
        let css = ".box { background-color: red; width: 200px; height: 100px; }";
        let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
        let (prims, _styles, _layout) = pipeline.recompute_styles(&doc, &stylesheets);

        assert!(
            !prims.fills.is_empty(),
            "style recompute should produce fills after dirty change"
        );
        assert!(
            prims.fills.len() > first.primitives.fills.len(),
            "adding background-color should increase fill count"
        );
    }

    /// 测试 crimson 色值 Rgba(220,20,60,255) 正确转换为渲染 Color。
    #[test]
    fn test_named_color_crimson_render() {
        let crimson = ColorValue::Rgba(220, 20, 60, 255);
        let color = color_value_to_render(&crimson);
        assert_eq!(color.r, 220);
        assert_eq!(color.g, 20);
        assert_eq!(color.b, 60);
        assert_eq!(color.a, 255);
    }

    /// 测试 hsla_to_rgba(0, 100, 50, 1.0) 生成正确的纯红 RGBA 值。
    #[test]
    fn test_hsla_to_rgba_pure_red() {
        let color = hsla_to_rgba(0.0, 100.0, 50.0, 1.0);
        assert_eq!(color.r, 255, "pure red R should be 255");
        assert_eq!(color.g, 0, "pure red G should be 0");
        assert_eq!(color.b, 0, "pure red B should be 0");
        assert_eq!(color.a, 255, "alpha=1.0 should map to 255");
    }
}

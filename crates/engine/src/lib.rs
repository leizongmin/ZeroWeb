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

    // ── 边界条件测试 ──────────────────────────────────────────

    /// 测试渲染管线基本流程：简单文档经 style + layout + paint 后产生渲染图元。
    ///
    /// 创建含 div 的 HTML 文档，通过 render_html 执行完整管线，
    /// 验证生成的填充图元和布局结果均有效。
    #[test]
    fn test_render_pipeline_basic() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = r#"<html><body><div class="box">Hello World</div></body></html>"#;
        let css = r#".box { background-color: #336699; width: 200px; height: 100px; }"#;

        let result = pipeline.render_html(html, css);

        // 管线应完成且布局缓存存在
        assert!(pipeline.layout().is_some(), "layout should be cached after render");
        assert!(result.layout.viewport_width > 0.0, "viewport width should be positive");

        // CSS 为 div 生成背景填充图元
        assert!(
            !result.primitives.fills.is_empty(),
            "pipeline should produce fill primitives for styled div"
        );

        // 计时信息有效
        assert!(result.timings.total_ms >= 0.0);
        assert!(result.timings.style_ms >= 0.0);
        assert!(result.timings.layout_ms >= 0.0);
        assert!(result.timings.paint_ms >= 0.0);
    }

    /// 测试样式变化后脏标记触发重新渲染。
    ///
    /// 首次渲染无 CSS 的文档，然后通过 recompute_styles 添加背景色样式，
    /// 验证第二次渲染产生的填充图元数量严格大于第一次。
    #[test]
    fn test_dirty_tracking_after_style_change() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = r#"<html><body><div class="target">Content</div></body></html>"#;

        // 首次渲染：无 CSS 背景
        let first = pipeline.render_html(html, "");
        let first_fill_count = first.primitives.fills.len();

        // 重新解析文档并添加背景色样式
        let doc = zero_dom::parse_html(html);
        let css = r#".target { background-color: red; width: 200px; height: 100px; }"#;
        let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
        let (prims, _styles, _layout) = pipeline.recompute_styles(&doc, &stylesheets);

        // 样式变化应产生更多填充图元
        assert!(
            !prims.fills.is_empty(),
            "recomputed styles should produce fills after adding background-color"
        );
        assert!(
            prims.fills.len() > first_fill_count,
            "style change should increase fill count: {} > {}",
            prims.fills.len(),
            first_fill_count,
        );

        // 布局缓存应更新
        assert!(pipeline.layout().is_some());
    }

    /// 测试两个重叠盒子不同 z-index 的合成排序。
    ///
    /// 创建两个重叠元素，分别设置 z-index=1 和 z-index=10，
    /// 验证合成后非根图层按 z-index 升序排列，
    /// 高 z-index 图层排在低 z-index 之后（后绘制 = 视觉在上层）。
    #[test]
    fn test_composite_z_index_ordering() {
        use crate::composite::promote_compositing_layers;
        use zero_layout_engine::types::OverflowClip;
        use zero_style_system::property::ZIndexValue;

        let mut doc = zero_dom::Document::new();
        let elem_low = doc.create_element("div");
        let elem_high = doc.create_element("div");

        let child_low = LayoutBox {
            node_id: Some(elem_low),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
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
        let child_high = LayoutBox {
            node_id: Some(elem_high),
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
            content_x: 50.0,
            content_y: 50.0,
            content_width: 100.0,
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
        let root_box = LayoutBox {
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
            children: vec![child_low, child_high],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = std::collections::HashMap::new();
        let mut style_low = ComputedStyle::default();
        style_low.z_index = ZIndexValue::Integer(1);
        styles.insert(elem_low, style_low);

        let mut style_high = ComputedStyle::default();
        style_high.z_index = ZIndexValue::Integer(10);
        styles.insert(elem_high, style_high);

        let layers = promote_compositing_layers(&root_box, &styles);

        // 根图层 + 2 个提升图层
        assert_eq!(layers.len(), 3, "root + 2 promoted layers");
        assert!(layers[0].is_root);

        // z-index 升序：1 在前（先绘制/底层），10 在后（后绘制/上层）
        assert_eq!(layers[1].z_index, 1, "first promoted layer should be z=1");
        assert_eq!(layers[2].z_index, 10, "second promoted layer should be z=10");
        assert!(
            layers[2].z_index > layers[1].z_index,
            "higher z-index should render after (on top of) lower z-index"
        );
    }

    /// 测试 border-radius 元素的 paint 生成正确的渲染图元。
    ///
    /// 创建带 border-radius 和背景色的元素，验证 paint 生成填充图元，
    /// 且填充矩形尺寸和颜色正确。圆角信息在当前架构下通过内部元数据标记。
    #[test]
    fn test_paint_border_radius_clip() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = LayoutBox {
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

        let mut styles = std::collections::HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(100, 149, 237, 255); // cornflower blue
        style.border_top_left_radius = zero_css_parser::values::LengthValue::Px(15.0);
        style.border_top_right_radius = zero_css_parser::values::LengthValue::Px(15.0);
        style.border_bottom_right_radius = zero_css_parser::values::LengthValue::Px(15.0);
        style.border_bottom_left_radius = zero_css_parser::values::LengthValue::Px(15.0);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        // 带圆角的背景仍生成填充图元
        assert_eq!(
            painter.primitives().fills.len(),
            1,
            "border-radius element should produce exactly 1 fill"
        );
        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.size.width, 200.0, "fill width should match element width");
        assert_eq!(fill.rect.size.height, 100.0, "fill height should match element height");
        assert_eq!(fill.color.r, 100);
        assert_eq!(fill.color.g, 149);
        assert_eq!(fill.color.b, 237);
        assert_eq!(fill.color.a, 255);
    }

    /// 测试 coral、tomato、steelblue 命名颜色转换为正确的 RGBA 渲染颜色。
    ///
    /// CSS 解析器将命名颜色在解析时转换为 Rgba 值，
    /// 验证通过 color_value_to_render 正确传播到渲染 Color。
    #[test]
    fn test_named_color_render_conversion() {
        // coral → Rgba(255, 127, 80, 255)
        let coral = ColorValue::Rgba(255, 127, 80, 255);
        let color = color_value_to_render(&coral);
        assert_eq!(color.r, 255, "coral R should be 255");
        assert_eq!(color.g, 127, "coral G should be 127");
        assert_eq!(color.b, 80, "coral B should be 80");
        assert_eq!(color.a, 255, "coral A should be 255");

        // tomato → Rgba(255, 99, 71, 255)
        let tomato = ColorValue::Rgba(255, 99, 71, 255);
        let color = color_value_to_render(&tomato);
        assert_eq!(color.r, 255, "tomato R should be 255");
        assert_eq!(color.g, 99, "tomato G should be 99");
        assert_eq!(color.b, 71, "tomato B should be 71");
        assert_eq!(color.a, 255, "tomato A should be 255");

        // steelblue → Rgba(70, 130, 180, 255)
        let steelblue = ColorValue::Rgba(70, 130, 180, 255);
        let color = color_value_to_render(&steelblue);
        assert_eq!(color.r, 70, "steelblue R should be 70");
        assert_eq!(color.g, 130, "steelblue G should be 130");
        assert_eq!(color.b, 180, "steelblue B should be 180");
        assert_eq!(color.a, 255, "steelblue A should be 255");
    }
}

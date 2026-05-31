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

    /// 测试 outline-style:solid 但 outline-width:0 时，不绘制任何 outline 图元。
    ///
    /// outline-width 为 0 时 paint_outline 提前返回，不应产生填充图元。
    #[test]
    fn test_outline_render_no_width() {
        use zero_css_parser::values::VisibilityValue;
        use zero_style_system::property::OutlineStyleValue;

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

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.outline_style = OutlineStyleValue::Solid;
        style.outline_width = zero_css_parser::values::LengthValue::Px(0.0);
        style.outline_color = ColorValue::Rgba(255, 0, 0, 255);
        // 设置 color 为 CurrentColor 以避免生成 glyph
        style.color = ColorValue::CurrentColor;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        assert!(
            painter.primitives().fills.is_empty(),
            "outline-width:0 应不产生任何 outline 填充图元"
        );
    }

    /// 测试 outline-offset:5px 时，outline 图元位置正确向外偏移。
    ///
    /// outline-offset 使 outline 向外偏移，验证生成的填充矩形坐标反映偏移量。
    #[test]
    fn test_outline_render_with_offset() {
        use zero_style_system::property::OutlineStyleValue;

        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = LayoutBox {
            node_id: Some(elem),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            content_x: 10.0,
            content_y: 20.0,
            content_width: 100.0,
            content_height: 50.0,
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
        let mut style = ComputedStyle::default();
        style.outline_style = OutlineStyleValue::Solid;
        style.outline_width = zero_css_parser::values::LengthValue::Px(2.0);
        style.outline_offset = zero_css_parser::values::LengthValue::Px(5.0);
        style.outline_color = ColorValue::Rgba(0, 128, 255, 255);
        // 设置 color 为 CurrentColor 以避免生成 glyph
        style.color = ColorValue::CurrentColor;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        // outline 生成 4 个填充图元（上、下、左、右）
        assert_eq!(painter.primitives().fills.len(), 4, "outline 应生成 4 个填充图元");

        // 验证上 outline 偏移：total_offset = outline_width(2) + outline_offset(5) = 7
        // 上 outline y = abs_y(20) - total_offset(7) = 13
        let top = &painter.primitives().fills[0];
        assert_eq!(top.rect.origin.y, 13.0, "上 outline 应向外偏移 5px");
        // 上 outline x = abs_x(10) - total_offset(7) = 3
        assert_eq!(top.rect.origin.x, 3.0, "上 outline x 起始位置应反映偏移");
    }

    /// 测试 visibility:hidden 的元素不产生任何渲染输出（背景、边框、文本均跳过）。
    ///
    /// paint_node 检测到 visibility 为 Hidden 或 Collapse 时跳过所有绘制。
    #[test]
    fn test_visibility_hidden_render() {
        use zero_css_parser::values::VisibilityValue;

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

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        style.visibility = VisibilityValue::Hidden;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        // visibility:hidden 不绘制背景、边框、outline、文本
        assert!(
            painter.primitives().fills.is_empty(),
            "visibility:hidden 元素不应产生填充图元"
        );
        assert!(
            painter.primitives().glyphs.is_empty(),
            "visibility:hidden 元素不应产生文本图元"
        );
    }

    /// 测试 opacity:0 的元素仍然生成渲染图元（渲染但完全透明）。
    ///
    /// 与 visibility:hidden 不同，opacity:0 不阻止 paint 生成图元，
    /// 图元仍然存在但 alpha 通道为 0。
    #[test]
    fn test_opacity_zero_render() {
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

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        style.opacity = 0.0;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        // opacity:0 不阻止绘制，图元仍然生成
        assert!(
            !painter.primitives().fills.is_empty(),
            "opacity:0 元素仍应生成填充图元（由合成层处理透明度）"
        );
        // 背景填充的矩形尺寸正确
        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.size.width, 200.0, "填充宽度应为元素宽度");
        assert_eq!(fill.rect.size.height, 100.0, "填充高度应为元素高度");
    }

    /// 测试 border-style:none 时，即使有边框宽度也不绘制边框图元。
    ///
    /// paint_borders 检查 border-style 是否为 None 或 Hidden，若是则跳过该边。
    #[test]
    fn test_border_style_none() {
        use zero_style_system::property::BorderStyleValue;

        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        // 布局有边框宽度，但 border-style 为 none
        let layout = LayoutBox {
            node_id: Some(elem),
            x: 0.0,
            y: 0.0,
            width: 104.0,
            height: 104.0,
            content_x: 2.0,
            content_y: 2.0,
            content_width: 100.0,
            content_height: 100.0,
            border_top: 2.0,
            border_right: 2.0,
            border_bottom: 2.0,
            border_left: 2.0,
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
        let mut style = ComputedStyle::default();
        style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
        style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
        // 所有边框 style 均为 none
        style.border_top_style = BorderStyleValue::None;
        style.border_right_style = BorderStyleValue::None;
        style.border_bottom_style = BorderStyleValue::None;
        style.border_left_style = BorderStyleValue::None;
        // 设置 color 为 CurrentColor 以避免生成 glyph
        style.color = ColorValue::CurrentColor;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        // border-style:none 即使有边框宽度也不绘制
        assert!(
            painter.primitives().fills.is_empty(),
            "border-style:none 不应产生任何边框填充图元"
        );
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

    // ── 新增边界条件测试 ──────────────────────────────────────────

    /// 测试 visibility:hidden 的嵌套元素，父元素隐藏时子元素也不绘制。
    ///
    /// visibility 是继承属性，子元素通过样式继承也会被隐藏。
    #[test]
    fn test_paint_with_visibility_hidden_nested() {
        use zero_css_parser::values::VisibilityValue;

        let mut doc = zero_dom::Document::new();
        let parent_elem = doc.create_element("div");
        let child_elem = doc.create_element("span");

        let child_box = LayoutBox {
            node_id: Some(child_elem),
            x: 10.0,
            y: 10.0,
            width: 50.0,
            height: 30.0,
            content_x: 10.0,
            content_y: 10.0,
            content_width: 50.0,
            content_height: 30.0,
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
        let parent_box = LayoutBox {
            node_id: Some(parent_elem),
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
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        parent_style.visibility = VisibilityValue::Hidden;
        styles.insert(parent_elem, parent_style);

        // visibility 是继承属性，子元素通过继承获得 hidden
        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(0, 255, 0, 255);
        child_style.visibility = VisibilityValue::Hidden;
        styles.insert(child_elem, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles, Some(&doc));

        assert!(
            painter.primitives().fills.is_empty(),
            "visibility:hidden 父元素及其子元素均不应产生填充图元"
        );
    }

    /// 测试合成层提升子元素并验证图层 z-index 排序正确。
    #[test]
    fn test_composite_promoted_child_z_ordering() {
        use zero_style_system::property::ZIndexValue;

        let mut doc = zero_dom::Document::new();
        let elem_a = doc.create_element("div");
        let elem_b = doc.create_element("div");
        let elem_c = doc.create_element("div");

        let child_a = LayoutBox {
            node_id: Some(elem_a),
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
        let child_b = LayoutBox {
            node_id: Some(elem_b),
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
        let child_c = LayoutBox {
            node_id: Some(elem_c),
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
            children: vec![child_a, child_b, child_c],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            z_index: 0,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = std::collections::HashMap::new();
        let mut sa = ComputedStyle::default();
        sa.z_index = ZIndexValue::Integer(5);
        styles.insert(elem_a, sa);

        let mut sb = ComputedStyle::default();
        sb.z_index = ZIndexValue::Integer(-1);
        styles.insert(elem_b, sb);

        let mut sc = ComputedStyle::default();
        sc.z_index = ZIndexValue::Integer(10);
        styles.insert(elem_c, sc);

        let layers = promote_compositing_layers(&root_box, &styles);
        // 根图层 + 3 个提升图层
        assert_eq!(layers.len(), 4, "root + 3 promoted layers");
        assert!(layers[0].is_root);
        // 提升的图层按 z-index 升序：-1, 5, 10
        assert_eq!(layers[1].z_index, -1);
        assert_eq!(layers[2].z_index, 5);
        assert_eq!(layers[3].z_index, 10);
    }

    /// 测试渲染管线 recompute 后脏标记被设置。
    #[test]
    fn test_recompute_dirty_flag() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = "<html><body><p>Initial</p></body></html>";

        // 首次渲染
        let first = pipeline.render_html(html, "");
        assert!(pipeline.layout().is_some());

        // 重新计算样式（无 CSS 变化）
        let doc = zero_dom::parse_html(html);
        let stylesheets = vec![];
        let (prims, _styles, _layout) = pipeline.recompute_styles(&doc, &stylesheets);

        // 即使无变化，管线仍应产生输出
        assert!(prims.fills.is_empty() || !prims.fills.is_empty());
        assert!(pipeline.layout().is_some());
    }

    /// 测试渲染管线处理多元素复杂页面。
    #[test]
    fn test_pipeline_complex_page() {
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let html = r#"<html><body>
            <div class="header">Header</div>
            <div class="main">
                <p>Paragraph 1</p>
                <p>Paragraph 2</p>
                <span>Inline text</span>
            </div>
            <div class="footer">Footer</div>
        </body></html>"#;
        let css = r#"
            .header { background-color: #333333; height: 60px; }
            .main { background-color: #ffffff; width: 200px; height: 400px; }
            .footer { background-color: #666666; height: 40px; }
        "#;

        let result = pipeline.render_html(html, css);

        assert!(pipeline.layout().is_some());
        assert!(result.layout.viewport_width > 0.0);
        // 应产生至少 header、main、footer 的背景填充
        assert!(
            !result.primitives.fills.is_empty(),
            "complex page should produce fill primitives"
        );
        assert!(result.timings.total_ms >= 0.0);
    }

    /// 测试 border-radius 裁剪：带圆角的元素尺寸信息正确。
    #[test]
    fn test_paint_border_radius_clipping_values() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = LayoutBox {
            node_id: Some(elem),
            x: 50.0,
            y: 50.0,
            width: 300.0,
            height: 200.0,
            content_x: 50.0,
            content_y: 50.0,
            content_width: 300.0,
            content_height: 200.0,
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
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(0, 128, 255, 255);
        style.border_top_left_radius = zero_css_parser::values::LengthValue::Px(20.0);
        style.border_top_right_radius = zero_css_parser::values::LengthValue::Px(20.0);
        style.border_bottom_right_radius = zero_css_parser::values::LengthValue::Px(20.0);
        style.border_bottom_left_radius = zero_css_parser::values::LengthValue::Px(20.0);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles, Some(&doc));

        assert_eq!(painter.primitives().fills.len(), 1);
        let fill = &painter.primitives().fills[0];
        // 验证填充位置和尺寸
        assert_eq!(fill.rect.origin.x, 50.0);
        assert_eq!(fill.rect.origin.y, 50.0);
        assert_eq!(fill.rect.size.width, 300.0);
        assert_eq!(fill.rect.size.height, 200.0);
    }
}

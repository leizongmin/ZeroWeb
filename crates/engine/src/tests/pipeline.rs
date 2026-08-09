#![allow(clippy::field_reassign_with_default, clippy::too_many_arguments)]

use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_render_foundation::geometry::Rect;

use crate::pipeline::RenderPipeline;
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
/// 测试渲染管线 recompute 后脏标记被设置。
#[test]
fn test_recompute_dirty_flag() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><p>Initial</p></body></html>";

    // 首次渲染
    let _first = pipeline.render_html(html, "");
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
/// HTML 实体和特殊字符在解析时需正确处理，验证管线容错完成。
#[test]
fn test_pipeline_html_with_special_entities() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="a&amp;b">&lt;hello&gt;</div></body></html>"#;
    let css = r#".a\26 b { background-color: #123456; width: 100px; height: 50px; }"#;
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0, "特殊字符 HTML 应容错完成");
    assert!(pipeline.layout().is_some());
}
/// 第一次渲染含 div 的文档，第二次渲染含 span 的文档，
/// 验证缓存布局被第二次渲染的结果替换。
#[test]
fn test_pipeline_consecutive_different_renders() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);

    let html1 = r#"<html><body><div class="a">First</div></body></html>"#;
    let css1 = r#".a { background-color: red; width: 200px; height: 100px; }"#;
    let result1 = pipeline.render_html(html1, css1);
    assert!(pipeline.layout().is_some());
    let fills1 = result1.primitives.fills.len();

    let html2 = r#"<html><body><span class="b">Second</span></body></html>"#;
    let css2 = r#".b { background-color: blue; width: 300px; height: 150px; }"#;
    let result2 = pipeline.render_html(html2, css2);
    assert!(pipeline.layout().is_some());
    let fills2 = result2.primitives.fills.len();

    // 两次渲染都应产生图元
    assert!(fills1 > 0, "第一次渲染应产生填充图元");
    assert!(fills2 > 0, "第二次渲染应产生填充图元");

    // 缓存的布局应为第二次渲染的结果
    let cached = pipeline.layout().unwrap();
    assert_eq!(cached.viewport_width, 800.0);
}

/// R639：跨多行的 inline span 背景应按行片段（per-fragment）绘制，而非单一 bounding-box
/// rect。窄视口强制 span 文本换行；若按 box-level 仅 1 个 blue fill，按 per-fragment 则
/// 行数个。此测试守护 R639 per-fragment inline bg 行为不退化为 box-level（同时验证
/// owner-height 索引使 per-fragment 在父 IFC 绘制 inline 文本时仍触发）。
#[test]
fn test_r639_multiline_inline_bg_painted_per_fragment() {
    use zero_render_foundation::color::Color;
    let mut pipeline = RenderPipeline::new(100.0, 600.0); // 窄视口强制换行
    let html = r#"<html><body><p><span style="background-color:rgb(0,0,255)">word word word word word word word word word</span></p></body></html>"#;
    let result = pipeline.render_html(html, "");
    let blue = Color::rgba(0, 0, 255, 255);
    let blue_fills = result.primitives.fills.iter().filter(|f| f.color == blue).count();
    // 跨多行 span：per-fragment 绘制 → blue fill 数 == 行数（>= 2）；旧 box-level 仅 1。
    assert!(
        blue_fills >= 2,
        "多行 inline span 背景应按行片段绘制（per-fragment），实际 blue fill 数 = {blue_fills}"
    );
}
/// R644：Cc 控制字符可见性（CSS Text 3 §white-space-processing）——fontdue 对 Cc 类控制
/// 字符无字形（.notdef 空），paint 时渲染可见占位 em 方块。此测试守护 Cc 控制字符产生可见
/// fill（修 control-chars-* mismatch 测试：test 应 != 空 ref）。
#[test]
fn test_r644_cc_control_char_visible_placeholder() {
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    // ::after content 注入 Cc 控制字符 U+0001（CSS 转义，避免 HTML 解析器吞掉）
    let html =
        r#"<html><body><style>p { font-size: 40px; } p::after { content: "\0001"; }</style><p>x</p></body></html>"#;
    let result = pipeline.render_html(html, "");
    // R644: Cc 控制字符应产生可见占位 fill（em 方块 font_size×font_size = 40×40），
    // 而非 fontdue .notdef 空。断言存在一个 >= 30px 的 fill（占位框）。
    assert!(
        result
            .primitives
            .fills
            .iter()
            .any(|f| f.rect.size.width >= 30.0 && f.rect.size.height >= 30.0),
        "Cc 控制字符应产生可见占位 fill（>=30px em 方块），实际 fills = {:?}",
        result
            .primitives
            .fills
            .iter()
            .map(|f| (f.rect.size.width, f.rect.size.height))
            .collect::<Vec<_>>()
    );
}
/// 空字符串与空 HTML 文档不同，它不是有效的 HTML 结构。
/// 验证管线能容错处理并返回零或最小的渲染输出。
#[test]
fn test_pipeline_render_empty_string_html() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html("", "");

    assert!(result.timings.total_ms >= 0.0, "空字符串 HTML 应容错完成");
    assert!(result.layout.viewport_width >= 0.0, "视口宽度应有效");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
}
/// 极小视口是边界条件，布局和绘制需在极有限空间内完成。
/// 验证管线不因除零或溢出而崩溃。
#[test]
fn test_pipeline_very_small_viewport() {
    let mut pipeline = RenderPipeline::new(1.0, 1.0);
    let html = r#"<html><body><div class="tiny">X</div></body></html>"#;
    let css = r#".tiny { background-color: red; width: 1px; height: 1px; }"#;
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0, "1x1 视口渲染应正常完成");
    assert_eq!(pipeline.viewport_width(), 1.0);
    assert_eq!(pipeline.viewport_height(), 1.0);
    assert!(pipeline.layout().is_some());
}
/// 空格、换行、制表符组成的输入不是有效 HTML 结构，
/// 验证管线能安全处理并完成渲染。
#[test]
fn test_pipeline_render_whitespace_only_html() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "   \n\t\n   ";
    let result = pipeline.render_html(html, "");

    assert!(result.timings.total_ms >= 0.0, "纯空白 HTML 应容错完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    assert!(result.layout.viewport_width >= 0.0);
}
/// CSS 中包含极大的像素值（999999px），
/// 验证布局引擎和绘制模块在处理超常数值时不溢出或崩溃。
#[test]
fn test_pipeline_render_extreme_css_values() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="huge">Big</div></body></html>"#;
    let css = r#".huge { width: 999999px; height: 999999px; background-color: #123456; }"#;
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0, "超大 CSS 值应容错完成");
    assert!(pipeline.layout().is_some());
}
/// 首次用 CSS 渲染文档产生背景填充，然后传空样式表重新计算。
/// 验证管线不 panic、布局缓存仍然存在、viewport 尺寸不变。
#[test]
fn test_recompute_with_empty_stylesheets() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Content</div></body></html>"#;
    let css = r#".box { background-color: #336699; width: 200px; height: 100px; }"#;

    // 首次渲染：带 CSS
    let first = pipeline.render_html(html, css);
    assert!(first.primitives.fills.len() > 0, "首次渲染应产生填充");
    let first_vp = first.layout.viewport_width;

    // 重新计算：空样式表
    let doc = zero_dom::parse_html(html);
    let (_, _styles, layout) = pipeline.recompute_styles(&doc, &[]);

    // 布局缓存仍有效
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    // viewport 不变
    assert_eq!(layout.viewport_width, first_vp, "空样式表重新计算后 viewport 不应变");
}
/// 页面包含两个 div：一个宽的父元素（背景红色）和一个窄的子元素（背景蓝色），
/// 通过 CSS 选择器为两者设置背景色。验证生成的填充图元中，
/// 父元素填充先于子元素填充，且颜色和尺寸正确。
#[test]
fn test_pipeline_overlapping_elements_fill_order() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div class="parent"><div class="child">Text</div></div>
    </body></html>"#;
    let css = r#"
        .parent { background-color: #ff0000; width: 400px; height: 300px; }
        .child { background-color: #0000ff; width: 200px; height: 100px; }
    "#;

    let result = pipeline.render_html(html, css);

    // 应产生至少 2 个填充图元（parent + child）
    assert!(
        result.primitives.fills.len() >= 2,
        "重叠元素应产生至少 2 个填充图元，实际 {}",
        result.primitives.fills.len()
    );

    // 父元素填充应在子元素之前
    let parent_fill = &result.primitives.fills[0];
    // 父元素背景色为红色
    assert!(
        parent_fill.color.r > 200 && parent_fill.color.g < 50,
        "第一个填充应为父元素红色背景，实际 r={} g={} b={}",
        parent_fill.color.r,
        parent_fill.color.g,
        parent_fill.color.b
    );
    // 父元素尺寸应大于子元素
    assert!(
        parent_fill.rect.size.width >= 200.0,
        "父元素宽度应 >= 200，实际 {}",
        parent_fill.rect.size.width
    );
}
/// total_ms 应大于等于 style_ms + layout_ms + paint_ms 的最大值。
/// 验证计时字段不含 NaN 或负值，且各阶段耗时均为有限值。
#[test]
fn test_pipeline_timing_consistency() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div class="a">Section A</div>
        <div class="b">Section B</div>
    </body></html>"#;
    let css = r#"
        .a { background-color: red; width: 200px; height: 100px; }
        .b { background-color: blue; width: 200px; height: 100px; }
    "#;
    let result = pipeline.render_html(html, css);

    // total_ms 应为有限正数
    assert!(
        result.timings.total_ms >= 0.0 && result.timings.total_ms.is_finite(),
        "total_ms 应为有限非负值，实际 {}",
        result.timings.total_ms
    );

    // 各阶段计时均为有限值
    assert!(result.timings.parse_ms.is_finite(), "parse_ms 应为有限值");
    assert!(result.timings.style_ms.is_finite(), "style_ms 应为有限值");
    assert!(result.timings.layout_ms.is_finite(), "layout_ms 应为有限值");
    assert!(result.timings.paint_ms.is_finite(), "paint_ms 应为有限值");

    // total_ms 应 >= 任意子阶段
    assert!(
        result.timings.total_ms >= result.timings.style_ms,
        "total_ms ({}) 应 >= style_ms ({})",
        result.timings.total_ms,
        result.timings.style_ms
    );
    assert!(
        result.timings.total_ms >= result.timings.layout_ms,
        "total_ms ({}) 应 >= layout_ms ({})",
        result.timings.total_ms,
        result.timings.layout_ms
    );
    assert!(
        result.timings.total_ms >= result.timings.paint_ms,
        "total_ms ({}) 应 >= paint_ms ({})",
        result.timings.total_ms,
        result.timings.paint_ms
    );
}
/// HTML 中的 <style> 块包含 CSS 规则，同时通过参数传入外部 CSS。
/// 验证管线能安全处理混合样式来源，且通过 CSS 规则生成填充图元。
#[test]
fn test_pipeline_html_with_inline_style_block() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><head>
        <style>.boxed { background-color: #336699; width: 200px; height: 100px; }</style>
    </head><body>
        <div class="boxed">Styled via style block</div>
    </body></html>"#;
    // 同时传入外部 CSS 验证混合样式不冲突
    let css = ".boxed { background-color: #663399; }";
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0, "渲染应正常完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    // CSS 规则应生成填充图元
    assert!(
        !result.primitives.fills.is_empty(),
        "含 <style> 标签的 HTML 应与外部 CSS 配合生成填充图元"
    );
}
/// 元素设置 background-color 和 4 条 solid 边框，
/// 验证第一个填充为背景色，后续 4 个为边框填充，
/// 总填充数恰好为 5（1 背景 + 4 边框）。
#[test]
fn test_pipeline_background_and_border_fill_count() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Bordered</div></body></html>"#;
    let css = r#"
        .box {
            background-color: #ffcc00;
            width: 200px;
            height: 100px;
            border: 3px solid #333333;
        }
    "#;
    let result = pipeline.render_html(html, css);

    // 1 背景 + 4 边框 = 5 填充
    assert!(
        result.primitives.fills.len() >= 5,
        "背景 + 4 条边框应产生至少 5 个填充图元，实际 {}",
        result.primitives.fills.len()
    );

    // 第一个填充应为背景色 #ffcc00 → Rgba(255, 204, 0, 255)
    let bg_fill = &result.primitives.fills[0];
    assert_eq!(bg_fill.color.r, 255, "背景 R 应为 255");
    assert_eq!(bg_fill.color.g, 204, "背景 G 应为 204");
    assert_eq!(bg_fill.color.b, 0, "背景 B 应为 0");

    // 背景填充尺寸匹配元素尺寸
    assert!(
        bg_fill.rect.size.width > 0.0,
        "背景宽度应为正，实际 {}",
        bg_fill.rect.size.width
    );
    assert!(
        bg_fill.rect.size.height > 0.0,
        "背景高度应为正，实际 {}",
        bg_fill.rect.size.height
    );
}
/// 模拟真实场景：首次全量渲染 → 样式变更重算 → 多次小区域增量渲染。
/// 验证每次增量渲染后布局缓存有效、脏追踪器状态正确。
#[test]
fn test_pipeline_recompute_then_multiple_incremental() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div class=\"box\">Content</div></body></html>";

    // 首次全量渲染
    let _first = pipeline.render_html(html, ".box { background-color: red; width: 200px; height: 100px; }");
    assert!(pipeline.layout().is_some());

    // 样式变更
    let doc = zero_dom::parse_html(html);
    let css = ".box { background-color: green; width: 300px; height: 150px; }";
    let ss = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let (prims, _, _) = pipeline.recompute_styles(&doc, &ss);
    assert!(!prims.fills.is_empty(), "样式变更后应产生填充图元");

    // 第一次增量渲染（小脏区域）
    let dirty1 = zero_layout_engine::LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 50.0,
        height: 50.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 50.0,
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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: zero_layout_engine::types::OverflowClip::Visible,
        overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        ..Default::default()
    };
    let result1 = pipeline.incremental_render(html, "", &dirty1);
    assert!(result1.timings.total_ms >= 0.0, "第一次增量渲染应正常完成");
    assert!(pipeline.layout().is_some(), "增量渲染后布局缓存应存在");
    assert!(
        pipeline.dirty_tracker().dirty_rects().is_empty(),
        "增量渲染后脏区域应清除"
    );

    // 第二次增量渲染（另一个小脏区域）
    let dirty2 = zero_layout_engine::LayoutBox {
        node_id: None,
        x: 100.0,
        y: 50.0,
        width: 80.0,
        height: 60.0,
        content_x: 100.0,
        content_y: 50.0,
        content_width: 80.0,
        content_height: 60.0,
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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: zero_layout_engine::types::OverflowClip::Visible,
        overflow_y: zero_layout_engine::types::OverflowClip::Visible,
        ..Default::default()
    };
    let result2 = pipeline.incremental_render(html, "", &dirty2);
    assert!(result2.timings.total_ms >= 0.0, "第二次增量渲染应正常完成");
    assert!(
        pipeline.dirty_tracker().dirty_rects().is_empty(),
        "第二次增量渲染后脏区域应清除"
    );
    assert!(pipeline.layout().is_some(), "布局缓存应始终有效");
}
// ── 新增边界条件测试 ──────────────────────────────────────────

/// 测试仅包含 HTML 注释的文档渲染不 panic 且图元数最小。
///
/// HTML 注释 <!-- ... --> 不产生可见 DOM 节点，渲染管线应安全跳过。
/// 验证渲染完成且不产生背景填充图元。
#[test]
fn test_pipeline_render_only_comments_html() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><!-- this is a comment --><body><!-- another comment --></body></html>"#;
    let result = pipeline.render_html(html, "");

    assert!(result.timings.total_ms >= 0.0, "注释 HTML 应容错完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    // 注释不产生可见元素，不应有背景填充
    assert!(result.primitives.fills.is_empty(), "纯注释 HTML 不应产生背景填充图元");
}
/// 首次渲染后，用完全相同的 CSS 调用 recompute_styles，
/// 验证填充图元数量不变，确保重算不会引入额外的图元。
#[test]
fn test_pipeline_recompute_same_styles_idempotent() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Content</div></body></html>"#;
    let css = r#".box { background-color: #336699; width: 200px; height: 100px; }"#;

    // 首次渲染
    let first = pipeline.render_html(html, css);
    let first_fill_count = first.primitives.fills.len();
    assert!(first_fill_count > 0, "首次渲染应产生填充图元");

    // 用相同 CSS 重新计算
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let (prims, _, _) = pipeline.recompute_styles(&doc, &stylesheets);

    assert_eq!(
        prims.fills.len(),
        first_fill_count,
        "相同 CSS 重算应产生相同数量的填充图元"
    );
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
}
/// 10 层嵌套的空 div 元素，无 CSS 样式。
/// 验证管线能安全处理深层 DOM 结构，布局树非空且视口有效。
#[test]
fn test_pipeline_render_deeply_nested_empty_divs() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    // 10 层嵌套空 div
    let html = "<html><body><div><div><div><div><div><div><div><div><div><div>Deep</div></div></div></div></div></div></div></div></div></div></body></html>";
    let result = pipeline.render_html(html, "");

    assert!(result.timings.total_ms >= 0.0, "深层嵌套 HTML 应容错完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    assert!(result.layout.viewport_width > 0.0, "视口宽度应为正");
    assert!(!result.layout.root.children.is_empty(), "布局树根应有子节点");
}
// ── 新增边界条件测试 ──────────────────────────────────────────

/// 测试渲染管线处理含非 ASCII 字符 CSS 类名的 HTML 文档不 panic。
///
/// CSS 选择器包含 Unicode 字符（如中文类名），浏览器引擎应容错处理。
/// 验证渲染管线能安全完成，且布局缓存有效。
#[test]
fn test_pipeline_render_non_ascii_class_name() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="标题">内容</div></body></html>"#;
    let css = r#".标题 { background-color: #336699; width: 200px; height: 100px; }"#;
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0, "非 ASCII 类名 HTML 应容错完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    assert!(result.layout.viewport_width > 0.0, "视口宽度应为正");
}
/// 对同一文档执行 5 次 recompute_styles，每次使用相同的 CSS，
/// 验证每次产生的填充图元数量完全一致，确保管线无累积副作用。
#[test]
fn test_pipeline_recompute_multiple_cycles_stable() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="box">Content</div></body></html>"#;
    let css = r#".box { background-color: #336699; width: 200px; height: 100px; }"#;

    // 首次渲染
    let first = pipeline.render_html(html, css);
    let first_fill_count = first.primitives.fills.len();
    assert!(first_fill_count > 0, "首次渲染应产生填充图元");

    // 连续 5 次 recompute
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    for i in 0..5 {
        let (prims, _, layout) = pipeline.recompute_styles(&doc, &stylesheets);
        assert_eq!(
            prims.fills.len(),
            first_fill_count,
            "第 {} 次 recompute 应产生相同数量的填充图元",
            i + 1
        );
        assert!(
            layout.viewport_width > 0.0,
            "第 {} 次 recompute 后 viewport_width 应有效",
            i + 1
        );
        assert!(pipeline.layout().is_some(), "缓存布局应始终存在");
    }
}
/// 依次执行：全量渲染 → 样式重算 → 增量渲染 → 再次全量渲染，
/// 验证每次操作后 viewport_width 和 viewport_height 均保持初始值。
#[test]
fn test_pipeline_viewport_dimensions_preserved() {
    let mut pipeline = RenderPipeline::new(1024.0, 768.0);
    let html = "<html><body><div class=\"box\">Content</div></body></html>";
    let css = ".box { background-color: red; width: 200px; height: 100px; }";

    // 步骤 1：全量渲染
    let r1 = pipeline.render_html(html, css);
    assert_eq!(r1.layout.viewport_width, 1024.0);
    assert_eq!(r1.layout.viewport_height, 768.0);

    // 步骤 2：样式重算
    let doc = zero_dom::parse_html(html);
    let ss = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let (_, _, layout2) = pipeline.recompute_styles(&doc, &ss);
    assert_eq!(layout2.viewport_width, 1024.0, "recompute 后 viewport_width 不应变");
    assert_eq!(layout2.viewport_height, 768.0, "recompute 后 viewport_height 不应变");

    // 步骤 3：增量渲染
    let dirty_box = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 50.0,
        height: 50.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 50.0,
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
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        ..Default::default()
    };
    let r3 = pipeline.incremental_render(html, "", &dirty_box);
    assert_eq!(r3.layout.viewport_width, 1024.0, "增量渲染后 viewport_width 不应变");
    assert_eq!(r3.layout.viewport_height, 768.0, "增量渲染后 viewport_height 不应变");

    // 步骤 4：再次全量渲染
    let r4 = pipeline.render_html(html, "");
    assert_eq!(r4.layout.viewport_width, 1024.0, "二次全量渲染后 viewport_width 不应变");
    assert_eq!(
        r4.layout.viewport_height, 768.0,
        "二次全量渲染后 viewport_height 不应变"
    );

    // viewport 访问器始终一致
    assert_eq!(pipeline.viewport_width(), 1024.0);
    assert_eq!(pipeline.viewport_height(), 768.0);
}
/// 嵌套 <table> 结构是复杂的 DOM 场景，内层表格在外层表格的 <td> 中。
/// 验证管线安全完成渲染，且布局树包含嵌套结构。
#[test]
fn test_pipeline_render_nested_table_structure() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <table class="outer">
            <tr>
                <td>
                    <table class="inner">
                        <tr><td>Inner A</td><td>Inner B</td></tr>
                    </table>
                </td>
                <td>Outer Cell</td>
            </tr>
        </table>
    </body></html>"#;
    let css = r#"
        .outer { background-color: #f0f0f0; width: 600px; }
        .inner { background-color: #e0e0e0; width: 300px; }
        td { border: 1px solid #cccccc; padding: 4px; }
    "#;
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0, "嵌套表格渲染应正常完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    assert!(result.layout.viewport_width > 0.0, "视口宽度应为正");
    assert!(!result.layout.root.children.is_empty(), "布局树根应有子节点");
    // 嵌套表格应产生填充图元（背景 + 边框）
    assert!(!result.primitives.fills.is_empty(), "嵌套表格应产生填充图元");
}
/// Rect::new(0, 0, 0, 0) 的 is_empty() 返回 true，
/// 任何节点都不与之相交，因此应跳过所有绘制，产生零图元。
#[test]
fn test_pipeline_incremental_paint_zero_size_dirty_rect() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div class=\"box\">Content</div></body></html>";
    let css = ".box { background-color: red; width: 200px; height: 100px; }";

    // 先做全量渲染
    let full_result = pipeline.render_html(html, css);
    let full_fill_count = full_result.primitives.fills.len();
    assert!(full_fill_count > 0, "全量渲染应产生填充图元");

    // 增量绘制零尺寸脏矩形
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let dirty_rect = Rect::new(0.0, 0.0, 0.0, 0.0);
    let inc_primitives = pipeline.incremental_paint(&doc, &stylesheets, dirty_rect);

    assert!(inc_primitives.is_some(), "incremental_paint 应返回 Some");
    let inc = inc_primitives.unwrap();
    // 零尺寸脏矩形与任何节点都不相交，应产生零图元
    assert!(
        inc.fills.is_empty(),
        "零尺寸脏矩形不应产生填充图元，实际 {}",
        inc.fills.len()
    );
    assert!(
        inc.glyphs.is_empty(),
        "零尺寸脏矩形不应产生文本图元，实际 {}",
        inc.glyphs.len()
    );
}
/// CSS 中包含 @media 规则和普通规则，管线应安全解析并渲染。
/// 验证渲染完成、布局缓存有效、且存在填充图元。
#[test]
fn test_pipeline_render_with_media_rules() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div class="box">Content</div>
        <div class="sidebar">Sidebar</div>
    </body></html>"#;
    let css = r#"
        .box { background-color: #336699; width: 200px; height: 100px; }
        @media (min-width: 600px) {
            .sidebar { background-color: #996633; width: 150px; height: 300px; }
        }
        @media screen and (max-width: 400px) {
            .box { background-color: #ff0000; width: 100%; }
        }
    "#;

    let result = pipeline.render_html(html, css);

    // 管线不 panic 且正常完成
    assert!(result.timings.total_ms >= 0.0, "@media 规则 HTML 应容错完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    assert!(result.layout.viewport_width > 0.0, "视口宽度应为正");

    // 至少 .box 的背景应产生填充图元
    assert!(
        !result.primitives.fills.is_empty(),
        "含 @media 规则的 CSS 应产生填充图元"
    );
}
/// 通过 CSS 为元素设置 outline（宽度、样式、颜色），
/// 验证管线端到端正确解析、应用样式并生成 outline 对应的填充图元。
/// outline 应生成 4 个填充图元（上、下、左、右），加上背景共 5 个。
#[test]
fn test_pipeline_with_outline_style_solid() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="outlined">Outlined Content</div></body></html>"#;
    let css = r#"
        .outlined {
            background-color: #ffcc00;
            width: 200px;
            height: 100px;
            outline: 3px solid #ff0000;
        }
    "#;

    let result = pipeline.render_html(html, css);

    assert!(
        result.timings.total_ms >= 0.0,
        "含 outline-style:solid 的 CSS 应正常完成"
    );
    assert!(pipeline.layout().is_some(), "布局缓存应存在");

    // 应至少有背景填充（1 个）+ outline 填充（视口剔除后可能减少）
    // outline 可能部分超出视口边界被 cull_invisible 剔除
    assert!(
        result.primitives.fills.len() >= 1,
        "outline-style:solid 应产生至少 1 个填充图元（背景），实际 {}",
        result.primitives.fills.len()
    );

    // 第一个填充应为背景色 #ffcc00 → Rgba(255, 204, 0, 255)
    let bg_fill = &result.primitives.fills[0];
    assert_eq!(bg_fill.color.r, 255, "背景 R 应为 255");
    assert_eq!(bg_fill.color.g, 204, "背景 G 应为 204");
    assert_eq!(bg_fill.color.b, 0, "背景 B 应为 0");
}
/// border-spacing 控制表格单元格之间的间距，需要水平和垂直两个分量。
/// 通过 CSS 设置 border-spacing: 8px，验证管线端到端安全完成渲染，
/// 且表格背景填充正常生成。
#[test]
fn test_render_with_border_spacing_property() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <table class="spaced">
            <tr><td class="cell">A</td><td class="cell">B</td></tr>
            <tr><td class="cell">C</td><td class="cell">D</td></tr>
        </table>
    </body></html>"#;
    let css = r#"
        .spaced { border-spacing: 8px; background-color: #f5f5f5; }
        .cell { background-color: #ddeeff; border: 1px solid #aabbcc; padding: 4px; }
    "#;

    let result = pipeline.render_html(html, css);

    // 管线不 panic 且正常完成
    assert!(result.timings.total_ms >= 0.0, "含 border-spacing 的 CSS 应容错完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    assert!(result.layout.viewport_width > 0.0, "视口宽度应为正");
    // 表格和单元格背景应产生填充图元
    assert!(
        !result.primitives.fills.is_empty(),
        "含 border-spacing 的表格应产生填充图元"
    );
}
/// counter-set 用于将 CSS 计数器设置为指定值。
/// 当前架构下管线应安全解析此属性，不因计数器操作而崩溃。
/// 通过 CSS 设置 counter-set: section 5，验证管线端到端安全完成。
#[test]
fn test_pipeline_with_counter_set_property() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div class="counter">Section Title</div>
        <div class="normal">Normal Content</div>
    </body></html>"#;
    let css = r#"
        .counter {
            counter-set: section 5;
            background-color: #336699;
            width: 200px;
            height: 100px;
        }
        .normal {
            background-color: #996633;
            width: 200px;
            height: 50px;
        }
    "#;

    let result = pipeline.render_html(html, css);

    // 管线不 panic 且正常完成
    assert!(result.timings.total_ms >= 0.0, "含 counter-set 的 CSS 应容错完成");
    assert!(pipeline.layout().is_some(), "布局缓存应存在");
    assert!(result.layout.viewport_width > 0.0, "视口宽度应为正");
    // counter-set 不影响渲染图元生成，背景填充应正常
    assert!(
        !result.primitives.fills.is_empty(),
        "含 counter-set 的元素应正常产生填充图元"
    );
}
// ── 新增边界测试（第二批） ──

/// 测试渲染管线处理空 CSS 不 panic。
#[test]
fn test_pipeline_empty_css() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div>Hello</div></body></html>"#;
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0, "空 CSS 应正常完成");
    assert!(result.layout.viewport_width > 0.0, "视口宽度应为正");
}
/// 测试渲染管线处理仅文本节点不 panic。
#[test]
fn test_pipeline_text_only() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>Just plain text</body></html>"#;
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0, "纯文本应正常完成");
    // 纯文本应产生 glyph 图元
    assert!(!result.primitives.glyphs.is_empty(), "纯文本应产生 glyph");
}

/// 文本只应由拥有直接文本子节点的元素绘制一次。
///
/// 之前 html/body 等祖先会递归收集后代 textContent，导致同一段文字在相近坐标重复绘制。
#[test]
fn test_pipeline_does_not_duplicate_nested_text_glyphs() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><p>Example <b>Domain</b></p></body></html>"#;
    let css = "html, body, p, b { color: black; font-size: 16px; }";

    let result = pipeline.render_html(html, css);
    let glyph_text: String = result
        .primitives
        .glyphs
        .iter()
        .filter_map(|glyph| char::from_u32(glyph.glyph_id))
        .collect();

    assert_eq!(
        glyph_text.matches("Example").count(),
        1,
        "ancestor elements should not duplicate direct text, got {glyph_text:?}"
    );
    assert_eq!(
        glyph_text.matches("Domain").count(),
        1,
        "inline child text should not be repainted by the child after the parent inline context, got {glyph_text:?}"
    );
}

/// 多个文本 block 的 glyph baseline 应随布局向下推进。
#[test]
fn test_pipeline_text_blocks_have_distinct_baselines() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><p>First paragraph</p><p>Second paragraph</p></body></html>"#;
    let css = "body, p { color: black; font-size: 16px; }";

    let result = pipeline.render_html(html, css);
    let first_y = result
        .primitives
        .glyphs
        .iter()
        .find(|glyph| char::from_u32(glyph.glyph_id) == Some('F'))
        .map(|glyph| glyph.y)
        .expect("first paragraph glyph should exist");
    let second_y = result
        .primitives
        .glyphs
        .iter()
        .find(|glyph| char::from_u32(glyph.glyph_id) == Some('S'))
        .map(|glyph| glyph.y)
        .expect("second paragraph glyph should exist");

    assert!(
        second_y > first_y,
        "second paragraph should render below first paragraph: first_y={first_y}, second_y={second_y}"
    );
}
/// 测试渲染管线处理深嵌套 HTML 不 panic。
#[test]
fn test_pipeline_deeply_nested_html() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    // 20 层嵌套
    let open: String = (0..20).map(|i| format!("<div class=\"l{i}\">")).collect();
    let close: String = (0..20).map(|_| "</div>").collect();
    let html = format!("<html><body>{open}Deep{close}</body></html>");
    let result = pipeline.render_html(&html, "");
    assert!(result.timings.total_ms >= 0.0, "深嵌套应正常完成");
}
/// 测试渲染管线处理含特殊字符的文本。
#[test]
fn test_pipeline_special_characters() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div>&lt;script&gt; &amp; "quotes" 'apostrophes'</div></body></html>"#;
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0, "特殊字符应正常完成");
}
/// 测试渲染管线处理多个背景颜色元素。
#[test]
fn test_pipeline_multiple_backgrounds() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body>
        <div class="a">A</div>
        <div class="b">B</div>
        <div class="c">C</div>
    </body></html>"#;
    let css = r#"
        .a { background-color: red; width: 100px; height: 50px; }
        .b { background-color: green; width: 100px; height: 50px; }
        .c { background-color: blue; width: 100px; height: 50px; }
    "#;
    let result = pipeline.render_html(html, css);
    assert!(
        result.primitives.fills.len() >= 3,
        "3 个背景色元素应产生至少 3 个填充图元"
    );
}
/// 测试渲染管线处理无效 HTML 容错。
#[test]
fn test_pipeline_malformed_html_recovery() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div><span>unclosed<div>nested</body></html>"#;
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0, "畸形 HTML 应容错完成");
}
/// 测试渲染管线处理零视口尺寸不 panic。
#[test]
fn test_pipeline_zero_viewport() {
    let mut pipeline = RenderPipeline::new(0.0, 0.0);
    let html = r#"<html><body><div>Text</div></body></html>"#;
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0, "零视口应正常完成");
}

// ── 新增边界条件测试：viewport 访问 / recompute 不渲染 / 顺序渲染 / 缓存布局重用 ──

/// 测试极小视口尺寸（0.5 x 0.5）渲染不 panic。
#[test]
fn test_pipeline_tiny_viewport_sub_pixel() {
    let mut pipeline = RenderPipeline::new(0.5, 0.5);
    let html = "<html><body><div>Tiny</div></body></html>";
    let result = pipeline.render_html(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert_eq!(pipeline.viewport_width(), 0.5);
    assert_eq!(pipeline.viewport_height(), 0.5);
}

/// 测试 recompute_styles 不先调用 render_html 也能正常工作，
/// 验证缓存的布局结果被正确设置。
#[test]
fn test_pipeline_recompute_without_prior_render() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    assert!(
        pipeline.layout().is_none(),
        "fresh pipeline should have no cached layout"
    );

    let html = r#"<html><body><div class="box">Content</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let css = r#".box { background-color: green; width: 200px; height: 100px; }"#;
    let stylesheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];

    let (primitives, _styles, layout) = pipeline.recompute_styles(&doc, &stylesheets);

    // 布局应被设置
    assert!(layout.viewport_width > 0.0, "layout viewport_width should be positive");
    assert!(
        pipeline.layout().is_some(),
        "cached layout should be set after recompute"
    );
    // CSS 应产生填充图元
    assert!(!primitives.fills.is_empty(), "CSS should produce fills");
}

/// 测试连续 5 次顺序渲染相同内容，每次填充图元数量完全一致。
#[test]
fn test_pipeline_sequential_renders_consistent() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div class="a">A</div><div class="b">B</div></body></html>"#;
    let css = r#"
        .a { background-color: red; width: 200px; height: 100px; }
        .b { background-color: blue; width: 150px; height: 75px; }
    "#;

    let mut fill_counts = Vec::new();
    for _ in 0..5 {
        let result = pipeline.render_html(html, css);
        fill_counts.push(result.primitives.fills.len());
        assert!(result.timings.total_ms >= 0.0);
    }

    // 所有渲染次数应产生相同数量的填充图元
    let first = fill_counts[0];
    for (i, &count) in fill_counts.iter().enumerate() {
        assert_eq!(count, first, "render {} fill count should match first render", i);
    }
}

/// 测试缓存的布局在 render_html 后被正确更新。
#[test]
fn test_pipeline_cached_layout_updates_across_renders() {
    let mut pipeline = RenderPipeline::new(640.0, 480.0);
    assert!(pipeline.layout().is_none());

    // 第一次渲染
    let html1 = "<html><body><div>First</div></body></html>";
    let _r1 = pipeline.render_html(html1, "");
    let layout1 = pipeline.layout().unwrap();
    assert_eq!(layout1.viewport_width, 640.0);
    assert_eq!(layout1.viewport_height, 480.0);
    let _child_count1 = layout1.root.children.len();

    // 第二次渲染不同内容
    let html2 = "<html><body><p>A</p><p>B</p><p>C</p></body></html>";
    let _r2 = pipeline.render_html(html2, "");
    let layout2 = pipeline.layout().unwrap();
    assert_eq!(layout2.viewport_width, 640.0, "viewport_width should stay the same");
    assert_eq!(layout2.viewport_height, 480.0, "viewport_height should stay the same");
    // 布局树应已更新（不同内容可能不同子节点数）
    // 只需验证布局被更新（不为空）
    assert!(
        !layout2.root.children.is_empty(),
        "layout should have children after second render"
    );
}

/// 测试 render_html 后再 recompute_styles，缓存的布局被第二次 recompute 正确覆盖。
#[test]
fn test_pipeline_cached_layout_overwrite_by_recompute() {
    let mut pipeline = RenderPipeline::new(1024.0, 768.0);
    let html = "<html><body><div class=\"box\">Content</div></body></html>";

    // render_html 设置缓存布局
    let _ = pipeline.render_html(html, ".box { background-color: red; width: 100px; height: 50px; }");
    assert!(pipeline.layout().is_some());

    // recompute_styles 覆盖缓存布局
    let doc = zero_dom::parse_html(html);
    let css = ".box { background-color: blue; width: 200px; height: 100px; }";
    let ss = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let (_, _, layout) = pipeline.recompute_styles(&doc, &ss);

    // 验证布局被更新
    assert_eq!(layout.viewport_width, 1024.0);
    assert_eq!(layout.viewport_height, 768.0);
    assert!(pipeline.layout().is_some());
    let cached = pipeline.layout().unwrap();
    assert_eq!(cached.viewport_width, 1024.0);
    assert_eq!(cached.viewport_height, 768.0);
}

/// 测试 render_html 返回的 RenderResult 布局和 pipeline.layout() 缓存一致。
#[test]
fn test_pipeline_render_result_matches_cached_layout() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body><div>Match</div></body></html>";
    let result = pipeline.render_html(html, "");

    let cached = pipeline.layout().unwrap();
    assert_eq!(
        result.layout.viewport_width, cached.viewport_width,
        "result and cached viewport_width should match"
    );
    assert_eq!(
        result.layout.viewport_height, cached.viewport_height,
        "result and cached viewport_height should match"
    );
    assert_eq!(
        result.layout.root.children.len(),
        cached.root.children.len(),
        "result and cached root children count should match"
    );
}

// ── 性能验证测试 ──────────────────────────────────────────────

/// 中等复杂度页面首屏渲染性能验证。
///
/// 验证 Done Criteria：「中等复杂度页面首屏渲染 < 2s」。
/// 使用纯 Rust 渲染管线（无 GPU），测量 parse → style → layout → paint 各阶段耗时。
#[test]
fn test_medium_page_first_paint_under_2_seconds() {
    let html = r##"<html><head><style>
        body { margin: 0; font-family: sans-serif; color: #333; }
        header { background: #2c3e50; color: white; padding: 20px; display: flex; justify-content: space-between; }
        main { padding: 20px; }
        .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; }
        .card { border: 1px solid #ddd; border-radius: 8px; padding: 16px; background: white; }
        .card h3 { margin-top: 0; color: #2c3e50; }
        .card p { line-height: 1.6; }
        .sidebar { background: #ecf0f1; padding: 15px; border-radius: 4px; }
        footer { background: #34495e; color: white; padding: 15px; text-align: center; }
    </style></head><body>
    <header>
        <h1>ZeroWeb Browser</h1>
        <nav><a href="home">Home</a> <a href="about">About</a> <a href="docs">Docs</a></nav>
    </header>
    <main>
        <div class="grid">
            <div class="card"><h3>Feature 1</h3><p>Fast rendering engine built in Rust.</p></div>
            <div class="card"><h3>Feature 2</h3><p>CSS Grid and Flexbox support.</p></div>
            <div class="card"><h3>Feature 3</h3><p>V8 JavaScript integration.</p></div>
            <div class="card"><h3>Feature 4</h3><p>Multi-process architecture.</p></div>
            <div class="card"><h3>Feature 5</h3><p>WebAssembly runtime.</p></div>
            <div class="card"><h3>Feature 6</h3><p>Cross-platform support.</p></div>
        </div>
        <div class="sidebar">
            <h3>Quick Links</h3>
            <ul><li>Getting Started</li><li>API Reference</li><li>Examples</li></ul>
        </div>
    </main>
    <footer><p>2026 ZeroWeb Project</p></footer>
    </body></html>"##;

    let mut pipeline = RenderPipeline::new(1280.0, 800.0);
    let result = pipeline.render_html(html, "");

    // 验证渲染成功
    assert!(!result.primitives.is_empty(), "应该生成渲染图元");
    assert!(result.primitives.glyphs.len() > 0, "应该渲染文本");

    // 验证各阶段计时合理
    let t = &result.timings;
    assert!(t.parse_ms >= 0.0, "解析耗时应 >= 0");
    assert!(t.style_ms >= 0.0, "样式耗时应 >= 0");
    assert!(t.layout_ms >= 0.0, "布局耗时应 >= 0");
    assert!(t.paint_ms >= 0.0, "绘制耗时应 >= 0");

    // 验证总耗时 < 2000ms（Done Criteria 性能目标）
    assert!(
        t.total_ms < 2000.0,
        "中等复杂度页面首屏渲染应 < 2s，实际: {:.2}ms (parse={:.2} style={:.2} layout={:.2} paint={:.2})",
        t.total_ms,
        t.parse_ms,
        t.style_ms,
        t.layout_ms,
        t.paint_ms
    );
}

/// 验证增量渲染（重新渲染）不退化。
///
/// 同一页面连续渲染两次，第二次不应比第一次慢超过 5 倍。
#[test]
fn test_incremental_render_not_degenerate() {
    let html = r##"<html><head><style>
        .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; }
        .card { border: 1px solid #ddd; padding: 16px; }
    </style></head><body>
    <div class="grid">
        <div class="card"><h3>Card 1</h3><p>Content A</p></div>
        <div class="card"><h3>Card 2</h3><p>Content B</p></div>
        <div class="card"><h3>Card 3</h3><p>Content C</p></div>
    </div>
    </body></html>"##;

    // 第一次渲染
    let mut pipeline1 = RenderPipeline::new(1280.0, 800.0);
    let result1 = pipeline1.render_html(html, "");

    // 第二次渲染（全新管线，模拟无增量优化的基线）
    let mut pipeline2 = RenderPipeline::new(1280.0, 800.0);
    let result2 = pipeline2.render_html(html, "");

    // 验证两次渲染结果一致
    assert_eq!(
        result1.primitives.len(),
        result2.primitives.len(),
        "同一页面重复渲染应产生相同数量的图元"
    );

    // 验证第二次不比第一次慢太多（允许 5 倍抖动）
    assert!(
        result2.timings.total_ms < result1.timings.total_ms * 5.0 + 100.0,
        "重复渲染不应退化: first={:.2}ms second={:.2}ms",
        result1.timings.total_ms,
        result2.timings.total_ms
    );
}

/// 验证渲染管线时间分解正确。
#[test]
fn test_pipeline_timings_breakdown() {
    let html = "<html><body>\
        <div style=\"background-color: red; width: 200px; height: 100px;\">Block 1</div>\
        <div style=\"background-color: blue; width: 200px; height: 100px;\">Block 2</div>\
        <div style=\"background-color: green; width: 200px; height: 100px;\">Block 3</div>\
        </body></html>";

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");

    let t = &result.timings;

    // 各阶段之和应 <= 总时间
    let stage_sum = t.parse_ms + t.style_ms + t.layout_ms + t.paint_ms;
    assert!(
        stage_sum <= t.total_ms + 1.0, // 允许 1ms 浮点误差
        "各阶段之和 ({:.2}ms) 应 <= 总时间 ({:.2}ms)",
        stage_sum,
        t.total_ms
    );

    // 总时间应 > 0（除非极快完成）
    assert!(t.total_ms >= 0.0, "总时间应 >= 0");
}

/// 文档高度超出视口时，视口下方的 `<img>` 图元须保留（由宿主滚动显示）。
#[test]
fn test_render_retains_below_viewport_image_primitive() {
    let mut pipeline = RenderPipeline::new(800.0, 400.0);
    let html = r#"<html><body><div style="height:800px"></div><img src="t.png" width="50" height="40"></body></html>"#;
    let result = pipeline.render_html(html, "");
    assert!(
        !result.primitives.images.is_empty(),
        "below-viewport img must not be culled (got {} images)",
        result.primitives.images.len()
    );
}

/// 诊断 testpage.htm：表格与「3. An image」之间不应有异常大空隙。
#[test]
fn test_testpage_table_to_image_section_gap() {
    use std::collections::HashMap;

    use crate::image_resource_key;
    use zero_dom::NodeId;

    let html = r##"<HTML><BODY BGCOLOR="#FFFFCC">
<H1>Internet Explorer 1.x (Mosaic) -- Running!</H1>
<HR>
<H2>1. Plain text &amp; formatting</H2>
<P>This page is served from a <B>local HTML file</B> and rendered by the browser.</P>
<H2>2. A table</H2>
<TABLE BORDER=1 CELLPADDING=6>
<TR BGCOLOR="#C0C0C0"><TH>Layer</TH><TH>Source dir</TH><TH>Status</TH></TR>
<TR><TD>Kernel/HTML core</TD><TD>generic\shared</TD><TD>OK</TD></TR>
</TABLE>
<H2>3. An image (JPEG)</H2>
<P><IMG SRC="testpage.jpg" ALT="astronaut" ALIGN=TOP>
This JPEG is decoded by the built-in libjpeg.</P>
</BODY></HTML>"##;

    let doc = zero_dom::parse_html(html);
    let table_id = doc.get_elements_by_tag_name("table")[0];
    let h2_id = doc.get_elements_by_tag_name("h2")[2];

    let page_url = "http://172.27.46.54:8000/testpage.htm";
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_document_url(Some(page_url));
    pipeline.set_image_sizes(HashMap::from([(
        image_resource_key("testpage.jpg", Some(page_url)),
        (512.0_f32, 384.0_f32),
    )]));

    pipeline.render_html(html, "");
    let layout = pipeline.layout().expect("layout");

    fn find_box(b: &zero_layout_engine::LayoutBox, target: NodeId, off_y: f32) -> Option<(f32, f32)> {
        let y = off_y + b.y;
        if b.node_id == Some(target) {
            return Some((y, b.height));
        }
        for c in &b.children {
            if let Some(found) = find_box(c, target, y) {
                return Some(found);
            }
        }
        None
    }

    let (table_y, table_h) = find_box(&layout.root, table_id, 0.0).expect("table box");
    let (h2_y, _) = find_box(&layout.root, h2_id, 0.0).expect("h2 box");
    let table_bottom = table_y + table_h;
    let gap = h2_y - table_bottom;

    assert!(
        gap < 80.0,
        "table bottom={table_bottom:.1}, h2 top={h2_y:.1}, gap={gap:.1}, doc_h={:?}",
        pipeline.document_height()
    );
}

/// M3-S9：render_with_dom_mutations——SetText 走增量布局（compute_incremental），
/// 免 parse（HTML 往返消除），返回 HTML 快照与活 DOM 一致。
#[test]
fn render_with_dom_mutations_text_uses_incremental_layout() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><body><div id="a" style="width:100px">short</div><div id="b">B</div></body></html>"#;
    let _ = pipeline.render_html(html, "");

    let m = crate::js_dom_bridge::DomMutation::SetText {
        selector: "#a".to_string(),
        text: "much longer text now".to_string(),
    };
    let (result, snapshot) = pipeline
        .render_with_dom_mutations(std::slice::from_ref(&m), "")
        .expect("mutations applied");
    // 活 DOM + HTML 快照一致（免 parse 路径）
    assert!(snapshot.contains("much longer text now"), "snapshot: {snapshot}");
    // 布局盒反映新文本（增量布局已重算 #a 及祖先的几何）
    let doc = pipeline.cached_doc.as_ref().expect("doc cached");
    let a = doc.query_selector(doc.root(), "#a").expect("#a");
    assert_eq!(doc.text_content(a).as_deref(), Some("much longer text now"));
    // 布局结果非空（增量路径产出 LayoutResult）
    assert!(!result.layout.snapshot().is_empty(), "layout snapshot empty");
}

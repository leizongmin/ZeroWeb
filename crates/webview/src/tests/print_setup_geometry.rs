//! T67（TV-1 遗留）：打印设置 → 预览渲染输出的程序化几何断言。
//!
//! T61 已钉「设置状态」（shell 级 `print_settings`）与「设置→页几何数字」
//! （engine `extract_print_page_geometry` 单测）；本模块补最后一段：设置经
//! `WebView::set_print_page_setup` → render 入口重放 → Print 模式根布局
//! containing block 宽 → **绘制图元宽度** 的端到端效果。断言读取真实
//! `FillPrimitive` 几何（guidelines #17：断言落在消费方可观测的渲染输出上，
//! 不读中间状态）。
//!
//! 期望值按 CSS Paged Media A4 @96dpi 规范公式就地推导（webview 不依赖
//! layout-engine；与 `PRINT_PAGE_WIDTH_A4`/`PRINT_PAGE_HEIGHT_A4` 同源）：
//! 宽 = 210mm/25.4×96 ≈ 793.7px，高 = 297mm/25.4×96 = 1122.5px。

use super::super::*;

/// 渲染一块 `width:100%` 纯红块，返回红色 fill 图元的最大宽度（px）。
///
/// Print 模式根布局 containing block 宽 = 打印页内容盒宽，`width:100%` 块的
/// 绘制宽度即该值——方向/边距变化直接反映为图元宽度变化。
fn rendered_red_block_width(setup: Option<zero_engine::PrintPageSetup>) -> f32 {
    let html = "<html><body style=\"margin:0;padding:0\">\
                <div style=\"width:100%;height:40px;background-color:rgb(255,0,0)\"></div>\
                </body></html>";
    let mut webview = WebView::new(WebViewConfig::default());
    webview.set_media_type(zero_engine::MediaType::Print);
    webview.set_print_page_setup(setup);
    webview.load_html(html, None);
    let result = webview.render();
    let red = result
        .primitives
        .fills
        .iter()
        .filter(|fill| fill.color.r == 255 && fill.color.g == 0 && fill.color.b == 0)
        .map(|fill| fill.rect.size.width)
        .fold(f32::NAN, f32::max);
    assert!(!red.is_nan(), "no red fill primitive in print render output");
    red
}

/// A4 页宽 @96dpi（210mm）。
const A4_WIDTH_PX: f32 = 210.0 / 25.4 * 96.0;
/// A4 页高 @96dpi（297mm）。
const A4_HEIGHT_PX: f32 = 1122.5;

/// T67：纵向 + 0 边距 → 内容块绘制宽度 = A4 页宽（≈793.7px），而非视口宽（800）。
/// 断言 793.7 而非 800 同时证明「Print 页几何已注入布局」与「未串到 Screen 视口宽」。
#[test]
fn print_portrait_no_margin_renders_at_a4_width() {
    let setup = zero_engine::PrintPageSetup {
        orientation: zero_engine::PrintOrientation::Portrait,
        margin_mm: 0.0,
    };
    let width = rendered_red_block_width(Some(setup));
    assert!(
        (width - A4_WIDTH_PX).abs() < 1.0,
        "portrait no-margin content block width {width} != A4 page width {A4_WIDTH_PX}"
    );
}

/// T67：横向 → 内容块绘制宽度 = A4 页高（1122.5px，宽高互换）。
#[test]
fn print_landscape_swaps_content_width() {
    let setup = zero_engine::PrintPageSetup {
        orientation: zero_engine::PrintOrientation::Landscape,
        margin_mm: 0.0,
    };
    let width = rendered_red_block_width(Some(setup));
    assert!(
        (width - A4_HEIGHT_PX).abs() < 1.0,
        "landscape content block width {width} != A4 page height {A4_HEIGHT_PX}"
    );
}

/// T67：横向 + 25.4mm 边距 → 内容宽 = 页宽 − 2×96px（mm→px 按 96dpi 四边收缩）。
#[test]
fn print_margin_shrinks_rendered_content_width() {
    let setup = zero_engine::PrintPageSetup {
        orientation: zero_engine::PrintOrientation::Landscape,
        margin_mm: 25.4,
    };
    let width = rendered_red_block_width(Some(setup));
    let want = A4_HEIGHT_PX - 2.0 * (25.4 / 25.4 * 96.0);
    assert!(
        (width - want).abs() < 1.0,
        "landscape + 25.4mm margin content block width {width} != {want}"
    );
}

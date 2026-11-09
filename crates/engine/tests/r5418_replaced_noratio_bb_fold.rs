//! R5418：no-ratio 替换元素 auto 尺寸注入的 box-sizing:border-box 语义折算——
//! css-ui-3 §box-sizing「border-box 只改写显式声明的 width/height 的解释盒」+
//! CSS2 §10.3.2/§10.6.2「auto 尺寸（固有维/default object size）恒 content-box
//! 语义（used width = intrinsic width，即 content 区）」。
//!
//! 案形（box-sizing-008/009，corpus 1.46%/4.83%）：img（no-ratio SVG，真实固有宽
//! 100/固有高 100）+ `box-sizing:border-box` + padding + CSS 尺寸双 auto。旧径把
//! content 语义注入值（w_opt=100/h_opt=150）直写 taffy size，BorderBox 节点被
//! taffy 按 border-box 扣 padding → content 80/130（chromium oracle：border box
//! 120×150 / 100×170，content 恒 100/150）。修复：both-auto 非 flex 注入臂在
//! border-box 节点补 frame 折算（bb = content + frame，frame_size 与 R4165 同源）。
//!
//! 测试判别形与字体无关；kill-switch `ZW_REPLACED_NORATIO_BB_FOLD=0`（OFF 轮由
//! corpus 单案精确回退覆盖，R3858 口径）。

use std::collections::HashMap;

use zero_engine::RenderPipeline;
use zero_engine::paint::simple_hash;

/// no-ratio SVG（无任何尺寸属性，与 corpus w100.svg 同走 no_ratio 通道——data URI
/// 带 width attr 会被 extract_svg_data_uri_size 解析走 attr 臂，信号差异）。真实
/// 固有宽 100 经 set_image_no_ratio 注入（Some(100), None），高缺 = default
/// object size 150 语义域。
const SRC: &str = "data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\" />";

/// 渲染单个 no-ratio img（CSS 尺寸双 auto），返回绿色 background fill 的
/// border-box 几何（background painting area = border box）。
fn render(img_style: &str) -> (f32, f32, f32, f32) {
    let html = format!(
        r#"<body><img src='{SRC}' style="background:#008000;{img_style}"></body>"#,
        SRC = SRC,
        img_style = img_style
    );
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let key = simple_hash(SRC.trim());
    let mut sizes = HashMap::new();
    sizes.insert(key, (100.0f32, 100.0f32));
    pipeline.set_image_sizes(sizes);
    // 真实固有维：宽 100（SVG width 属性）、高缺（default object size 150 语义域）。
    let mut no_ratio: HashMap<u64, (Option<f32>, Option<f32>)> = HashMap::new();
    no_ratio.insert(key, (Some(100.0), None));
    pipeline.set_image_no_ratio(no_ratio);
    let mut natural = HashMap::new();
    natural.insert(key, (100.0f32, 100.0f32));
    pipeline.set_image_natural_sizes(natural);
    let result = pipeline.render_html(&html, "");
    let green = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == 0 && f.color.g == 128 && f.color.b == 0 && f.color.a == 255)
        .expect("img background fill");
    (
        green.rect.origin.x,
        green.rect.origin.y,
        green.rect.size.width,
        green.rect.size.height,
    )
}

/// ★load-bearing（box-sizing-008 t01 形）：no-ratio img + `border-box` +
/// padding-left 20 + CSS 尺寸双 auto——border box 应 120（content 100 + padding
/// 20；chromium oracle 实证 bb 120×150）。旧径注入 100 被 taffy 按 border-box 扣
/// 成 content 80/bb 100，本针红。
#[test]
fn r5418_noratio_border_box_hpadding_folds() {
    let (x, y, w, h) = render("box-sizing:border-box;padding-left:20px;");
    assert_eq!(x, 8.0, "img 左缘 = body margin 8");
    assert_eq!(y, 8.0, "img 顶缘 = body margin 8");
    assert!(
        (w - 120.0).abs() < 0.5,
        "border-box 节点注入固有宽 100 应折算 bb 120（100+20），实测 {w}"
    );
    assert!(
        (h - 150.0).abs() < 0.5,
        "default object size 高 150（垂直无 padding 不折），实测 {h}"
    );
}

/// ★load-bearing（box-sizing-008 t02 形，垂直轴）：no-ratio img + `border-box` +
/// padding-bottom 20 + CSS 尺寸双 auto——border box 高应 170（default 150 +
/// padding 20；chromium oracle 实证 bb 100×170）。旧径 150 致后继行整体上移 20px。
#[test]
fn r5418_noratio_border_box_vpadding_folds() {
    let (_, _, w, h) = render("box-sizing:border-box;padding-bottom:20px;display:block;");
    assert!(
        (h - 170.0).abs() < 0.5,
        "border-box 节点 default 高 150 应折算 bb 170（150+20），实测 {h}"
    );
    assert!((w - 100.0).abs() < 0.5, "水平无 padding 不折（固有宽 100），实测 {w}");
}

/// 守卫（显式声明臂不触碰）：`border-box` + width:120px 显式 + padding-left 20
/// ——converter 已按 bb 语义写入 120（content = 120−20 = 100），本修复不得重复
/// 折算（重复计会得 140，本针即红）。
#[test]
fn r5418_border_box_declared_width_not_double_folded() {
    let (_, _, w, _) = render("box-sizing:border-box;padding-left:20px;width:120px;");
    assert!(
        (w - 120.0).abs() < 0.5,
        "显式 width:120 border-box 语义 bb 恒 120，实测 {w}"
    );
}

/// 守卫（content-box 路径不受折算污染）：默认 content-box + padding-left 20 +
/// CSS 尺寸双 auto——注入 content 100、taffy ContentBox 加回 padding → bb 恒
/// 120。折算若错误地作用于 content-box（+frame 双重计）会得 140，本针即红。
#[test]
fn r5418_content_box_path_untouched() {
    let (_, _, w, _) = render("padding-left:20px;");
    assert!(
        (w - 120.0).abs() < 0.5,
        "content-box 注入 100 + padding 20 → bb 120（折算不得介入），实测 {w}"
    );
}

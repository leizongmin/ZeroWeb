//! R5268（css-backgrounds-4 §background-clip:text 逐层 clip）：混合 clip 页
//! （如 `text, border-box`）不再整盒抑制——非 Text 层照常铺盒（文字域 ⊆ 非 Text
//! clip 域，其色同时作为字形混合 backdrop 进入全栈合成）；bg **色**按末层 clip
//! 绘制（R4528，末层 Text 时不铺盒矩形）。判据锚 background-clip-text-and-blend-mode
//! box2/3。整盒抑制回退测试在 `crates/engine/tests/`（独立进程，R3858 教训）。

use crate::pipeline::RenderPipeline;

/// 渲染返回（字形色集合，矩形填充色集合，渐变色标色集合）。
fn render(html: &str) -> (Vec<(u8, u8, u8, u8)>, Vec<(u8, u8, u8, u8)>, Vec<(u8, u8, u8, u8)>) {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let glyphs = p
        .glyphs
        .iter()
        .map(|g| (g.color.r, g.color.g, g.color.b, g.color.a))
        .collect();
    let fills = p
        .fills
        .iter()
        .map(|f| (f.color.r, f.color.g, f.color.b, f.color.a))
        .collect();
    let grad_stops = p
        .gradients
        .iter()
        .flat_map(|g| g.stops.iter().map(|s| (s.color.r, s.color.g, s.color.b, s.color.a)))
        .collect();
    (glyphs, fills, grad_stops)
}

/// box2 形：clip `text, border-box`——底 border-box 层（red）照常铺盒；顶 Text 层
/// （green，difference）矩形抑制、其色经全栈合成入字形：difference(green, red
/// OVER white) = yellow。旧整盒抑制下 red 渐变缺失、glyph 同色——red 存在性为判据。
#[test]
fn mixed_clip_text_border_box_keeps_non_text_layer_fill() {
    let html = r#"<body style="margin:0"><div style="font-size:100px; background-image: linear-gradient(rgb(0,255,0), rgb(0,255,0)), linear-gradient(rgb(255,0,0), rgb(255,0,0)); background-clip: text, border-box; background-color: rgb(255,255,255); background-blend-mode: difference, normal; color: transparent">X</div></body>"#;
    let (glyphs, _fills, grad_stops) = render(html);
    assert!(!glyphs.is_empty(), "字形应发射");
    assert!(
        glyphs.iter().all(|c| *c == (255, 255, 0, 255)),
        "字形应承载全栈合成 difference(green, red)=yellow，实际 {glyphs:?}"
    );
    assert!(
        grad_stops.contains(&(255, 0, 0, 255)),
        "border-box 层应照常铺盒（red 渐变存在），实际 {grad_stops:?}"
    );
    assert!(
        !grad_stops.contains(&(0, 255, 0, 255)),
        "Text 层矩形应抑制（无 green 渐变），实际 {grad_stops:?}"
    );
}

/// box3 形：clip `border-box, text`——顶 border-box 层（green）照常铺盒；底 Text
/// 层（blue）矩形抑制；bg 色按末层 clip=text 不铺盒矩形（R4528）。字形全栈合成
/// normal：green OVER blue OVER yellow = green。旧整盒抑制下 green 渐变缺失。
#[test]
fn border_box_text_clip_suppresses_text_layer_and_bg_color_fill() {
    let html = r#"<body style="margin:0"><div style="font-size:100px; background-image: linear-gradient(rgb(0,255,0), rgb(0,255,0)), linear-gradient(rgb(0,0,255), rgb(0,0,255)); background-clip: border-box, text; background-color: rgb(255,255,0); color: transparent">X</div></body>"#;
    let (glyphs, fills, grad_stops) = render(html);
    assert!(!glyphs.is_empty(), "字形应发射");
    assert!(
        glyphs.iter().all(|c| *c == (0, 255, 0, 255)),
        "字形应承载全栈合成 green OVER blue OVER yellow = green，实际 {glyphs:?}"
    );
    assert!(
        grad_stops.contains(&(0, 255, 0, 255)),
        "border-box 层应照常铺盒（green 渐变存在），实际 {grad_stops:?}"
    );
    assert!(
        !grad_stops.contains(&(0, 0, 255, 255)),
        "Text 层矩形应抑制（无 blue 渐变），实际 {grad_stops:?}"
    );
    assert!(
        !fills.contains(&(255, 255, 0, 255)),
        "bg 色按末层 clip=text 不铺盒矩形（R4528，无 yellow 填充），实际 {fills:?}"
    );
}

/// url 层锚（box2 全形，R5268 cyclic per-layer longhand）：url 层提取的
/// repeat/size/attachment 判定按 css-backgrounds-3 §3.1 `i % len` 取值——旧
/// `.get(i)?` 在次层（声明值为单值 list）取 None → 整页提取禁用 → 旧整盒行为。
/// 注入 solid_colors 模拟 runner 面：green 实层 + 失败加载透明哨兵（R5268
/// resources.rs else 臂语义）。字形 = difference(green, lighten(透明哨兵, cyan))
/// = blue；Text 层抑制 → 无图像图元。
#[test]
fn multi_layer_url_extraction_survives_undeclared_per_layer_longhands() {
    use crate::paint::helpers::image_resource_key;
    let mut solid = std::collections::HashMap::new();
    solid.insert(image_resource_key("resources/green-100.png", None), [0, 255, 0, 255]);
    solid.insert(image_resource_key("resources/blue.png", None), [0, 0, 0, 0]);
    let html = r#"<body style="margin:0"><div style="font-size:100px; background-image: url('resources/green-100.png'), url('resources/blue.png'); background-clip: text, border-box; background-color: cyan; background-blend-mode: difference, lighten; color: transparent">X</div></body>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_image_solid_colors(solid);
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let glyph = p.glyphs.first().map(|g| (g.color.r, g.color.g, g.color.b, g.color.a));
    assert_eq!(
        glyph,
        Some((0, 0, 255, 255)),
        "字形应承载全栈合成 difference(green, cyan)=blue"
    );
    // Text 层（green）矩形抑制 = 无 green 图像图元。border-box 层（blue.png 失败
    // 加载）仍发图元、渲染期绘不出（既有未解码图像行为，视觉为空），不在此判。
    use crate::paint::helpers::simple_hash;
    let green_key = simple_hash("resources/green-100.png");
    assert!(
        p.images.iter().all(|i| i.image_key.0 != green_key),
        "Text 层（green url）矩形应抑制（无 green 图像图元），实际 keys {:?}",
        p.images.iter().map(|i| i.image_key.0).collect::<Vec<_>>()
    );
}

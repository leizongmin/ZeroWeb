//! R4058 回归：content-visibility:hidden 元素的**替换内容**不绘制（css-contain-2 §5）。
//!
//! R2251 只挡了直属文本（paint_text），替换内容（img 位图 / canvas 位图 / svg 栅格 /
//! video 帧）漏挡——cv-021（hidden img 露出位图）/ cv-canvas（hidden canvas 露出像素）
//! 露出内容，ref 为纯 background+border 盒。修 = paint 4b 家族（img/canvas/svg/video）
//! 统一挂 `!content_hidden` gate；UA display:inline 的替换元素（svg 根）为 atomic
//! inline，content-visibility 对其生效（replaced 臂）。
//!
//! driving: css-contain/content-visibility/content-visibility-021 / -022 / -canvas。

use crate::pipeline::RenderPipeline;

/// hidden img：图片位图不进 primitives（元素盒装饰——背景/边框——照常）。
#[test]
fn r4058_cv_hidden_img_suppresses_image_primitive() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    // img 有 CSS 宽高（元素盒 202×202 含边框），src 指向不存在资源 → 无解码像素，
    // 旧实现在无解码时以 CSS 盒尺寸发 ImagePrimitive（get_img_intrinsic_size 回退）。
    let html = "<html><body style=\"margin:0\">\
<img src=\"nonexistent-r4058.png\" style=\"width:200px; height:200px; content-visibility:hidden; background:lightblue; border:1px solid black\">\
</body></html>";
    let result = pipeline.render_html(html, "");
    let images = &result.primitives().images;
    assert!(
        images.is_empty(),
        "R4058: content-visibility:hidden img 不应发 ImagePrimitive（替换内容不绘制），got {}",
        images.len()
    );
}

/// 对照：visible img 正常发 ImagePrimitive（gate 不误伤普通替换元素绘制）。
#[test]
fn r4058_cv_visible_img_still_paints() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<img src=\"nonexistent-r4058b.png\" style=\"width:200px; height:200px\">\
</body></html>";
    let result = pipeline.render_html(html, "");
    let images = &result.primitives().images;
    assert!(!images.is_empty(), "R4058 对照: visible img 应照常发 ImagePrimitive");
}

//! R5155（css-contain-1 §3.1 paint containment 圆角裁剪）：contain:paint 的裁剪
//! 区域 = padding box 圆角形状——溢出图片内容按「矩形窗口 + 四角圆角」剔除。
//! driving: contain-paint-clip-015/016（50×50 border-radius:50% 圆 + padding:25 +
//! 175×100 img，矩形裁剪在圆角外角隙露红 0.13%/0.24%）。

use crate::pipeline::RenderPipeline;

/// corpus 案形：圆角容器内溢出 img 应带圆角窗口（窗口 = padding box ∩ img rect，
/// 四角半径 = 50% 圆角收缩后）。
#[test]
fn r5155_contain_paint_radius_rounds_image_clip() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"contain: paint; border-radius: 50%; width:50px; height:50px; padding:25px\">\
<img style=\"display:block\" src=\"support/swatch-red.png\" width=\"175\" height=\"100\"></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    assert!(!p.images.is_empty(), "img 应生成 image 图元");
    for img in &p.images {
        let clip = img.clip.expect("contain:paint 裁剪容器内 img 应有窗口");
        // 窗口 = padding-box(0,0,100,100) ∩ img rect(25,25,175×100) = (25,25) 75×75
        assert!(
            (clip.origin.x - 25.0).abs() < 0.5
                && (clip.origin.y - 25.0).abs() < 0.5
                && (clip.size.width - 75.0).abs() < 0.5
                && (clip.size.height - 75.0).abs() < 0.5,
            "窗口应为 (25,25) 75×75，got {:?}",
            img.clip
        );
        let radii = img
            .clip_radius
            .expect("contain:paint + border-radius 应写入 clip_radius");
        assert!(
            radii.iter().all(|r| (r - 50.0).abs() < 0.5),
            "四角半径应为 50（50% of 100 圆），got {:?}",
            radii
        );
    }
}

/// 窄门对照：overflow:hidden + border-radius（无 contain:paint）不启用圆角窗口
///（通用 overflow 圆角裁剪未接，FIXME 案驱动；现状 = 矩形窗口）。
#[test]
fn r5155_overflow_radius_without_containment_unchanged() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"overflow: hidden; border-radius: 50%; width:50px; height:50px; padding:25px\">\
<img style=\"display:block\" src=\"support/swatch-red.png\" width=\"175\" height=\"100\"></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    assert!(!p.images.is_empty(), "img 应生成 image 图元");
    assert!(
        p.images.iter().all(|img| img.clip_radius.is_none()),
        "无 containment 的 overflow 圆角不应写 clip_radius（现状矩形窗口）"
    );
}

/// 对照：contain:paint 无 border-radius → 矩形窗口（clip_radius None，既有行为）。
#[test]
fn r5155_containment_without_radius_unchanged() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:0\">\
<div style=\"contain: paint; width:50px; height:50px; padding:25px\">\
<img style=\"display:block\" src=\"support/swatch-red.png\" width=\"175\" height=\"100\"></div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    assert!(!p.images.is_empty(), "img 应生成 image 图元");
    assert!(
        p.images
            .iter()
            .all(|img| img.clip.is_some() && img.clip_radius.is_none()),
        "无圆角的 contain:paint 裁剪应保持矩形窗口（clip_radius None）"
    );
}

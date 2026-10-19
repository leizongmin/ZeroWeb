//! R5202（CSS2 §17.2.1.1 匿名表包装 + §17.4.1 caption-side）：孤立 table-internal
//! 兄弟经 merge_orphan_table_run 合成的匿名 table 包装盒内，caption 的定位与
//! 计高与真表路径同语义。driving: caption-position-001（① cell+top-caption 案：
//! caption 滞留表底且丢 UA margin 1em；② bottom-caption 案：cat 图溢出 div 边框）。

use crate::pipeline::RenderPipeline;

/// ① caption-side:top caption 产于 table-cell 之后：caption 置顶（taffy DOM 序
/// 修正），cell 下移 caption 高；caption 的 margin-top 由包装盒 y 承接（真表由
/// taffy 逃逸，包装盒自补），cell 行偏移 = caption 高（margin-bottom 不推行——
/// 与真表 position_cells 的 top_caption_extent 同口径）。
#[test]
fn r5202_anon_table_top_caption_above_cell() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:8px\">\
<div style=\"border:3px solid black; padding:16px; width:98px\">\
<p style=\"display:table-cell\"><img src=\"support/cat.png\" width=\"155\" height=\"124\"></p>\
<p style=\"caption-side:top; display:table-caption\">ABOVE</p>\
</div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let img_y = p.images.iter().map(|i| i.rect.origin.y).fold(f32::MAX, f32::min);
    let text_y = p
        .glyphs
        .iter()
        .filter(|g| g.glyph_id == b'A' as u32)
        .map(|g| g.y)
        .fold(f32::MAX, f32::min);
    // caption 文字在图上方（基线 y 小于图顶）；UA p margin 1em → 内容整体 +16
    assert!(
        text_y < img_y,
        "caption-side:top 应置顶（text_y={text_y} 须 < img_y={img_y}）"
    );
    // 与真表路径同语义的基准：margin-top 16 + caption 行 18.4 → 图顶 61
    assert!(
        (img_y - 61.0).abs() < 1.0,
        "图顶应在 61（margin 16 + caption 高 18.4 + 内容顶 27），got {img_y}"
    );
}

/// ② caption-side:bottom caption（含 img 内容）+ 后继 table-cell：cell 置顶，
/// caption 移到行内容之后；包装盒高含 bottom caption 占位——外层 div 边框盒
/// 须包住 caption 内容（cat 图不溢出）。
#[test]
fn r5202_anon_table_bottom_caption_below_cell_contained() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let html = "<html><body style=\"margin:8px\">\
<div style=\"border:3px solid black; padding:16px; width:98px\">\
<p style=\"caption-side:bottom; display:table-caption\">\
<img src=\"support/cat.png\" width=\"155\" height=\"124\"></p>\
<p style=\"display:table-cell\">ABOVE</p>\
</div>\
</body></html>";
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let img_y = p.images.iter().map(|i| i.rect.origin.y).fold(f32::MAX, f32::min);
    let img_bottom = p
        .images
        .iter()
        .map(|i| i.rect.origin.y + i.rect.size.height)
        .fold(f32::MIN, f32::max);
    let text_y = p
        .glyphs
        .iter()
        .filter(|g| g.glyph_id == b'A' as u32)
        .map(|g| g.y)
        .fold(f32::MAX, f32::min);
    // cell 文字（基线）在 cat 图上方
    assert!(
        text_y < img_y,
        "cell 应在 bottom caption 之上（text_y={text_y} 须 < img_y={img_y}）"
    );
    // 图底须在 div 边框盒内：div 顶 8 + 边框 3 + padding 16 + cell 行 18.4 + 图 124
    // + padding 16 + 边框 3 → 图底 ≤ 8+3+16+19+124+16+3 = 189（容差覆盖行高取整）
    assert!(
        img_bottom < 190.0,
        "cat 图底应被 div 边框盒包住（< 190），got {img_bottom}"
    );
}

//! R3893 diagnostic：ref 页 paint glyph dump——定位「div 文本被吸收进 body IFC」的串联源。

use crate::pipeline::RenderPipeline;

#[test]
fn r3893_dump_ref_page_glyphs() {
    let html = r#"<html><head><style>
  .quote { color: green; }
</style></head><body>
“
<div><span class="quote">‘</span>Should not crash or assert and all six quotes should be displayed.’</div>
<div><span class="quote">‘</span>Should not crash or assert and all six quotes should be displayed.’</div>
”</body></html>"#;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");
    let glyphs = &result.primitives().glyphs;
    eprintln!("=== R3893 ref page glyphs: {} total ===", glyphs.len());
    // 打印前 80 个 glyph 的 (x, y, ch, r,g,b)
    use std::collections::BTreeMap;
    let mut by_y: BTreeMap<i32, Vec<char>> = BTreeMap::new();
    for g in glyphs.iter() {
        let ch = char::from_u32(g.glyph_id).unwrap_or('?');
        by_y.entry(g.y as i32).or_default().push(ch);
    }
    for (y, chs) in &by_y {
        eprintln!("  y={}: {} glyphs: {}", y, chs.len(), chs.iter().collect::<String>());
    }
    for g in glyphs.iter().filter(|g| g.glyph_id == 0x2018) {
        eprintln!("  QUOTE‘ at y={} rgba({},{},{})", g.y, g.color.r, g.color.g, g.color.b);
    }
    for g in glyphs.iter().take(0) {
        eprintln!(
            "  ch={:?} x={:.1} y={:.1} fs={} rgba({},{},{},{})",
            char::from_u32(g.glyph_id).unwrap_or('?'),
            g.x,
            g.y,
            g.font_size,
            g.color.r,
            g.color.g,
            g.color.b,
            g.color.a
        );
    }
}

//! R5205（css-overflow-4 #line-clamp，paint Path B 分带窗口 R4417/R4418 订正）：
//! 重跑 IFC 的行盒 y 是内容盒局部坐标（0 起），fragments 已加盒 content 原点
//!（border+padding）——分带窗口须按绝对 glyph y 锚定，否则带整体偏移 δ：δ>0.5
//! 的盒（页面上方有 margin/border/padding 的常见形态）末可见行落出末带窗口，
//! line_ys 少一行 → 省略号附到 cap−1 行（line-clamp-023：serif 簇 "Line 3…"
//! + "Line 4" 裸行）。driving: css-overflow line-clamp-023.tentative。

use crate::pipeline::{RenderPipeline, RenderResult};

/// 023 DOM：serif（非 Ahem → paint Path B 重跑 IFC），clamp 盒上方有 body margin
/// + border + padding（δ=18）。4 行预算 6 行内容，省略号应在第 4 行末。
#[test]
fn r5205_path_b_ellipsis_anchors_to_last_visible_row() {
    let html = r#"<html><head><style>
.clamp {
  line-clamp: 4;
  font: 16px / 32px serif;
  padding: 4px;
  border: 2px solid black;
}
.inner { margin: 4px; }
.inner .inner { white-space: pre; }
</style></head><body>
<div class="clamp"><div class="inner"><div class="inner">Line 1
Line 2
Line 3
Line 4
Line 5
Line 6</div></div></div>
</body></html>"#;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result: RenderResult = pipeline.render_html(html, "");
    let p = result.primitives();
    let mut rows: Vec<(f32, char)> = p
        .glyphs
        .iter()
        .filter(|g| g.font_size > 0.0)
        .map(|g| (g.y, char::from_u32(g.glyph_id).unwrap_or('?')))
        .collect();
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    // 按 y 聚合行（0.5px 容差），行内按 x 排序还原文本。
    let mut lines: Vec<(f32, Vec<(f32, char)>)> = Vec::new();
    for (y, ch) in rows {
        match lines.last_mut() {
            Some((ly, cs)) if (*ly - y).abs() < 0.5 => cs.push((0.0, ch)),
            _ => lines.push((y, vec![(0.0, ch)])),
        }
    }
    assert_eq!(lines.len(), 4, "cap=4 应恰 4 可见行");
    // 修前：省略号在行 3（"Line 3…"）且行 4 裸；修后行 4 末尾有 '…'。
    let row_text = |i: usize| -> String { lines[i].1.iter().map(|(_, c)| c).collect() };
    assert_eq!(row_text(0), "Line 1");
    assert_eq!(row_text(1), "Line 2");
    assert_eq!(row_text(2), "Line 3", "行 3 不应带省略号");
    assert_eq!(row_text(3), "Line 4…", "省略号应附在末可见行（行 4）末尾");
}

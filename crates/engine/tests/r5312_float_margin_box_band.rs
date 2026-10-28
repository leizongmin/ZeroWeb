//! R5312：float 排除带 margin-box 口径回归钉（CSS 2.1 §9.5）。
//!
//! 根因：compute_final own_floats、remeasure 与 postprocess R4234 三处
//! `FloatExclusion` 构建按 `x: c.x + margin_left` 组带——taffy native float（M3
//! step(b)）下 c.x/c.y 已是 border-box 位（margin 含在坐标内），再 `+margin` 即
//! 双计：ml:5 float 的带成 [10,25]（margin box 应 [0,15]）。runner 端（outline-028
//! 案 2）同源 span 被挤到第二行 + 右移 10px；RenderPipeline 端双计表现为测量面
//! 多折一行（容器 auto 高 10 → 20）。
//!
//! Ahem 10px 确定性断言：max-content 容器内 float ml:5（margin box [0,15]）+ span
//! ——修复后容器恰 1 行高（10px）、span 字形停在 float 字形右缘 +10px 同行；修复前
//! 带 [10,25] 把 span 折到第二行（容器高 20px）。
//!
//! 诊断链记档：
//! docs/goal/rendering-compat/evidence/r5312-float-margin-box-band-2026-10-28.md
//!
//! https://www.w3.org/TR/CSS21/visuren.html#floats

use zero_engine::RenderPipeline;

const CASE: &str = r#"<!DOCTYPE html>
<style>
body { margin: 0; }
.grid { display: grid; grid-template-columns: repeat(4, 100px); }
.oc { width: max-content; margin: 30px 0; font: 10px/1 Ahem; }
#f { float: left; margin-left: 5px; }
</style>
<div class="grid"><div><div class="oc"><div id="f">X</div><span>Y</span></div></div></div>"#;

/// float 排除带须按 margin box 组带：ml:5 float 的带 [0,15] 容得下同源 span
///（首行 1 行盒）。修复前带 [10,25]，span 左缘 25 触发折行 → 容器高多一行。
#[test]
fn float_band_uses_margin_box_width() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(CASE, "");
    let snapshot = result.layout.snapshot();

    // 容器（max-content = float margin box 15 + span 10 = 25 宽）auto 高须为 1 行。
    let container_h = snapshot
        .lines()
        .find(|l| l.contains("size=(25.00,"))
        .and_then(|l| {
            let idx = l.find("size=(")? + "size=(".len();
            l[idx..]
                .split(',')
                .nth(1)?
                .split(')')
                .next()?
                .trim()
                .parse::<f32>()
                .ok()
        })
        .unwrap_or_else(|| panic!("快照缺少 25px 宽容器行；实际快照：\n{snapshot}"));
    assert!(
        (container_h - 10.0).abs() < 0.5,
        "float ml:5 的排除带按 margin box [0,15] 组带时容器应恰 1 行（高 10px）；\
         实际高 {container_h}px——带按 `x + margin_left` 双计成 [10,25] 时 span 折行、容器高变 20px。\
         快照：\n{snapshot}"
    );

    // 可见契约：span 字形停在 float margin box 右缘（= float 字形 x + 10）且同行。
    let glyphs = &result.primitives().glyphs;
    let pos_of = |ch: char| {
        glyphs
            .iter()
            .find(|g| g.glyph_id == ch as u32)
            .map(|g| (g.x, g.y))
            .unwrap_or_else(|| panic!("缺少 '{ch}' 字形图元；glyphs={}", glyphs.len()))
    };
    let (fx, fy) = pos_of('X');
    let (sx, sy) = pos_of('Y');
    assert!(
        (sx - (fx + 10.0)).abs() < 0.5 && (sy - fy).abs() < 0.5,
        "span 字形须停在 float 右缘同行（float=({fx},{fy})）；实际 span=({sx},{sy})"
    );
}

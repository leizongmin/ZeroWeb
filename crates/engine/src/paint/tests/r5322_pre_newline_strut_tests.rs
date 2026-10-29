//! R5322（CSS Text 3 §5.2 + CSS2 §10.8.1）：`white-space:pre` 容器内 `\n` 强制
//! 断行标记与 `<br>` 同权——行首连续 `\n` 须产出 strut 行盒（driving:
//! white-space-008，green pre `\n\nX` 须 3 行与 red `<br/><br/>X` 同高覆盖）。
//! 空行 strut 高度取容器 used line-height（R5298 strut 口径），非硬编码。

use crate::pipeline::RenderPipeline;
use zero_render_foundation::primitive::FillPrimitive;

fn fills_dump(fills: &[FillPrimitive]) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    for f in fills {
        let _ = writeln!(
            s,
            "rect=(x={:.1},y={:.1},w={:.1},h={:.1}) color={:?}",
            f.rect.left(),
            f.rect.top(),
            f.rect.size.width,
            f.rect.size.height,
            f.color
        );
    }
    s
}

fn find_fill(fills: &[FillPrimitive], rgba: (u8, u8, u8, u8)) -> &FillPrimitive {
    let c = zero_render_foundation::color::Color::rgba(rgba.0, rgba.1, rgba.2, rgba.3);
    fills
        .iter()
        .find(|f| f.color == c)
        .unwrap_or_else(|| panic!("未找到 {:?} 填充，fills:\n{}", rgba, fills_dump(fills)))
}

/// white-space-008 最小镜像：green pre `\n\nX`（2 空行 + X = 3 行）vs red
/// `X<br>` + 源码换行 `X<br>` + 空行 + `X`（normal 折叠后 3 行，corpus 实测
/// ZW red 侧本就 54px = 3 行）。断言：两矩形同高（red 完全被 green 覆盖）。
#[test]
fn pre_leading_newlines_match_br_strut_lines() {
    let html = "<html><body style=\"margin:0\"><div style=\"position:relative\">\
<div style=\"background:#00ff00;white-space:pre\">\n\nX</div>\
<div style=\"position:absolute;top:0;left:0;width:300px;background:#ff0000;z-index:-1\">X<br>\n    X<br>\n\n    X</div>\
</div></body></html>";
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(html, "");
    let prims = result.primitives();
    eprintln!("=== fills ===\n{}", fills_dump(&prims.fills));
    let g = find_fill(&prims.fills, (0, 255, 0, 255)).rect.size.height;
    let r = find_fill(&prims.fills, (255, 0, 0, 255)).rect.size.height;
    assert!(
        (g - r).abs() <= 2.0,
        "pre 行首 \\n\\n 应与 <br><br> 同构 3 行 strut：green h={g:.1} vs red h={r:.1}"
    );
}

/// strut 高度取容器 line-height：pre `line-height:30px` 的 `\n\nX` = 3×30 = 90px
/// （防「用硬编码 20px 修行数」的伪修法）。
#[test]
fn pre_empty_line_strut_uses_container_line_height() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"background:#00ff00;white-space:pre;line-height:30px\">\n\nX</div>\
</body></html>";
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(html, "");
    let prims = result.primitives();
    eprintln!("=== fills ===\n{}", fills_dump(&prims.fills));
    let g = find_fill(&prims.fills, (0, 255, 0, 255)).rect.size.height;
    assert!(
        (g - 90.0).abs() <= 3.0,
        "空行 strut 须为容器 line-height（3×30=90），实际 {g:.1}"
    );
}

/// 控制腿：`<br><br>X`（R1286 路径）不回退——仍 3 行 strut。
#[test]
fn br_br_x_control_still_three_lines() {
    let html = r#"<html><body style="margin:0"><div style="background:#00ff00"><br><br>X</div></body></html>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(html, "");
    let prims = result.primitives();
    eprintln!("=== fills ===\n{}", fills_dump(&prims.fills));
    let g = find_fill(&prims.fills, (0, 255, 0, 255)).rect.size.height;
    assert!(g >= 50.0, "<br><br>X 控制腿应 ≥3 行（~55px），实际 {g:.1}");
}

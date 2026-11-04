//! R5414：块级 abspos 静态位锚改取容器 laid content extent——R5227 行底标量含 OOF
//! 自身幻影行（block-in-inline 单 IFC 模型给 OOF 子独立行），锚位随幻影行越界。
//! css-position-3 §static-position：真静态位 = 假想盒在流位置 = 前驱在流内容底＝
//! taffy 内容底（与后续 in-flow 兄弟落位同源，自洽）。
//!
//! 案形（css/CSS2/positioning/abspos-007.xht）：`.inline` span 内文本 + abspos
//! 绿块（10em×1em）+ in-flow 红块（同尺寸）——修复前绿块 y=36（幻影行底），
//! 红块 y=16，错位 20px 红条整面露出。
//!
//! kill-switch `ZW_R5227_CONTENT_EXTENT=0`（OFF 精确回退由 corpus A/B 双窗覆盖；
//! OnceLock 进程级缓存不支撑同进程双极性测试）。

use zero_engine::RenderPipeline;

const SHAPE_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <style>
   .container div { height: 1em; width: 10em; }
   .test { position: absolute; background: green; z-index: 1; }
   .control { background: red; }
   .inline { display: inline; }
  </style>
</head>
<body>
  <div class="container">
   <div class="inline">
     Test passes if there is <strong>no red</strong>.
     <div class="test"></div>
     <div class="control"></div>
   </div>
  </div>
</body>
</html>"#;

fn fill_rect(result: &zero_engine::RenderResult, r: u8, g: u8, b: u8, tag: &str) -> (f32, f32, f32, f32) {
    let f = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == r && f.color.g == g && f.color.b == b && f.color.a == 255)
        .unwrap_or_else(|| panic!("{tag} 填充缺失"));
    (f.rect.origin.x, f.rect.origin.y, f.rect.size.width, f.rect.size.height)
}

fn assert_rect(actual: (f32, f32, f32, f32), expected: (f32, f32, f32, f32), tag: &str) {
    for (a, e, axis) in [
        (actual.0, expected.0, "x"),
        (actual.1, expected.1, "y"),
        (actual.2, expected.2, "w"),
        (actual.3, expected.3, "h"),
    ] {
        assert!(
            (a - e).abs() < 0.5,
            "{tag}: {axis} = {a} 应 {e}（实际 {actual:?} 应 {expected:?}）"
        );
    }
}

/// ★load-bearing（abspos-007 案形）：块级 abspos 静态位 = 前驱在流内容底 16
/// ＝ in-flow 红块落位——绿块与红块同位覆盖。修复前绿块 y=36（幻影行底标量）。
#[test]
fn r5414_abspos_static_anchor_at_content_extent() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(SHAPE_HTML, "");
    let green = fill_rect(&result, 0, 128, 0, "abspos 绿块");
    let red = fill_rect(&result, 255, 0, 0, "in-flow 红块");
    // x/w/h 绝对恒（10em=160、1em=16、body margin 8）；y 断言走相对等 + 幻影行底
    // 上界——debug/release 文本行度量不同（IFC/taffy 缝隙既有面），绝对常量跨构建
    // 不稳（release 探针 y=16、本测试构建 y=24）。修复前幻影行底锚（release 36）
    // 落在上界之外，回归即红。
    assert_rect(green, (red.0, green.1, red.2, red.3), "绿块几何对齐红块");
    assert_eq!(green, red, "绿块须与红块同位（静态位＝在流内容底）");
    assert!(
        green.1 < 40.0,
        "绿块 y = {} 应在前驱内容底（幻影行底锚会越界）",
        green.1
    );
}

/// 守卫：无在流前驱的块级 abspos 不进本 pass（has_in_flow_pred 门）——静态位维持
/// 容器内容顶 0，不被 content extent 锚误移。
#[test]
fn r5414_no_in_flow_predecessor_stays_at_content_top() {
    let html = r#"<body style="margin:0"><div style="position:absolute;background:green;height:16px;width:160px"></div></body>"#;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");
    let green = fill_rect(&result, 0, 128, 0, "首子 abspos");
    assert_rect(green, (0.0, 0.0, 160.0, 16.0), "无前驱 abspos");
}

/// 守卫：inline-level abspos 维持 R5166 行顶锚——is_block_level 门不因本修复越界
/// （行底/content extent 锚只写块级域）。
#[test]
fn r5414_inline_level_abspos_keeps_line_top_anchor() {
    let html = r#"<body style="margin:0"><div style="font-size:16px">text <span style="position:absolute;background:green;width:50px;height:16px"></span></div></body>"#;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");
    let green = fill_rect(&result, 0, 128, 0, "inline-level abspos");
    assert!(
        (green.1 - 0.0).abs() < 0.5,
        "inline-level abspos y = {} 应 0（行顶锚，非行底 16）",
        green.1
    );
}

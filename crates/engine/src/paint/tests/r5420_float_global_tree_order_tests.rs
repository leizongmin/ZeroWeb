//! R5420：非 positioned floats 跨块全局树序绘制（CSS 2.2 Appendix E step 4）。
//!
//! 规范语义：floats 属 SC 根的 step 4 统一平面化绘制层，不嵌套于后代块的递归序
//!（chromium PaintLayer 平面化同型：普通块不建 layer，其内 floats 挂最近 layer
//! 祖先按全局树序）。ZW 旧实现＝per-container 局部后置排序（child_paint_sort_key
//! floats (2,0) 仅相对直接兄弟）——外层 float 树序早于嵌套容器内 float 时序倒置。
//! 驱动案 overhanging-float-paint-order：body red float（树 1）→ container 内
//! blue（树 2）/green（树 3），旧序 container 子树先画完、red 后画 → red 盖 green；
//! chromium/ref 页语义 green 盖 red（oracle 实测布局全对、错序纯绘制层）。
//! 修复＝scope 收集（遇 float 收集即停，子树随其递归 paint 内既有序）+ in-flow 块
//! 后按树序 flush。kill-switch `ZW_FLOAT_GLOBAL_TREE_ORDER=0`（OFF 轮由 corpus
//! 单案 ✗1.25% 精确回退覆盖，不设进程内单测——R3858 口径）。

use crate::pipeline::{RenderPipeline, RenderResult};

/// fills 按色的绘制序 index（首命中）。
fn fill_index(result: &RenderResult, rgb: [u8; 3]) -> Option<usize> {
    result
        .primitives()
        .fills
        .iter()
        .position(|f| f.color.r == rgb[0] && f.color.g == rgb[1] && f.color.b == rgb[2] && f.color.a == 255)
}

/// ★load-bearing（overhanging-float-paint-order 精确案形）：body red float（100×60）
/// → container（普通块 100×80）内 blue float（mt:-20，60 高）+ green float（mt:-100，
/// 80 高）。修复后全局树序 red→blue→green：green 后绘盖 red/blue 重叠区（ref 页
/// 语义＝绿 [0,80]+蓝 [88,108] 可见，red 全被盖）。旧行为 red 最后绘 → red idx
/// 最大（红 [0,60] 外露）。
#[test]
fn r5420_overhanging_floats_paint_in_global_tree_order() {
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#red { width: 100px; height: 60px; background: red; float: left; }
#container { width: 100px; height: 80px; }
#blue { width: 100px; height: 60px; background: blue; float: left; margin-top: -20px; }
#green { width: 100px; height: 80px; background: green; float: left; margin-top: -100px; }
</style>
<div id="red"></div>
<div id="container">
  <div id="blue"></div>
  <div id="green"></div>
</div>"#;
    let result = RenderPipeline::new(200.0, 200.0).render_html(html, "");
    let red = fill_index(&result, [255, 0, 0]);
    let blue = fill_index(&result, [0, 0, 255]);
    let green = fill_index(&result, [0, 128, 0]);
    let (red, blue, green) = (
        red.expect("red fill"),
        blue.expect("blue fill"),
        green.expect("green fill"),
    );
    assert!(
        red < blue && blue < green,
        "floats 应按全局树序 red({red}) < blue({blue}) < green({green}) 绘制（旧行为 red 最后绘）"
    );
}

/// ★跨容器简化形：外层 float A（树 1）在前，后续普通块内的 float B（树 2）几何
/// 重叠——B 应盖 A（旧序 A 随 body 主循环最后绘，A 盖 B）。用黄/紫两色错开。
#[test]
fn r5420_later_tree_float_paints_over_earlier_container_float() {
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#a { width: 100px; height: 100px; background: yellow; float: left; }
#wrap { width: 100px; height: 100px; }
#b { width: 100px; height: 100px; background: purple; float: left; margin-top: -100px; }
</style>
<div id="a"></div>
<div id="wrap"><div id="b"></div></div>"#;
    let result = RenderPipeline::new(200.0, 200.0).render_html(html, "");
    let yellow = fill_index(&result, [255, 255, 0]);
    let purple = fill_index(&result, [128, 0, 128]);
    let (yellow, purple) = (yellow.expect("yellow fill"), purple.expect("purple fill"));
    assert!(
        yellow < purple,
        "树序靠后的嵌套容器 float 应后绘（yellow {yellow} < purple {purple}；旧行为倒置）"
    );
}

/// ★同容器 float 树序守卫：blue（树 1）/green（树 2）同为 container 直接子——
/// 既有 per-container 序（green 后绘）保持不变（收集器不打乱同容器序）。
#[test]
fn r5420_same_container_floats_keep_tree_order() {
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#wrap { width: 100px; height: 100px; }
#blue { width: 100px; height: 100px; background: blue; float: left; }
#green { width: 100px; height: 100px; background: green; float: left; margin-top: -100px; }
</style>
<div id="wrap">
  <div id="blue"></div>
  <div id="green"></div>
</div>"#;
    let result = RenderPipeline::new(200.0, 200.0).render_html(html, "");
    let blue = fill_index(&result, [0, 0, 255]);
    let green = fill_index(&result, [0, 128, 0]);
    let (blue, green) = (blue.expect("blue fill"), green.expect("green fill"));
    assert!(blue < green, "同容器 floats 树序不变：blue({blue}) < green({green})");
}

/// ★float 子树嵌套 float DFS 前序守卫：A float（树 1）内嵌 B float（树 2，右半
/// 偏移——负 margin 上拉形会被 paint_in_rect 脏区快剔整支剔除，不构成有效驱动）、
/// 外层后续 C float（树 3，负 margin 上拉与 A/B 重叠）——收集停语义下 A 收集
///（B 随其递归画）、C 收集，flush 序 A→C ⇒ 全局序 A < B < C 保持（B 不得晚于 C；
/// 旧序 wrap 子树先画完、A 后画 → green 首位断红）。
#[test]
fn r5420_nested_float_inside_float_keeps_dfs_order() {
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#a { width: 100px; height: 100px; background: yellow; float: left; }
#b { width: 50px; height: 100px; background: red; float: left; margin-left: 50px; }
#wrap { width: 100px; height: 100px; }
#c { width: 100px; height: 100px; background: green; float: left; margin-top: -100px; }
</style>
<div id="a"><div id="b"></div></div>
<div id="wrap"><div id="c"></div></div>"#;
    let result = RenderPipeline::new(200.0, 200.0).render_html(html, "");
    let yellow = fill_index(&result, [255, 255, 0]);
    let red = fill_index(&result, [255, 0, 0]);
    let green = fill_index(&result, [0, 128, 0]);
    let (yellow, red, green) = (
        yellow.expect("yellow fill"),
        red.expect("red fill"),
        green.expect("green fill"),
    );
    assert!(
        yellow < red && red < green,
        "float 内嵌 float 的 DFS 前序即全局树序：yellow({yellow}) < red({red}) < green({green})"
    );
}

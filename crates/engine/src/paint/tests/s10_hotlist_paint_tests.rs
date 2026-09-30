//! slice10：热榜行叠字 paint 级回归——float 子 + inline 兄弟共存容器（百度热榜 li 形态）。
//!
//! 形态：`li > [a(float, overflow:hidden, [i(inline-block 图标), span(inline 标题文本)]),
//! mark(inline-block)]`。缺陷链（修复前，glyph 基线实测 52.16，正确 24.85）：
//! ① 容器含 float 子 + inline 级兄弟时 R1280 float_not_block 使 `has_direct_paintable_text`
//! 为真 → 容器 paint_text 运行（floats-006 语义，正确）；② 但 paint Path B 以空 styles
//! 重跑 IFC，collector 的 float 臂需真实 styles 而不可达、float 子不在 block_child_nodes
//! → float 子落入 inline 递归，其子树文本被吸收进容器 IFC（CSS2 §9.5 违例：float 脱离
//! 常规流，其内容在 float 自身 BFC 内排版，不参与容器行内流）；③ 容器行盒把标题文本
//! 重排到 float 下方第二行（line_top=36 → 基线 52.16），且 painted_inline_nodes 去重
//! 随后抑制 float 盒自身的正确 Path B 绘制 → 叠字。修复：painter 从 computed styles
//! 注入 `float_child_nodes` 真值，collector 空 styles 臂同发 FloatAnchor（R3784 语义），
//! float 子树不再被吸收，float 盒自身绘制不再被抑制。
//!
//! 判别力证据：repro4.html 单因子实验（活体）——去掉 mark 兄弟后同形态绘制即正确
//! （r3a 基线 24.848 vs r3f 52.16，Δ27.3px）；本测试在引擎内复现同一分叉。
//! 断言值：fs16/lh36 标题基线 = half-leading(10) + ascent(≈14.85) ≈ 24.85（A 容器 IFC
//! strut 组合行）；修复前错误基线 52.16（容器第二行）。

use crate::pipeline::RenderPipeline;

const A_STYLE: &str = "float:left;display:block;width:369px;height:36px;line-height:36px;\
font-size:14px;overflow:hidden;white-space:nowrap;color:#222;";
const ICON_STYLE: &str = "display:inline-block;position:relative;top:-2px;vertical-align:middle;\
width:18px;height:18px;line-height:18px;font-size:18px;color:#f63051;";
const TITLE_STYLE: &str = "display:inline;line-height:36px;font-size:16px;color:#222;";
const MARK: &str =
    "<span style=\"display:inline-block;vertical-align:middle;width:4px;height:16px;margin-left:4px;\"></span>";

fn li_shell(inner: &str) -> String {
    format!(
        "<html><body><ul style=\"list-style:none;margin:0;padding:0;width:760px;\">\
<li style=\"float:left;width:369px;height:36px;line-height:36px;font-size:12px;white-space:nowrap;clear:both;\">{inner}</li></ul></body></html>"
    )
}

fn a_html() -> String {
    format!(
        "<a style=\"{A_STYLE}\">\
<i style=\"{ICON_STYLE}\"></i>\
<span style=\"{TITLE_STYLE}\">TITLE</span>\
</a>"
    )
}

fn row_html(mark: bool) -> String {
    let mark_html = if mark { MARK } else { "" };
    li_shell(&format!("{}{}", a_html(), mark_html))
}

fn title_glyph_baselines(html: &str) -> Vec<f32> {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "body { margin: 0 }");
    let mut ys: Vec<f32> = result
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id == 'T' as u32)
        .map(|g| g.y)
        .collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys
}

/// 驱动形态（含 mark 兄弟）：标题基线必须落在 A 容器 IFC 首行（≈24.85），
/// 不得被容器 IFC 吸收到 float 下方第二行（修复前 52.16）。
#[test]
fn s10_hotlist_float_plus_inline_sibling_no_text_absorption() {
    let ys = title_glyph_baselines(&row_html(true));
    assert!(ys.len() >= 2, "TITLE 含两个 T，应有 ≥2 个 glyph：{ys:?}");
    for (i, y) in ys.iter().enumerate() {
        assert!(
            (23.0..27.0).contains(y),
            "TITLE glyph[{i}] 基线应 ≈24.85（A 容器 IFC 首行，CSS2 §10.8.1 strut 组合行），\
实际 {y}——若 ≈52 则 float 子树文本被容器 IFC 吸收（CSS2 §9.5 违例回归）"
        );
    }
}

/// 对照形态（无 mark 兄弟）：修复不得破坏纯 float 行的既有正确绘制。
#[test]
fn s10_hotlist_float_only_row_baseline_unchanged() {
    let ys = title_glyph_baselines(&row_html(false));
    assert!(ys.len() >= 2, "TITLE 含两个 T，应有 ≥2 个 glyph：{ys:?}");
    for (i, y) in ys.iter().enumerate() {
        assert!((23.0..27.0).contains(y), "TITLE glyph[{i}] 基线应 ≈24.85：{y}");
    }
}

/// 邻近变体臂（审查 TE-2）：mark 兄弟在 float a **之前**（DOM 序相反）。
/// 触发面相同——容器仍同时含 float 子与 inline 级兄弟，paint_text 照常运行；
/// 行内序不同不得改变 float 子树不入容器 IFC 的语义（CSS2 §9.5）。
#[test]
fn s10_hotlist_mark_before_float_no_text_absorption() {
    let html = li_shell(&format!("{}{}", MARK, a_html()));
    let ys = title_glyph_baselines(&html);
    assert!(ys.len() >= 2, "TITLE 含两个 T，应有 ≥2 个 glyph：{ys:?}");
    for (i, y) in ys.iter().enumerate() {
        assert!(
            (23.0..27.0).contains(y),
            "TITLE glyph[{i}] 基线应 ≈24.85（mark 前置变体同语义）：{y}"
        );
    }
}

//! R5396（CSS2 §9.2.1.1 inline formatting + CSS2 §16 字体继承）：R2156
//! inline-box-model coherence 摘除「含嵌套原子行内的 display:inline 子盒」（taffy
//! 树仅提升原子后代）后，其文本归属容器 IFC（collect_items R1576 递归臂）。此时
//! R4043 deep-path 排除（`subtree_font_differs_from` 字体分歧等）的前提「行内子盒
//! 自绘」不再成立——子盒已不存在，容器是唯一绘制者；容器 paint_text 的
//! `has_direct_paintable_text` gate 早退曾把整段行内文本孤儿化（layout IFC 全量
//! runs、paint 零字形；github trending 标题 h2>a>[svg+span(400)+text(600)]
//! 页面标题整体消失实证）。修复 = gate 按盒树实况放行（R5235 信号集成员无
//! LayoutBox 即已被摘除）。kill-switch `ZW_UNBOXED_INLINE_TEXT=0`（独立二进制
//! `r5396_unboxed_atomic_inline_text_kill_switch.rs`，R3858：env 锚不与并行单测共进程）。

use zero_engine::RenderPipeline;

/// 容器行内文本按 x 排序拼接（去空白），供存在性断言。
fn rendered_text(result: &zero_engine::RenderResult) -> String {
    let mut glyphs: Vec<_> = result.primitives().glyphs.to_vec();
    glyphs.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    glyphs
        .iter()
        .map(|g| char::from_u32(g.glyph_id).unwrap_or('?'))
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn repro_html() -> String {
    // 缺陷形：容器 600 > inline a > [inline-block 原子 + span 400 + 直系文本 600]。
    // 字重切换触发 R4043 字体分歧排除，原子触发 R2156 摘除——修复前两 gate 叠加
    // 使容器早退且无行内子盒可让渡，文本整体丢失。
    r##"<body style="margin:0"><div style="font-family:Ahem;font-size:20px;font-weight:600"><a href="#"><span style="display:inline-block;width:16px;height:16px"></span><span style="font-weight:400">repo /</span> agent</a></div></body>"##.to_string()
}

#[test]
fn unboxed_atomic_inline_keeps_text_across_font_weight_switch() {
    let html = repro_html();
    let result = RenderPipeline::new(400.0, 300.0).render_html(&html, "");
    let text = rendered_text(&result);
    assert!(
        text.contains("repo") && text.contains("agent"),
        "字重切换 × 原子行内的整段行内文本必须全部绘出，实际 {text:?}"
    );
    // 恰好绘制一次（无行内子盒可让渡，容器 Path B 是唯一绘制者；双绘 = 泄漏）。
    assert_eq!(
        text.chars().count(),
        10,
        "非空白字形应恰 repo/agent 10 个（去重双绘），实际 {text:?}"
    );
    // 原子 advance 保持：span 400 文本起点在 16px 原子之后。
    let prims = result.primitives();
    let mut xs: Vec<f32> = prims
        .glyphs
        .iter()
        .filter(|g| char::from_u32(g.glyph_id) == Some('r'))
        .map(|g| g.x)
        .collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        xs.first().is_some_and(|&x| x >= 15.5),
        "span 文本应排在 16px 原子行内盒之后，实际首 'r' x={xs:?}"
    );
}

/// 邻近边界（回归守卫）：同拓扑无字重切换（uniform 600）与无原子（纯 inline）
/// 两形修复前后行为不变——前者本就经 PHASEA/stored 路径放行，后者行内子盒存在、
/// 由子盒自绘。
#[test]
fn neighbor_forms_unaffected() {
    // uniform 字重 + 原子：deep-path 字体分歧排除不触发，修复前已绘。
    let uniform = r##"<body style="margin:0"><div style="font-family:Ahem;font-size:20px;font-weight:600"><a href="#"><span style="display:inline-block;width:16px;height:16px"></span> agent</a></div></body>"##;
    let result = RenderPipeline::new(400.0, 300.0).render_html(uniform, "");
    let text = rendered_text(&result);
    assert!(
        text.contains("agent"),
        "无字重切换形（修复前已绘）必须保持，实际 {text:?}"
    );

    // 字重切换但无原子：行内子盒（a/span）存在，由子盒 Path B 自绘，不经本修复面。
    let no_atomic = r##"<body style="margin:0"><div style="font-family:Ahem;font-size:20px;font-weight:600"><a href="#"><span style="font-weight:400">repo /</span> agent</a></div></body>"##;
    let result = RenderPipeline::new(400.0, 300.0).render_html(no_atomic, "");
    let text = rendered_text(&result);
    assert!(
        text.contains("repo") && text.contains("agent"),
        "纯 inline 形（子盒自绘）必须保持，实际 {text:?}"
    );
}

//! R5396 kill-switch 钉（独立测试进程，R3858：env 锚不与并行单测互踩）：
//! `ZW_UNBOXED_INLINE_TEXT=0` 回退旧 gate 早退行为（行内文本孤儿化，即修复前
//! 缺陷面）——证明 `r5396_unboxed_atomic_inline_text` 的主断言咬合在修复路径上。

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

#[test]
fn kill_switch_restores_pre_fix_text_loss() {
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）；本文件仅此一测，
    // 独占进程环境（R3858）。
    unsafe { std::env::set_var("ZW_UNBOXED_INLINE_TEXT", "0") };
    // 缺陷形：容器 600 > inline a > [inline-block 原子 + span 400 + 直系文本 600]。
    let html = r##"<body style="margin:0"><div style="font-family:Ahem;font-size:20px;font-weight:600"><a href="#"><span style="display:inline-block;width:16px;height:16px"></span><span style="font-weight:400">repo /</span> agent</a></div></body>"##;
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let text = rendered_text(&result);
    assert!(
        !text.contains("repo") && !text.contains("agent"),
        "kill-switch 关断放行后应回退旧早退行为（无文本），实际 {text:?}"
    );
}

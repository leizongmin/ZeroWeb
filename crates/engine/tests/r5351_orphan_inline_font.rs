//! R5351（CSS2 §9.2.1.1 anonymous inline boxes）：orphan inline 元素文本的
//! **自身字体度量**锚。前置块级兄弟（`<p>`）触发形下 inline div 无 LayoutBox、
//! 其成员进 R109 匿名块片段盒；片段 IFC 曾被 compute_final 的混合排除
//!（!has_block_elem，按容器全部 DOM 子判定）拒之门外 → 度量表不存 → paint
//! Path B 重建时 flatten run 回落 16px 默认字体（自身 `font-size:40px` 不落
//! shaping，颜色伪样式不受影响）。修复 = 片段盒按 fragment_node_ids 成员含
//! inline 元素/文本节点放行（收集本就被成员表圈定，无块子树吸收面）。
//! kill-switch `ZW_ORPHAN_FRAG_IFC=0`。
//!
//! 独立测试进程锚默认开态（R3858：进程内 set_var 与并行单测互踩）。

use zero_engine::RenderPipeline;

/// orphan inline div 的字形按自身 40px 整形（修复前 16px 默认字体，advance
/// 8.8px；修复后 advance = 1em = 40px）。
#[test]
fn orphan_inline_text_shapes_at_own_font_size() {
    let html = r#"<body style="margin:0"><p style="margin:0">intro</p><div id="t" style="display:inline; color: fuchsia; font-size: 40px; font-family: Ahem">aa</div></body>"#;
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let prims = result.primitives();
    let orphan_glyphs: Vec<_> = prims
        .glyphs
        .iter()
        .filter(|g| g.color == zero_render_foundation::color::Color::rgba(255, 0, 255, 255))
        .collect();
    assert_eq!(
        orphan_glyphs.len(),
        2,
        "orphan div 应恰 2 个字形，实际 {orphan_glyphs:?}"
    );
    for g in &orphan_glyphs {
        assert!(
            (g.font_size - 40.0).abs() <= 0.5,
            "orphan inline 字形应按自身 font-size 40 整形，实际 fs={}（16 = Path B 度量表缺失回落默认）",
            g.font_size
        );
    }
    // Ahem advance = 1em：40px 字形间距 40（16px 回落时为 8.8）。
    let dx = orphan_glyphs[1].x - orphan_glyphs[0].x;
    assert!((dx - 40.0).abs() <= 0.5, "Ahem advance 应 = 1em = 40，实际 {dx}");
}

/// 对照：无前置块级兄弟（inline-first，div 持 LayoutBox）与 block 形不受修复
/// 影响——两形本来就走 styled IFC，字体从未丢失（回归守卫）。
#[test]
fn non_orphan_forms_keep_own_font_size() {
    for (name, html) in [
        (
            "inline-first",
            r#"<body style="margin:0"><div id="t" style="display:inline; font-size: 40px; font-family: Ahem">aa</div></body>"#,
        ),
        (
            "block",
            r#"<body style="margin:0"><p style="margin:0">intro</p><div id="t" style="font-size: 40px; font-family: Ahem">aa</div></body>"#,
        ),
    ] {
        let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
        let prims = result.primitives();
        let glyphs: Vec<_> = prims.glyphs.iter().filter(|g| g.font_size > 1.0).collect();
        assert!(
            glyphs.iter().any(|g| (g.font_size - 40.0).abs() <= 0.5),
            "{name} 形 40px 字形应存在（styled IFC 原语义），实际 fs 集合 {:?}",
            glyphs.iter().map(|g| g.font_size).collect::<Vec<_>>()
        );
    }
}

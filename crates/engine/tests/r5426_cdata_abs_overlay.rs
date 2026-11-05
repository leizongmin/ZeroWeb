//! R5426（CDATA 段内实体不解码）e2e：content-177 案形——XHTML `<style>` CDATA
//! 段内字符引用按字面保留（XML 1.0 §2.7），两个 abs span 同位叠合的 glyph 序列
//! 全等。CSS 解码单测见 `src/tests/r5426_cdata_entity_tests.rs`。
//!
//! 已知预存杂音（非本修复面，base 同在）：父 IFC 对 OOF abs 子发射一枚
//! glyph_id=0 占位 glyph——断言一律过滤。
//!
//! 已知残留缺口（本文件不断言）：abs 盒直系文本经 paint display 门漏过继承字号
//! 落 16px 默认（真实字体窗口 content-177 2.35% 实证）；播种修复臂窗口 A/B 使
//! css-ruby/abs-in-ruby 族 3 案翻红已回退，待布局层字体修复同轮落地。

use zero_engine::RenderPipeline;

/// 收集 glyph 原语的 (glyph_id, font_size, x, y)，过滤 OOF 占位 notdef（见模块注释）。
fn glyph_tuples(result: &zero_engine::RenderResult) -> Vec<(u32, f32, f32, f32)> {
    result
        .primitives()
        .glyphs
        .iter()
        .filter(|g| g.glyph_id != 0)
        .map(|g| (g.glyph_id, g.font_size, g.x, g.y))
        .collect()
}

/// ★① e2e（content-177 案形，XHTML 原文含 CDATA + 实体）：两个 abs span 同位
/// 叠合——.test 的 :before content（CSS 串，实体字面保留）与 .control 的解析文本
///（`&amp;` 解码成 `&` 后字面串）同一字符序列，glyph 全等叠合。旧径 CDATA 段内
/// 实体被解码 → :before 成 "TEST FAIL" → 序列短且无字面 `&#`，本针红。
#[test]
fn r5426_content177_abs_spans_overlay_identically() {
    let html = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <style type="text/css"><![CDATA[
   div { position: relative; font: 3em sans-serif; color: red; }
   span { position: absolute; top: 0; left: 0; }
   .control { color: green; }
   .test:before { content: 'TEST &#x46;&#x41;&#x49;&#x4c;'; }
  ]]></style>
 </head>
 <body>
  <div><span class="test">ED</span><span class="control">TEST &amp;#x46;&amp;#x41;&amp;#x49;&amp;#x4c;ED</span></div>
 </body>
</html>"#;
    let result = RenderPipeline::new(800.0, 600.0).render_html(html, "");
    let glyphs = glyph_tuples(&result);
    // "TEST &#x46;&#x41;&#x49;&#x4c;ED" 非空白字符 30 枚 × 2 span（空白折叠不发射 glyph）。
    assert_eq!(glyphs.len(), 60, "两个 abs span 各 30 字符 glyph，实测 {glyphs:?}");
    // 字面实体标记：控制串前缀 "TEST &#x46;" 非空白序 = T E S T & # x 4 6 ;。
    assert_eq!(glyphs[4].0, 0x26, "第 5 字符应为字面 '&'（CDATA 段内实体未解码）");
    assert_eq!(glyphs[5].0, 0x23, "第 6 字符应为字面 '#'");
    // 两 span 同位叠合：逐 glyph (id, size, x, y) 全等。
    for i in 0..30 {
        let (a, b) = (&glyphs[i], &glyphs[i + 30]);
        assert_eq!(a.0, b.0, "glyph {i} 字符不等（实体解码/折行分岔）");
        assert!(
            (a.1 - b.1).abs() < 0.5 && (a.2 - b.2).abs() < 0.5 && (a.3 - b.3).abs() < 0.5,
            "glyph {i} 几何不等：{a:?} vs {b:?}"
        );
    }
}

//! R5316（CSS2 §10.8.1 va:text-* × 负 leading）：text-top/text-bottom 对齐参照 =
//! 父 content area（基线相对），盒参与行盒围合——负 leading（line-height < A+D）
//! 时行盒按基线锚定需求生长。
//!
//! 案 vertical-align-negative-leading-001（✗19.98%）：容器 line-height:10px +
//! font-size:30px Ahem（half-leading −10），chromium 行高 = [10,30,10,10,20,20]
//!（va:top/bottom 不参与行盒围合 → 10；text-top/text-bottom 钉父 content
//! 顶/底 → 20/30），ZW 旧 half_leading 扩展被 `.max(0.0)` 钳零恒 10。修复 = 基线
//! 锚定需求与 half_leading 校准值取 max（正 leading 域逐字节同旧行为）。
//!
//! kill-switch 反证（`ZW_VA_TEXT_BASELINE_PIN=0`）为进程级 LazyLock 快照，
//! 由 corpus/env-off 轮承担（evidence 记录）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;

fn find_box(root: &LayoutBox, node_id: zero_dom::NodeId) -> Option<f32> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b.height);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

fn container_heights(result: &crate::engine::LayoutResult, doc: &zero_dom::Document) -> Vec<f32> {
    ["c1", "c2", "c3", "c4", "c5", "c6"]
        .iter()
        .map(|id| {
            let n = doc.get_element_by_id(id).unwrap_or_else(|| panic!("{id}"));
            find_box(&result.root, n).unwrap_or_else(|| panic!("{id} 盒"))
        })
        .collect()
}

/// 存面 fragment 行内绝对基线（祖先 y 累加 + line.y + frag.baseline_y）：存面行可挂
/// 匿名子盒（y 相对该盒），按 fragment 归因 node_id 全树首中即返回。
fn frag_baseline(root: &crate::types::LayoutBox, frag_node: zero_dom::NodeId) -> Option<f32> {
    let mut stack = vec![(root, 0.0_f32)];
    while let Some((b, acc)) = stack.pop() {
        let abs_y = acc + b.y;
        if let Some(lines) = &b.inline_layout {
            for line in lines {
                for f in &line.fragments {
                    if f.node_id == Some(frag_node) {
                        return Some(abs_y + line.y + f.baseline_y);
                    }
                }
            }
        }
        stack.extend(b.children.iter().map(|c| (c, abs_y)));
    }
    None
}

/// 驱动案六容器形：va:top/bottom 行不生长（10），text-top/text-bottom 行按父
/// content area（A=24/D=6 基线相对）围合生长（20/30/20/20）。修复前全 10。
#[test]
fn r5316_negative_leading_line_heights_match_chromium() {
    let html = r#"<html><body style="margin:0">
<div id=c1 style="margin:30px 0; line-height:10px; font-size:30px; font-family:Ahem;">
<span style="vertical-align:top">XX</span><span>XX</span><span style="vertical-align:bottom">XX</span>
</div>
<div id=c2 style="margin:30px 0; line-height:10px; font-size:30px; font-family:Ahem;">
<span style="vertical-align:text-top">XX</span><span>XX</span><span style="vertical-align:text-bottom">XX</span>
</div>
<div id=c3 style="margin:30px 0; line-height:10px; font-size:30px; font-family:Ahem;"><span style="vertical-align:top">XX</span></div>
<div id=c4 style="margin:30px 0; line-height:10px; font-size:30px; font-family:Ahem;"><span style="vertical-align:bottom">XX</span></div>
<div id=c5 style="margin:30px 0; line-height:10px; font-size:30px; font-family:Ahem;"><span style="vertical-align:text-top">XX</span></div>
<div id=c6 style="margin:30px 0; line-height:10px; font-size:30px; font-family:Ahem;"><span style="vertical-align:text-bottom">XX</span></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let hs = container_heights(&result, &doc);
    let expect = [10.0, 30.0, 10.0, 10.0, 20.0, 20.0];
    for (i, (h, e)) in hs.iter().zip(expect).enumerate() {
        assert!(
            (h - e).abs() < 0.5,
            "容器 {} 行盒高应 {}（chromium 判例），实际 {}（全部 {:?}）",
            i + 1,
            e,
            h,
            hs
        );
    }
}

/// 正 leading 域零扰动守卫：fs16/lh20（half +2）单 text-bottom 行 = 旧行为 22
///（top 扩 half、基线 16.8，扩展后底合围不新增）。
#[test]
fn r5316_positive_leading_text_bottom_unchanged() {
    let html = r#"<html><body style="margin:0">
<div id=c1 style="margin:30px 0; line-height:20px; font-size:16px; font-family:Ahem;"><span style="vertical-align:text-bottom">XX</span></div>
<div id=c2 style="margin:30px 0; line-height:20px; font-size:16px; font-family:Ahem;"><span style="vertical-align:text-top">XX</span></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let h1 = find_box(&result.root, doc.get_element_by_id("c1").expect("c1")).expect("c1 盒");
    let h2 = find_box(&result.root, doc.get_element_by_id("c2").expect("c2")).expect("c2 盒");
    assert!(
        (h1 - 22.0).abs() < 0.5,
        "正 leading 单 text-bottom 行盒高应 22（旧行为），实际 {h1}"
    );
    assert!(
        (h2 - 22.0).abs() < 0.5,
        "正 leading 单 text-top 行盒高应 22（旧行为），实际 {h2}"
    );
}

/// R5316 Phase 3：text-top/text-bottom run 字形基线按**父 content area**锚定（存面
/// fragment.baseline_y，容器相对）。chromium 判例（ref up10/down10/up5/down5）：
/// c2 text-top +14（= B−A_parent+leaded_asc）、text-bottom +34；同线 baseline span
/// 24 不动；c5 text-top 14、c6 text-bottom 24（容器相对；glyph 顶 = 基线 − A，
/// 如 c5 glyph 90+14−24=80..110）。修复前 text-* 全部滞留行主基线（24/14）。
#[test]
fn r5316_va_text_glyph_baseline_anchored_to_parent_content_area() {
    let html = r#"<html><body style="margin:0">
<div id=c2 style="margin:30px 0; line-height:10px; font-size:30px; font-family:Ahem;">
<span id=s_tt style="vertical-align:text-top">XX</span><span id=s_b>XX</span><span id=s_tb style="vertical-align:text-bottom">XX</span>
</div>
<div id=c5 style="margin:30px 0; line-height:10px; font-size:30px; font-family:Ahem;"><span id=s5 style="vertical-align:text-top">XX</span></div>
<div id=c6 style="margin:30px 0; line-height:10px; font-size:30px; font-family:Ahem;"><span id=s6 style="vertical-align:text-bottom">XX</span></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let base = |frag: &str, expect: f32| {
        let f = doc.get_element_by_id(frag).unwrap_or_else(|| panic!("{frag}"));
        let b = frag_baseline(&result.root, f).unwrap_or_else(|| panic!("{frag} 存面基线"));
        assert!(
            (b - expect).abs() < 0.5,
            "{frag} 字形基线应 {expect}（页绝对），实际 {b}"
        );
    };
    // 页绝对：c2 顶 30、c5 顶 90、c6 顶 140（margin 30 链）+ 容器相对 14/24/34/14/24。
    base("s_tt", 44.0);
    base("s_b", 54.0);
    base("s_tb", 64.0);
    base("s5", 104.0);
    base("s6", 164.0);
}

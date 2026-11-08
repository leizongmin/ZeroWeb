//! R5471：own_floats 排除带 margin-box 判弃/裁顶口径测试。
//!
//! 谱系：WPT `css/CSS2/css1/c5525-fltmrgn-000.xht` test4（CSS2 §9.5 float margin box）。
//!
//! 根因链：compute_final_inline_layouts 的 own_floats 构造旧按 border 顶
//! `rel_y < 0` 整带丢弃——负 margin-top float（c5525 test4 img mt−30，border 顶
//! p-rel −30、margin-box 带 [0,15]）带被丢 → stored IFC float-unaware（首行
//! x=0 应 45）→ paint use_stored 重放错位墨，float 图元右侧红标记露出（c5525
//! ✓0.28% 残余 p4-band 主体）。修复：判弃按 margin-box 带底（整带在容器顶上方
//! 才丢）、跨顶带裁顶保留可见段，对齐步 6 remeasure R5442 同构臂（stored 与
//! 重测口径一致）。`ZW_FLOAT_BAND_MARGINBOX=0` 回退旧口径——flag 走 corpus
//! OFF 轮（进程内 set_var 不可靠，R3858 教训）。

use crate::engine::LayoutEngine;

fn compute(html: &str) -> std::sync::Arc<crate::types::LayoutBox> {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles).root
}

fn find_by<'a>(
    root: &'a crate::types::LayoutBox,
    pred: &dyn Fn(&crate::types::LayoutBox) -> bool,
) -> Option<&'a crate::types::LayoutBox> {
    if pred(root) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_by(c, pred))
}

fn first_text_x(root: &crate::types::LayoutBox) -> f32 {
    let div = find_by(root, &|b: &crate::types::LayoutBox| (b.width - 120.0).abs() < 0.5).expect("div 盒（w=120）");
    let p = div.children.first().expect("p 盒");
    let il = p.inline_layout.as_ref().expect("含 float 排除带的 p 应存 IFC");
    il[0].fragments[0].x
}

/// 跨顶带：float span mt−30 h45 → margin-box 带 [0,15] x[0,45)。裁顶保留可见段
/// → stored IFC 首行避让（frag.x=45）。修复前 border 顶 rel_y=−30<0 整带丢 →
/// frag.x=0（float-unaware，paint 重放错位墨）。
#[test]
fn negative_mt_float_band_clipped_stored_ifc_avoids() {
    let root = compute(
        r#"<html><body style="margin:0"><div style="width:120px; font:15px/15px Ahem;"><p style="margin:0"><span style="float:left; width:45px; height:45px; margin-top:-30px"></span>x</p></div></body></html>"#,
    );
    let fx = first_text_x(&root);
    assert!((fx - 45.0).abs() < 0.5, "首行 frag 应避让裁顶带（x=45），实际 {}", fx);
}

/// 整带在容器顶上方：mt−60 h45 → margin-box 带 [−60,−15]，带底 ≤ 0 不约束任何
/// 行 → 丢弃 → 首行不避让（frag.x=0）。与跨顶带案共同钉住判弃口径边界。
#[test]
fn fully_above_band_dropped_lines_unaffected() {
    let root = compute(
        r#"<html><body style="margin:0"><div style="width:120px; font:15px/15px Ahem;"><p style="margin:0"><span style="float:left; width:45px; height:45px; margin-top:-60px"></span>x</p></div></body></html>"#,
    );
    let fx = first_text_x(&root);
    assert!(fx.abs() < 0.5, "整带在容器上方不约束行排（x=0），实际 {}", fx);
}

/// 常规正带对照：mt 0 h45 → 带 [0,45]（border 顶 rel_y=0 不触旧判弃）→ 新旧口径
/// 同为避让（frag.x=45）。钉住修复不改变既有 rel_y ≥ 0 带行为。
#[test]
fn positive_band_avoids_same_as_before() {
    let root = compute(
        r#"<html><body style="margin:0"><div style="width:120px; font:15px/15px Ahem;"><p style="margin:0"><span style="float:left; width:45px; height:45px"></span>x</p></div></body></html>"#,
    );
    let fx = first_text_x(&root);
    assert!((fx - 45.0).abs() < 0.5, "常规 float 带首行应避让（x=45），实际 {}", fx);
}

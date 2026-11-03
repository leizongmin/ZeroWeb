//! R5399：BFC root auto 高终末 float 包含补扩 + R4140 unhoist 兄弟间距保留回归测试。
//!
//! 谱系：WPT `css/CSS2/floats-clear/floats-145.xht`（CSS 2.1 §9.5/§10.6.7，蓝条整体
//! 缺失 ✗2.06% → ✓0.00%）双缺口：
//!
//! ① **float 父高塌 0**——step 5 `recompute_bfc_auto_height`（float_positioning.rs）跑时
//!    float 子内容高尚未物化（R3787：taffy 对 float 叶子欠计、子树未终化），其 R4069
//!    「空块子折叠 mb」臂把未测量 float 子按空块折叠 → float div（BFC root）h=0。
//!    float 高在 step 6+ inline 终化物化后无人重访 BFC 祖先。修复 =
//!    `bfc_float_containment_late`（postprocess.rs，终末 grow-only max 补扩，
//!    §10.6.7）。
//! ② **R4140 unhoist 丢兄弟间距**——taffy 把 inline-block 原子首子的 mt 塌穿容器
//!    （img 的 UA display 为 InlineBlock 近似），R4140 unhoist 把容器 y 上提
//!    over_collapse 时丢弃 taffy 塌进链的前驱块 mb 份额 → REF 页 div 落 p_bb+0 应
//!    p_bb+16。修复 = `unhoist_inline_first_atom_mt` 兄弟间距保留
//!    `clamp(prev.mb − 声明 mt, 0, over_collapse)`。
//!
//! kill-switch 敏感性（外部 env 实跑取证，2026-11-03，test-guard 包裹 runner）：
//! `ZW_BFC_FLOAT_CONTAIN_LATE=0` → 145 test 形态隔离单跑 ✗（outer h 回 0）；
//! `ZW_UNHOIST_SIBLING_GAP=0` → 145 ref 形态隔离单跑 ✗（div y 回 p_bb+0）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::{Document, NodeKind};
use zero_style_system::StyleSystem;

fn find_box<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_box(id, doc, c))
}

fn parent_of<'a>(root: &'a LayoutBox, target: &LayoutBox) -> Option<&'a LayoutBox> {
    for c in &root.children {
        if std::ptr::eq(c as *const LayoutBox, target as *const LayoutBox) {
            return Some(root);
        }
        if let Some(p) = parent_of(c, target) {
            return Some(p);
        }
    }
    None
}

fn compute(html: &str, css: &str) -> (Document, std::sync::Arc<LayoutBox>) {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result.root)
}

/// 145 主形态：float 父 div 唯一子为 float span（行高 90 + mb 30）→ 父 auto 高 120
///（§10.6.7 BFC root 含 float 子 margin-box 底；旧塌 0，蓝色背景整体缺失）。
#[test]
fn test_float_parent_auto_height_contains_float_child() {
    let html = r#"<html><body style="margin:8px">
<div id="outer" style="background:blue;float:left;font:30px/3 Ahem"><span id="inner" style="float:left;margin-bottom:1em">Text sample</span></div>
</body></html>"#;
    let (doc, root) = compute(html, "");
    let outer = find_box("outer", &doc, &root).expect("outer float div");
    let inner = find_box("inner", &doc, &root).expect("inner float span");

    // ★ R5399 load-bearing：旧塌 0；span 行高 90 + mb 30。
    assert!(
        (inner.height - 90.0).abs() <= 0.5,
        "inner float span line box height must be 90; h={}",
        inner.height
    );
    assert!(
        (outer.height - 120.0).abs() <= 0.5,
        "float parent BFC auto height must contain float child margin-box bottom (120); h={}",
        outer.height
    );
}

/// 145 REF 页形态：p mb 16 + 后继 div（首子 img mt 90 塌穿）→ div.y = p_bb + 16
///（§8.3.1 邻接兄弟对 gap 保留；旧 unhoist 丢 p.mb → div 落 p_bb+0）。
#[test]
fn test_unhoist_keeps_preceding_sibling_margin_gap() {
    let html = r#"<html><body>
<p id="para">Test passes if there is a blue bar.</p>
<div id="dv"><img id="pic" src="" style="margin-top:90px;width:330px;height:30px"></div>
</body></html>"#;
    let (doc, root) = compute(html, "");
    let para = find_box("para", &doc, &root).expect("p");
    let dv = find_box("dv", &doc, &root).expect("div");

    // ★ R5399 load-bearing：div 在 p border-box 底 + max(p.mb 16, div 声明 mt 0)。
    // 旧 unhoist 全额上提 over_collapse → div.y = 18（p_bb，gap 0）。
    let p_bb = para.y + para.height;
    assert!(
        (para.margin_bottom - 16.0).abs() <= 0.5,
        "p default margin-bottom must be 16; mb={}",
        para.margin_bottom
    );
    assert!(
        (dv.y - (p_bb + 16.0)).abs() <= 0.5,
        "div must sit at p_bb + collapsed sibling gap 16; div.y={} p_bb={}",
        dv.y,
        p_bb
    );
    let _ = parent_of(&root, dv); // parent 链存在性（结构守卫，不承载断言）
}

/// R4140 原语义守卫：容器无前驱块兄弟（body 首子 inline-block mt 32）→ unhoist
/// 全额上提不变（R5399 prev_gap=0 与旧行为逐值一致）。
#[test]
fn test_unhoist_first_child_no_prev_sibling_unchanged() {
    let html = r#"<html><body style="margin:8px">
<div id="atom" style="display:inline-block;margin-top:32px;width:40px;height:20px"></div>
</body></html>"#;
    let (doc, root) = compute(html, "");
    let body = find_box("atom", &doc, &root)
        .and_then(|atom| parent_of(&root, atom))
        .expect("body box");

    // R4140 探针谱系（debug_r4140_layout_dump）：body margin-top 8，无前驱块兄弟 →
    // unhoist 后 body 边缘 = 声明 margin 8（原子 mt 32 由 IFC 行内定位自持）。
    assert!(
        (body.y - 8.0).abs() <= 0.5,
        "body with first inline-block atom child must stay at declared margin 8; y={}",
        body.y
    );
}

/// 既有语义守卫：非 BFC auto 高块的 float 排除收缩（§10.5.1，step 5.2 既有域）不被
/// R5399 终末补扩反转——overflow:visible div 的 float 子不撑高父。
#[test]
fn test_non_bfc_auto_height_still_excludes_floats() {
    let html = r#"<html><body style="margin:8px">
<div id="host" style="background:yellow"><div id="flo" style="float:left;width:100px;height:50px;background:red"></div></div>
</body></html>"#;
    let (doc, root) = compute(html, "");
    let host = find_box("host", &doc, &root).expect("host div");

    assert!(
        host.height <= 0.5,
        "non-BFC auto-height block must exclude float child from height (10.5.1); h={}",
        host.height
    );
}

/// 既有语义守卫：overflow:hidden BFC root 的 float 子计入高（step 5 既有臂）与终末
/// 补扩幂等——float 已定位（y=0）时补扩值与 step 5 臂一致。
#[test]
fn test_overflow_hidden_bfc_contains_float() {
    let html = r#"<html><body style="margin:8px">
<div id="host" style="overflow:hidden;background:blue"><div id="flo" style="float:left;width:100px;height:50px;background:red"></div></div>
</body></html>"#;
    let (doc, root) = compute(html, "");
    let host = find_box("host", &doc, &root).expect("host div");

    assert!(
        (host.height - 50.0).abs() <= 0.5,
        "BFC root (overflow:hidden) auto height must contain float child; h={}",
        host.height
    );
}

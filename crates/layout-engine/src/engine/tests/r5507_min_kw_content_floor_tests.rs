//! R5507（css-sizing-3 §5.2/§5.3 used 值约束）：min-width content 关键字 + 自身
//! definite width 的内容基下限。
//!
//! 缺陷机制（双刀）：① `kw_aware_max_width` 对 min-width 关键字盒（bare fit-content
//! 经 R1018 映射 MaxContent，min-content/max-content 直映射）在自身 definite width
//! 在场时返回被 `box_content_max_width_inner` 尾部 walk 的 definite-width 守卫压制
//! 的 full 口径（own_explicit > 0 → atom/inline 段核算 walk 不运行 → children_inner
//! 塌 0 → 叶回退 = 声明宽）——min 关键字下限整体丢失（width:10px + min-width:
//! fit-content 内容 100 → 测 10 应 100）。② R4149 写入臂把 size.width 清成 auto——
//! taffy 块级 auto 拉伸到 CB，宽 CB 下过填充（应 100 非 800）。
//!
//! 修复：① min 关键字贡献 = max(声明宽, 内容测量)（content-only 直测，guard 门
//! own_explicit ≤ 0.5 不触发即 walk 运行）；② 写入只设 min_size，保留 converter
//! 的 size.width，taffy 约束求解 min ≥ size 取大。
//!
//! 驱动案：corpus css-sizing fit-content-min-inline-size（chromium oracle 定谳
//! 100×100 绿方块；修复前 ZW 13px 窄条）。锚全用定宽原子子，字体无关。
//!
//! 谱系：R1018（bare fit-content→MaxContent）、R4149/R4151（min/max-width 关键字
//! 贡献钳）、R5037（content-only 口径——fit-content() 公式 W_max）、R5375
//!（max-width:max-content 内容基贡献）；本钉收 min-width 关键字 × definite width
//! 交叉域（R4151 is_kw 已含 min 关键字但消费臂只挂 max_w 分支）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<f32> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b.width);
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

/// 主判例（corpus fit-content-min-inline-size 同构）：body 直下 inner div
/// `min-width: fit-content; width: 10px` 含两个 50px 定宽 inline-block——min
/// 关键字地板 100 精确生效（修复前塌声明宽 10），且**不**被 taffy auto 拉伸到
/// CB 宽（修复②语义：宽 CB 下精确 100）。
#[test]
fn r5507_min_kw_floor_exact_content_in_wide_cb() {
    let html = r#"<html><body style="margin:0">
<div style="min-width: fit-content; width: 10px;">
  <span style="display: inline-block; width: 50px;"></span><span style="display: inline-block; width: 50px;"></span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let id = *divs.first().expect("inner div");
    let w = find_box(&result.root, id).expect("inner box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R5507: min-width:fit-content 应地板到内容 100（修复前 10；不得 CB 满宽 800），实际 {w}"
    );
}

/// 链式主形（corpus 原页）：外层 `width: fit-content` + 内层 min 关键字地板——
/// 内层 100 后外层 shrink-to-fit 跟随 100（修复前外层测 10 塌同宽）。
#[test]
fn r5507_bare_fitcontent_outer_follows_floored_inner() {
    let html = r#"<html><body style="margin:0">
<div style="width: fit-content; height: 100px; background: green;">
  <div style="min-width: fit-content; width: 10px;">
    <span style="display: inline-block; width: 50px;"></span><span style="display: inline-block; width: 50px;"></span>
  </div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let outer = *divs.first().expect("outer div");
    let inner = *divs.last().expect("inner div");
    let ow = find_box(&result.root, outer).expect("outer box");
    let iw = find_box(&result.root, inner).expect("inner box");
    assert!((iw - 100.0).abs() < 0.5, "R5507: 内层地板 100，实际 {iw}");
    assert!(
        (ow - 100.0).abs() < 0.5,
        "R5507: 外层 fit-content 随地板后内容收缩 100（修复前 10），实际 {ow}"
    );
}

/// R4151 既有域 parity：`min-width: min-content` 关键字同形同修（is_kw 同桶，
/// 缺陷与修复均沿同一路径——修复前塌 10 同证）。
#[test]
fn r5507_min_content_keyword_same_domain() {
    let html = r#"<html><body style="margin:0">
<div style="min-width: min-content; width: 10px;">
  <span style="display: inline-block; width: 50px;"></span><span style="display: inline-block; width: 50px;"></span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let id = *divs.first().expect("inner div");
    let w = find_box(&result.root, id).expect("inner box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R5507: min-width:min-content 同域地板 100，实际 {w}"
    );
}

/// own_def 侧守卫（css-sizing-3 §5.2 max(声明宽, 内容测量) 取大）：width:200px +
/// min-width:min-content + 窄内容 50px——声明宽胜出，used 保持 200（min 关键字
/// 只地板不收缩；修复②保留声明宽的直接验证）。
#[test]
fn r5507_definite_width_wins_when_content_narrower() {
    let html = r#"<html><body style="margin:0">
<div style="min-width: min-content; width: 200px;">
  <span style="display: inline-block; width: 50px;"></span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let id = *divs.first().expect("inner div");
    let w = find_box(&result.root, id).expect("inner box");
    assert!(
        (w - 200.0).abs() < 0.5,
        "R5507: 声明宽 200 > 内容 50 时 used 保持 200（min 关键字只地板），实际 {w}"
    );
}

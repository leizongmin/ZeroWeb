//! R5074（CSS2 §10.3.7 rule 3/4）：abspos auto margin 分配门——auto margin 均分/单边
//! 分配（视觉居中）仅当 inset 三元组（left/width/right，垂直同理）**全部非 auto**；
//! 任一 inset 为 auto 时 auto margin 归 0、解该 inset（rule 4）。上游 taffy 门只看
//! auto margin 计数，left:auto + right:0 + 定宽 + margin:auto 被错误居中。
//! driving: css/CSS2/positioning abspos-009（绿条应贴 body CB 右缘）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

/// kill-switch 用例以 env var 表达（`ZW_ABSPOS_AUTO_MARGIN_INSET`），本文件内所有
/// 用例先抢锁防并行互踩；毒化容忍（前序 panic 不连坐）。
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.x, b.y, b.width));
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

/// abspos-009 形态：body position:relative（CB 宽 784）+ abspos width:160 + right:0 +
/// margin:auto + left/bottom auto → 贴右缘（相对 x = 784−160 = 624，绝对 632），
/// 不居中（taffy 旧渲绝对 320）。
#[test]
fn r5074_auto_margin_with_auto_inset_anchors_to_definite_inset() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin: 8px; position: relative;">
<div style="position: absolute; width: 160px; height: 16px; right: 0; margin: auto; background: green;"></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let did = *doc.get_elements_by_tag_name("div").first().expect("abspos div");
    let (x, _y, w) = find_box(&result.root, did).expect("abspos box");
    assert_eq!(w, 160.0, "定宽保持");
    // find_box 返回相对 body 的 x：624 + body abs x 8 = 绝对 632（CB 右缘 792 − 160）。
    assert!(
        (x - 624.0).abs() < 1.0,
        "R5074: left:auto + right:0 + margin:auto 应贴右缘（相对 624），实际 {x}"
    );
}

/// rule 3 保真：left/right/width 三者全非 auto + 双 auto margin → 居中语义不变
///（绝对 320 = 8 + (784−160)/2）。
#[test]
fn r5074_over_constrained_centering_preserved() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin: 8px; position: relative;">
<div style="position: absolute; width: 160px; height: 16px; left: 0; right: 0; margin: auto; background: green;"></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let did = *doc.get_elements_by_tag_name("div").first().expect("abspos div");
    let (x, _y, _w) = find_box(&result.root, did).expect("abspos box");
    assert!(
        (x - 312.0).abs() < 1.0,
        "R5074: 三者全非 auto + 双 auto margin 居中语义保持（相对 312），实际 {x}"
    );
}

/// kill-switch 双态：`ZW_ABSPOS_AUTO_MARGIN_INSET=0` 回退上游门 → 009 形态错误居中
///（相对 312 = 624/2 旧基线）。
#[test]
fn r5074_kill_switch_reverts_to_centering() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin: 8px; position: relative;">
<div style="position: absolute; width: 160px; height: 16px; right: 0; margin: auto; background: green;"></div>
</body></html>"#;
    // SAFETY: 锁内同步设置/读取；无其他测试并发读 ZW_ABSPOS_AUTO_MARGIN_INSET。
    unsafe {
        std::env::set_var("ZW_ABSPOS_AUTO_MARGIN_INSET", "0");
    }
    let (_, result) = layout(html);
    unsafe {
        std::env::remove_var("ZW_ABSPOS_AUTO_MARGIN_INSET");
    }
    let doc = zero_dom::parse_html(html);
    let did = *doc.get_elements_by_tag_name("div").first().expect("abspos div");
    let (x, _y, _w) = find_box(&result.root, did).expect("abspos box");
    assert!(
        (x - 312.0).abs() < 1.0,
        "R5074: kill-switch 下应回退居中基线（相对 312），实际 {x}"
    );
}

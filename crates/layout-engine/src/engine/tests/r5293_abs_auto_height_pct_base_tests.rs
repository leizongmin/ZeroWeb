//! R5293（CSS2 §10.5 × §10.6.4 × §8.3.1）：auto 高 abs 盒的高度来自内容，不得
//! 回灌为子代百分比高解析基——包含块高依赖内容时子代 `height:<percentage>`
//! 计算为 auto，末子 bottom margin 借 collapse-through 计入 abs 盒高。
//! 定高回灌曾锁死 auto 高（inner own_margins 断开，80px margin 丢失）。
//! kill-switch `ZW_ABSPOS_AUTO_H_PCT_BASE=0` 整体回退。
//! corpus 定谳：child-bottom-margin-in-unresolvable-percentage-height ✓0.00；
//! 已知代价 margin-applies-to-015 翻红 1.54%（表 wrapper bottom_margin CS/PL
//! 不对称被本修正现形，FIXME(table-margin) 挂账表域）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// child-bottom-margin-in-unresolvable-percentage-height 结构：红 100×100 内
/// abs 绿（width:100%，高 auto）> inner（height:100%）> gc（h:20 + mb:80）。
/// inner 百分比高 → auto，gc mb:80 塌穿 inner 溢入绿盒底（abs 立 BFC 拦停）→
/// 绿盒高 = 20 + 80 = 100 盖满红盒（修前绿盒 20，红盒下 80px 露红）。
#[test]
fn r5293_abs_auto_height_child_bottom_margin_escapes_into_height() {
    let html = r#"<html><body style="margin:0">
<div id="red" style="position:relative; width:100px; height:100px; background:red;">
  <div id="green" style="position:absolute; width:100%; background:green;">
    <div id="inner" style="height:100%;">
      <div id="gc" style="margin-bottom:80px; height:20px;"></div>
    </div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let green_id = doc.get_element_by_id("green").expect("green");
    let inner_id = doc.get_element_by_id("inner").expect("inner");
    let gc_id = doc.get_element_by_id("gc").expect("gc");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let green = find(&result.root, green_id).unwrap();
    let inner = find(&result.root, inner_id).unwrap();
    let gc = find(&result.root, gc_id).unwrap();
    assert!(
        (green.height - 100.0).abs() < 0.5,
        "绿盒高 = gc 20 + 塌穿溢入 mb 80 = 100（定高回灌锁死为 20），实得 {}",
        green.height
    );
    assert!(
        (inner.height - 20.0).abs() < 0.5,
        "inner 高 = gc 内容 20，实得 {}",
        inner.height
    );
    assert!(
        (gc.y - green.y).abs() < 0.5 && (gc.height - 20.0).abs() < 0.5,
        "gc 贴绿盒内容顶 h=20，实得 rel_y={} h={}",
        gc.y - green.y,
        gc.height
    );
}

/// absolute-non-replaced-max-height-002 守护：重算高须仍受 max-height 钳
///（ContentSize 模式叶盒忽略自身 min/max，采信重排高时补外层钳）。
#[test]
fn r5293_abs_auto_height_reclamp_max_height() {
    let html = r#"<html><body style="margin:0">
<div id="cb" style="position:relative; width:200px; height:200px;">
  <div id="abs" style="position:absolute; right:0; max-height:50px; background:blue; font:100px/1 Ahem;">&nbsp;</div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let abs_id = doc.get_element_by_id("abs").expect("abs");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let abs = find(&result.root, abs_id).unwrap();
    assert!(
        (abs.height - 50.0).abs() < 0.5,
        "abs 内容高 100 应被 max-height:50 钳住（采信重排高须补钳），实得 {}",
        abs.height
    );
}

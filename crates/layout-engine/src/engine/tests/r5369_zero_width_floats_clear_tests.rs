//! R5369：嵌套 float 自带 clear 的底边预测 + R4222 回加范围扩到 float 祖先容器的
//! clear 子（WPT `css/CSS2/floats/zero-width-floats.html` 4.17% → 0.00%）。
//!
//! 谱系与机理：
//!  - fix A（`nested_float_bottoms` 清预测，`ZW_NESTED_FLOAT_CLEAR_PRED=0` 回退）：
//!    R1392 嵌套 float 底边扫描发生在父 walk 主循环内、早于递归子 walk 落实嵌套
//!    float 自身 `clear` 的抬升 → 底边虚短 → 后续 clear 兄弟 clearance 短 50
//!   （fr clear:left 底 200 被记 150，g2 静态位 200 应 234）。
//!  - fix B（R4222 回加范围，`ZW_R4222_FLOAT_ANCESTOR_SCOPE=0` 回退）：float walk
//!    对 clear 子四臂无条件重写 child.y、丢掉 taffy 烘焙的 block-level relative
//!    inset（g2 top:-100px 被抹）；旧回加范围只认「直接子含 float」容器，深层
//!    float 的祖先容器（body→container→float）不回加 → g2 停在 clearance 静态位。
//!    非 clear 子是否被重写随臂而异（clear-on-parent-with-margins-no-clearance
//!    外层 relative 块 y 未被重写，扩围回加 = 双计），故仅扩到 **clear** 子。
//!
//! 两开关正交承重：仅 A = 8.33%（inset 仍丢）、仅 B = 2.08%（clearance 仍短 50）、
//! 双开 = 0.00%、双关 = 4.17%（基线）。

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

/// zero-width-floats 结构：container(ml:50, w:100) 内 fl(float:left w:0 h:50) +
/// fr(float:right clear:left w:0 h:150) + g1(overflow:hidden m:0 -50px h:100)；
/// body 后继 g2(clear:both relative top:-100px w:200 h:100)。
///
/// 钉位（content-rel，CSS2 §9.5/§9.5.1/§9.5.2 + css-position-3 §relative-positioning）：
///  - fr.y == 50：clear:left 抬到 fl 底（fix A：嵌套扫描预测同值）。
///  - g2.y == 134：静态 clearance 位 234（fr 底 200 + p 段 34）+ 回加 inset −100
///    （fix B）。绘出 abs = 16 + 134 = 150，与 g1 底（16+134）拼成 200×200 连续方块。
///  - g2.x == 0：回流位（未误入 BFC 避让旁置）。
#[test]
fn r5369_zwf_nested_clear_bottom_and_relpos_inset() {
    let html = r#"<html><body>
<p>Test passes if there is a filled green square and <strong>no red</strong>.</p>
<div id="container" style="margin-left:50px; width:100px;">
  <div id="fl" style="float:left; width:0; height:50px;"></div>
  <div id="fr" style="float:right; clear:left; width:0; height:150px;"></div>
  <div id="g1" style="overflow:hidden; margin:0 -50px; height:100px; background:green;"></div>
</div>
<div id="g2" style="position:relative; top:-100px; overflow:hidden; clear:both; width:200px; height:100px; background:green;"></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let ids: Vec<(&str, NodeId)> = ["container", "fl", "fr", "g1", "g2"]
        .iter()
        .map(|n| (*n, doc.get_element_by_id(n).expect("id")))
        .collect();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let box_of = |name: &str| {
        let id = ids.iter().find(|(n, _)| *n == name).expect("name").1;
        find(&result.root, id).unwrap_or_else(|| panic!("{name} box"))
    };
    let (fl, fr, g1, g2) = (box_of("fl"), box_of("fr"), box_of("g1"), box_of("g2"));
    assert!((fl.y - 0.0).abs() < 0.5, "fl.y = {}", fl.y);
    assert!((fr.y - 50.0).abs() < 0.5, "fr.y（clear:left 抬升）= {}", fr.y);
    assert!(
        (g1.y - 0.0).abs() < 0.5 && (g1.x + 50.0).abs() < 0.5,
        "g1 = {},{}",
        g1.x,
        g1.y
    );
    assert!(
        (g2.y - 134.0).abs() < 0.5,
        "g2.y（clearance 234 + inset -100）= {}",
        g2.y
    );
    assert!((g2.x - 0.0).abs() < 0.5, "g2.x = {}", g2.x);
}

/// min-height-106 结构（简化去 p）：red(float:left min-height:200 overflow:auto w:200) +
/// green(clear:left relative top:-200px overflow:auto w:200 h:200)。
///
/// 钉位：green.y == 0（walk 重写 clearance 位 200、R4222 回加 top:-200；绘出覆盖
/// float 顶部成全绿）；green.x == 0——walk 重写后 y=200 已无 float 重叠，BFC 避让
/// 不得旁置（首版 walk 内 +rel_dy 使 y=0 触发避让 → x=200 的缺陷签名）。
#[test]
fn r5369_min_height_106_clear_relpos_no_beside_slip() {
    let html = r#"<html><body>
<div id="red" style="font:100px/1 Ahem; overflow:auto; width:200px; background:red; float:left; min-height:200px;">XXX</div>
<div id="green" style="font:100px/1 Ahem; overflow:auto; width:200px; background:green; clear:left; height:200px; position:relative; top:-200px;">XX</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let red_id = doc.get_element_by_id("red").expect("red");
    let green_id = doc.get_element_by_id("green").expect("green");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let red = find(&result.root, red_id).expect("red box");
    let green = find(&result.root, green_id).expect("green box");
    assert!(
        (red.y - 0.0).abs() < 0.5 && (red.height - 200.0).abs() < 0.5,
        "red = {}",
        red.y
    );
    assert!(
        (green.y - 0.0).abs() < 0.5,
        "green.y（clearance 200 + inset -200）= {}",
        green.y
    );
    assert!((green.x - 0.0).abs() < 0.5, "green.x（不得旁置避让）= {}", green.x);
}

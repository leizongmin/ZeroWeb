//! R5370：display:table/inline-table 内 OOF 子（absolute/fixed）不生成匿名 cell、
//! 不入空 grid 收缩（WPT `css/css-transforms/transform-abspos-006/007` 6.28% →
//! 0.00%）。
//!
//! 机理（css-position-3 §9.1 + CSS2 §17.2.1 + §9.8）：build_grid 的 R3815 匿名
//! cell 包裹臂旧只排 float（R3817）、不排 absolute/fixed——OOF 子被包成 cell 后，
//! cell 布局按流内语义把其 definite 宽/高覆写为表/格几何（200×100 + left:50/
//! top:100 → 100×200@表原点）；`ZW_TABLE_OOF_CHILD_EXCLUDE=0` 回退旧行为。
//! build_grid 排除后 grid 空 → 空表臂的 shrink_table_to_block_content 同款排除
//! OOF 子（否则收缩臂仍会覆写 definite 宽），表保持 taffy 对 definite 宽/高的
//! 解析，OOF 子由 taffy abspos 通道按 CB inset 定位定尺寸（与 block 容器同路径）。

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

fn compute(html: &str, ids: &[&'static str]) -> (crate::types::LayoutResult, Vec<(&'static str, NodeId)>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let pairs: Vec<(&'static str, NodeId)> = ids
        .iter()
        .map(|n| (*n, doc.get_element_by_id(n).expect("id")))
        .collect();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    (engine.compute(&doc, &styles), pairs)
}

/// transform-abspos-006 结构（去 transform 隔离本缺陷层；body margin:0 去掉
/// CB 原点偏移）：table(100×200) + abspos 子(200×100 left:50 top:100)。
///
/// 钉位：table 保持 definite 100×200（OOF 子不参与收缩/格记账）；navy 200×100
/// （definite 宽高不被 cell 几何覆写）@(50,100)（CB inset 定位）。旧缺陷签名：
/// navy wh=(100,200)、y=0（当 cell 贴表原点）。
#[test]
fn r5370_table_abspos_child_definite_size_and_insets() {
    let html = r#"<html><body style="margin:0">
<div id="tbl" style="display:table; width:100px; height:200px;">
  <div id="navy" style="position:absolute; left:50px; top:100px; width:200px; height:100px; background:navy;"></div>
</div>
</body></html>"#;
    let (result, ids) = compute(html, &["tbl", "navy"]);
    let tbl = find(&result.root, ids[0].1).expect("tbl box");
    let navy = find(&result.root, ids[1].1).expect("navy box");
    assert!(
        (tbl.width - 100.0).abs() < 0.5 && (tbl.height - 200.0).abs() < 0.5,
        "tbl wh = {},{}",
        tbl.width,
        tbl.height
    );
    assert!(
        (navy.width - 200.0).abs() < 0.5 && (navy.height - 100.0).abs() < 0.5,
        "navy wh（definite 尺寸不被 cell 覆写）= {},{}",
        navy.width,
        navy.height
    );
    assert!(
        (navy.x - 50.0).abs() < 0.5 && (navy.y - 100.0).abs() < 0.5,
        "navy xy（CB inset 定位）= {},{}",
        navy.x,
        navy.y
    );
}

/// transform-abspos-007 形态：inline-table 容器 + position:fixed 子（fixed CB =
/// 视口，inset 即视口坐标）。
#[test]
fn r5370_inline_table_fixed_child_viewport_insets() {
    let html = r#"<html><body style="margin:0">
<div id="itbl" style="display:inline-table; width:100px; height:200px;">
  <div id="navy" style="position:fixed; left:20px; top:30px; width:150px; height:80px; background:navy;"></div>
</div>
</body></html>"#;
    let (result, ids) = compute(html, &["itbl", "navy"]);
    let itbl = find(&result.root, ids[0].1).expect("itbl box");
    let navy = find(&result.root, ids[1].1).expect("navy box");
    assert!(
        (itbl.width - 100.0).abs() < 0.5 && (itbl.height - 200.0).abs() < 0.5,
        "itbl wh = {},{}",
        itbl.width,
        itbl.height
    );
    assert!(
        (navy.width - 150.0).abs() < 0.5 && (navy.height - 80.0).abs() < 0.5,
        "navy wh = {},{}",
        navy.width,
        navy.height
    );
    assert!(
        (navy.x - 20.0).abs() < 0.5 && (navy.y - 30.0).abs() < 0.5,
        "navy xy（视口 inset）= {},{}",
        navy.x,
        navy.y
    );
}

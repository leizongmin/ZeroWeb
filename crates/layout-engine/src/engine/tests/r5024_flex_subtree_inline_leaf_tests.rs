//! R5024 回归：content-sized 子树（flex/grid item、inline-block、float 祖先）内的
//! 「文本 + inline 元素」混合级须走 R1024 leaf 路径，inline 文本才参与固有尺寸聚合。
//!
//! 缺陷形态（活体 baidu SERP 标题，r2s3 几何取证）：flex row（title-box）中的 h3
//! （flex:0 1 auto）子树 h3 > a > span > em+text——span 级 R1024 语境不满足走「仅元素
//! 子」路径，inline 文本兄弟被丢出 taffy 树 → h3 flex base size = em 段 94px（应 433px），
//! paint 侧 IFC 仍按 94px 折 5 行 120px 高叠压下方摘要。修复后单行 434×24。
//!
//! 依据 css-flexbox-1 §9.9.1（item 固有主轴尺寸由内容 max-content 馈入）+ css-sizing-3
//! §5（固有尺寸沿嵌套块传播）。kill-switch：`ZW_FLEX_SUBTREE_INLINE_LEAF=0` 回退。

use crate::engine::LayoutEngine;
use crate::types::{LayoutBox, LayoutResult};
use zero_dom::{Document, NodeId};
use zero_style_system::StyleSystem;

const TITLE_REST: &str = ",极简·轻量化·无广告·更智能的全新浏览器";

fn find_box(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    if root.node_id == Some(id) {
        return Some(root);
    }
    for c in &root.children {
        if let Some(b) = find_box(c, id) {
            return Some(b);
        }
    }
    None
}

fn compute(markup: &str) -> (Document, LayoutResult) {
    let doc = zero_dom::parse_html(&format!(r#"<html><body style="margin:0">{markup}</body></html>"#));
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// R5024 主案：flex row 中的 h3（flex item）子树含 em+text 混合 inline 级——
/// h3 须取全文 max-content 宽（>300）单行（h=24），不得塌到 em 段宽（~94）折多行。
#[test]
fn r5024_flex_item_nested_em_text_max_content() {
    let (doc, result) = compute(&format!(
        r#"<div style="display:flex;width:608px"><h3 id="h3" style="font-size:18px;line-height:24px;word-break:break-all"><a style="display:block"><span><em>ZERO浏览器</em>{}</span></a></h3><a style="flex:1 1 auto"></a></div>"#,
        TITLE_REST
    ));
    let h3_id = doc.get_element_by_id("h3").expect("h3");
    let h3 = find_box(&result.root, h3_id).expect("h3 LayoutBox");
    println!("[R5024] flex h3 w={:.1} h={:.1}", h3.width, h3.height);
    assert!(
        h3.width > 300.0,
        "flex item 应取全文 max-content 宽（~433），不得塌到 em 段宽，got {}",
        h3.width
    );
    assert_eq!(h3.height, 24.0, "全文单行应 24px 高（缺陷态折 5 行 120px paint 叠压）");
}

/// R5024 变体：inline-block 祖先子树内的混合 inline 级同样入 leaf 路径——
/// inline-block 收缩到全文宽（>300），不得收缩到 em 段宽。
#[test]
fn r5024_inline_block_subtree_nested_em_text_shrink_to_fit() {
    let (doc, result) = compute(&format!(
        r#"<div id="ib" style="display:inline-block;font-size:18px;line-height:24px"><div><a style="display:block"><span><em>ZERO浏览器</em>{}</span></a></div></div>"#,
        TITLE_REST
    ));
    let ib_id = doc.get_element_by_id("ib").expect("ib");
    let ib = find_box(&result.root, ib_id).expect("ib LayoutBox");
    println!("[R5024] inline-block w={:.1} h={:.1}", ib.width, ib.height);
    assert!(
        ib.width > 300.0,
        "inline-block 应收缩到全文 max-content 宽（~433），got {}",
        ib.width
    );
    assert_eq!(ib.height, 24.0, "全文单行应 24px 高");
}

/// R5024 邻臂（负对照基线）：无 em 的纯文本同结构——修复前后均应为全文宽单行
///（守住 gate 不因收窄语境而破坏 R1024 原有形态）。
#[test]
fn r5024_neighbor_plain_text_flex_item_baseline() {
    let (doc, result) = compute(&format!(
        r#"<div style="display:flex;width:608px"><h3 id="h3" style="font-size:18px;line-height:24px;word-break:break-all"><a style="display:block"><span>ZERO浏览器{}</span></a></h3><a style="flex:1 1 auto"></a></div>"#,
        TITLE_REST
    ));
    let h3_id = doc.get_element_by_id("h3").expect("h3");
    let h3 = find_box(&result.root, h3_id).expect("h3 LayoutBox");
    println!("[R5024] plain h3 w={:.1} h={:.1}", h3.width, h3.height);
    assert!(h3.width > 300.0, "纯文本邻臂应保持全文宽，got {}", h3.width);
    assert_eq!(h3.height, 24.0);
}

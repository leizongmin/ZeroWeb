//! slice19：inline 元素内原子行内级后代（input 等 replaced/inline-block）的盒子几何钉。
//!
//! 症状（活体 + 最小静态页 ZW vs Chrome/154 双取证，800×600）：
//! `div > div > span > input(inline-block)` 结构下，span 及其后代 gBCR 全 0×0
//!（Chrome：span [0,324,624.5,17]、input [0,322.5,520,22]）；整段 inline 子树
//! 不参与绘制（静态页 B 区像素空白）。直接位于块容器下的同款 input（`div > input`）
//! 几何正常——差异变量是 **inline 中间层**（R2156 inline_box_model_coherence skip 路径）。
//!
//! 钉的是布局层契约（CSS2 §9.2.1.1 inline formatting；§10.3.1 replaced inline；
//! §10.6.1 内容高度）：原子行内级后代参与父 IFC 行排（行高证据：容器高 44 = inline-block
//! 高）的同时，必须保有自身布局盒（gBCR/命中面/绘制的共同真值源）。
//!
//! 负控制：display:none 祖先内 input 两引擎均 0×0（盒缺席是正确语义）——防钉退化恒真。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::sync::Arc;
use zero_style_system::StyleSystem;

fn find_by_id<'a>(b: &'a LayoutBox, doc: &zero_dom::Document, id: &str) -> Option<&'a LayoutBox> {
    fn walk<'a>(b: &'a LayoutBox, doc: &zero_dom::Document, id: &str, hits: &mut Vec<&'a LayoutBox>) {
        if let Some(nid) = b.node_id
            && doc.get_attribute(nid, "id").is_some_and(|v| v == id)
        {
            hits.push(b);
        }
        for c in &b.children {
            walk(c, doc, id, hits);
        }
    }
    let mut hits = Vec::new();
    walk(b, doc, id, &mut hits);
    hits.into_iter().next()
}

fn compute_root(html: &str) -> (zero_dom::Document, Arc<LayoutBox>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result.root)
}

/// 主钉（对应活体 #kw2/#su2 零几何）：span 内 inline-block input 必须有非零布局盒。
/// 挂账（同轴不钉）：inline 包装盒自身（`#mid` span）的 gBCR 仍为盒缺席——Chrome 报
/// 行内容面积（[624.5,17]），ZW 需 inline 包装盒合成机制（slice13/15 inline_reported_rect
/// 同族），不在本切片（原子后代盒子）半径内。
#[test]
fn s19_atomic_inline_inside_inline_span_keeps_box() {
    let html = r#"<html><body style="margin:0">
<div style="width:800px"><div id="outer"><span id="mid"><input id="kw" style="display:inline-block;width:512px;height:16px"><input id="su" style="width:100px;height:44px"></span></div></div>
</body></html>"#;
    let (doc, root) = compute_root(html);
    let kw = find_by_id(&root, &doc, "kw").expect("inline-block input in span 应有布局盒");
    assert!(
        kw.width > 0.0 && kw.height > 0.0,
        "span 内 inline-block input 盒尺寸应非零（Chrome [520,22]），实际 {}×{}",
        kw.width,
        kw.height
    );
    let su = find_by_id(&root, &doc, "su").expect("inline submit input in span 应有布局盒");
    assert!(
        su.width > 0.0 && su.height > 0.0,
        "span 内 submit input 盒尺寸应非零（Chrome [100,44]），实际 {}×{}",
        su.width,
        su.height
    );
}

/// R2156 动机案例对照（`<p><label>text <input></label></p>`，37-form-controls 形态）：
/// label 被 skip 后其原子行内后代 input 的盒子契约同题。
#[test]
fn s19_r2156_label_with_input_keeps_input_box() {
    let html = r#"<html><body style="margin:0">
<p id="p"><label id="lab">text <input id="in" style="width:120px;height:24px"></label></p>
</body></html>"#;
    let (doc, root) = compute_root(html);
    let input = find_by_id(&root, &doc, "in").expect("label 内 input 应有布局盒");
    assert!(
        input.width > 0.0 && input.height > 0.0,
        "R2156 skip 路径下 label 内 input 盒尺寸应非零，实际 {}×{}",
        input.width,
        input.height
    );
}

/// 负控制：display:none 祖先内 input 盒缺席是正确语义（Chrome 亦 0×0）。
#[test]
fn s19_negative_hidden_input_has_no_box() {
    let html = r#"<html><body style="margin:0">
<div id="hidden" style="display:none"><input id="hkw" style="width:512px;height:16px"></div>
</body></html>"#;
    let (doc, root) = compute_root(html);
    let found = find_by_id(&root, &doc, "hkw");
    match found {
        None => {}
        Some(b) => assert!(
            b.width == 0.0 && b.height == 0.0,
            "display:none 内 input 不应有非零盒，实际 {}×{}",
            b.width,
            b.height
        ),
    }
}

/// 守卫钉（S-TE1，pr61 testeff te1 形态）：被 R2156 skip 的 span 内**并置**可见
/// input 与 display:none input——收集臂必须经 `DisplayValue::None => continue`
/// 把隐藏分支挡在提升集之外（可见兄弟使 skip 谓词成立、下探真实可达）。
/// 基线两态皆绿（base 整树 skip 隐藏臂同样无盒；head 靠收集守卫）——守卫回退
///（隐藏臂被推入提升集获得 taffy 子树）时本钉变红，非 RED/GREEN 判别钉。
#[test]
fn s19_negative_hidden_inside_skipped_span_has_no_box() {
    let html = r#"<html><body style="margin:0">
<div><span><input id="vis" style="display:inline-block;width:120px;height:24px"><input id="hid" style="display:none;width:120px;height:24px"></span></div>
</body></html>"#;
    let (doc, root) = compute_root(html);
    let found = find_by_id(&root, &doc, "hid");
    match found {
        None => {}
        Some(b) => assert!(
            b.width == 0.0 && b.height == 0.0,
            "被 skip span 内 display:none input 不应有非零盒，实际 {}×{}",
            b.width,
            b.height
        ),
    }
}

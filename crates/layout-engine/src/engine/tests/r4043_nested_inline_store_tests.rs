//! R4043（css-content-3 §quotes / CSS2 §9.2.1.1）：块容器独子 inline 含纯 inline 嵌套树
//! 的 stored IFC 收窄守卫——深路径检测器单测。
//!
//! `<p><q>one <q>two</q> three</q></p>`（容器无直接文本，唯一 inline 子含嵌套 inline 元素）：
//! R207 的 PHASEA_STORE_EXT 守卫要求 inline 子为「叶文本容器」（无元素子节点），把此类
//! 结构排除在 stored IFC 之外 → 容器文本三行堆叠（quotes-005/013/015/019 嵌套 `<q>` 独子族，
//! 全部 1.05-1.34% 过阈失败）。R4043 把守卫收窄为三类**深路径域**，纯 inline 同字体嵌套树
//! 照常存储。端到端行为由导入的 WPT 常驻断言锚（quotes-015 / outline-004 / ruby-whitespace-001）
//! 守护；本文件只测检测器谓词本身。
//!
//! 深路径排除面回归锚（本轮 A/B 实证，改窄守卫时不得放宽）：
//! - 块级后代：inline-box-002 族（R109 碎片化域，R207 原始动机）
//! - `<br>` 嵌套：outline-004（`<div><span>xx<br>xx</span>`）换行丢失 → 0.00%→1.23%
//! - ruby 语义：ruby-whitespace-001 ref 页（`<rb><span>…</span></rb>` 含自身形态）行序错乱
//! - 字体度量：outline-022（`#target{font-size:80px}` 嵌三层 span）字形塌缩

use crate::inline_finalization::{subtree_font_differs_from, subtree_has_block_elem};
use zero_style_system::StyleSystem;

fn compute_styles(
    html: &str,
) -> (
    zero_dom::Document,
    std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    (doc, styles)
}

/// 深路径检测①：inline 子树含块级后代（R109 碎片化域）→ 检出。
#[test]
fn r4043_block_descendant_detected() {
    let (doc, styles) = compute_styles(r#"<html><body><span><div></div></span></body></html>"#);
    let spans = doc.get_elements_by_tag_name("span");
    let span_id = *spans.last().expect("span");
    assert!(subtree_has_block_elem(&doc, &styles, span_id), "块级后代应检出");
}

/// 深路径检测②：inline 子树含嵌套 `<br>` 后代 → 检出（含 br 直子的形态由 collect 层处理）。
#[test]
fn r4043_nested_br_structure() {
    // 纯 inline 嵌套（无深路径域）子树：嵌套 span 不触发块级/字体检测。
    let (doc, styles) = compute_styles(r#"<html><body><span>text<span>nested</span>tail</span></body></html>"#);
    let spans = doc.get_elements_by_tag_name("span");
    let outer = spans[0];
    assert!(
        !subtree_has_block_elem(&doc, &styles, outer),
        "纯 inline 嵌套不应误报块级后代"
    );
    assert!(
        !subtree_font_differs_from(&doc, &styles, outer, &styles[&outer]),
        "同字体嵌套不应误报字体度量差"
    );
}

/// 深路径检测③：inline 子树含字体度量不同的后代（font-size 80px）→ 检出。
#[test]
fn r4043_font_metric_differs_detected() {
    let (doc, styles) =
        compute_styles(r#"<html><body><span>text<span style="font-size: 80px">big</span></span></body></html>"#);
    let spans = doc.get_elements_by_tag_name("span");
    // 文档序第一个 span = 外层。
    let outer = spans[0];
    assert!(
        subtree_font_differs_from(&doc, &styles, outer, &styles[&outer]),
        "嵌套 font-size:80px 后代应检出字体度量差"
    );
}

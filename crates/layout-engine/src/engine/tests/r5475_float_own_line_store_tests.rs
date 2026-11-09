//! R5475：单行 float 容器自身行盒落盘（行盒归属主案 floats-149 形切片）。
//!
//! `div.block > div[display:inline] > span(文本, float:left 50%) + span(空, float:left)`
//! 形（css/CSS2/floats-clear/floats-149）：子 span 按 §9.7 块化为 float = BFC 根，文本行
//! 归属 float 自身 IFC（chromium 同构）。ZW 侧 span IFC 已正确产 1 行，旧 store 门因
//! 「非 pure-Ahem 且无 float 排除带」拒存 → 全树零行盒（探针定谳：display:inline 中间层
//! 物化 + 父级收集器视块化 span 为块级不发锚下探），paint Path B 逐盒重跑兜住视觉但
//! 行盒消费面（滚动溢出行臂等）失据。本切片放行单行 float 落盘。
//! `ZW_FLOAT_OWN_LINE_STORE=0` 回退旧行为。driving floats-149（✓0.16% 残余经 R5470
//! 证据门回退臂，行盒归属修复为销账前置）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

/// 手抠 <style> 文本（collect_stylesheets 不在依赖内，r5167 探针同口径）。
fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn find_box(root: &LayoutBox, id: zero_dom::NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

const FLOATS_149_HTML: &str = r#"<html><head><style>
.block { display: block; background: green; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; background: white; }
</style></head><body style="margin:0">
<div class="block" id="blk">
 <div class="inline" id="inl">
  <span>There should be a green bar to the right</span>
  <span></span>
 </div>
</div>
</body></html>"#;

/// 文本 float 自持行盒：span1（块化 float，含文本）inline_layout = 1 行 ≥1 片段；
/// 空 float span2 无行盒；display:inline 中间层不落盘（无双重 owner）。
#[test]
fn r5475_text_float_owns_its_line_box() {
    let doc = zero_dom::parse_html(FLOATS_149_HTML);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(FLOATS_149_HTML));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // span1：<div class="inline"> 首个 span 子（含文本）。
    let inline_div = doc.get_element_by_id("inl").expect("div.inline 存在");
    let span_ids: Vec<zero_dom::NodeId> = doc
        .child_nodes(inline_div)
        .into_iter()
        .filter(|&c| {
            doc.get(c)
                .is_some_and(|n| matches!(n.kind, zero_dom::NodeKind::Element(_)))
        })
        .collect();
    assert_eq!(span_ids.len(), 2, "div.inline 应恰有 2 个 span 子");

    let span1 = find_box(&result.root, span_ids[0]).expect("span1 盒应存在");
    let lines = span1
        .inline_layout
        .as_ref()
        .expect("文本 float 应自持行盒（R5475 落盘放行）");
    assert_eq!(
        lines.len(),
        1,
        "该形文本恰为一行（float 宽 50% 容得下），实际 {} 行",
        lines.len()
    );
    assert!(!lines[0].fragments.is_empty(), "行盒应含文本片段");
    let joined: String = lines[0].fragments.iter().map(|f| f.text.as_str()).collect();
    assert!(
        joined.replace(' ', "").contains("Thereshouldbe"),
        "行盒片段应为 span 文本（词级分片），实际 {:?}",
        lines[0].fragments.iter().map(|f| &f.text).collect::<Vec<_>>()
    );

    let span2 = find_box(&result.root, span_ids[1]).expect("span2 盒应存在");
    assert!(
        span2.inline_layout.is_none(),
        "空 float 无文本不落盘（gate-no-text 域不变）"
    );
}

/// display:inline 中间层不因本切片获得第二 owner：div.inline（非块级，I1 门）与
/// div.block（收集器视块化 span 为块级 → 0 文本行）均无 inline_layout，全树唯一
/// 行盒 owner = 文本 float span1。
#[test]
fn r5475_no_second_line_owner_in_inline_middle_layer() {
    let doc = zero_dom::parse_html(FLOATS_149_HTML);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(FLOATS_149_HTML));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let inline_div = doc.get_element_by_id("inl").expect("div.inline 存在");
    let block_div = doc.get_element_by_id("blk").expect("div.block 存在");
    for (tag, id) in [("div.inline", inline_div), ("div.block", block_div)] {
        let b = find_box(&result.root, id).unwrap_or_else(|| panic!("{tag} 盒应存在"));
        assert!(
            b.inline_layout.is_none(),
            "{tag} 不应落盘行盒（中间层/零文本收集域不变），实际 {} 行",
            b.inline_layout.as_ref().map(|l| l.len()).unwrap_or(0)
        );
    }
}
